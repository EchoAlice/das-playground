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
Notes:
    1.  E & T add discv5 peers within set_topology
    2.  Understand Rust's mapping syntax (need this for node_ids and enrs)

Questions:
    1.  Why is the discovery protocol wrapped in Arc?  
        Maybe:  It allows for persisting data for all sockets
    2.  How do I add more peers to my table?  Can I ask bootnodes in this simulation???
*/

// This function creates the discv5 service + main discovery protocol for EACH NODE! 
pub async fn create_discovery(i: u16) -> Arc<Discovery> {
    // Each task needs to have its own port... OS things  :P
    // Listeing port and address.  Why do we need both? 
    let port_start = 9000 + i;
    let listen_ip = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap(); 

    // Generates local node's random enr key and new enr.  *Base the secp256k1 on our node's public key*
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
   
    // Construct the discv5 server
    let mut discv5 = Discv5::new(enr, enr_key, config).unwrap();
    let ip4 = discv5.local_enr().ip4().unwrap();
    let udp4 = discv5.local_enr().udp4().unwrap();
   
    // Currently:  Adds bootnode like Brechy  
    // Later:  Implement routing tables like model-das in set_topology()
    let ef_bootnode_enr = Enr::from_str(BOOTNODE).unwrap();
    discv5.add_enr(ef_bootnode_enr).expect("bootnode error");    



    /*
    ET allow each DASNode to run a task which contains a loop that (continuously?) processes events from the eventstream 
    Why does their outer for loop use "for __ in discv5_servers"?  Feels like not the best design choice.  Ask later

        for (i, discv5) in discv5_servers.into_iter().enumerate() {
            tokio::spawn {
                loop{} 
            }
        }
    */

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

    // Initialize our protocol
    let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 
    
    discovery
}