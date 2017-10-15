use network::*;
use game::*;
use std::collections::HashSet;
use rand::{thread_rng, Rng};

// TODO import
macro_rules! durak_error {
    ($t:ident, $x:expr) => (DurakError::new(DurakErrorType::$t, $x))
}

pub trait GameRules {
    fn apply(&self, &mut GameState, &Vec<ClientHash>, GameAction) -> Result<GameState>;
}

#[derive(Clone)]
pub struct DefaultRules {
    cards_per_player: usize,
}

impl DefaultRules {
    pub fn new() -> DefaultRules {
        DefaultRules { cards_per_player: 5 }
    }
}

impl GameRules for DefaultRules {
    fn apply(
        &self,
        state: &mut GameState,
        players: &Vec<ClientHash>,
        action: GameAction,
    ) -> Result<GameState> {
        let mut rng = thread_rng();
        match action {
            GameAction::DealCards => {
                let mut cards = Vec::new();
                for suite in [Suite::Hearts, Suite::Diamonds, Suite::Clubs, Suite::Spades]
                    .into_iter()
                {
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
                rng.shuffle(cards.as_mut_slice());
                for player in players {
                    state.player_cards.insert(*player, HashSet::new());
                    for _ in 0..self.cards_per_player {
                        state.player_cards.get_mut(player).unwrap().insert(
                            cards
                                .pop()
                                .unwrap(),
                        );
                    }
                }
                state.card_stack = cards.clone();
                state.trump = state.card_stack.last().map(|x| x.suite.clone());
                Ok(state.clone())
            }
            GameAction::PutCard(card) => Err(durak_error!(Unimplemented, "")),
        }
    }
}
