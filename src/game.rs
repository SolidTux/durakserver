use network::*;

pub struct Game {}

impl Game {
    pub fn new() -> Game {
        Game {}
    }

    pub fn handle_command(cmd: Command) -> Option<Answer> {
        println!("{:?}", cmd);

        None
    }
}
