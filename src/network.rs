use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::net::TcpListener;
use std::result;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use rand::random;
use game::*;

type ClientHash = u64;

#[derive(Debug)]
pub struct DuplexChannel<A: Clone, B: Clone> {
    tx: mpsc::Sender<A>,
    rx: mpsc::Receiver<B>,
}

pub struct Server {
    listener: TcpListener,
    channels: HashMap<ClientHash, DuplexChannel<Answer, Command>>,
    game: Game,
}

struct Player {
    name: String,
}

pub type Result<T> = result::Result<T, DurakError>;

#[derive(Debug)]
pub enum DurakError {
    IOError(io::Error),
    ChannelSendError(String),
    ChannelRecvError(String),
    ParserError(String),
}

#[derive(Debug, Clone)]
pub enum Command {
    Player(PlayerCommand),
}

#[derive(Debug, Clone)]
pub enum Answer {
    Nothing,
}

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Name(String),
}

impl Server {
    pub fn new<S: Into<String>>(address: S) -> Result<Server> {
        Ok(Server {
            listener: TcpListener::bind(address.into())?,
            channels: HashMap::new(),
            game: Game::new(),
        })
    }

    pub fn listen(&mut self) -> Result<()> {
        let listener = self.listener.try_clone()?;
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || for stream in listener.incoming() {
            let tx = tx.clone();
            let stream = stream.unwrap();
            let id: ClientHash = random();
            let (remote_channel, local_channel) = DuplexChannel::new();
            tx.send((id, remote_channel)).unwrap();
            let tx = local_channel.tx;
            thread::spawn(move || {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {
                            println!("line: {}", line);
                            match Command::parse(line) {
                                Ok(cmd) => {
                                    println!("send {:016X} {:?}", id, cmd);
                                    tx.send(cmd).unwrap();
                                }
                                Err(e) => {
                                    println!("error {:016X} {:?}", id, e);
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            });
            let rx = local_channel.rx;
            thread::spawn(move || for answer in rx {
                println!("{:?}", answer);
            });
        });

        loop {
            match rx.try_recv() {
                Ok((clienthash, channel)) => {
                    self.channels.insert(clienthash, channel);
                }
                Err(_) => {}
            }
            for (clienthash, channel) in &self.channels {
                match channel.try_recv() {
                    Ok(command) => println!("{:016X} {:?}", clienthash, command),
                    Err(_) => {}
                }
            }
        }
    }
}

impl From<io::Error> for DurakError {
    fn from(e: io::Error) -> DurakError {
        DurakError::IOError(e)
    }
}

impl From<mpsc::TryRecvError> for DurakError {
    fn from(e: mpsc::TryRecvError) -> DurakError {
        DurakError::ChannelRecvError(e.description().into())
    }
}

impl<T: Send> From<mpsc::SendError<T>> for DurakError {
    fn from(e: mpsc::SendError<T>) -> DurakError {
        DurakError::ChannelSendError(e.description().into())
    }
}


impl Command {
    pub fn parse<S: Into<String>>(line: S) -> Result<Command> {
        let line: String = line.into().trim().into();
        let mut parts = line.splitn(2, ' ');

        match parts.next() {
            Some("player") => {
                match parts.next() {
                    Some(tail) => Ok(Command::Player(PlayerCommand::parse(tail)?)),
                    None => Err(DurakError::ParserError("player tail".into())),
                }
            }
            Some(x) => Err(DurakError::ParserError(format!("unknown command {}", x))),
            None => Err(DurakError::ParserError("no command".into())),
        }
    }
}

impl PlayerCommand {
    pub fn parse<S: Into<String>>(line: S) -> Result<PlayerCommand> {
        let line: String = line.into().trim().into();
        let mut parts = line.splitn(2, ' ');

        match parts.next() {
            Some("name") => {
                match parts.next() {
                    Some(name) => Ok(PlayerCommand::Name(name.into())),
                    None => Err(DurakError::ParserError("player name tail".into())),
                }
            }
            Some(_) => Err(DurakError::ParserError("player command unknown".into())),
            None => Err(DurakError::ParserError("player no command".into())),
        }
    }
}

impl<A: Clone + Send, B: Clone> DuplexChannel<A, B> {
    pub fn new() -> (DuplexChannel<A, B>, DuplexChannel<B, A>) {
        let (txa, rxa) = mpsc::channel();
        let (txb, rxb) = mpsc::channel();
        (
            DuplexChannel { tx: txa, rx: rxb },
            DuplexChannel { tx: txb, rx: rxa },
        )
    }

    pub fn try_recv(&self) -> Result<B> {
        self.rx.try_recv().map_err(|e| e.into())
    }

    pub fn send(&self, t: A) -> Result<()> {
        self.tx.send(t).map_err(|e| e.into())
    }
}

impl<A: Clone, B: Clone> IntoIterator for DuplexChannel<A, B> {
    type Item = B;
    type IntoIter = mpsc::IntoIter<B>;

    fn into_iter(self) -> Self::IntoIter {
        self.rx.into_iter()
    }
}
