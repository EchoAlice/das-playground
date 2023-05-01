use discv5_overlay::{
    portalnet::{
        discovery::Discovery, 
        overlay::OverlayProtocol, 
        storage::MemoryContentStore, 
        types::distance::XorMetric
    }, 
};
use std::sync::Arc;

use crate::content_key::{
    DASContentKey, 
    DASValidator,
    SecureDASContentKey,
    SecureDASValidator,
};


#[derive(Clone)]
pub struct DASNode {
    pub discovery: Arc<Discovery>,
    pub overlay: Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>,
    pub secure_overlay: Arc<OverlayProtocol<SecureDASContentKey, XorMetric, SecureDASValidator, MemoryContentStore>>,
    
    samples: [u8; 8],
    pub handled_ids: i32,
}

// The DASNode within Model-DAS returns itself AND an overlay service!
impl DASNode {
    pub fn new(
        discovery: Arc<Discovery>,
        overlay: Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>,
        secure_overlay: Arc<OverlayProtocol<SecureDASContentKey, XorMetric, SecureDASValidator, MemoryContentStore>>,
    ) -> Self {
        Self {
            discovery,
            overlay,
            secure_overlay,
            samples: [0; 8],       
            handled_ids: 0,
        }
    }
}