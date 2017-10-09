use rand::random;
use std::collections::{HashMap, HashSet};
use std::fmt;
use network::*;
use rules::*;

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
    pub total_cards: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableState {
    Idle,
    Game,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Card {
    value: CardValue,
    suite: Suite,
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
                    None => Some((
                        AnswerTarget::Direct,
                        Answer::Error(
                            DurakError::GameError("Player not found.".into()),
                        ),
                    )),
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
                                    Some((
                                        AnswerTarget::Direct,
                                        Answer::Error(
                                            DurakError::GameError("Already joined a table.".into()),
                                        ),
                                    ))
                                }
                            } else {
                                Some((
                                    AnswerTarget::Direct,
                                    Answer::Error(DurakError::GameError(
                                        "Player not found. Please call \"player name\".".into(),
                                    )),
                                ))
                            }
                        } else {
                            Some((
                                AnswerTarget::Direct,
                                Answer::Error(
                                    DurakError::GameError("Unable to join table.".into()),
                                ),
                            ))
                        }
                    }
                    None => Some((
                        AnswerTarget::Direct,
                        Answer::Error(
                            DurakError::GameError("Table not found.".into()),
                        ),
                    )),
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
                            Some((
                                AnswerTarget::Direct,
                                Answer::Error(
                                    DurakError::GameError("Table not found.".into()),
                                ),
                            ))
                        }
                    } else {
                        Some((
                            AnswerTarget::Direct,
                            Answer::Error(
                                DurakError::GameError("No table joined.".into()),
                            ),
                        ))
                    }
                } else {
                    Some((
                        AnswerTarget::Direct,
                        Answer::Error(DurakError::GameError(
                            "Player not found. Please call \"player name\".".into(),
                        )),
                    ))
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
                                    None => Some((
                                        AnswerTarget::Direct,
                                        Answer::Error(
                                            DurakError::GameError("Table not found.".into()),
                                        ),
                                    )),
                                }
                            }
                            None => Some((
                                AnswerTarget::Direct,
                                Answer::Error(
                                    DurakError::GameError("No table joined yet.".into()),
                                ),
                            )),
                        }
                    }
                    None => Some((
                        AnswerTarget::Direct,
                        Answer::Error(
                            DurakError::GameError("Player not found.".into()),
                        ),
                    )),
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
                                        table.state = TableState::Game;
                                        table.game_state = Some(GameState::new());
                                        if let Some(ref mut state) = table.game_state {
                                            table.rules.apply(
                                                state,
                                                &self.players.keys().collect(),
                                                GameAction::DealCards,
                                            )
                                        };
                                        println!("{:?}", table.game_state);
                                        Some((
                                            AnswerTarget::Direct,
                                            Answer::Error(DurakError::Unimplemented),
                                        ))
                                    }
                                    None => Some((
                                        AnswerTarget::Direct,
                                        Answer::Error(
                                            DurakError::GameError("Table not found.".into()),
                                        ),
                                    )),
                                }
                            }
                            None => Some((
                                AnswerTarget::Direct,
                                Answer::Error(
                                    DurakError::GameError("No table joined.".into()),
                                ),
                            )),
                        }
                    }
                    None => Some((
                        AnswerTarget::Direct,
                        Answer::Error(
                            DurakError::GameError("Player not found.".into()),
                        ),
                    )),
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
        let mut cards = Vec::new();
        for suite in [Suite::Hearts, Suite::Diamonds, Suite::Clubs, Suite::Spades].into_iter() {
            for value in [
                CardValue::Number6,
                CardValue::Number7,
                CardValue::Number8,
                CardValue::Number9,
                CardValue::Number10,
                CardValue::Jack,
                CardValue::Queen,
                CardValue::Ace,
            ].into_iter()
            {
                cards.push(Card {
                    suite: suite.clone(),
                    value: value.clone(),
                });
            }
        }
        GameState {
            player_cards: HashMap::new(),
            total_cards: cards,
        }
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:?}", self.value, self.suite)
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

impl fmt::Debug for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CardValue::Number6 => write!(f, "6"),
            &CardValue::Number7 => write!(f, "7"),
            &CardValue::Number8 => write!(f, "8"),
            &CardValue::Number9 => write!(f, "9"),
            &CardValue::Number10 => write!(f, "10"),
            &CardValue::Jack => write!(f, "J"),
            &CardValue::Queen => write!(f, "Q"),
            &CardValue::Ace => write!(f, "A"),
        }
    }
}
