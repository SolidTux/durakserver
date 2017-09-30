use rand::random;

#[derive(Clone)]
pub struct Player {
    name: String,
}

#[derive(Clone)]
pub struct Table {
    pub id: u64,
    pub name: String,
    players: Vec<Player>,
}

impl Player {
    pub fn new<S: Into<String>>(name: S) -> Player {
        Player { name: name.into() }
    }
}

impl Table {
    pub fn new<S: Into<String>>(name: S) -> Table {
        Table {
            id: random(),
            name: name.into(),
            players: Vec::new(),
        }
    }
}
