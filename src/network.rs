use std::u64;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, ToSocketAddrs};
use std::num;
use std::process;
use std::result;
use std::sync::mpsc;
use std::thread;
use std::time;

use rand::random;
use game::*;
use rules::*;

macro_rules! durak_error {
    ($t:ident, $x:expr) => (DurakError::new(DurakErrorType::$t, $x))
}

pub type ClientHash = u64;

#[derive(Debug)]
pub struct DuplexChannel<A, B> {
    tx: mpsc::Sender<A>,
    rx: mpsc::Receiver<B>,
}

pub struct Server<T: GameRules + Clone + Send> {
    listener: TcpListener,
    channels: HashMap<ClientHash, DuplexChannel<Answer<T>, Command<T>>>,
    room: Room<T>,
}

pub type Result<T> = result::Result<T, DurakError>;

#[derive(Debug, Clone)]
pub struct DurakError {
    error_type: DurakErrorType,
    message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DurakErrorType {
    IOError,
    ChannelSendError,
    ChannelRecvError,
    ParserError,
    GameError,
    Unimplemented,
}

#[derive(Clone, Debug)]
pub enum Command<T: GameRules + Clone + Send> {
    Player(PlayerCommand),
    Table(TableCommand),
    Game(GameCommand),
    Answer(Answer<T>),
    Quit,
}

#[derive(Clone, Debug)]
pub enum Answer<T: GameRules + Clone + Send> {
    PlayerList(HashMap<ClientHash, Player>),
    PlayerState(ClientHash, Player),
    TableList(HashMap<TableHash, Table<T>>),
    Error(DurakError),
    Chat(ClientHash, String),
    GameState(GameState),
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
    State,
    Action(GameAction),
}

#[derive(Debug, Clone)]
pub enum GameAction {
    DealCards,
    PutCard(Card, Option<usize>),
}

impl<T: GameRules + Debug + Clone + Send + 'static> Server<T> {
    pub fn new<S: ToSocketAddrs>(address: S, rules: T) -> Result<Server<T>> {
        Ok(Server {
            listener: TcpListener::bind(address)?,
            channels: HashMap::new(),
            room: Room::new(rules),
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
                                        table.get_state(),
                                        table.name
                                    ))
                                    .unwrap();
                            }
                            writer.write_fmt(format_args!("\n")).unwrap();
                        }
                        Answer::Error(error) => {
                            writer
                                .write_fmt(format_args!("ERROR {}\n", error.to_string()))
                                .unwrap();
                        }
                        Answer::Chat(sender, message) => {
                            writer
                                .write_fmt(format_args!("chat {:016X} {}\n", sender, message))
                                .unwrap();
                        }
                        Answer::GameState(gamestate) => {
                            writer
                                .write_fmt(format_args!(
                                    "cards {}\n",
                                    gamestate.player_cards.get(&id).unwrap().iter().fold(
                                        String::new(),
                                        |acc, x| {
                                            if acc.len() == 0 {
                                                format!("{}", x)
                                            } else {
                                                format!("{} {}", acc, x)
                                            }
                                        },
                                    )
                                ))
                                .unwrap();
                            writer
                                .write_fmt(format_args!("trump {}\n", gamestate))
                                .unwrap();
                            writer
                                .write_fmt(format_args!(
                                    "table {}\n",
                                    gamestate.table_stacks.iter().fold(String::new(), |acc,
                                     &(ref x,
                                       ref y)| {
                                        if acc.len() == 0 {
                                            match y {
                                                &Some(ref c) => format!("{}/{}", x, c),
                                                &None => format!("{}/--", x),
                                            }
                                        } else {
                                            match y {
                                                &Some(ref c) => format!("{} {}/{}", acc, x, c),
                                                &None => format!("{} {}/--", acc, x),
                                            }
                                        }
                                    })
                                ))
                                .unwrap();
                            if let Some(p) = gamestate.target_player {
                                writer
                                    .write_fmt(format_args!("target {:016X}\n", p))
                                    .unwrap();
                            }

                        }
                    }
                    let _ = writer.flush();
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
                    Ok(Command::Quit) => process::exit(0),
                    Ok(command) => {
                        println!("{:016X}: {:?}", clienthash, command);
                        match self.room.handle_command(clienthash, command) {
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

impl DurakError {
    pub fn new<S: Into<String>>(t: DurakErrorType, m: S) -> DurakError {
        DurakError {
            error_type: t,
            message: m.into(),
        }
    }
}

impl From<io::Error> for DurakError {
    fn from(e: io::Error) -> DurakError {
        DurakError::new(DurakErrorType::IOError, e.description())
    }
}

impl From<mpsc::TryRecvError> for DurakError {
    fn from(e: mpsc::TryRecvError) -> DurakError {
        DurakError::new(DurakErrorType::ChannelRecvError, e.description())
    }
}

impl From<num::ParseIntError> for DurakError {
    fn from(e: num::ParseIntError) -> DurakError {
        DurakError::new(DurakErrorType::ParserError, e.description())
    }
}

impl<T: Send> From<mpsc::SendError<T>> for DurakError {
    fn from(e: mpsc::SendError<T>) -> DurakError {
        DurakError::new(DurakErrorType::ChannelSendError, e.description())
    }
}


impl<T: GameRules + Clone + Send> Command<T> {
    pub fn parse<S: Into<String>>(line: S) -> Result<Command<T>> {
        let line: String = line.into().trim().into();
        let mut parts = line.splitn(2, ' ');

        match parts.next() {
            Some("quit") => Ok(Command::Quit),
            Some("player") => {
                match parts.next() {
                    Some(tail) => Ok(Command::Player(PlayerCommand::parse(tail)?)),
                    None => Err(durak_error!(ParserError, "No player command specified.")),
                }
            }
            Some("table") => {
                match parts.next() {
                    Some(tail) => Ok(Command::Table(TableCommand::parse(tail)?)),
                    None => Err(durak_error!(ParserError, "No table command specified.")),
                }
            }
            Some("game") => {
                match parts.next() {
                    Some(tail) => Ok(Command::Game(GameCommand::parse(tail)?)),
                    None => Err(durak_error!(ParserError, "No game command specified.")),
                }
            }
            Some(x) => Err(durak_error!(ParserError, format!("Unknown command {}.", x))),
            None => Err(durak_error!(ParserError, "No command specified.")),
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
                    None => Err(durak_error!(ParserError, "No name specified.")),
                }
            }
            Some("list") => Ok(PlayerCommand::List),
            Some("state") => Ok(PlayerCommand::State),
            Some(x) => Err(durak_error!(
                ParserError,
                format!("Unknown player command {}.", x)
            )),
            None => Err(durak_error!(ParserError, "No player command specified.")),
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
                    None => Err(durak_error!(ParserError, "No table name specified.")),
                }
            }
            Some("join") => {
                match parts.next() {
                    Some(id) => {
                        match TableHash::from_str_radix(id, 16) {
                            Ok(tablehash) => Ok(TableCommand::Join(tablehash)),
                            Err(_) => Err(durak_error!(ParserError, "Could not parse table hash.")),
                        }
                    }
                    None => Err(durak_error!(ParserError, "No table hash specified.")),
                }
            }
            Some("list") => Ok(TableCommand::List),
            Some("leave") => Ok(TableCommand::Leave),
            Some("chat") => {
                match parts.next() {
                    Some(message) => Ok(TableCommand::Chat(message.into())),
                    None => Err(durak_error!(ParserError, "No message specified.")),
                }
            }
            Some(x) => Err(durak_error!(
                ParserError,
                format!("Unknown table command {}.", x)
            )),
            None => Err(durak_error!(ParserError, "No table command specified.")),
        }
    }
}

impl GameCommand {
    pub fn parse<S: Into<String>>(line: S) -> Result<GameCommand> {
        let line: String = line.into().trim().into();
        let mut parts = line.splitn(2, ' ');

        match parts.next() {
            Some("start") => Ok(GameCommand::Start),
            Some("state") => Ok(GameCommand::State),
            Some("put") => {
                match parts.next() {
                    Some(tail) => {
                        let mut parts = tail.split(' ');
                        match parts.next() {
                            Some(card) => {
                                match parts.next() {
                                    Some(tail) => Ok(GameCommand::Action(GameAction::PutCard(
                                        card.parse()?,
                                        Some(tail.parse()?),
                                    ))),
                                    None => Ok(GameCommand::Action(
                                        GameAction::PutCard(card.parse()?, None),
                                    )),
                                }
                            }
                            None => Err(durak_error!(ParserError, "No card specified.")),
                        }
                    }
                    None => Err(durak_error!(ParserError, "No card specified.")),
                }
            }
            Some(x) => Err(durak_error!(
                ParserError,
                format!("Unknown game command {}.", x)
            )),
            None => Err(durak_error!(ParserError, "No table command specified.")),
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
        if self.error_type == DurakErrorType::Unimplemented {
            "Unimplemented feature.".into()
        } else {
            self.message.clone()
        }
    }
}
