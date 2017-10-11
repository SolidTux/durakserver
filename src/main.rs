extern crate durak;

use durak::network::*;
use durak::rules::*;

fn main() {
    let mut server = Server::new("0.0.0.0:2342", DefaultRules::new()).unwrap();
    server.listen().unwrap();
}
