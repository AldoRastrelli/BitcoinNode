use std::error::Error;
//use std::nte::BufReader;
use rusteze::node::bitnode::BitcoinNode;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(_node) = BitcoinNode::build() else {
        return Err("No node".into());
    };
    Ok(())
}
