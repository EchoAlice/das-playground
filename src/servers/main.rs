#![allow(unused)]
use discv5::{
    Discv5,
    Discv5ConfigBuilder, 
    enr, 
    enr::CombinedKey,
};
use discv5_overlay::portalnet::discovery::Discovery;
// use std::fmt::Debug;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

// mod node_struct;
use crate::servers::node_struct::DASNode;

// use node_struct::DASNode;




/*
    Arc<T>'s, aka Atomic Reference Counted types, are smart pointers that allow for 
    a single value to have multiple owners.  ***These multiple owners can be shared across multiple threads/tasks***
*/

// Spins up tasks that create nodes
#[tokio::main]
async fn main() {


    // -------------------------------------------------------------------------------------------------------------------- 
    // Create nodes outside of tasks.  You'll be able to manpulate the nodes easier.  
    // Tasks should be utilized somewhere else within the program
    for i in 0..4 {
        let i = i as u16; 
        let my_node = create_node(i).await;
        println!("My node: {:?}", my_node);
    }

    
// -------------------------------------------------------------------------------------------------------------------- 
    let mut handles = Vec::new();
    // let mut nodes = Vec::new(); 
    for i in 0..4 {
        let handle = tokio::spawn(async move {
            let i = i as u16; 
            let my_node = create_node(i).await;
            println!("Node: {:?}", my_node);
            println!("\n");
            
            // Ask a node to send their routing table info! Print tables 
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    // How can I store the nodes and interact with them inside of the main function??? 
    
    // //  Implement debug within data structure so we can print this thing out!
    // for node in nodes {
    //     println!("Node's ENR: {}", node.discovery);
    // }
}






/*
          Add debug trait to DASNode and discv5 server structs!
             Look back at what Timofey said about doing so!
 */

 /*
DASNode
    1. Discovery Protocol   (Check)  
    2. Libp2p Service
    3. Samples
    4. Handled_ids
    5. Overlay Protocol 
*/
async fn create_node(i: u16) -> DASNode {
    
    // 1. Discovery Protocol
    let discv5 = create_discv5_server(i).await;
    let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 

    // Creates node.  Add to fields as project progresses  
    let mut my_node = DASNode::new(discovery);
    
    //  Implement debug within data structure so we can print this thing out!
    // println!("Our node's neighbors: {:?}", discovery.discv5.kbuckets());
    
    my_node 
}


// The main Discv5 Service struct. 
// This provides the user-level API for performing queries and interacting with the underlying service.
async fn create_discv5_server(i: u16) -> Discv5 {
    
    // Should UDP ports increment? 
    let port_start = 9000 + i;       
    let listen_ip = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap(); 

    // Generates a node's random enr key and new enr.  *Base the secp256k1 on our nodes' public keys*
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = {
        let mut builder = enr::EnrBuilder::new("v4");
        builder.ip4(listen_ip);
        builder.udp4(port_start);
        builder.build(&enr_key).unwrap()
    }; 
   
    // Discv5 configureation 
    let mut config_builder = Discv5ConfigBuilder::default();
    config_builder.request_retries(10);
    config_builder.filter_max_nodes_per_ip(None);
    config_builder.request_timeout(Duration::from_secs(60));
    config_builder.query_timeout(Duration::from_secs(60)); 
    let config = config_builder.build();
    
    // Is it the object a node references to manipulate it's own understanding of the network (also how it interacts with the world)
    let mut discv5 = Discv5::new(enr, enr_key, config).unwrap();
    let ip4 = discv5.local_enr().ip4().unwrap();
    let udp4 = discv5.local_enr().udp4().unwrap();
    
    discv5.start(format!("{}:{}", ip4, udp4).parse().unwrap())
        .await
        .unwrap();
    
    discv5
}


pub fn run_servers() {
    crate::servers::main::main();
    println!("Runs servers module")
}



// fn populate_table(mut discv5_server) -> Discv5 {
// };