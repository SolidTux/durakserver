use network::*;
use game::*;
use std::collections::HashSet;
use rand::{thread_rng, Rng};

// TODO import
macro_rules! durak_error {
    ($t:ident, $x:expr) => (DurakError::new(DurakErrorType::$t, $x))
}

pub trait GameRules {
    fn apply(&self, &ClientHash, &mut GameState, &Vec<ClientHash>, GameAction)
        -> Result<GameState>;
}

#[derive(Clone, Debug)]
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
        origin: &ClientHash,
        state: &mut GameState,
        players: &Vec<ClientHash>,
        action: GameAction,
    ) -> Result<GameState> {
        println!("Action: {:?}", action);
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
                let mut p = players.clone();
                rng.shuffle(p.as_mut_slice());
                state.target_player = p.get(0).map(|x| x.clone()); //TODO
            }
            GameAction::PutCard(card, stack_ind) => {
                // TODO test if player is allowed to put card
                match state.player_cards.get_mut(origin) {
                    Some(cards) => {
                        let is_target = match state.target_player {
                            Some(player) => player == *origin,
                            None => return Err(durak_error!(GameError, "No target player.")),
                        };
                        match stack_ind {
                            Some(ind) => {
                                if !is_target {
                                    return Err(durak_error!(
                                        GameError,
                                        "Only target player can defend."
                                    ));
                                }
                                match state.table_stacks.get_mut(ind) {
                                    Some(stack) => {
                                        if let (a, None) = stack.clone() {
                                            if !cards.remove(&card) {
                                                return Err(
                                                    durak_error!(GameError, "Card not found."),
                                                );
                                            }
                                            *stack = (a, Some(card.clone()));
                                        } else {
                                            return Err(durak_error!(
                                                GameError,
                                                "Card already defended."
                                            ));
                                        }
                                    }
                                    None => return Err(durak_error!(GameError, "Stack not found.")),
                                }
                            }
                            None => {
                                if is_target {
                                    return Err(durak_error!(
                                        GameError,
                                        "Defending player cannot start a new stack."
                                    ));
                                }
                                state.table_stacks.push((card.clone(), None));
                            }
                        }
                    }
                    None => return Err(durak_error!(GameError, "Player not found.")),
                }
            }
        }
        Ok(state.clone())
    }
}
