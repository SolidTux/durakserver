use std::io::Result;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::u64;
use rand::random;
use game::*;


pub struct Server {
    listener: TcpListener,
    channels: HashMap<u16, mpsc::Sender<AnswerAction>>,
    players: HashMap<u64, Player>,
    tables: Vec<Table>,
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
    AddTable(String),
    ListTables(u16),
    GetPlayer(u16, u64),
}

pub enum AnswerAction {
    Player(Option<Player>),
    TableList(Vec<Table>),
}

impl Server {
    pub fn new(address: &'static str) -> Result<Server> {
        Ok(Server {
            listener: TcpListener::bind(address)?,
            channels: HashMap::new(),
            players: HashMap::new(),
            tables: Vec::new(),
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
                }
                SendAction::AddTable(name) => {
                    self.tables.push(Table::new(name));
                }
                SendAction::ListTables(id) => {
                    let channel = self.channels.get(&id).unwrap();
                    channel
                        .send(AnswerAction::TableList(self.tables.clone()))
                        .unwrap();
                }
                SendAction::GetPlayer(id, hash) => {
                    let channel = self.channels.get(&id).unwrap();
                    match self.players.get(&hash) {
                        Some(player) => {
                            channel
                                .send(AnswerAction::Player(Some(player.clone())))
                                .unwrap();
                        }
                        None => {
                            channel.send(AnswerAction::Player(None)).unwrap();
                        }
                    }
                }
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
        writer.write_fmt(format_args!("{}\n", message.into()))?;
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
                                        self.write_message(format!("Your hash is {:08X}.", hash))?
                                    }
                                }
                            }
                            None => self.write_error("No name provided.")?,
                        }
                    }
                    Some(_) => {}
                    None => {}
                }
            }
            Some(command) => {
                match self.player {
                    Some(hash) => {
                        match command {
                            "table" => {
                                match partiter.next() {
                                    Some("new") => {
                                        match partiter.next() {
                                            Some(name) => {
                                                self.tx
                                                    .send(SendAction::AddTable(String::from(name)))
                                                    .unwrap();
                                            }
                                            None => self.write_error("No name provided.")?,
                                        }
                                    }
                                    Some("list") => {
                                        self.tx.send(SendAction::ListTables(self.id)).unwrap();
                                        match self.rx.recv() {
                                            Ok(AnswerAction::TableList(tables)) => {
                                                for table in tables {
                                                    self.write_message(
                                                        format!("{:08X} {}", table.id, table.name),
                                                    )?;
                                                }
                                            }
                                            Ok(_) => {}
                                            Err(_) => {}
                                        }
                                    }
                                    Some(_) => {}
                                    None => {}
                                }
                            }
                            x => self.write_error(format!("Unknown command \"{}\".", x))?,
                        }
                    }
                    None => {
                        self.write_error(
                            "Please register with the \"player name\" command.",
                        )?
                    }
                }
            }
            None => {}
        }

        Ok(())
    }
}
