use game::*;
use network::*;

pub trait GameRules {
    fn apply(&self, &mut GameState, GameAction);
}

#[derive(Clone)]
pub struct DefaultRules {}

impl DefaultRules {
    pub fn new() -> DefaultRules {
        DefaultRules {}
    }
}

impl GameRules for DefaultRules {
    fn apply(&self, state: &mut GameState, action: GameAction) {
        match action {
            GameAction::DrawCards => {}
        }
    }
}
