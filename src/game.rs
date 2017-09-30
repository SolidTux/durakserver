pub struct Player {
    name: String,
}

impl Player {
    pub fn new<S: Into<String>>(name: S) -> Player {
        Player { name: name.into() }
    }
}
