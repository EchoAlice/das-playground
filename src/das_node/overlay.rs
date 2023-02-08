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
        // TContentKey,
        // TValidator,
    } 
};

const DAS_PROTOCOL_ID: &str = "DAS";
const SECURE_DAS_PROTOCOL_ID: &str = "SECURE_DAS";


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

*/

// Modify parameters and return values of the function with generics.

pub async fn create_overlay<TContentKey, TValidator>(discovery: Arc<Discovery>, protocol: ProtocolId, validator:Arc<TValidator>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) -> ( 
// pub async fn create_overlay<TContentKey, TValidator>(discovery: Arc<Discovery>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) -> ( 
    // Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>, 
    // OverlayService<DASContentKey, XorMetric, DASValidator, MemoryContentStore>,
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
    
    // Modify logic 
    // let protocol = ProtocolId::Custom(DAS_PROTOCOL_ID.to_string());
    // let protocol = ProtocolId::Custom(SECURE_DAS_PROTOCOL_ID.to_string());

    let storage = {
        Arc::new(parking_lot::RwLock::new(MemoryContentStore::new(
            discovery.discv5.local_enr().node_id(),
            DistanceFunction::Xor,
        )))
    };
    
    // Modify logic. 
    // Use this IF we decide to return an enum and can't figure out how to add the validtaor<> trait to our generi trait    

    // Maybe if we can tell which protocol is being implemented, we can assert that our validator is either DAS or SecureDAS
    // Would this require us to create and return an enum OverlayType?
    
    // The Validator type is what's telling our OverlayProtocol::new( ) the type of overlay we're creating
    // let validator = Arc::new(DASValidator);
    // let validator = Arc::new(SecureDASValidator);
    // match protocol {
    //     ProtocolId::Custom(DAS_PROTOCOL_ID.to_string()) => {
    //         println!("DAS Protocol")
    //     },
    //     ProtocolId::Custom(SECURE_DAS_PROTOCOL_ID.to_string()) => {
    //         println!("Secure DAS Protocol")
    //     },
    //     _ => {} 
    // }


    // Do we need to modify our OverlayProtocol's constructor?
    /*
    pub fn new(
        config: OverlayConfig,
        discovery: Arc<Discovery>,
        utp_listener_tx: UnboundedSender<UtpListenerRequest>,
        store: Arc<RwLock<TStore>>,
        data_radius: Distance,
        protocol: ProtocolId,
        validator: Arc<TValidator>,
    ) -> (Self, OverlayService<TContentKey, TMetric, TValidator, TStore>) {    
    */
   
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