#![allow(unused)]
use discv5::{
    Discv5,
    Discv5ConfigBuilder,
    Discv5Event, 
    enr,
    Enr, 
    enr::CombinedKey,
};
use discv5_overlay::portalnet::discovery::Discovery;
use std::str::FromStr;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use crate::das_node::config::BOOTNODE;

/*
    The Node Discovery Protocol v5 (discv5) is the p2p network that Ethereum Nodes use
    to establish network connections with other nodes.  It acts as a database of all live nodes
    in the network, performing 3 main functions:
        1. Sampling the set of all live node
        2. Searching for nodes providing a certain service (topic advertisement) 
        3. Authoratative resolution of node records with a node's id
*/

// This function creates the discv5 service + main discovery protocol for a node! 
pub async fn create_discovery(i: u16) -> Arc<Discovery> {
    // Each task needs to have its own port... OS things  :P
    // Listeing port and address.  Why do we need both? 
    let port_start = 9000 + i;
    let listen_ip = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap(); 

    // Generates local node's random enr key and new enr.  *Base the secp256k1 on our node's public key*
    // There's a lot to talk about wrt ENR things.  Create a summary here soon 
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = {
        let mut builder = enr::EnrBuilder::new("v4");
        builder.ip4(listen_ip);
        builder.udp4(port_start);
        builder.build(&enr_key).unwrap()
    }; 
    
    // Discv5 configureation.  Not EXACTLY sure why these options were chosen, but I'm following E+T 
    let mut config_builder = Discv5ConfigBuilder::default();
    config_builder.request_retries(10);
    config_builder.filter_max_nodes_per_ip(None);
    config_builder.request_timeout(Duration::from_secs(60));
    config_builder.query_timeout(Duration::from_secs(60)); 
    let config = config_builder.build();
   
    // Construct the discv5 server
    let mut discv5 = Discv5::new(enr, enr_key, config).unwrap();
    let ip4 = discv5.local_enr().ip4().unwrap();
    let udp4 = discv5.local_enr().udp4().unwrap();
   
    // Bootnode functionality.  Might utilize later 
    // let ef_bootnode_enr = Enr::from_str(BOOTNODE).unwrap();
    // discv5.add_enr(ef_bootnode_enr).expect("bootnode error");    

    // Brechy adds event stream here.  E+T place it in the main app function because....  ___________

    // Start the discv5 server
    discv5.start(format!("{}:{}", ip4, udp4).parse().unwrap())
        .await
        .unwrap();

/*
    Shared State:
    - Arc allows state to be referenced concurrently by many tasks and/or threads (aka sharing state) 
    - When you're shared state is complex (like the discovery struct), you'll want a task to manage the state and
      utilize message passing to operate on it
    - What state within our Discovery data structure is needing to be shared?
    - Throughout Tokio, the term "handle" is used to reference a value that *provides access* to some shared state
*/
    
    // Initializes our protocol.  What's this Portal Config?
    let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 
    discovery
}