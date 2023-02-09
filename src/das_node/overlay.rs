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
        ValidatorType, 
        // TContentKey,
        // TValidator,
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


    Currently modifying parameters and return values of the function with generics!
*/


// Passes in an enum as our validator type
// pub async fn create_overlay<TContentKey, TValidator>(discovery: Arc<Discovery>, protocol: ProtocolId, validator:Arc<ValidatorType>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) -> ( 
// This one's our superstar
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