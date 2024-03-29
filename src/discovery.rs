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
    The Node Discovery Protocol v5 (discv5) is the UDP-based p2p network that Ethereum Nodes use
    to establish network connections with other nodes.  It acts as a database of all live nodes
    in the network, performing 3 main functions:
        1. Sampling the set of all live node
        2. Searching for nodes providing a certain service (topic advertisement) 
        3. Authoratative resolution of node records with a node's id

    Discv5 implements message passing via UDP transport!  
        This allows for fast ping/pong message between nodes (necessary to establish real connection).
        A more formal connection via TCP (or UTP wrt DAS Prototype + Trin?) is then established.
        
        "For discovery, where a node simply wants to make its presence known in order to then 
            establish a formal connection with a peer, UDP is sufficient"
*/

// Questions:
//      Why does our discv5 struct have no table entries?

// Creates discovery protocol struct + service for a node! 
pub async fn create_discovery(i: u16) -> Arc<Discovery> {
    // UDP port to find peers  +  IP address to connect to peers to have its record relayed in the DHT
    // I believe this is a client-side (ephemeral) port 
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
    
    // Discv5 configuration.  Following Eric and Timofey's lead
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
   
    // Initializes our protocol struct 
    let discovery = Arc::new(Discovery::new_raw(discv5, Default::default())); 
     
    discovery
}