use discv5_overlay::{
    portalnet::{
        discovery::Discovery,
        overlay::{
            OverlayConfig, 
            OverlayProtocol
        },
        overlay_service::OverlayService,
        storage::{DistanceFunction, MemoryContentStore},
        types::{
            distance::{Distance, XorMetric},
            messages::ProtocolId
        }
    },
    utp::stream::UtpListenerRequest,
};
use std::{
    sync::Arc,
    time::Duration
};
use tokio::sync::mpsc;

use crate::das_node::{
    content_key::{
        DASContentKey,
        DASValidator,
        SecureDASContentKey, 
        SecureDASValidator, 
        // ValidatorType, 
    } 
};

const DAS_PROTOCOL_ID: &str = "DAS";
const SECURE_DAS_PROTOCOL_ID: &str = "SECURE_DAS";


/*
    "The Overlay protocol is a layer on top of discv5 that handles all requests from the overlay networks
    (state, history etc.) and dispatch them to the discv5 protocol TalkReq. Each network should
    implement the overlay protocol and the overlay protocol is where we can encapsulate the logic for
    handling common network requests/responses."  
    -Trin repo


Note:
    - Currently modifying parameters and return values of the function with generics!
    - The Validator type what tells our OverlayProtocol::new( ) the type of overlay we're creating!
*/




/*
//-----------------------------
// GENERALIZE OVERLAY CREATION
//-----------------------------
//
// Passes in an enum as our validator type
// pub async fn create_overlay<TContentKey, TValidator>(discovery: Arc<Discovery>, protocol: ProtocolId, validator:Arc<ValidatorType>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) -> ( 

// This function definition feels right
pub async fn create_overlay<TContentKey, TValidator>(discovery: Arc<Discovery>, protocol: ProtocolId, validator:Arc<TValidator>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) -> ( 
    Arc<OverlayProtocol<TContentKey, XorMetric, TValidator, MemoryContentStore>>, 
    OverlayService<TContentKey, XorMetric, TValidator, MemoryContentStore>,
    ) { 
        let config = OverlayConfig {
        bootnode_enrs: discovery.clone().discv5.table_entries_enr(),
        ping_queue_interval: Some(Duration::from_secs(10000)),
        query_num_results: usize::MAX,
        query_timeout: Duration::from_secs(60),
        query_peer_timeout: Duration::from_secs(30),
        ..Default::default()
    };

    let storage = {
        Arc::new(parking_lot::RwLock::new(MemoryContentStore::new(
            discovery.discv5.local_enr().node_id(),
            DistanceFunction::Xor,
        )))
    };
    
    // Use this IF we decide to return an enum and can't figure out how to add the validtaor<> trait to our generic trait    
    // match protocol {
    //     ProtocolId::Custom(DAS_PROTOCOL_ID.to_string()) => {
    //     },
    //     ProtocolId::Custom(SECURE_DAS_PROTOCOL_ID.to_string()) => {
    //     },
    //     _ => {} 
    // }
  
    // This shouldn't be here.  Testing what happens when "TValidator<>" trait bound errors aren't happening
    let protocol = ProtocolId::Custom(DAS_PROTOCOL_ID.to_string());
    let validator = Arc::new(DASValidator);

    let (overlay, service) = OverlayProtocol::new(
        config,
        discovery.clone(),
        utp_listener_tx,
        storage,
        Distance::MAX,
        protocol,
        validator,
    );
   
    let overlay = Arc::new(overlay);
    ( 
        overlay, 
        service 
    )
}
*/





// -----------------------------------
//         INTERMEDIATE PHASE
//
// Just implement overlays seperately.
// -----------------------------------
//
// I'm spending a lot of time on complexities within Rust.  Make simple overlay creation functions for now.
// Circle back once I've implemented the message proxy
pub async fn create_das_overlay(discovery: Arc<Discovery>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) -> (
    Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>, 
    OverlayService<DASContentKey, XorMetric, DASValidator, MemoryContentStore>,
){
    let config = OverlayConfig {
        bootnode_enrs: discovery.clone().discv5.table_entries_enr(),
        ping_queue_interval: Some(Duration::from_secs(10000)),
        query_num_results: usize::MAX,
        query_timeout: Duration::from_secs(60),
        query_peer_timeout: Duration::from_secs(30),
        ..Default::default()
    };
    // println!("Overlay config bootnodes: {:?}", config.bootnode_enrs);
    let storage = {
        Arc::new(parking_lot::RwLock::new(MemoryContentStore::new(
            discovery.discv5.local_enr().node_id(),
            DistanceFunction::Xor,
        )))
    };
  
    let protocol = ProtocolId::Custom(DAS_PROTOCOL_ID.to_string());
    let validator = Arc::new(DASValidator);

    let (overlay, service) = OverlayProtocol::new(
        config,
        discovery.clone(),
        utp_listener_tx,
        storage,
        Distance::MAX,
        protocol,
        validator,
    );
   
    let overlay = Arc::new(overlay);
    ( 
        overlay, 
        service 
    )
} 


pub async fn create_secure_das_overlay(discovery: Arc<Discovery>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) -> (
    Arc<OverlayProtocol<SecureDASContentKey, XorMetric, SecureDASValidator, MemoryContentStore>>, 
    OverlayService<SecureDASContentKey, XorMetric, SecureDASValidator, MemoryContentStore>,
){

        let config = OverlayConfig {
        bootnode_enrs: discovery.clone().discv5.table_entries_enr(),
        ping_queue_interval: Some(Duration::from_secs(10000)),
        query_num_results: usize::MAX,
        query_timeout: Duration::from_secs(60),
        query_peer_timeout: Duration::from_secs(30),
        ..Default::default()
    };

    let storage = {
        Arc::new(parking_lot::RwLock::new(MemoryContentStore::new(
            discovery.discv5.local_enr().node_id(),
            DistanceFunction::Xor,
        )))
    };
  
    let protocol = ProtocolId::Custom(SECURE_DAS_PROTOCOL_ID.to_string());
    let validator = Arc::new(SecureDASValidator);

    let (overlay, service) = OverlayProtocol::new(
        config,
        discovery.clone(),
        utp_listener_tx,
        storage,
        Distance::MAX,
        protocol,
        validator,
    );
   
    let overlay = Arc::new(overlay);
    ( 
        overlay, 
        service 
    )
} 