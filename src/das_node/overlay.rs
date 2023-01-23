use async_trait::async_trait;
use discv5_overlay::{
    portalnet::{
        discovery::Discovery,
        overlay::{OverlayConfig, OverlayProtocol},
        overlay_service::OverlayService,
        storage::{DistanceFunction, MemoryContentStore},
        types::{
            content_key::OverlayContentKey,
            distance::{Distance, XorMetric},
            messages::ProtocolId
        }
    },
    types::validation::Validator,
    utp::stream::UtpListenerRequest,
};
use std::{
    fmt,
    fmt::{Display, Formatter},
    sync::Arc,
    time::Duration
};
use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};
use tokio::sync::mpsc;

const DAS_PROTOCOL_ID: &str = "DAS";

/*
    I don't know what's going on with these DASContentKey + DASValidator traits.
    Copy + pasted from Model-DAS's overlay.rs!
*/
/// A content key in the DAS overlay network.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
#[ssz(enum_behaviour = "union")]
pub enum DASContentKey {
    Sample([u8; 32]),
}

#[allow(clippy::from_over_into)]
impl Into<Vec<u8>> for DASContentKey {
    fn into(self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
}

impl TryFrom<Vec<u8>> for DASContentKey {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match DASContentKey::from_ssz_bytes(&value) {
            Ok(key) => Ok(key),
            Err(_err) => {
                println!("unable to decode DASContentKey");
                Err("Unable to decode SSZ")
            }
        }
    }
}

impl Display for DASContentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Sample(b) => format!("sample: {}", hex::encode(b)),
        };

        write!(f, "{}", s)
    }
}

impl OverlayContentKey for DASContentKey {
    fn content_id(&self) -> [u8; 32] {
        match self {
            DASContentKey::Sample(b) => b.clone(),
        }
    }
}

pub struct DASValidator;

#[async_trait]
impl Validator<DASContentKey> for DASValidator {
    async fn validate_content(
        &self,
        content_key: &DASContentKey,
        content: &[u8],
    ) -> anyhow::Result<()>
// where
        //     DASContentKey: 'async_trait,
    {
        match content_key {
            DASContentKey::Sample(_) => Ok(()),
        }
    }
}

/*
    Goal: 
        Understand how these utp channels work to facilitate overlay communication

    Notes:
        - E&T use the uTP protocol for reliable message passing over UDP.
              Check out the reasons why reliable messaging is needed in DAS --> https://hackmd.io/@timofey/SyqzhA4vo#712-Reliable-UDP-over-Discv5
        - I don't understand what's going on above these comments... 
    Questions:
        - Should our create_overlay() also spawn our client task manager? 
        - Why do our protocols need an atomically reference-counted pointer?
        - What are our message tasks?  What is our client task? 


    The Overlay protocol is a layer on top of discv5 that handles all requests from the overlay networks
    (state, history etc.) and dispatch them to the discv5 protocol TalkReq. Each network should
    implement the overlay protocol and the overlay protocol is where we can encapsulate the logic for
    handling common network requests/responses.
*/

// Creates the entire overlay protocol within this function.  Reference Model DAS's impl DASNode{}    
// Make more generalizable content keys so i can reuse this function for second overlay...  -->  <TContentKey, TStore, etc.>.  Look at overlay protocol 
pub async fn create_overlay(discovery: Arc<Discovery>, utp_listener_tx: mpsc::UnboundedSender<UtpListenerRequest>) ->  
    ( 
    Option<Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>>>, 
    OverlayService<DASContentKey, XorMetric, DASValidator, MemoryContentStore> 
    ) 
    { 
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
    ( Some(overlay), service )
}