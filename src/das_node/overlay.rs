#![allow(unused)]
use async_trait::async_trait;
use discv5_overlay::portalnet::discovery::Discovery;
use discv5_overlay::portalnet::overlay::{OverlayConfig, OverlayProtocol};
use discv5_overlay::portalnet::types::distance::XorMetric;
use discv5_overlay::portalnet::storage::MemoryContentStore;
use discv5_overlay::types::validation::Validator;
use std::time::Duration;
use std::sync::Arc;
use ssz_derive::{Decode, Encode};

const DAS_PROTOCOL_ID: &str = "DAS";

/// A content key in the DAS overlay network.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
#[ssz(enum_behaviour = "union")]
pub enum DASContentKey {
    Sample([u8; 32]),
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

    Questions:
        - Should our create_overlay() also spawn our client task manager? 
        - Why do our protocols need an atomically reference-counted pointer?
        - What are our message tasks?  What is our client task? 
*/
pub async fn create_overlay(discovery: Arc<Discovery>) { /* -> Arc<OverlayProtocol<DASContentKey, XorMetric, DASValidator, MemoryContentStore>> */
    // Create the entire overlay protocol within this function.  Reference Model DAS's impl DASNode{}    
    let config = OverlayConfig {
        bootnode_enrs: discovery.clone().discv5.table_entries_enr(),
        ping_queue_interval: Some(Duration::from_secs(10000)),
        query_num_results: usize::MAX,
        query_timeout: Duration::from_secs(60),
        query_peer_timeout: Duration::from_secs(30),
        ..Default::default()
    };
    
    // Continue here
    // let protocol = ProtocolId::Custom(DAS_PROTOCOL_ID.to_string());


}