#![allow(unused)]
use crate::das_node::node_struct;
use crate::das_node::discovery;


/*
    Create nodes that contain servers for each protocol the DASNode supports.  Each server has a task!
*/
#[tokio::main]
async fn main() {
    let mut node_futures = Vec::new(); 
    for i in 0..4 {
        let i = i as u16; 
        let my_node = create_node(i);
        node_futures.push(my_node);
    }
    
    // Makes sure all nodes have been fully instantiated.  Moves  
    let mut nodes = Vec::new();
    for node in node_futures {
        let out = node.await; 
        nodes.push(out); 
        
    }

    /*
        This is where we can start manipulating nodes!
            1. Figure out how to access a nodes peers within Discv5 
    */
    for node in nodes {
        println!("{:?}", node);
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
    
    // Creates node.  Add to fields as project progresses  
    let mut my_node = node_struct::DASNode::new(discovery);
    
    my_node 
}




pub fn run_nodes() {
    crate::das_node::main::main();
}