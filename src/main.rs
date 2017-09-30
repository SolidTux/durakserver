mod server;

use server::*;

fn main() {
    println!("Hello, world!");
    let server = Server::new("0.0.0.0:2342").unwrap();
    server.run();
}
