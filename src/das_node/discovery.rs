use discv5::{
    Discv5,
    Discv5ConfigBuilder,
    Discv5Event, 
    enr,
    Enr, 
    enr::CombinedKey,
};
use discv5_overlay::portalnet::discovery::Discovery;
use std::{
    net::Ipv4Addr,
    str::FromStr,
    sync::Arc,
    time::Duration,
};


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
    // A node using the discovery protocol needs to provide an IP address and UDP port to have its record relayed in the DHT
    // Do we need to have incremental IP addresses?
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
    
    // Discv5 configuration.  Not sure why these specific parameters were chosen, but I'm following E+T 
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

    // Initializes the discv5 service.  Starts the required tasks and begins listening on a given UDP SocketAddr
    discv5.start(format!("{}:{}", ip4, udp4).parse().unwrap())
        .await
        .unwrap();
   
    // Initializes our protocol (How Timofey and Eric do things).  Portal Config? 
    let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 
    println!("Discovery in use: {:?}", discovery);

    /*
        Initialize discovery.start() instead of discv5.start().  It calls discv5.start() AND sets up the proxy to sit between different overlay protocols and our Discv5. 
        We need to create the logic that passes TalkReqs to the correct subnetwork.  See our events.rs 
        
        Each node is using  IpV4 Socket: Some(75.130.204.11:4242) + listen_socket: 0.0.0.0:4242.  Does this need to change this?  
    */
    // let discovery = Discovery::new(Default::default()); 
    // let discovery = Arc::new(discovery.unwrap());
    // println!("Discovery we want: {:?}", discovery);
   
    // Talk Req rx will allow us to listen to all overlay requests!
    // let talk_req_rx = discovery.start().await.unwrap(); 

    discovery
}