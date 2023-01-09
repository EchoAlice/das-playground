#![allow(unused)]
use crate::das_node::main::run_nodes;
use crate::servers::main::run_servers;

mod das_node;
mod servers;


// Get rid of servers directory once DAS_node is functional
fn main() {
    run_nodes();
}