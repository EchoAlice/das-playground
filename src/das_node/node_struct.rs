#![allow(unused)]
use discv5_overlay::{portalnet::discovery::Discovery, utp::stream::UtpListenerRequest};
use std::sync::Arc;

//  Only accurate field is discovery.  Fix the rest one at a time...
#[derive(Debug, Clone)]
pub struct DASNode {
    // Discovery field is public for testing purposes! 
    pub discovery: Arc<Discovery>,
    libp2p: String,
    samples: [u8; 8],
    overlay: String,
    pub handled_ids: i32,
}

/*
    Begin to import fields from other crates!
*/
impl DASNode {
    pub fn new(
        discovery: Arc<Discovery>,
        // utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>,
        // libp2p: Libp2pService,
    ) -> Self {
        Self {
            discovery,
            libp2p: String::from("None"),
            samples: [0; 8],       // Correct number of samples???
            handled_ids: 0,
            overlay: String::from("None"),
        }
    }
}