use rand::random;
use std::collections::{HashMap, HashSet};
use std::fmt;
use network::*;
use rules::*;

macro_rules! direct_error {
    ($t:ident, $x: expr) => (Some((AnswerTarget::Direct,
        Answer::Error(DurakError::new(DurakErrorType::$t, $x)))));
}

pub type TableHash = u64;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub cards: Vec<Card>,
    pub table: Option<TableHash>,
}

pub struct Room<T: GameRules + Clone + Send> {
    players: HashMap<ClientHash, Player>,
    tables: HashMap<TableHash, Table<T>>,
    rules: T,
}

#[derive(Clone)]
pub struct Table<T: GameRules + Clone + Send> {
    pub name: String,
    pub players: Vec<ClientHash>,
    pub trump: Option<Suite>,
    pub max_players: usize,
    pub min_players: usize,
    pub state: TableState,
    game_state: Option<GameState>,
    rules: T,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub player_cards: HashMap<ClientHash, HashSet<Card>>,
    pub card_stack: Vec<Card>,
    pub trump: Option<Suite>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableState {
    Idle,
    Game,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Card {
    pub value: CardValue,
    pub suite: Suite,
}

#[derive(Clone, PartialEq, Eq, Hash)]
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

#[derive(Clone, PartialEq, Eq, Hash)]
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

impl fmt::Display for TableState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &TableState::Idle => write!(f, "Idle"),
            &TableState::Game => write!(f, "Game"),
        }
    }
}

impl<T: GameRules + Clone + Send> Room<T> {
    pub fn new(rules: T) -> Room<T> {
        Room {
            players: HashMap::new(),
            tables: HashMap::new(),
            rules: rules,
        }
    }

    pub fn handle_command(
        &mut self,
        client: &ClientHash,
        command: Command<T>,
    ) -> Option<(AnswerTarget, Answer<T>)> {
        match command {
            Command::Player(PlayerCommand::Name(name)) => {
                self.players.entry(*client).or_insert(Player::new()).name = name;
                None
            }
            Command::Player(PlayerCommand::List) => {
                Some((
                    AnswerTarget::Direct,
                    Answer::PlayerList(self.players.clone()),
                ))
            }
            Command::Player(PlayerCommand::State) => {
                match self.players.get(client) {
                    Some(player) => Some((
                        AnswerTarget::Direct,
                        Answer::PlayerState(*client, player.clone()),
                    )),
                    None => direct_error!(GameError, "Player not found."),
                }
            }
            Command::Table(tablecommand) => self.handle_table_command(client, tablecommand),
            Command::Game(gamecommand) => self.handle_game_command(client, gamecommand),
            Command::Answer(answer) => Some((AnswerTarget::Direct, answer)),
        }
    }

    fn handle_table_command(
        &mut self,
        client: &ClientHash,
        command: TableCommand,
    ) -> Option<(AnswerTarget, Answer<T>)> {
        match command {
            TableCommand::New(name) => {
                self.tables.insert(
                    random(),
                    Table::new(name, self.rules.clone()),
                );
                None
            }
            TableCommand::List => Some((
                AnswerTarget::Direct,
                Answer::TableList(self.tables.clone()),
            )),
            TableCommand::Join(tablehash) => {
                match self.tables.get_mut(&tablehash) {
                    Some(table) => {
                        if (table.state == TableState::Idle) &&
                            (table.players.len() < table.max_players)
                        {
                            if let Some(player) = self.players.get_mut(client) {
                                if let None = player.table {
                                    player.table = Some(tablehash);
                                    table.players.push(*client);
                                    None
                                } else {
                                    direct_error!(GameError, "Already joined a table.")
                                }
                            } else {
                                direct_error!(
                                    GameError,
                                    "Player not found. Please call \"player name\"."
                                )
                            }
                        } else {
                            direct_error!(GameError, "Unable to join table.")
                        }
                    }
                    None => direct_error!(GameError, "Table not found."),
                }
            }
            TableCommand::Leave => {
                if let Some(player) = self.players.get_mut(client) {
                    if let Some(tablehash) = player.table {
                        player.table = None;
                        if let Some(table) = self.tables.get_mut(&tablehash) {
                            table.players.retain(|&x| x != *client);
                            None
                        } else {
                            direct_error!(GameError, "Table not found.")
                        }
                    } else {
                        direct_error!(GameError, "No table joined.")
                    }
                } else {
                    direct_error!(GameError, "Player not found. Please call \"player name\".")
                }
            }
            TableCommand::Chat(message) => {
                match self.players.get(client) {
                    Some(player) => {
                        match player.table {
                            Some(tablehash) => {
                                match self.tables.get(&tablehash) {
                                    Some(table) => Some((
                                        AnswerTarget::List(table.players.clone()),
                                        Answer::Chat(*client, message),
                                    )),
                                    None => direct_error!(GameError, "Table not found."),
                                }
                            }
                            None => direct_error!(GameError, "No table joined yet."),
                        }
                    }
                    None => direct_error!(GameError, "Player not found."),
                }
            }
        }
    }

    fn handle_game_command(
        &mut self,
        client: &ClientHash,
        command: GameCommand,
    ) -> Option<(AnswerTarget, Answer<T>)> {
        match command {
            GameCommand::Start => {
                match self.players.get(client) {
                    Some(player) => {
                        match player.table {
                            Some(tablehash) => {
                                match self.tables.get_mut(&tablehash) {
                                    Some(table) => {
                                        if table.players.len() >= table.min_players {
                                            table.state = TableState::Game;
                                            table.game_state = Some(GameState::new());
                                            match table.game_state {
                                                Some(ref mut state) => {
                                                    table.rules.apply(
                                                        state,
                                                        &table.players,
                                                        GameAction::DealCards,
                                                    );
                                                    println!("{:?}", state);
                                                    Some((
                                                        AnswerTarget::List(table.players.clone()),
                                                        Answer::GameState(state.clone()),
                                                    ))
                                                }
                                                None => {
                                                    direct_error!(
                                                        GameError,
                                                        "Unable to start game."
                                                    )
                                                }
                                            }
                                        } else {
                                            direct_error!(GameError, "Not enough players.")
                                        }
                                    }
                                    None => direct_error!(GameError, "Table not found."),
                                }
                            }
                            None => direct_error!(GameError, "No table joined."),
                        }
                    }
                    None => direct_error!(GameError, "Player not found."),
                }
            }
            GameCommand::State => {
                match self.players.get(client) {
                    Some(player) => {
                        match player.table {
                            Some(tablehash) => {
                                match self.tables.get_mut(&tablehash) {
                                    Some(table) => {
                                        match table.game_state {
                                            Some(ref state) => Some((
                                                AnswerTarget::Direct,
                                                Answer::GameState(state.clone()),
                                            )),
                                            None => direct_error!(GameError, "No game running."),
                                        }
                                    }
                                    None => direct_error!(GameError, "Table not found."),
                                }
                            }
                            None => direct_error!(GameError, "No table joined."),
                        }
                    }
                    None => direct_error!(GameError, "Player not found."),
                }
            }
        }
    }
}

impl<T: GameRules + Clone + Send> Table<T> {
    pub fn new<S: Into<String>>(name: S, rules: T) -> Table<T> {
        Table {
            name: name.into(),
            players: Vec::new(),
            trump: None,
            max_players: 6,
            min_players: 2,
            state: TableState::Idle,
            game_state: None,
            rules: rules,
        }
    }
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            player_cards: HashMap::new(),
            card_stack: Vec::new(),
            trump: None,
        }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.card_stack.last() {
            Some(card) => write!(f, "{}", card),
            None => {
                match self.trump {
                    Some(ref suite) => write!(f, "-{}", suite),
                    None => write!(f, "--"),
                }
            }
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.value, self.suite)
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:?}", self.value, self.suite)
    }
}

impl fmt::Display for Suite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Suite::Diamonds => write!(f, "1"),
            &Suite::Hearts => write!(f, "2"),
            &Suite::Spades => write!(f, "3"),
            &Suite::Clubs => write!(f, "4"),
        }
    }
}

impl fmt::Debug for Suite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Suite::Hearts => write!(f, "♥"),
            &Suite::Diamonds => write!(f, "♦"),
            &Suite::Clubs => write!(f, "♣"),
            &Suite::Spades => write!(f, "♠"),
        }
    }
}

impl fmt::Display for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CardValue::Number6 => write!(f, "6"),
            &CardValue::Number7 => write!(f, "7"),
            &CardValue::Number8 => write!(f, "8"),
            &CardValue::Number9 => write!(f, "9"),
            &CardValue::Number10 => write!(f, "0"),
            &CardValue::Jack => write!(f, "J"),
            &CardValue::Queen => write!(f, "Q"),
            &CardValue::Ace => write!(f, "A"),
        }
    }
}
