use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpListener;
use std::result;
use std::thread;

pub struct Server {
    listener: TcpListener,
}

type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    ParserError(String),
}

#[derive(Debug)]
enum Command {
    Player(PlayerCommand),
}

#[derive(Debug)]
enum PlayerCommand {
    Name(String),
}

impl Server {
    pub fn new<S: Into<String>>(address: S) -> Result<Server> {
        Ok(Server { listener: TcpListener::bind(address.into())? })
    }

    pub fn listen(&self) -> Result<()> {
        for stream in self.listener.incoming() {
            let stream = stream?;
            thread::spawn(|| {
                let mut reader = BufReader::new(stream);
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => {
                            break;
                        }
                        Ok(_) => {
                            let cmd = Command::parse(line);
                            println!("{:?}", cmd);
                        }
                        Err(_) => {
                            println!("Error while reading message.");
                            break;
                        }
                    }
                }
            });
        }
        Ok(())
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IOError(e)
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
                    None => Err(Error::ParserError("player tail".into())),
                }
            }
            Some(x) => Err(Error::ParserError(format!("unknown command {}", x))),
            None => Err(Error::ParserError("no command".into())),
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
                    None => Err(Error::ParserError("player name tail".into())),
                }
            }
            Some(_) => Err(Error::ParserError("player command unknown".into())),
            None => Err(Error::ParserError("player no command".into())),
        }
    }
}
