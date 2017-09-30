use std::io::Result;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub struct Server {
    listener: TcpListener,
}

pub struct Client {
    id: u16,
    stream: TcpStream,
}

impl Server {
    pub fn new(address: &'static str) -> Result<Server> {
        Ok(Server { listener: TcpListener::bind(address)? })
    }

    pub fn run(&self) -> Result<()> {
        let mut id = 0;

        for stream in self.listener.incoming() {
            let client = Client::new(id, stream?);
            thread::spawn(move || client.handle());
            id = id + 1;
        }

        Ok(())
    }
}

impl Client {
    pub fn new(id: u16, stream: TcpStream) -> Client {
        Client {
            id: id,
            stream: stream,
        }
    }

    pub fn handle(&self) {
        let mut reader = BufReader::new(&self.stream);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    println!("{}: Connecion closed.", self.id);
                    break;
                }
                Ok(length) => println!("{} ({}): {}", self.id, length, line.trim()),
                Err(_) => {
                    println!("Error while reading message.");
                    break;
                }
            }
        }
    }
}
