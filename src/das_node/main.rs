#![allow(unused)]
use discv5::TalkRequest;
use discv5_overlay::{
    portalnet::{
        overlay_service::{
            OverlayCommand,
            OverlayService
        },
        storage::MemoryContentStore,
        types::distance::XorMetric, 
    },
    utp::stream::{UtpListener, UtpListenerRequest, UtpListenerEvent}
};
use rand::Rng;
use tokio::{
    select, 
    sync::mpsc::{UnboundedReceiver, UnboundedSender}
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::debug;

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
/*
DASNode --> order of operations
    1. Discovery Protocol                 [X]  
    2. Overlay Protocol                   [X]
        - DAS Overlay Network             [ ]    (Subprotocol) 
        - Secure DAS Overlay Network      [ ]    (Subprotocol) 
    4. Samples                            [ ]
    5. Handled_ids ???                    [ ]
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
// Using multiple for loops to break up the major steps
#[tokio::main]
async fn main() {
    
    let mut nodes = Vec::new();
    let mut overlay_services = Vec::new(); 
    let mut utp_events_txs = Vec::new(); 
    let mut utp_listener_rxs = Vec::new(); 
    
    // Creates our DASNodes which implement the discv5 and overlay protocols
    // Grabs utp communication channels instantiated from within create_node().  We need these for overlay communication!
    for i in 0..NUMBER_OF_NODES {
        // Is it possible to/should I not call await? And gather info (joinhandles await) in a seperate for loop? 
        // I believe that this is no longer creating nodes concurrently.  Is this true? Is this a bad thing? 
        let (node, 
            overlay_service, 
            utp_events_tx, 
            utp_listener_rx
        ) = create_node(i as u16).await;
        
        /*
            Maybe move other code from main for loops in here...?
            
            But I'm unsure of how our asyncronous code plays into everything... 
            For now, allow all nodes to be created within one loop, then initialize our message processing in another loop
        */
       

        // Gathers information for each node in the network.  If we move code inside same function, we can get rid of these vectors! 
        nodes.push(node);
        overlay_services.push(overlay_service);
        utp_events_txs.push(utp_events_tx);
        utp_listener_rxs.push(utp_listener_rx)
    }


    // Obtains event streams.  Populates our nodes' routing tables   
    let mut event_streams = Vec::new();
    for i in 0..NUMBER_OF_NODES {
        let mut event_str = ReceiverStream::new(nodes[i].discovery.discv5.event_stream().await.unwrap());
        event_streams.push(event_str); 
        
        populate_routing_table(i, nodes.clone());
        
        // Can we check when nodes are added to the routing table via event streams?  Could be good practice
    }

    /*
        So far, our program runs syncronously... All good (maybe?), 
        but once we've gotten our nodes set up, we want for communication between nodes to happen concurrently (simulates communication within a network).  

        To set up communication between nodes, we need to:
            1.  Spawn a task to process messages, sent internally from protocol struct to service, to manipulate state of our protocols (data stores?). 
                Do we only need one message processing task per node?  
                
                For context on manipulating shared state through message passing -->  https://tokio.rs/tokio/tutorial/channels
            
            2.  Make some sort of proxy to handle the different subnetworks messaging (What Trin does!)  
                See here: https://github.com/ethereum/trin/blob/master/trin-core/src/portalnet/discovery.rs#L174

        Once message processing tasks are set up within nodes, we can connect nodes to one another.
    */
    
    //  Instantiates task manager to process ALL messages for each node.  
    //  For now, we're ony dealing with overlay requests
    for i in 0..NUMBER_OF_NODES {
        // Is the correct service always matching to the correct node?  
        let mut overlay_service = overlay_services.remove(0);
          
        tokio::spawn(async move {
        
        /*
            Continually processes inbound overlay commands
            Our overlay_service.command_rx is doing the same thing as Tokio's TCP listener in tutuorial --> https://tokio.rs/tokio/tutorial/spawning
            
            Sockets are accepted in a loop.  Each socket is processed then closed.  (Socket might not be the right word)
        */
            loop {
                select! {
                    Some(command) = overlay_service.command_rx.recv() => {
                        match command {
                            OverlayCommand::Request(request) => overlay_service.process_request(request),
                            // **See line 324 of Model DAS to implement other OverlayCommand!**  
                            _ => {}    
                        }
                    }
                }
            } 
        });
    }
    
    // Shows that the Discv5 and Overlay protocols within a node are instantiated!
    println!("Our node's enr according to discovery protocol: {:?}", nodes[2].discovery.local_enr());
    println!("Our node's enr according to overlay protocol: {:?}", nodes[2].overlay.local_enr());
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
