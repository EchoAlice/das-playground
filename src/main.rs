#![allow(unused)]
use discv5::Discv5;
use discv5_overlay::portalnet::discovery::Discovery;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

mod node_struct;
use node_struct::DASNode;


/*
    1.  Spin up DASNodes that have their own Discovery Protocol. (Does this include a server?)  
    2.  Try adding information to their routing tables!  

    Arc<T>'s, aka Atomic Reference Counted types, are smart pointers that allow for 
    a single value to have multiple owners.  ***These multiple owners can be shared across multiple threads***
*/

#[tokio::main]
async fn main() {
    let mut handles = Vec::new();
    for i in 0..4 {
        let mut my_node = DASNode::new();
        let handle = tokio::spawn(async move {
            instantiate_node(i, my_node).await;
        });
        handles.push(handle);
    } 
    
    for handle in handles {
        handle.await.unwrap();
    }
}

// Tinkering with changing up nodes in an async fashing
async fn instantiate_node(i: i32, mut node: DASNode) {
    println!("i: {i}"); 
    node.handled_ids = i; 
    node.ping += 1;
    create_server(node).await;
}

//  Spin up new nodes that all contain discv5 servers 
//  Follow through with line 152 within DASPrototype,
async fn create_server(mut node: DASNode) {
    sleep(Duration::from_millis(1000)).await;
    println!("Create a server for each node");
}
