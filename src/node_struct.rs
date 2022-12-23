use discv5_overlay::portalnet::discovery::Discovery;
use std::sync::Arc;

// These fields aren't accurate.  Fix them one at a time...
#[derive(Debug, Clone)]
pub struct DASNode {
    // discovery: Arc<Discovery>,
    libp2p: String,
    samples: [u8; 8],
    overlay: String,
    pub handled_ids: i32,
    pub ping: i32,
}

/*
    Begin to import fields from other crates!
*/
impl DASNode {
    // pub fn new(discovery: Arc<Discovery>) -> Self {
    pub fn new() -> Self {
        Self {
            // discovery,
            libp2p: String::from("None"),
            samples: [0; 8],       // Correct number of samples???
            handled_ids: 0,
            overlay: String::from("None"),
            ping: 0,
        }
    }
}