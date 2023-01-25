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

#[derive(Clone)]
pub struct DASNode {
    pub discovery: Arc<Discovery>,
    pub overlay: Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>,
    samples: [u8; 8],
    pub handled_ids: i32,
}

// The DASNode within Model-DAS returns itself AND an overlay service!
impl DASNode {
    pub fn new(
        discovery: Arc<Discovery>,
        overlay: Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>
    ) -> Self {
        Self {
            discovery,
            overlay,
            samples: [0; 8],       
            handled_ids: 0,
        }
    }
}