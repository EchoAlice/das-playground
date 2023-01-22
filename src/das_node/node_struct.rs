#![allow(unused)]
use discv5::Discv5Event;
// Clean this up
use discv5_overlay::portalnet::overlay::OverlayProtocol;
use discv5_overlay::portalnet::types::distance::XorMetric;
use discv5_overlay::{portalnet::discovery::Discovery, utp::stream::UtpListenerRequest};
use discv5_overlay::portalnet::storage::MemoryContentStore; 

use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;

use crate::das_node::overlay::{DASContentKey, DASValidator};


// Figure out how to create a DAS Node that is initialized without an overlay field, but can be added down the line

// The only accurate field here is discovery!  All other fields have dummy types right now.
#[derive(Clone)]
pub struct DASNode {
    // Discovery field is public for testing purposes 
    pub discovery: Arc<Discovery>,
    pub overlay: Option<Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>>,
    samples: [u8; 8],
    pub handled_ids: i32,
}


// The DASNode within Model-DAS returns itself AND an overlay service!
impl DASNode {
    pub fn new(
        discovery: Arc<Discovery>,
        // overlay: Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>,
    ) -> Self {
        Self {
            discovery,
            overlay: None,
            samples: [0; 8],       // Correct number of samples???
            handled_ids: 0,
        }
    }

    // pub fn add_overlay() {
    //        ???
    // }
}