use std::collections::HashMap;
use network::*;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
}

pub struct Game {
    players: HashMap<ClientHash, Player>,
}

impl Player {
    pub fn new() -> Player {
        Player { name: String::new() }
    }
}

impl Game {
    pub fn new() -> Game {
        Game { players: HashMap::new() }
    }

    pub fn handle_command(&mut self, client: &ClientHash, command: Command) -> Option<Answer> {
        match command {
            Command::Player(PlayerCommand::Name(name)) => {
                self.players.entry(*client).or_insert(Player::new()).name = name;
                None
            }
            Command::Player(PlayerCommand::List) => Some(Answer::PlayerList(
                self.players.values().cloned().collect(),
            )),
        }
    }
}
