#![allow(unused)]
use discv5::{
    Discv5Event, 
    TalkRequest
};
use discv5_overlay::{
    portalnet::{
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

use crate::das_node::{
    discovery,
    node_struct::DASNode,
    overlay, 
    content_key::{
        DASContentKey, 
        DASValidator,
        SecureDASContentKey,
        SecureDASValidator,
    }
};

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
                See Trin: https://github.com/ethereum/trin/blob/master/trin-core/src/portalnet/discovery.rs#L174 


    Notes:
        - Discv5 IP addresses might be wrong
*/


#[tokio::main]
async fn main() {
    //============================ 
    //   Part 1:  Node Creation
    //============================ 
    let mut nodes = Vec::new();
    
    // Instantiates protocol structs and message processing within each node
    for i in 0..NUMBER_OF_NODES {
        let (
            starter_node, 
            mut overlay_service,
            mut secure_overlay_service, 
            utp_events_tx, 
            utp_listener_rx
        ) = create_node(i as u16).await;
      
        // I don't think we can obtain the event stream using discovery.start() instead of discv5.start() 
        let mut event_str = ReceiverStream::new(starter_node.discovery.discv5.event_stream().await.unwrap());

        // It doesn't feel clean copying the entire node to pass info into our task manager  :P 
        let node = starter_node.clone(); 
        nodes.push(starter_node);

        /* 
        Big Question:
            How should I handle messages from different subnetworks? 
                - One big event loop or proxy?
                           
            See Trin for Proxy: https://github.com/ethereum/trin/blob/master/trin-core/src/portalnet/discovery.rs#L174
        */
        
        // This is the server side of our nodes. Instantiates task manager to continually process ALL messages for each node.
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
                        let chan = format!("{i} {}", node.discovery.discv5.local_enr().node_id().to_string());
                        match event {
                            Discv5Event::TalkRequest(req) => {
                                println!("Stream {}: Discv5 TalkReq received", chan);  
                                
                                let node = node.clone(); 
                                tokio::spawn(async move {
                                    let protocol = ProtocolId::from_str(&hex::encode_upper(req.protocol())).unwrap();

                                    if protocol == ProtocolId::Custom(DAS_PROTOCOL_ID.to_string()) {
                                        println!("Enters DAS Protocol");  
                                        let talk_resp = match node.overlay.process_one_request(&req).await {
                                            Ok(response) => discv5_overlay::portalnet::types::messages::Message::from(response).into(),
                                            Err(err) => {
                                                error!("Node {chan} Error processing request: {err}");
                                                return;
                                            },
                                        };

                                        if let Err(err) = req.respond(talk_resp) {
                                            println!("Error");  
                                            error!("Unable to respond to talk request: {}", err);
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
                                                error!("Node {chan} Error processing request: {err}");
                                                return;
                                            },
                                        };

                                        if let Err(err) = req.respond(talk_resp) {
                                            println!("Error");  
                                            error!("Unable to respond to talk request: {}", err);
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
    
    // Populates our nodes' routing tables.  
    for i in 0..NUMBER_OF_NODES {
        // ***Figure out how to populate our overlay networks***
        populate_routing_table(i, nodes.clone());
    }

    // View of Routing Tables
    // ----------------------------
    // Discv5 
    println!("Nodes connected to local node's discv5 routing table: {:?}", nodes[2].discovery.connected_peers()); 
    // DAS Overlay 
    println!("Nodes connected to local node's overlay routing table: {:?}", nodes[2].overlay.table_entries_id()); 
    // SecureDAS Overlay 
    println!("Nodes connected to local node's secure overlay routing table: {:?}", nodes[2].secure_overlay.table_entries_id()); 
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
    // TODO: Send find nodes 
    // TODO: Send find content 

    // Secure Overlay Messaging
    // -------------------------- 
    let secure_das_ping = nodes[1].secure_overlay.send_ping(nodes[2].secure_overlay.local_enr());
    secure_das_ping.await;
    // TODO: Send find nodes 
    // TODO: Send find content 

    //================================ 
    //         Sanity Check 
    //================================ 
    println!("Overlay Protocol ID: {:?}", nodes[2].overlay.protocol()); 
    println!("Secure Overlay Protocol ID: {:?}", nodes[2].secure_overlay.protocol()); 

}




async fn create_node(i: u16) -> (
        DASNode, 
        OverlayService<DASContentKey, XorMetric, DASValidator, MemoryContentStore>,
        OverlayService<SecureDASContentKey, XorMetric, SecureDASValidator, MemoryContentStore>,
        UnboundedSender<TalkRequest>,
        UnboundedReceiver<UtpListenerEvent>
    ) {
    // 1. Discovery Protocol 
    //------------------------
    let discovery = discovery::create_discovery(i).await;


    // UTP Channels (Ignore)
    // -----------------------
    // DAS Overlay UTP Channel 
    let ( utp_events_tx, 
            utp_listener_tx, mut utp_listener_rx, 
            mut utp_listener,
    ) = UtpListener::new(discovery.clone());
    tokio::spawn(async move { utp_listener.start().await });
    
    // Secure DAS Overlay UTP Channel 
    let ( secure_utp_events_tx, 
            secure_utp_listener_tx, mut secure_utp_listener_rx, 
            mut secure_utp_listener,
    ) = UtpListener::new(discovery.clone());
    tokio::spawn(async move { secure_utp_listener.start().await });

   
    //--------------------
    // INTERMEDIATE PHASE 
    //--------------------
    // 2. DAS Overlay Protocol   
    let (overlay, overlay_service) = overlay::create_das_overlay(discovery.clone(), utp_listener_tx).await;  
    // 3. Secure DAS Overlay Protocol   
    let (secure_overlay, secure_overlay_service) = overlay::create_secure_das_overlay(discovery.clone(), secure_utp_listener_tx).await;  

/*
    //-----------------------------
    // GENERALIZE OVERLAY CREATION
    //-----------------------------
    // 2. DAS Overlay Protocol   
    let das_protocol = ProtocolId::Custom(DAS_PROTOCOL_ID.to_string());
    let das_validator = Arc::new(DASValidator);
    let (overlay, overlay_service) = overlay::create_overlay(discovery.clone(), das_protocol, das_validator, utp_listener_tx).await;  
    
    // 3. Secure Overlay Protocol
    let secure_protocol = ProtocolId::Custom(SECURE_DAS_PROTOCOL_ID.to_string());
    let secure_das_validator = Arc::new(SecureDASValidator);
    let (overlay, overlay_service) = overlay::create_overlay(discovery.clone(), secure_protocol, secure_das_validator, utp_listener_tx).await;  
*/

    //  Samples: TODO
    
    //  Handled_ids: TODO 


    // Creates node (Timofey creates node with utp_listener_tx) 
    let mut my_node = DASNode::new(discovery, overlay, secure_overlay);
    
    (
        my_node,
        overlay_service,
        secure_overlay_service,
        utp_events_tx,
        utp_listener_rx
    ) 
}


// Adds nodes from within the simulation to routing tables.  
fn populate_routing_table(local_index: usize, mut nodes: Vec<DASNode>) {
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
        // ==================================================================
        // ==================================================================
        // ***How can I populate all routing tables with the same 3 nodes?***
        // ==================================================================
        // ==================================================================
        // Not hanlding errors right now.  Getting concept down   :P 
        
        if invalid_index == false {
            nodes[local_index].discovery.discv5.add_enr(nodes[rand].discovery.discv5.local_enr().clone());
            // nodes[local_index].overlay.bucket_entries(somehow add ENRs to this routing table);
            // nodes[local_index].secure_overlay.bucket_entries(somehow add ENRs to this routing table);
            
            
            
            
            // Figure out control flow later.  This is original code!
            match nodes[local_index].discovery.discv5.add_enr(nodes[rand].discovery.discv5.local_enr().clone()) {
                Ok(_) => {
                    used_indexes.push(rand);
                    n -= 1;
                },
                // Might need to change this 
                Err(_) => continue,
            }
        }
    }
}



pub fn run_nodes() {
    crate::das_node::main::main();
}
