use discv5::{
    Discv5,
    Discv5ConfigBuilder, 
    enr, 
    enr::CombinedKey,
};
use discv5_overlay::portalnet::discovery::Discovery;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

/*
Notes:
    1.  Add bootnodes to the ENR!
    2.  Event stream- Notifies subscriptor when something has occured within the server
    3.  E & T add discv5 peers within set_topology

Questions:
    1.  Why is the discovery protocol wrapped in Arc?  
        Maybe:  It allows for persisting data for all sockets
    2.  Understand Rust's mapping syntax 
*/

// This function creates the discv5 service + main discovery protocol for each node 
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
   
    // Add bootnode!  Look into DAS Prototype for same functionality... Compare
    // let ef_bootnode_enr = Enr::from_str(BOOTNODE).unwrap();
    // discv5.add_enr(ef_bootnode_enr).expect("bootnode error");    

    // Start the discv5 server
    discv5.start(format!("{}:{}", ip4, udp4).parse().unwrap())
        .await
        .unwrap();
  
    // Arc allows state to be referenced concurrently by many tasks and/or threads! 
    // Think:  A lot of nodes will be writing to and reading from a specific node's server
    let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 
    
    discovery
}