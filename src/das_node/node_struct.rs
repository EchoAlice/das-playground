#![allow(unused)]
use discv5::Discv5Event;
use discv5_overlay::{portalnet::discovery::Discovery, utp::stream::UtpListenerRequest};
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;

/*
    The only accurate field here is discovery!  All other fields have dummy types right now.
    Instantiate the overlay field next. 

    Having the dv5_event_stream field doesn't allow for the "clone" trait.  
    Might need move the field if this becomes a problem down the line.
*/
#[derive(Debug)]
pub struct DASNode {
    // Discovery field is public for testing purposes! dv5_event_stream was manually placed here by me. 
    pub discovery: Arc<Discovery>,
    pub dv5_event_stream: ReceiverStream<Discv5Event>, 
    
    libp2p: String,
    samples: [u8; 8],
    overlay: String,
    pub handled_ids: i32,
}

impl DASNode {
    pub fn new(
        discovery: Arc<Discovery>,
        dv5_event_stream: ReceiverStream<Discv5Event>, 
        // utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>,
        // libp2p: Libp2pService,
    ) -> Self {
        Self {
            discovery,
            dv5_event_stream, 
            libp2p: String::from("None"),
            samples: [0; 8],       // Correct number of samples???
            handled_ids: 0,
            overlay: String::from("None"),
        }
    }
}