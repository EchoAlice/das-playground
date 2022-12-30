#![allow(unused)]

mod nodes;
mod servers;

use crate::nodes::main::run_nodes;
use crate::servers::main::run_servers;

fn main() {
    println!("Test"); 
    run_nodes();
    run_servers();
}