extern crate rand;

mod network;
mod game;

use network::*;

fn main() {
    let mut server = Server::new("0.0.0.0:2342").unwrap();
    server.listen().unwrap();
}
