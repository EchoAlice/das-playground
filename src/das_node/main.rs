#![allow(unused)]
use tokio_stream::wrappers::ReceiverStream;

use crate::das_node::node_struct;
use crate::das_node::discovery;


/*
    Create nodes that contain servers for each protocol the DASNode supports.  
    Our discv5 server runs tasks that are abstracted from us.

    "To process connections concurrently, a new task is spawned for each inbound connection. The connection is processed on this task."
*/
#[tokio::main]
async fn main() {
    let mut node_futures = Vec::new(); 
    let mut nodes = Vec::new();
    
    // Create our DASNodes 
    for i in 0..4 {
        let i = i as u16; 
        let my_node = create_node(i);
        node_futures.push(my_node);
    }
    for node in node_futures {
        let out = node.await; 
        nodes.push(out); 
    }

    /*
    Concepts to implement:
        1. Create event streams for each node's discv5 service.  
           How should I store this?  Within a data structure or out in the wild? 
        2. Add nodes to the discv5 network 
    */
    for node in nodes {
        //  Discv5 Routing tables are empty.  Change this!
        println!("{:?}", node);
        println!("Our node's discv5 peers: {:?}", node.discovery.discv5.table_entries_enr());
        println!("\n");
    }
}


/*
DASNode
    1. Discovery Protocol   (Check)  
    2. Libp2p Service
    3. Samples
    4. Handled_ids
    5. Overlay Protocol 
*/
async fn create_node(i: u16) -> node_struct::DASNode {
    // 1. Discovery Protocol 
    let discovery = discovery::create_discovery(i).await;
    let mut event_stream = ReceiverStream::new(discovery.discv5.event_stream().await.unwrap());
    
    // Creates node.  Add to fields as project progresses  
    let mut my_node = node_struct::DASNode::new(discovery, event_stream);
    
    my_node 
}




pub fn run_nodes() {
    crate::das_node::main::main();
}