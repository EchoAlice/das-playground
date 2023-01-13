#![allow(unused)]
use tokio_stream::wrappers::ReceiverStream;

use crate::das_node::{config::NUMBER_OF_NODES, discovery ,node_struct};
// use crate::das_node::node_struct;
// use crate::das_node::discovery;

use super::node_struct::DASNode;


/*
    Goal: 
        Create DASNodes that contain servers for each protocol the DASNode supports.  


    Concepts to implement:
        - Add nodes to our routing tables 
        - Play with the event stream 

    Notes:
        "To process connections concurrently, a new task is spawned for each inbound connection. The connection is processed on this task."

    Questions:
        1.  Why is the discovery protocol wrapped in Arc?  
            Maybe:  It allows for persisting data for all sockets
        2.  How do I add more peers to my table?  Can I ask bootnodes in this simulation???
*/
#[tokio::main]
async fn main() {
    let mut node_futures = Vec::new(); 
    let mut nodes = Vec::new();
    let mut event_streams = Vec::new(); 

    // Create our DASNodes 
    for i in 0..NUMBER_OF_NODES {
        let i = i as u16; 
        let my_node = create_node(i);
        node_futures.push(my_node);
    }
    for node in node_futures {
        let out = node.await; 
        nodes.push(out); 
    }

    // Create event streams vector so we can access streams while still being able to clone our DASNode 
    for i in 0..NUMBER_OF_NODES {
        let mut event_str = ReceiverStream::new(nodes[i].discovery.discv5.event_stream().await.unwrap());
        event_streams.push(event_str); 
        println!("Our node's enr: {:?}", nodes[i].discovery);
        
        // Add peers to table
        set_topology(i, nodes.clone()); 
        
        println!("Discv5 peers: {:?}", nodes[i].discovery.discv5.table_entries_enr());
        println!("\n");
    
    }
    // Instantiate event loops for discv5, overlay, and libp2p logic
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
    
    // 2. Overlay Protocol
    
    // 3. Libp2p Service

    // 4. Samples

    // 5. Handled Ids



    // Creates node.  Add fields as project progresses  
    let mut my_node = DASNode::new(discovery);
    my_node 
}


/*
    ET's set_topology() and Brechy's find_peers() both populate our routing tables.
    But they're doing two different things:  
        - set_topology() is adding nodes from within the simulation to the routing tables
        - find_peers() is asking the real deal network for random nodes
*/
// Pass all nodes to the function to implement function like E+T do
// Adds nodes from within the simulation to the routing tables
fn set_topology(i: usize, mut nodes: Vec<DASNode>) {
    // What is this variable? 
    let mut n = 3;

    // Make sure I can modify the node
    
    // // Get random value    
    // while n != 0 {
    //     let i = rng.gen_range(0usize..discv5_servers.len() - 1);

    //     match s.add_enr(discv5_servers[i].local_enr().clone()) {
    //         Ok(_) => n -= 1,
    //         Err(_) => continue,
    //     }
    // }
}



pub fn run_nodes() {
    crate::das_node::main::main();
}
