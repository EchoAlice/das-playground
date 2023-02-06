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
    } 
};

const DAS_PROTOCOL_ID: &str = "DAS";


/*
    Goal: 
        Create our subnetworks via the overlay protocol 

    "The Overlay protocol is a layer on top of discv5 that handles all requests from the overlay networks
    (state, history etc.) and dispatch them to the discv5 protocol TalkReq. Each network should
    implement the overlay protocol and the overlay protocol is where we can encapsulate the logic for
    handling common network requests/responses."  -Trin repo
*/


/*
How can I make more generalizable content keys so i can reuse this function for secure overlay?  -->  <TContentKey, TStore, etc.>  
Maybe take in our content key as a parameter.  Look at overlay protocol!

This may help... Found in Trin's overlay service

impl<
        TContentKey: 'static + OverlayContentKey + Send + Sync,
        TMetric: Metric + Send + Sync,
        TValidator: 'static + Validator<TContentKey> + Send + Sync,
        TStore: 'static + ContentStore + Send + Sync,
    > OverlayService<TContentKey, TMetric, TValidator, TStore>
*/


pub async fn create_overlay(discovery: Arc<Discovery>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) -> ( 
    Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>, 
    OverlayService<DASContentKey, XorMetric, DASValidator, MemoryContentStore>
    ) { 
    let config = OverlayConfig {
        bootnode_enrs: discovery.clone().discv5.table_entries_enr(),
        ping_queue_interval: Some(Duration::from_secs(10000)),
        query_num_results: usize::MAX,
        query_timeout: Duration::from_secs(60),
        query_peer_timeout: Duration::from_secs(30),
        ..Default::default()
    };
    let protocol = ProtocolId::Custom(DAS_PROTOCOL_ID.to_string());
    let storage = {
        Arc::new(parking_lot::RwLock::new(MemoryContentStore::new(
            discovery.discv5.local_enr().node_id(),
            DistanceFunction::Xor,
        )))
    };
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