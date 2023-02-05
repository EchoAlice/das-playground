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
use std::env;
use std::str::FromStr;
use tokio::{
    select, 
    sync::mpsc::{UnboundedReceiver, UnboundedSender}
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info};
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
    Goals: 
        - Create DASNodes that contain the protocols and subprotocols listed above.  
        - Pass information back and forth between these created nodes via overlay 
               "The request is TALKREQ [ req-id, protocol, data ]" <-- https://github.com/ethereum/devp2p/issues/156
        - Design the code to be easily understandable (educational resource) 

    Questions:
        What all do I need to do to make custom overlay networks?

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
      
        // Idk if we can obtain the event stream using discovery.start() instead of discv5.start() 
        let mut event_str = ReceiverStream::new(starter_node.discovery.discv5.event_stream().await.unwrap());

        // It doesn't feel clean copying the entire node to pass info into our task manager  :P 
        let node = starter_node.clone(); 
        nodes.push(starter_node);

        /* 
        Big Question:
            How should I handle messages from different subnetworks? 
            Can I just spawn another task within our event to handle the secure overlay?  
            Or should i create a proxy to sit in between all TalkReqs and DAS + Secure DAS Subnetworks 
            
            See Trin: https://github.com/ethereum/trin/blob/master/trin-core/src/portalnet/discovery.rs#L174
         */
        
         // This is the server side of our nodes. Instantiates task manager to continually process ALL messages for each node.
        tokio::spawn(async move {
            loop {
                select! {
                    // Discv5: 
                    //      Implement discv5 message processing used in DAS Prototype.
                    Some(event) = event_str.next() => {
                        let chan = format!("{i} {}", node.discovery.discv5.local_enr().node_id().to_string());
                        match event {
                            Discv5Event::TalkRequest(req) => {
                                println!("Our node's enr inside of task: {:?}", node.discovery.local_enr());
                                println!("Stream {}: Discv5 TalkReq received", chan);  
                                
                                let node = node.clone(); 
                                tokio::spawn(async move {
                                    let protocol = ProtocolId::from_str(&hex::encode_upper(req.protocol())).unwrap();

                                    if protocol == ProtocolId::Custom(DAS_PROTOCOL_ID.to_string()) {
                                        println!("Enters DAS Protocol");  
                                        let talk_resp = match node.overlay.process_one_request(&req).await {
                                            Ok(response) => discv5_overlay::portalnet::types::messages::Message::from(response).into(),
                                            Err(err) => {
                                                error!("Node {chan} Error processing request: {err}");
                                                return;
                                            },
                                        };

                                        if let Err(err) = req.respond(talk_resp) {
                                            println!("Error");  
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
    }
    
    // Populates our nodes' routing tables.   
    for i in 0..NUMBER_OF_NODES {
        populate_routing_table(i, nodes.clone());
    }


    //================================ 
    //   Part 2: Node Communication
    //================================ 

    // Creates simple communication between nodes 
    let result = nodes[1].overlay.send_ping(nodes[2].overlay.local_enr());
    result.await;
    
    
    // ***We can no longer run any code after awaiting our tasks.  The tasks are designed to never stop running! 
    //================================ 
    //         Sanity Check 
    //================================ 
    let discovery_enr = nodes[2].discovery.local_enr();
    let overlay_enr = nodes[2].overlay.local_enr(); 
    if discovery_enr == overlay_enr {
        println!("Discovery and overlay protocol *structs* are instantiated")
    } else {
        println!("Discovery and overlay protocol *structs* are not instantiated")
    };
    println!("Discovery Enr: {}", nodes[2].discovery.local_enr());
}






async fn create_node(i: u16) -> (
        DASNode,
        OverlayService<DASContentKey, XorMetric, DASValidator, MemoryContentStore>,
        UnboundedSender<TalkRequest>,
        UnboundedReceiver<UtpListenerEvent>
    ) {
    // 1. Discovery Protocol 
    let discovery = discovery::create_discovery(i).await;
  
    // Create uTP channel for overlay messaging
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
