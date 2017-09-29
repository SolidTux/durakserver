use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};

fn run_server(address: &'static str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;

    let mut id = 0;

    for stream in listener.incoming() {
        handle_client(id, stream?);
        id = id + 1;
    }

    Ok(())
}

fn handle_client(id: u32, stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("{}: Connecion closed.", id);
                break
            },
            Ok(length) => println!("{} ({}): {}", id, length, line.trim()),
            Err(_) => {
                println!("Error while reading message.");
                break
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
    run_server("0.0.0.0:2342").unwrap();
}
