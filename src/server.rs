use std::io::Result;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use rand::random;
use game::*;


pub struct Server {
    listener: TcpListener,
    channels: HashMap<u16, mpsc::Sender<AnswerAction>>,
    players: HashMap<u64, Player>,
}

pub struct Client {
    id: u16,
    stream: TcpStream,
    tx: mpsc::Sender<SendAction>,
    rx: mpsc::Receiver<AnswerAction>,
    player: Option<u64>,
}

pub enum SendAction {
    AnswerChannel(u16, mpsc::Sender<AnswerAction>),
    AddPlayer(u64, String),
    GetPlayers(u16),
    Nothing,
}

pub enum AnswerAction {

}

impl Server {
    pub fn new(address: &'static str) -> Result<Server> {
        Ok(Server {
            listener: TcpListener::bind(address)?,
            channels: HashMap::new(),
            players: HashMap::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let listener = self.listener.try_clone().unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut id = 0;
            for stream in listener.incoming() {
                let (tx2, rx2) = mpsc::channel();
                tx.send(SendAction::AnswerChannel(id, tx2)).unwrap();
                let mut client = Client::new(id, stream.unwrap(), rx2, tx.clone());
                thread::spawn(move || client.handle());
                id = id + 1;
            }
        });

        for message in rx {
            match message {
                SendAction::AnswerChannel(id, channel) => {
                    self.channels.insert(id, channel);
                }
                SendAction::AddPlayer(hash, name) => {
                    self.players.insert(hash, Player::new(name));
                    println!("{} inserted", hash);
                }
                SendAction::GetPlayers(_) => {}
                SendAction::Nothing => {}
            }
        }

        Ok(())
    }
}

impl Client {
    pub fn new(
        id: u16,
        stream: TcpStream,
        rx: mpsc::Receiver<AnswerAction>,
        tx: mpsc::Sender<SendAction>,
    ) -> Client {
        Client {
            id: id,
            tx: tx,
            rx: rx,
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
                            Some(name) => {
                                match self.player {
                                    Some(_) => self.write_error("Already registered.")?,
                                    None => {
                                        let hash: u64 = random();
                                        self.player = Some(hash);
                                        self.tx
                                            .send(SendAction::AddPlayer(hash, String::from(name)))
                                            .unwrap();
                                    }
                                }
                            }
                            None => self.write_error("No name provided.")?,
                        }
                    }
                    Some("list") => {}
                    Some(_) => {}
                    None => {}
                }
            }
            Some(x) => self.write_error(format!("Unknown command \"{}\"\n", x))?,
            None => {}
        }

        Ok(())
    }
}
