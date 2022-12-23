#![allow(unused)]
use discv5::{
    Discv5,
    Discv5ConfigBuilder, 
    enr, 
    enr::CombinedKey,
};
use discv5_overlay::portalnet::discovery::Discovery;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

mod node_struct;
use node_struct::DASNode;



/*
    Arc<T>'s, aka Atomic Reference Counted types, are smart pointers that allow for 
    a single value to have multiple owners.  ***These multiple owners can be shared across multiple threads***
*/

// Spins up tasks that create nodes
#[tokio::main]
async fn main() {
    let mut handles = Vec::new();
    for i in 0..4 {
        let mut my_node = DASNode::new();
        let handle = tokio::spawn(async move {
            spin_up_node(i, my_node).await;
        });
        handles.push(handle);
    } 
    
    for handle in handles {
        handle.await.unwrap();
    }
}


async fn spin_up_node(i: i32, mut node: DASNode) {
    instantiate_server(node, i).await;
}


//  Why do Eric and Timofey use a for loop to spin up servers? 
async fn instantiate_server(mut node: DASNode, i: i32) {
    let port_start = (10 + i) as u16;       
    println!("Create a server for each node");
    
    let discv5 = create_discv5_server(port_start).await;    
    // let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 
}

//  Instantiates the main discv5 service struct.  This provides the user-level API 
//  for performing queries and interacting with the underlying service. 
async fn create_discv5_server(port_start: u16) -> Discv5{
    let port_udp = String::from("9000"); 
    let listen_ip = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap(); 
    
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = {
        let mut builder = enr::EnrBuilder::new("v4");
        builder.ip4(listen_ip);
        // builder.udp4(port_start as u16 + i as u16);
        builder.udp4(port_start);
        builder.build(&enr_key).unwrap()
    }; 
    println!("enr --> {}", enr);
   
    // default configuration
    let mut config_builder = Discv5ConfigBuilder::default();
    config_builder.request_retries(10);
    config_builder.filter_max_nodes_per_ip(None);
    config_builder.request_timeout(Duration::from_secs(60));
    config_builder.query_timeout(Duration::from_secs(60)); 
    let config = config_builder.build();
    
    // Returns a discv5 service struct 
    let discv5 = Discv5::new(enr, enr_key, config).unwrap();
    
    // TO DO!!! 
    // set_topolygy(discv5);
    
    discv5
}

// TO DO: Set topology for service, THEN return discv5    
// fn set_topolygy(mut discv5_server) -> Discv5 {
// };