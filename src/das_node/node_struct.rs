use discv5_overlay::{
    portalnet::{
        discovery::Discovery, 
        overlay::OverlayProtocol, 
        storage::MemoryContentStore, 
        types::distance::XorMetric
    }, 
};
use std::sync::Arc;

use crate::das_node::content_key::{
    // TContentKey,
    // TValidator, 
    DASContentKey, 
    DASValidator,
};


// Should our DASNode contain generics?  Or should all generics be used when creating our individual overlays?
// Get rid of generics from struct and impl definitions if they're not necessary!
#[derive(Clone)]
// pub struct DASNode<TContentKey, TValidator> {
pub struct DASNode {
    pub discovery: Arc<Discovery>,
    // I don't think this is correct
    // pub overlay: Arc<OverlayProtocol<TContentKey, XorMetric, TValidator, MemoryContentStore>>,
    
    pub overlay: Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>,
    // pub secure_overlay: Arc<OverlayProtocol<SecureDASContentKey, XorMetric, SecureDASValidator, MemoryContentStore>>,
    
    samples: [u8; 8],
    pub handled_ids: i32,
}

// The DASNode within Model-DAS returns itself AND an overlay service!
// impl<TContentKey, TValidator> DASNode<TContentKey, TValidator> {
impl DASNode {
    pub fn new(
        discovery: Arc<Discovery>,
        // overlay: Arc<OverlayProtocol<TContentKey, XorMetric, TValidator, MemoryContentStore>>,
        overlay: Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>,
        // secure_overlay: Arc<OverlayProtocol<SecureDASContentKey, XorMetric, SecureDASValidator, MemoryContentStore>>,
    ) -> Self {
        Self {
            discovery,
            overlay,
            samples: [0; 8],       
            handled_ids: 0,
        }
    }
}