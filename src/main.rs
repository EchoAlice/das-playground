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

mod node_struct;
use node_struct::DASNode;




/*
    Arc<T>'s, aka Atomic Reference Counted types, are smart pointers that allow for 
    a single value to have multiple owners.  ***These multiple owners can be shared across multiple threads/tasks***
*/

// Spins up tasks that create nodes
#[tokio::main]
async fn main() {
    
    let mut handles = Vec::new();
    for i in 0..4 {
        let mut my_node = DASNode::new();
        let handle = tokio::spawn(async move {
            let i = i as u16; 
            println!("Create a node");
            println!("-----------------------------------"); 
            create_node(i, my_node).await;
        });
        handles.push(handle);
    } 
    
    for handle in handles {
        handle.await.unwrap();
    }
}


// I'm just wanting to create the DASNode.  Worry about instantiating the overlay service later
/*
DASNode
    1. Discovery Protocol
    2. Libp2p Service
    3. Samples
    4. Handled_ids
    5. Overlay Protocol 
*/
async fn create_node(i: u16, mut node: DASNode) {
    // 1. Discovery Protocol
    let discv5 = create_discv5_server(i).await;
    // println!("create_node DiscV5 Server: {:?}", discv5);

    // Base Node Discovery Protocol v5 layer
    let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 
    println!("Discovery: {:?}", discovery);
    println!("the node's service channel: {}", discovery.discv5.service_channel);
    
    //Access each node's peers in their routing tables 
}


// The main Discv5 Service struct. This provides the user-level API for performing queries and
// interacting with the underlying service.
async fn create_discv5_server(i: u16) -> Discv5 {
    let port_start = 9000 + i;       
    let listen_ip = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap(); 
    let listen_addr = format!("{}:{}", listen_ip, port_start + i)
        .parse::<SocketAddr>()
        .unwrap(); 
    // Looks like only even numbers.  I bet the "i" gets increased within the function more than once (or is within a loop?) 
    println!("Listen address: {}", listen_addr);

    // Generates new enrs for each id.  Is this ok? 
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = {
        let mut builder = enr::EnrBuilder::new("v4");
        builder.ip4(listen_ip);
        builder.udp4(port_start + i);
        // builder.udp4(port_start);
        builder.build(&enr_key).unwrap()
    }; 
   
    // Discv5 configureation 
    let mut config_builder = Discv5ConfigBuilder::default();
    config_builder.request_retries(10);
    config_builder.filter_max_nodes_per_ip(None);
    config_builder.request_timeout(Duration::from_secs(60));
    config_builder.query_timeout(Duration::from_secs(60)); 
    let config = config_builder.build();
    
    // Is discv5 a server? 
    // Is it the object a node references to manipulate it's own understanding of the network (also how it interacts with the world)
    let mut discv5 = Discv5::new(enr, enr_key, config).unwrap();
    // let mut discv5 = start_server(&mut discv5); 
    let ip4 = discv5.local_enr().ip4().unwrap();
    let udp4 = discv5.local_enr().udp4().unwrap();
 
    //
    discv5.start(listen_addr)
    // discv5.start(format!("{}:{}", ip4, udp4).parse().unwrap())
        .await
        .unwrap();
    
    discv5
}















// async fn start_server(discv5: &mut Discv5) -> Discv5 {
// }

// fn populate_table(mut discv5_server) -> Discv5 {
// };