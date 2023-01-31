#![allow(unused)]
use discv5::{
    Discv5Event, 
    TalkRequest
};
use discv5_overlay::{
    portalnet::{
        overlay_service::{
            OverlayCommand,
            OverlayService
        },
        storage::MemoryContentStore,
        types::{
            distance::XorMetric,
            messages::ProtocolId
        }, 
    },
    utp::stream::{UtpListener, UtpListenerRequest, UtpListenerEvent}
};
use futures::stream::{FuturesOrdered, FuturesUnordered};
use futures::{AsyncWriteExt, FutureExt, StreamExt};  //pin_mut
use rand::Rng;
use std::str::FromStr;
use tokio::{
    select, 
    sync::mpsc::{UnboundedReceiver, UnboundedSender}
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::debug;
use tracing::log::error;

use crate::das_node::{
    discovery,
    node_struct::DASNode,
    overlay, 
    overlay::{
        DASContentKey, 
        DASValidator,
    }
};

pub const NUMBER_OF_NODES: usize = 10;
const DAS_PROTOCOL_ID: &str = "DAS";


/*
DASNode --> order of operations
    1. Discovery Protocol                 [X]  
    2. Overlay Protocol                   [X]
        - DAS Overlay Network             [ ]    (Subprotocol) 
        - Secure DAS Overlay Network      [ ]    (Subprotocol) 
    4. Samples                            [ ]
    5. Handled_ids ???                    [ ]

    ***To facilitate communication we must create message processing to allow for manipulation of each node's state.
*/
/*
    Goals: 
        - Create DASNodes that contain servers for each protocol the DASNode supports.  
        - Pass information back and forth between these created nodes via overlay 
               "The request is TALKREQ [ req-id, protocol, data ]" <-- https://github.com/ethereum/devp2p/issues/156
        - Design the code to be easily understandable (educational resource) 

    Answered Questions:
        - Each node has a protocol struct that stores its own information related to that protocol.  
          Each node has a service that interacts with that node's protocol struct.
        - The protocol structs within DASNode are wrapped in Arc<> purely for prototyping purposes!
          Allows the structs to be updated in both event loop routine and in simulation starting function

    Notes:
        - "To process connections concurrently, a new task is spawned for each inbound connection. The connection is processed on this task."
        - Event streams are stored within the main function, not within a data structure
        - E&T as to why they're passing around messages with uTP https://hackmd.io/@timofey/SyqzhA4vo#712-Reliable-UDP-over-Discv5
        - Obtain the discv5 event stream so we can spawn a manager task for our _____________       
        - After instantiating the overlay service,  I'll need to pass messages to nodes to test the thing!

        Shared State:
            - Arc allows state to be referenced concurrently by many tasks and/or threads (aka sharing state) 
            - When you're shared state is complex (like the discovery struct), you'll want a task to manage the state and utilize message passing to operate on it
            - Throughout Tokio, the term "handle" is used to reference a value that *provides access* to some shared state
    
    Questions:
        1.  How do i send a message from Discv5's TalkReq/Resp?  
            Does it have to be through the overlay?  Or is it accessible at the disv5 protocol
        2.  Are the different tasks that request to change the node's protocol struct the reason why node's protocol struct needs to have Arc<> fields?
            (Reword this question when you understand things better)
        3.  When is it super important for us to start running asyncronous code within the *main* function?  Aka asyncronysity BETWEEN nodes
*/



#[tokio::main]
async fn main() {
    //============================ 
    //   Part 1:  Node Creation
    //============================ 
    let mut nodes = Vec::new();
    
    // Instantiates protocol structs and message processing within each node
    for i in 0..NUMBER_OF_NODES {
        let (starter_node, 
            mut overlay_service, 
            utp_events_tx, 
            utp_listener_rx
        ) = create_node(i as u16).await;
      
        let mut event_str = ReceiverStream::new(starter_node.discovery.discv5.event_stream().await.unwrap());

        // This doesn't feel clean... But might be the best we can do      :P 
        let node = starter_node.clone(); 
        nodes.push(starter_node);

        /*
            This is the server side of our node! Instantiates task manager to continually process ALL messages for each node.
            Later, create task manager(s?) to handle the two protocols discv5 and overlay.  See Trin: https://github.com/ethereum/trin/blob/master/trin-core/src/portalnet/discovery.rs#L174
        */
        tokio::spawn(async move {
            loop {
                select! {
                    // Discv5: 
                    //      Implement discv5 message processing used in DAS Prototype.
                    Some(event) = event_str.next() => {
                        let chan = format!("{i} {}", node.discovery.discv5.local_enr().node_id().to_string());
                        match event {
                            Discv5Event::TalkRequest(req) => {
                                debug!("Stream {}: Talk request received", chan);
                                // msg_counter.send(MsgCountCmd::Increment);
                                // clone_all!(node, opts, enr_to_libp2p, node_ids, utp_events_tx);
                                let node = node.clone(); 
                                let utp_events_tx = utp_events_tx.clone();
                                tokio::spawn(async move {
                                    let protocol = ProtocolId::from_str(&hex::encode_upper(req.protocol())).unwrap();

                                    if protocol == ProtocolId::Utp {
                                        utp_events_tx.send(req).unwrap();
                                        return;
                                    }

                                    if protocol == ProtocolId::Custom(DAS_PROTOCOL_ID.to_string()) {
                                        let talk_resp = match node.overlay.process_one_request(&req).await {
                                            Ok(response) => discv5_overlay::portalnet::types::messages::Message::from(response).into(),
                                            Err(err) => {
                                                error!("Node {chan} Error processing request: {err}");
                                                return;
                                            },
                                        };

                                        if let Err(err) = req.respond(talk_resp) {
                                            error!("Unable to respond to talk request: {}", err);
                                            return;
                                        }

                                        return;
                                    }
                                    // let resp = handle_talk_request(req.node_id().clone(), req.protocol(), req.body().to_vec(), node, opts, enr_to_libp2p, node_ids, i).await;
                                    // let resp = handle_talk_request(req.node_id().clone(), req.protocol(), req.body().to_vec(), node).await;
                                    // req.respond(resp);
                                });
                            },
                            _ => {}    
                        }
                    },
                    // Overlay:  
                    //      Add other overlay message types (Line 324 of Model DAS) 
                    Some(command) = overlay_service.command_rx.recv() => {
                        match command {
                            // Print something here bc idk if this is reacting
                            OverlayCommand::Request(request) => overlay_service.process_request(request), 
                            _ => {}    
                        }
                    }
                }
            } 
        });
        // nodes.push(node);
    }
    
    // Populates our nodes' routing tables.   
    for i in 0..NUMBER_OF_NODES {
        populate_routing_table(i, nodes.clone());
    }


    //================================ 
    //   Part 2: Node Communication
    //================================ 

    // Create simple communication between nodes!  How can I see when a node processes the incoming ping? 
    let result = nodes[1].overlay.send_ping(nodes[2].overlay.local_enr());



    // Shows that the Discv5 and Overlay protocols within a node are instantiated!
    println!("Our node's enr according to discovery protocol: {:?}", nodes[2].discovery.local_enr());
    println!("\n"); 
    println!("Our node's enr according to overlay protocol: {:?}", nodes[2].overlay.local_enr());
    println!("\n"); 
    println!("Subprotocol: {:?}", nodes[2].overlay.protocol());
}


async fn create_node(i: u16) -> (
        DASNode,
        OverlayService<DASContentKey, XorMetric, DASValidator, MemoryContentStore>,
        UnboundedSender<TalkRequest>,
        UnboundedReceiver<UtpListenerEvent>
    ) {
    // 1. Discovery Protocol 
    let discovery = discovery::create_discovery(i).await;
   
    // Create uTP channel for overlay messaging.  What's the deal with this vs the overlay? 
    let ( utp_events_tx, 
            utp_listener_tx, mut utp_listener_rx, 
            mut utp_listener,
    ) = UtpListener::new(discovery.clone());

    // Starts the main uTP service used to listen and handle all uTP connections and streams.
    // The utp listener accepts inbound uTP sockets (first thing).  
    // Where should the listener be started + stored?  Investigate details of UTPListener.start()
    tokio::spawn(async move { utp_listener.start().await });
    
    // 2. Instantiates our Overlay Protocol.  Return our overlay and overlay service! (overlay goes inside DASNode) 
    let (overlay, overlay_service) = overlay::create_overlay(discovery.clone(), utp_listener_tx).await;  

    //  Samples: TODO
    
    //  Handled_ids: TODO 


    // Creates node (Timofey creates node with utp_listener_tx) 
    let mut my_node = DASNode::new(discovery, overlay);
    
    // utp_events_tx and utp_listener_rx are used within each node's message processing.
    (
        my_node,
        overlay_service,
        utp_events_tx,
        utp_listener_rx
    ) 
}


// Adds nodes from within the simulation to routing tables.  
fn populate_routing_table(local_index: usize, mut nodes: Vec<DASNode>) {
    // Number of peers a node adds to their routing table 
    let mut n = 3;
    let mut used_indexes = Vec::new();

    while n != 0 {
        let mut invalid_index = false; 
        let mut rng = rand::thread_rng();
        let rand = rng.gen_range(0usize..NUMBER_OF_NODES);

        // Makes sure we aren't duplicating nodes within our routing table 
        for i in 0..used_indexes.len() {
            if rand == used_indexes[i] || rand == local_index {
                invalid_index = true; 
            }
        } 
        if invalid_index == false {
            match nodes[local_index].discovery.discv5.add_enr(nodes[rand].discovery.discv5.local_enr().clone()) {
                Ok(_) => {
                    used_indexes.push(rand);
                    n -= 1;
                },
                Err(_) => continue,
            }
        }
    }
}



pub fn run_nodes() {
    crate::das_node::main::main();
}
