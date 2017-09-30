extern crate rand;

mod game;
mod server;

use server::*;

fn main() {
    let mut server = Server::new("0.0.0.0:2342").unwrap();
    server.run().unwrap();
}
