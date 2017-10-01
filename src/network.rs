use std::u64;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::net::TcpListener;
use std::result;
use std::sync::mpsc;
use std::thread;
use std::time;

use rand::random;
use game::*;

pub type ClientHash = u64;

#[derive(Debug)]
pub struct DuplexChannel<A, B> {
    tx: mpsc::Sender<A>,
    rx: mpsc::Receiver<B>,
}

pub struct Server {
    listener: TcpListener,
    channels: HashMap<ClientHash, DuplexChannel<Answer, Command>>,
    game: Game,
}

pub type Result<T> = result::Result<T, DurakError>;

#[derive(Debug)]
pub enum DurakError {
    IOError(String),
    ChannelSendError(String),
    ChannelRecvError(String),
    ParserError(String),
    GameError(String),
}

#[derive(Debug, Clone)]
pub enum Command {
    Player(PlayerCommand),
    Table(TableCommand),
}

#[derive(Debug)]
pub enum Answer {
    PlayerList(Vec<Player>),
    TableList(HashMap<TableHash, Table>),
    Error(DurakError),
}

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Name(String),
    List,
}

#[derive(Debug, Clone)]
pub enum TableCommand {
    New(String),
    Join(TableHash),
    List,
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
            let local_stream = stream.try_clone().unwrap();
            thread::spawn(move || {
                let mut reader = BufReader::new(local_stream);
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {
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
            let local_stream = stream.try_clone().unwrap();
            thread::spawn(move || {
                let mut writer = BufWriter::new(local_stream);
                for answer in rx {
                    match answer {
                        Answer::PlayerList(list) => {
                            for player in list {
                                writer
                                    .write_fmt(format_args!("\t{}\n", player.name))
                                    .unwrap();
                                writer.flush().unwrap();
                            }
                        }
                        Answer::TableList(list) => {
                            for (tablehash, table) in list {
                                writer
                                    .write_fmt(format_args!(
                                        "\t{:016X} {} {} {} {}\n",
                                        tablehash,
                                        table.players.len(),
                                        table.min_players,
                                        table.max_players,
                                        table.name
                                    ))
                                    .unwrap();
                                writer.flush().unwrap();
                            }
                        }
                        Answer::Error(error) => {
                            writer
                                .write_fmt(format_args!("ERROR {}\n", error.to_string()))
                                .unwrap();
                            writer.flush().unwrap();
                        }
                    }
                }
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
                    Ok(command) => {
                        match self.game.handle_command(clienthash, command) {
                            Some(answer) => channel.send(answer).unwrap(),
                            None => {}
                        }
                    }
                    Err(_) => {}
                }
            }
            thread::sleep(time::Duration::from_millis(1));
        }
    }
}

impl From<io::Error> for DurakError {
    fn from(e: io::Error) -> DurakError {
        DurakError::IOError(e.description().into())
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
            Some("table") => {
                match parts.next() {
                    Some(tail) => Ok(Command::Table(TableCommand::parse(tail)?)),
                    None => Err(DurakError::ParserError("table tail".into())),
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
                    Some(name) => Ok(PlayerCommand::Name(name.trim().into())),
                    None => Err(DurakError::ParserError("player name tail".into())),
                }
            }
            Some("list") => Ok(PlayerCommand::List),
            Some(_) => Err(DurakError::ParserError("player command unknown".into())),
            None => Err(DurakError::ParserError("player no command".into())),
        }
    }
}

impl TableCommand {
    pub fn parse<S: Into<String>>(line: S) -> Result<TableCommand> {
        let line: String = line.into().trim().into();
        let mut parts = line.splitn(2, ' ');

        match parts.next() {
            Some("new") => {
                match parts.next() {
                    Some(name) => Ok(TableCommand::New(name.trim().into())),
                    None => Err(DurakError::ParserError("table new tail".into())),
                }
            }
            Some("join") => {
                match parts.next() {
                    Some(id) => {
                        match TableHash::from_str_radix(id, 16) {
                            Ok(tablehash) => Ok(TableCommand::Join(tablehash)),
                            Err(_) => Err(DurakError::ParserError(
                                "Could not parse table hash.".into(),
                            )),
                        }
                    }
                    None => Err(DurakError::ParserError("player name tail".into())),
                }
            }
            Some("list") => Ok(TableCommand::List),
            Some(_) => Err(DurakError::ParserError("player command unknown".into())),
            None => Err(DurakError::ParserError("player no command".into())),
        }
    }
}

impl<A: Send, B: Clone> DuplexChannel<A, B> {
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

impl<A, B> IntoIterator for DuplexChannel<A, B> {
    type Item = B;
    type IntoIter = mpsc::IntoIter<B>;

    fn into_iter(self) -> Self::IntoIter {
        self.rx.into_iter()
    }
}

impl ToString for DurakError {
    fn to_string(&self) -> String {
        match self {
            &DurakError::IOError(ref error) => error.clone(),
            &DurakError::ChannelSendError(ref error) => error.clone(),
            &DurakError::ChannelRecvError(ref error) => error.clone(),
            &DurakError::ParserError(ref error) => error.clone(),
            &DurakError::GameError(ref error) => error.clone(),
        }
    }
}
