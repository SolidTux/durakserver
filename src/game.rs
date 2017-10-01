use rand::random;
use std::collections::HashMap;
use network::*;

pub type TableHash = u64;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    cards: Vec<Card>,
    table: Option<TableHash>,
}

pub struct Game {
    players: HashMap<ClientHash, Player>,
    tables: HashMap<TableHash, Table>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub players: HashMap<ClientHash, Player>,
    pub trump: Option<Suite>,
    pub max_players: u8,
    pub min_players: u8,
    pub state: TableState,
    pub draw_stack: Vec<Card>,
    pub table_stacks: Vec<(Card, Option<Card>)>,
}

#[derive(Debug, Clone)]
pub enum TableState {
    Idle,
    Game,
}

#[derive(Debug, Clone)]
pub struct Card {
    value: CardValue,
    suite: Suite,
}

#[derive(Debug, Clone)]
pub enum CardValue {
    Number6,
    Number7,
    Number8,
    Number9,
    Number10,
    Jack,
    Queen,
    Ace,
}

#[derive(Debug, Clone)]
pub enum Suite {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Player {
    pub fn new() -> Player {
        Player {
            name: String::new(),
            cards: Vec::new(),
            table: None,
        }
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: HashMap::new(),
            tables: HashMap::new(),
        }
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
            Command::Table(TableCommand::New(name)) => {
                self.tables.insert(random(), Table::new(name));
                None
            }
            Command::Table(TableCommand::List) => Some(Answer::TableList(self.tables.clone())),
        }
    }
}

impl Table {
    pub fn new<S: Into<String>>(name: S) -> Table {
        Table {
            name: name.into(),
            players: HashMap::new(),
            trump: None,
            max_players: 6,
            min_players: 2,
            state: TableState::Idle,
            draw_stack: Vec::new(),
            table_stacks: Vec::new(),
        }
    }
}
