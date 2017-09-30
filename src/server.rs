use std::io::Result;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::thread;
use rand::random;
use game::*;


pub struct Server {
    listener: TcpListener,
    players: HashMap<u64, Player>,
}

pub struct Client {
    id: u16,
    stream: TcpStream,
    player: Option<u64>,
}

impl Server {
    pub fn new(address: &'static str) -> Result<Server> {
        Ok(Server {
            listener: TcpListener::bind(address)?,
            players: HashMap::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut id = 0;

        for stream in self.listener.incoming() {
            let mut client = Client::new(id, stream?);
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
            player: None,
        }
    }

    pub fn handle(&mut self) {
        let read_stream = self.stream.try_clone().unwrap();
        let mut reader = BufReader::new(read_stream);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    println!("{}: Connecion closed.", self.id);
                    break;
                }
                Ok(_) => {
                    self.parse_line(line).unwrap();
                }
                Err(_) => {
                    println!("Error while reading message.");
                    break;
                }
            }
        }
    }

    fn write_error<S: Into<String>>(&self, message: S) -> Result<()> {
        self.write_message(format!("error {}", message.into()))
    }

    fn write_message<S: Into<String>>(&self, message: S) -> Result<()> {
        let mut writer = BufWriter::new(&self.stream);
        writer.write_fmt(format_args!("{}", message.into()))?;
        writer.flush()?;

        Ok(())
    }

    fn parse_line(&mut self, line: String) -> Result<()> {
        let mut partiter = line.split_whitespace();
        match partiter.next() {
            Some("player") => {
                match partiter.next() {
                    Some("name") => {
                        match partiter.next() {
                            Some(x) => {
                                match self.player {
                                    Some(_) => self.write_error("Already registered.")?,
                                    None => {
                                        let hash: u64 = random();
                                        println!("{:04X}", hash);
                                    }
                                }
                            }
                            None => self.write_error("No name provided.")?,
                        }
                    }
                    Some(x) => println!("{}", x),
                    None => {}
                }
            }
            Some(x) => self.write_error(format!("Unknown command \"{}\"\n", x))?,
            None => {}
        }

        Ok(())
    }
}
