extern crate rand;

mod network;
mod game;
mod rules;

use network::*;
use rules::*;

fn main() {
    let mut server = Server::new("0.0.0.0:2342", DefaultRules::new()).unwrap();
    server.listen().unwrap();
}
