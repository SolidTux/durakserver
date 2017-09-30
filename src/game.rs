use std::io::{Result, Error, ErrorKind};
use rand::random;

#[derive(Clone)]
pub struct Player {
    name: String,
}

#[derive(Clone)]
pub enum TableState {
    Idle,
    Full,
    Game,
}

#[derive(Clone)]
pub struct Table {
    max_players: usize,
    pub name: String,
    pub state: TableState,
    players: Vec<u64>,
}

impl Player {
    pub fn new<S: Into<String>>(name: S) -> Player {
        Player { name: name.into() }
    }
}

impl Table {
    pub fn new<S: Into<String>>(name: S) -> Table {
        Table {
            max_players: 4,
            name: name.into(),
            state: TableState::Idle,
            players: Vec::new(),
        }
    }

    pub fn add_player(&mut self, hash: u64) -> Result<()> {
        // TODO test, if already joined or full
        if self.players.len() < self.max_players {
            self.players.push(hash);
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Table is full"))
        }
    }
}
