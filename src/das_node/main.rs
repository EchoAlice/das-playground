#![allow(unused)]
use discv5_overlay::utp::stream::{UtpListener, UtpListenerRequest};
use rand::Rng;
use tokio_stream::wrappers::ReceiverStream;

use crate::das_node::overlay;
use super::node_struct::DASNode;
use crate::das_node::{
    config::NUMBER_OF_NODES,
    discovery,
    node_struct,
};



/*
    Goals: 
        - Create DASNodes that contain servers for each protocol the DASNode supports.  
        - Pass information back and for between these created nodes via the overlay and/or discv5's TalkReq/TalkResp
               "The request is TALKREQ [ req-id, protocol, data ]" <-- https://github.com/ethereum/devp2p/issues/156

    Notes:
        - "To process connections concurrently, a new task is spawned for each inbound connection. The connection is processed on this task."
        - Event streams are stored within the main function, not within a data structure
        - E&T as to why they're passing around messages with uTP https://hackmd.io/@timofey/SyqzhA4vo#712-Reliable-UDP-over-Discv5
        - Obtain the discv5 event stream so we can spawn a manager task for our _____________       

    Questions:
        1.  Why is the discovery protocol wrapped in Arc?  
            Maybe:  It allows for persisting data for all sockets
        2.  How do i send a message from Discv5's TalkReq/Resp?  
            Does it have to be through the overlay?  Or is it accessible at the disv5 protocol
*/
#[tokio::main]
async fn main() {
    // Create our DASNodes 
    let mut node_futures = Vec::new(); 
    let mut nodes = Vec::new();
    for i in 0..NUMBER_OF_NODES {
        let i = i as u16; 
        let my_node = create_node(i);
        node_futures.push(my_node);
    }
    for node in node_futures {
        let out = node.await; 
        nodes.push(out); 
    }
   
    // Create event streams and add nodes to routing tables
    let mut event_streams = Vec::new(); 
    for i in 0..NUMBER_OF_NODES {
        // Listen for events! 
        let mut event_str = ReceiverStream::new(nodes[i].discovery.discv5.event_stream().await.unwrap());
        event_streams.push(event_str); 
       
        // Populate our nodes' routing tables
        populate_routing_table(i, nodes.clone());
       
        // Create uTP channel for overlay messaging 
        let ( // Channel to process uTP TalkReq packets from main protal event handler
              utp_events_tx, 
              // Channel to process portal overlay requests
              utp_listener_tx, mut utp_listener_rx, 
              // Main uTP service used to listen and handle all uTP connections and streams
              mut utp_listener, 
        ) = UtpListener::new(nodes[i].discovery.clone());
        tokio::spawn(async move { utp_listener.start().await });
        
        /*
        Spawn manager task to handle overlay messages from our utp channel.  For context -->  https://tokio.rs/tokio/tutorial/channels
            - Task manages our client resource and is a channel that acts as a buffer 
            - Requires listening to your event stream!
            - Where should I place the proxy used to delegate different overlays' request 
                handling logic?  (Trin)
        */


    }

    // Example node
    println!("Our node's discovery protocol: {:?}", nodes[3].discovery);
    println!("Event stream: {:?}", event_streams[3]);
    println!("\n");



}


/*
DASNode --> order of operations
    1. Discovery Protocol   (Check)  
    2. Overlay Protocol 
    3. Libp2p Service
    4. Samples
    5. Handled_ids
*/
async fn create_node(i: u16) -> DASNode {
    // 1. Discovery Protocol 
    let discovery = discovery::create_discovery(i).await;
    
    // 2. Overlay Protocol.  
    // let overlay = overlay::create_overlay(discovery.clone()).await;  

    // 3. Libp2p Service

    // 4. Samples

    // 5. Handled Ids


    // Creates node.  Add fields as project progresses  
    // let mut my_node = DASNode::new(discovery, overlay);
    let mut my_node = DASNode::new(discovery);
    my_node 
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
