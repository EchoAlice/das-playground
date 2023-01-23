use discv5::Discv5Event;
use discv5_overlay::{
    portalnet::{
        discovery::Discovery, 
        overlay::OverlayProtocol, 
        storage::MemoryContentStore, 
        types::distance::XorMetric
    }, 
    utp::stream::UtpListenerRequest
};
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
    ) -> Self {
        Self {
            discovery,
            overlay: None,
            samples: [0; 8],       // Correct number of samples???
            handled_ids: 0,
        }
    }
}