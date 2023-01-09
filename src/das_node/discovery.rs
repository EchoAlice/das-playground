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



// Summarize what this thing is!
pub async fn create_discovery(i: u16) -> Arc<Discovery> {
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