#![allow(unused)]

use discv5::{
    Discv5,
    Discv5ConfigBuilder, 
    enr, 
    enr::CombinedKey,
};
use discv5_overlay::portalnet::discovery::Discovery;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

use crate::das_node::node_struct;
use crate::servers::node_struct::DASNode;



/*
    Create nodes that contain servers for each protocol the DASNode supports.  Each server has a task!
*/
#[tokio::main]
async fn main() {
    let mut node_futures = Vec::new(); 
    for i in 0..4 {
        let i = i as u16; 
        let my_node = create_node(i);
        node_futures.push(my_node);
    }
    
    // Makes sure all nodes have been fully instantiated.  
    let mut nodes = Vec::new();
    for node in node_futures {
        let out = node.await; 
        nodes.push(out); 
        
    }

    /*
        This is where we can start manipulating nodes!
        Figure out how to access a nodes peers within Discv5 
    */
    for node in nodes {
        println!("Node: {:?}", node);
        println!("\n");
    }
}


 /*
DASNode
    1. Discovery Protocol   (Check)  
    2. Libp2p Service
    3. Samples
    4. Handled_ids
    5. Overlay Protocol 


???
impl DASNode {
    discovery:
    utp_listener_tx:
    libp2p:
}
*/
async fn create_node(i: u16) -> DASNode {
    // 1. Discovery Protocol 
    let discovery = create_discovery(i).await;
    
    // Creates node.  Add to fields as project progresses  
    let mut my_node = DASNode::new(discovery);
    
    my_node 
}


// Summarize what this thing is!
async fn create_discovery(i: u16) -> Arc<Discovery> {
    // Each task needs to have its own port... OS things  :P 
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
    
    let mut discv5 = Discv5::new(enr, enr_key, config).unwrap();
    let ip4 = discv5.local_enr().ip4().unwrap();
    let udp4 = discv5.local_enr().udp4().unwrap();
   
    // Add bootnode.  Look into DAS Prototype for same functionality... Compare
    // let ef_bootnode_enr = Enr::from_str(BOOTNODE).unwrap();
    // discv5.add_enr(ef_bootnode_enr).expect("bootnode error");    

    discv5.start(format!("{}:{}", ip4, udp4).parse().unwrap())
        .await
        .unwrap();
    
        let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 
    
    discovery
}






pub fn run_nodes() {
    crate::das_node::main::main();
}