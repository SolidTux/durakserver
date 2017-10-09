use game::*;
use network::*;
use std::collections::HashSet;
use rand::{thread_rng, Rng};

pub trait GameRules {
    fn apply(&self, &mut GameState, &Vec<ClientHash>, GameAction);
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
    fn apply(&self, state: &mut GameState, players: &Vec<ClientHash>, action: GameAction) {
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
            }
        }
    }
}
