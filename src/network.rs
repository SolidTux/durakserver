use std::u64;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, ToSocketAddrs};
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

#[derive(Debug, Clone)]
pub enum DurakError {
    IOError(String),
    ChannelSendError(String),
    ChannelRecvError(String),
    ParserError(String),
    GameError(String),
    Unimplemented,
}

#[derive(Debug, Clone)]
pub enum Command {
    Player(PlayerCommand),
    Table(TableCommand),
    Game(GameCommand),
    Answer(Answer),
}

#[derive(Debug, Clone)]
pub enum Answer {
    PlayerList(HashMap<ClientHash, Player>),
    PlayerState(ClientHash, Player),
    TableList(HashMap<TableHash, Table>),
    Error(DurakError),
    Chat(ClientHash, String),
}

pub enum AnswerTarget {
    Direct,
    List(Vec<ClientHash>),
}

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Name(String),
    State,
    List,
}

#[derive(Debug, Clone)]
pub enum TableCommand {
    New(String),
    Join(TableHash),
    Chat(String),
    Leave,
    List,
}

#[derive(Debug, Clone)]
pub enum GameCommand {
    Start,
}

impl Server {
    pub fn new<S: ToSocketAddrs>(address: S) -> Result<Server> {
        Ok(Server {
            listener: TcpListener::bind(address)?,
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
                                    tx.send(Command::Answer(Answer::Error(e))).unwrap();
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
                            for (hash, player) in list {
                                writer
                                    .write_fmt(format_args!("\t{:016X} {}\n", hash, player.name))
                                    .unwrap();
                                writer.flush().unwrap();
                            }
                        }
                        Answer::PlayerState(hash, player) => {
                            writer
                                .write_fmt(format_args!("\thash  {:016X}\n", hash))
                                .unwrap();
                            writer
                                .write_fmt(format_args!("\tname  {}\n", player.name))
                                .unwrap();
                            match player.table {
                                Some(table) => {
                                    writer
                                        .write_fmt(format_args!("\ttable {:016X}\n", table))
                                        .unwrap()
                                }
                                None => {}
                            }
                            writer.flush().unwrap();
                        }
                        Answer::TableList(list) => {
                            for (tablehash, table) in list {
                                writer
                                    .write_fmt(format_args!(
                                        "\t{:016X} {} {} {} {} {}\n",
                                        tablehash,
                                        table.players.len(),
                                        table.min_players,
                                        table.max_players,
                                        table.state,
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
                        Answer::Chat(sender, message) => {
                            writer
                                .write_fmt(format_args!("chat {:016X} {}\n", sender, message))
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
                            Some((target, answer)) => {
                                match target {
                                    AnswerTarget::Direct => {
                                        match self.channels.get(&clienthash) {
                                            Some(ch) => ch.send(answer.clone()).unwrap(),
                                            None => {}
                                        }
                                    }
                                    AnswerTarget::List(targets) => {
                                        for target in targets {
                                            match self.channels.get(&target) {
                                                Some(ch) => ch.send(answer.clone()).unwrap(),
                                                None => {}
                                            }
                                        }
                                    }
                                }
                            }
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
                    None => Err(DurakError::ParserError(
                        "No player command specified.".into(),
                    )),
                }
            }
            Some("table") => {
                match parts.next() {
                    Some(tail) => Ok(Command::Table(TableCommand::parse(tail)?)),
                    None => Err(DurakError::ParserError(
                        "No table command specified.".into(),
                    )),
                }
            }
            Some("game") => {
                match parts.next() {
                    Some(tail) => Ok(Command::Game(GameCommand::parse(tail)?)),
                    None => Err(DurakError::ParserError("No game command specified.".into())),
                }
            }
            Some(x) => Err(DurakError::ParserError(format!("Unknown command {}.", x))),
            None => Err(DurakError::ParserError("No command specified.".into())),
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
                    None => Err(DurakError::ParserError("No name specified.".into())),
                }
            }
            Some("list") => Ok(PlayerCommand::List),
            Some("state") => Ok(PlayerCommand::State),
            Some(x) => Err(DurakError::ParserError(
                format!("Unknown player command {}.", x),
            )),
            None => Err(DurakError::ParserError(
                "No player command specified.".into(),
            )),
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
                    None => Err(DurakError::ParserError("No table name specified.".into())),
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
                    None => Err(DurakError::ParserError("No table hash specified.".into())),
                }
            }
            Some("list") => Ok(TableCommand::List),
            Some("leave") => Ok(TableCommand::Leave),
            Some("chat") => {
                match parts.next() {
                    Some(message) => Ok(TableCommand::Chat(message.into())),
                    None => Err(DurakError::ParserError("No message specified.".into())),
                }
            }
            Some(x) => Err(DurakError::ParserError(
                format!("Unknown table command {}.", x),
            )),
            None => Err(DurakError::ParserError(
                "No table command specified.".into(),
            )),
        }
    }
}

impl GameCommand {
    pub fn parse<S: Into<String>>(line: S) -> Result<GameCommand> {
        let line: String = line.into().trim().into();
        let mut parts = line.splitn(2, ' ');

        match parts.next() {
            Some("start") => Ok(GameCommand::Start),
            Some(x) => Err(DurakError::ParserError(
                format!("Unknown game command {}.", x),
            )),
            None => Err(DurakError::ParserError(
                "No table command specified.".into(),
            )),
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
            &DurakError::Unimplemented => "Unimplemented function.".into(),
        }
    }
}
