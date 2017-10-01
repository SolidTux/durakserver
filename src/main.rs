mod network;

use network::*;

fn main() {
    let server = Server::new("0.0.0.0:2342").unwrap();
    server.listen().unwrap();
}
