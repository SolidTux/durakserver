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

                // TODO
                state.target_player = players.get(1).cloned();
                state.attack_player = players.get(0).cloned();
                state.neighbor_player = players.get(2).cloned();
            }
            GameAction::PutCard(card, stack_ind) => {
                let target = match state.target_player {
                    Some(player) => player,
                    None => return Err(durak_error!(GameError, "No target player.")),
                };
                let attack = match state.attack_player {
                    Some(player) => player,
                    None => return Err(durak_error!(GameError, "No attacking player.")),
                };
                let neighbor = match state.neighbor_player {
                    Some(player) => player,
                    None => return Err(durak_error!(GameError, "No attacking player.")),
                };
                let target_num_cards = match state.player_cards.clone().get(&target) {
                    Some(cards) => cards.len(),
                    None => return Err(durak_error!(GameError, "Target player cards not found.")),
                };
                match state.player_cards.get_mut(origin) {
                    Some(cards) => {
                        match stack_ind {
                            Some(ind) => {
                                if !(target == *origin) {
                                    return Err(durak_error!(
                                        GameError,
                                        "Only target player can defend."
                                    ));
                                }
                                match state.table_stacks.get_mut(ind) {
                                    Some(stack) => {
                                        if let (a, None) = stack.clone() {
                                            match state.trump {
                                                Some(ref trump) => {
                                                    match card.better_as(a.clone(), trump.clone()) {
                                                        Some(false) => {
                                                            return Err(durak_error!(
                                                                GameError,
                                                                "Defending is only possible with better card."
                                                            ))
                                                        }
                                                        Some(true) => {}
                                                        None => {
                                                            return Err(durak_error!(
                                                                GameError,
                                                                "Defending is only possible with matching card."
                                                            ))
                                                        }
                                                    }
                                                }
                                                None => {
                                                    return Err(durak_error!(
                                                        GameError,
                                                        "No trump suite defined."
                                                    ))
                                                }
                                            }
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
                                println!("{:016X} {:016X} {:016X}", origin, attack, neighbor);
                                if (state.table_stacks.len() == 0) && !(attack == *origin) {
                                    return Err(durak_error!(
                                        GameError,
                                        "Only attacking player can start."
                                    ));
                                }
                                if !((attack == *origin) || (neighbor == *origin)) {
                                    return Err(durak_error!(
                                        GameError,
                                        "Only attacking player and neighbor can start a new stack."
                                    ));
                                }
                                if state
                                    .table_stacks
                                    .iter()
                                    .filter(|&&(_, ref b)| b.is_none())
                                    .count() >=
                                    target_num_cards
                                {
                                    return Err(durak_error!(
                                        GameError,
                                        "No more stacks than cards allowed"
                                    ));
                                }
                                if !cards.remove(&card) {
                                    return Err(durak_error!(GameError, "Card not found."));
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
