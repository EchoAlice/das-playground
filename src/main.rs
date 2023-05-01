#![allow(unused)]
use discv5::{
    Discv5Event, 
    TalkRequest
};
use discv5_overlay::{
    portalnet::{
        discovery::Discovery,
        overlay_service::{
            OverlayCommand,
            OverlayService
        },
        storage::MemoryContentStore,
        types::{
            distance::XorMetric,
            messages::ProtocolId
        }, 
    },
    utp::stream::{UtpListener, UtpListenerEvent}
};
use futures::StreamExt; 
use rand::Rng;
use std::{
    str::FromStr,
    sync::Arc,
};
use tokio::{
    select, 
    sync::mpsc::{UnboundedReceiver, UnboundedSender}
};
use tokio_stream::wrappers::ReceiverStream;
use tracing;
use tracing::log::error;

use crate::{
    node_struct::DASNode,
    content_key::{
        DASContentKey, 
        DASValidator,
        SecureDASContentKey,
        SecureDASValidator,
    }
};

pub mod content_key;
pub mod discovery;
pub mod node_struct;
pub mod overlay;

pub const NUMBER_OF_NODES: usize = 10;
const DAS_PROTOCOL_ID: &str = "DAS";
const SECURE_DAS_PROTOCOL_ID: &str = "SECURE_DAS";

/*
    Goals 
        - Create DASNodes that contain the protocols and subprotocols needed for a backup, 
          validator-only DHT, discv5 overlay network so participates in the repo can communicate.  

    Questions:
        - Are data shard validator committees the same committees used within Light Client things?  
        - Should I create a proxy for message processing to handle my two seperate overlays?


    Notes:
        - Discv5 IP addresses might be wrong
        - Trin uses a proxy for message processing:  https://github.com/ethereum/trin/blob/master/trin-core/src/portalnet/discovery.rs#L174 
*/


#[tokio::main]
async fn main() {
    //============================ 
    //   Part 1:  Node Creation
    //============================ 
    let mut discv5_structs = Vec::new();
    let mut nodes = Vec::new();

    // Create all Discv5 servers, then pass these into create_nodes.
    for i in 0.. NUMBER_OF_NODES {
        let discv5_struct = discovery::create_discovery(i as u16).await;
        discv5_structs.push(discv5_struct)
    }

    // Populate discv5 tables
    for i in 0..NUMBER_OF_NODES {
        populate_discv5_table(i, discv5_structs.clone());
    }

    // Instantiates protocol structs and message processing within each node
    for i in discv5_structs.into_iter() {
        let (
            starter_node, 
            mut overlay_service,
            mut secure_overlay_service, 
            utp_events_tx, 
            utp_listener_rx
        ) = create_node(i).await;
     
        let mut event_str = ReceiverStream::new(starter_node.discovery.discv5.event_stream().await.unwrap());

        // Copying the entire node to pass info into our task manager  :P 
        let node = starter_node.clone(); 
        nodes.push(starter_node);
        
        // Instantiates task manager to continually process ALL messages for each node (server side of node).
        // Wrap message processing code into a function so it's easy to read.
        tokio::spawn(async move {
            loop {
                /// "Select!" randomly picks one of these match branches to process an event 
                select! {
                    // =========================== 
                    // Overlay Message Processing:  
                    // =========================== 
                    // Request 
                    Some(command) = overlay_service.command_rx.recv() => {
                        match command {
                            OverlayCommand::Request(request) => { 
                                println!("Processing Overlay Request"); 
                                overlay_service.process_request(request)
                            }, 
                            _ => {}    
                        }
                    }
                    // Response 
                    Some(response) = overlay_service.response_rx.recv() => {
                        // Look up active request that corresponds to the response.
                        let optional_active_request = overlay_service.active_outgoing_requests.write().remove(&response.request_id);
                        if let Some(active_request) = optional_active_request {
                            println!("Send overlay response");
                            println!("\n");
                            // Send response to responder if present.
                            if let Some(responder) = active_request.responder {
                                let _ = responder.send(response.response.clone());
                            }

                            // Perform background processing.
                            match response.response {
                                Ok(response) => overlay_service.process_response(response, active_request.destination, active_request.request, active_request.query_id),
                                Err(error) => overlay_service.process_request_failure(response.request_id, active_request.destination, error),
                            }

                        } else {
                            println!("No request found for response");
                        }
                    } 
                    // ================================== 
                    // Secure Overlay Message Processing:  
                    // ================================== 
                    // Request 
                    Some(command) = secure_overlay_service.command_rx.recv() => {
                        match command {
                            OverlayCommand::Request(request) => { 
                                println!("Processing Secure Overlay Request"); 
                                secure_overlay_service.process_request(request)
                            }, 
                            _ => {}    
                        }
                    }
                    // Response 
                    Some(response) = secure_overlay_service.response_rx.recv() => {
                        // Look up active request that corresponds to the response.
                        let optional_active_request = secure_overlay_service.active_outgoing_requests.write().remove(&response.request_id);
                        if let Some(active_request) = optional_active_request {
                            println!("Send secure overlay response");
                            println!("\n");
                            // Send response to responder if present.
                            if let Some(responder) = active_request.responder {
                                let _ = responder.send(response.response.clone());
                            }

                            // Perform background processing.
                            match response.response {
                                Ok(response) => secure_overlay_service.process_response(response, active_request.destination, active_request.request, active_request.query_id),
                                Err(error) => secure_overlay_service.process_request_failure(response.request_id, active_request.destination, error),
                            }

                        } else {
                            println!("No request found for response");
                        }
                    } 
                    // ==========================
                    // Discv5 Message Processing: 
                    // ==========================
                    // Incoming event
                    Some(event) = event_str.next() => {
                        // let chan = format!("{:?} {i}", node.discovery.discv5.local_enr().node_id().to_string());
                        match event {
                            Discv5Event::TalkRequest(req) => {
                                // println!("Stream {}: Discv5 TalkReq received", chan);  
                                
                                let node = node.clone(); 
                                tokio::spawn(async move {
                                    let protocol = ProtocolId::from_str(&hex::encode_upper(req.protocol())).unwrap();

                                    if protocol == ProtocolId::Custom(DAS_PROTOCOL_ID.to_string()) {
                                        println!("Enters DAS Protocol");  
                                        let talk_resp = match node.overlay.process_one_request(&req).await {
                                            Ok(response) => discv5_overlay::portalnet::types::messages::Message::from(response).into(),
                                            Err(err) => {
                                                error!("Error processing request:");
                                                // error!("Node {chan} Error processing request: {err}");
                                                return;
                                            },
                                        };

                                        if let Err(err) = req.respond(talk_resp) {
                                            println!("Error");  
                                            error!("Unable to respond to talk request: ");
                                            // error!("Unable to respond to talk request: {}", err);
                                            return;
                                        }

                                        return;
                                    }

                                    // See if we can watch for a secure overlay event.
                                    if protocol == ProtocolId::Custom(SECURE_DAS_PROTOCOL_ID.to_string()) {
                                        println!("Enters SecureDAS Protocol");  
                                        let talk_resp = match node.overlay.process_one_request(&req).await {
                                        // let talk_resp = match node.secure_overlay.process_one_request(&req).await {
                                            Ok(response) => discv5_overlay::portalnet::types::messages::Message::from(response).into(),
                                            Err(err) => {
                                                error!("Node Error processing request: ");
                                                // error!("Node {chan} Error processing request: {err}");
                                                return;
                                            },
                                        };

                                        if let Err(err) = req.respond(talk_resp) {
                                            println!("Error");  
                                            error!("Unable to respond to talk request: ");
                                            // error!("Unable to respond to talk request: {}", err);
                                            return;
                                        }

                                        return;
                                    }
                                    // let resp = handle_talk_request(req.node_id().clone(), req.protocol(), req.body().to_vec(), node, opts, enr_to_libp2p, node_ids, i).await;
                                    // let resp = handle_talk_request(req.node_id().clone(), req.protocol(), req.body().to_vec(), node).await;
                                    // req.respond(resp);
                                });
                            },
                            _ => {}    
                        }
                    },
                }
            } 
        });
    }

    // View of a node's routing table
    // ----------------------------
    println!("Node's discv5 routing table: {:?}", nodes[2].discovery.connected_peers()); 
    println!("\n");
    println!("Node's overlay routing table: {:?}", nodes[2].overlay.table_entries_id()); 
    println!("\n");
    println!("Node's secure overlay routing table: {:?}", nodes[2].secure_overlay.table_entries_id()); 
    println!("\n");


    /* 
    ================================== 
       Part 2: Node Communication
    ================================== 
       Creates simple communication between nodes. We need to pass our overlay messages 
       through peers' routing tables.  Implement concurrent communication later.     
    
       Overlay Protocol struct --> calls our Overlay Service   
    */

    // Overlay Messaging
    // ------------------ 
    let das_ping = nodes[1].overlay.send_ping(nodes[2].overlay.local_enr());
    das_ping.await;
    // TODO: Send find_nodes 
    // TODO: Send find_content 

    // Secure Overlay Messaging
    // -------------------------- 
    let secure_das_ping = nodes[1].secure_overlay.send_ping(nodes[2].secure_overlay.local_enr());
    secure_das_ping.await;
    // TODO: Send find_nodes 
    // TODO: Send find_content 

    //================================ 
    //         Sanity Check 
    //================================ 
    println!("Overlay Protocol ID: {:?}", nodes[2].overlay.protocol()); 
    println!("Secure Overlay Protocol ID: {:?}", nodes[2].secure_overlay.protocol()); 

}


async fn create_node(discv5_struct: Arc<Discovery>) -> (
        DASNode, 
        OverlayService<DASContentKey, XorMetric, DASValidator, MemoryContentStore>,
        OverlayService<SecureDASContentKey, XorMetric, SecureDASValidator, MemoryContentStore>,
        UnboundedSender<TalkRequest>,
        UnboundedReceiver<UtpListenerEvent>
    ) {

    // DAS Overlay UTP Channel 
    let ( utp_events_tx, 
            utp_listener_tx, mut utp_listener_rx, 
            mut utp_listener,
    ) = UtpListener::new(discv5_struct.clone());
    tokio::spawn(async move { utp_listener.start().await });
    
    // Secure DAS Overlay UTP Channel 
    let ( secure_utp_events_tx, 
            secure_utp_listener_tx, mut secure_utp_listener_rx, 
            mut secure_utp_listener,
    ) = UtpListener::new(discv5_struct.clone());
    tokio::spawn(async move { secure_utp_listener.start().await });

    // DAS and Secure DAS Overlay Protocols
    let (overlay, overlay_service) = overlay::create_das_overlay(discv5_struct.clone(), utp_listener_tx).await;
    let (secure_overlay, secure_overlay_service) = overlay::create_secure_das_overlay(discv5_struct.clone(), secure_utp_listener_tx).await;  

    //  Samples: TODO
    
    //  Handled_ids: TODO 

    // Creates node (Timofey creates node with utp_listener_tx) 
    let mut my_node = DASNode::new(discv5_struct, overlay, secure_overlay);
    
    (
        my_node,
        overlay_service,
        secure_overlay_service,
        utp_events_tx,
        utp_listener_rx
    ) 
}


// Adds nodes from within the simulation to routing tables.  
fn populate_discv5_table(local_index: usize, mut structs: Vec<Arc<Discovery>>) {
    // Number of peers a node adds to their routing table 
    let mut n = 3;
    let mut used_indexes = Vec::new();

    while n != 0 {
        let mut invalid_index = false; 
        let mut rng = rand::thread_rng();
        let rand = rng.gen_range(0usize..NUMBER_OF_NODES);

        // Makes sure we aren't duplicating nodes within our routing table 
        for i in 0..used_indexes.len() {
            if rand == used_indexes[i] || rand == local_index {
                invalid_index = true; 
            }
        }
        
        if invalid_index == false {
            match structs[local_index].discv5.add_enr(structs[rand].discv5.local_enr().clone()) {
                Ok(_) => {
                    used_indexes.push(rand);
                    n -= 1;
                },
                Err(_) => continue,
            }
        }
    }
}