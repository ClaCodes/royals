use itertools::{iproduct, Itertools};
use std::{collections::HashSet, iter::once};
use strum::IntoEnumIterator;

use crate::{
    card::Card,
    event::EventEntry,
    play::{Action, Play},
    player::PlayerId,
};

pub struct PlayerState {
    protected: bool,
    hand: Vec<Card>,
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            protected: false,
            hand: vec![],
        }
    }
    pub fn protected(&self) -> bool {
        self.protected
    }

    pub fn set_protected(&mut self, value: bool) {
        self.protected = value;
    }

    pub fn hand(&self) -> &Vec<Card> {
        &self.hand
    }

    pub fn hand_mut(&mut self) -> &mut Vec<Card> {
        &mut self.hand
    }

    pub fn is_active(&self) -> bool {
        !&self.hand().is_empty()
    }
}

pub struct GameState<'a> {
    pub players: Vec<PlayerState>,
    pub played_card_count: usize,
    pub players_turn: PlayerId,
    pub deck: &'a [Card],
}

impl<'a> GameState<'a> {
    pub fn new(player_count: usize, deck: &'a [Card], log: &mut Vec<EventEntry>) -> Self {
        let mut state = GameState {
            players: vec![],
            played_card_count: 0,
            players_turn: 0,
            deck,
        };
        for i in 0..player_count {
            state.players.push(PlayerState::new());
            state.pick_up_card(i, log);
        }
        state.pick_up_card(state.players_turn, log);
        state
    }
    pub fn valid_actions(&self) -> (Option<PlayerId>, Vec<Action>) {
        let actions: Vec<Action> = once(Action::GiveUp)
            .chain(self.possible_actions())
            .filter(|a| self.is_valid(a))
            .collect();
        (
            if actions.is_empty() {
                None
            } else {
                Some(self.players_turn)
            },
            actions,
        )
    }

    pub fn possible_actions(&self) -> Vec<Action> {
        // todo is there an alternative way to also iterate over None
        let others = self.other_active_players();
        let mut optional_players = others.iter().map(|p| Some(*p)).collect_vec();
        optional_players.push(None);

        let mut optional_card = Card::iter().map(Some).collect_vec();
        optional_card.push(None);

        iproduct!(Card::iter(), optional_players.iter(), optional_card.iter())
            .map(|(card, &opponent, &guess)| {
                Action::Play(Play {
                    card,
                    opponent,
                    guess,
                })
            })
            .collect_vec()
    }

    pub fn is_valid(&self, action: &Action) -> bool {
        if self.game_over() {
            return false;
        }
        match action {
            Action::GiveUp => true,
            Action::Play(play) => {
                if !self.players[self.players_turn].hand().contains(&play.card) {
                    return false;
                }
                if self.players[self.players_turn]
                    .hand()
                    .contains(&Card::Countess)
                    && (play.card == Card::Prince || play.card == Card::King)
                {
                    return false;
                }
                if !play.card.needs_opponent() {
                    if play.opponent.is_some() {
                        return false;
                    }
                } else if !self.all_protected() && play.opponent.is_none() {
                    return false;
                }

                if !play.card.needs_guess() {
                    if play.guess.is_some() {
                        return false;
                    }
                } else if !self.all_protected() && play.guess.is_none() {
                    return false;
                }

                if let Some(op) = play.opponent {
                    if op == self.players_turn {
                        return false;
                    }
                    if op >= self.players.len() {
                        return false;
                    }
                    if !self.players[op].is_active() {
                        return false;
                    }
                }
                true
            }
        }
    }

    pub fn active_players(&self) -> HashSet<PlayerId> {
        self.players
            .iter()
            .enumerate()
            .filter(|&(_, p)| p.is_active())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn other_players(&self) -> HashSet<PlayerId> {
        self.players
            .iter()
            .enumerate()
            .map(|(i, _)| i)
            .filter(|&id| id != self.players_turn)
            .collect()
    }

    pub fn other_active_players(&self) -> HashSet<PlayerId> {
        self.other_players()
            .intersection(&self.active_players())
            .cloned()
            .collect::<HashSet<_>>()
    }

    pub fn all_protected(&self) -> bool {
        self.other_active_players()
            .iter()
            .all(|&id| self.players[id].protected())
    }

    pub fn game_over(&self) -> bool {
        // last card is ussually not used
        self.deck.len() - self.played_card_count <= 1 || self.active_players().len() <= 1
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{card::Card, game_state::GameState, game_state::PlayerState};

    #[test]
    fn active_players_should_return_player_ids_with_non_empty_hand() {
        let deck = &Card::deck();
        let state = GameState {
            deck,
            players: vec![
                PlayerState {
                    protected: false,
                    hand: vec![],
                },
                PlayerState {
                    protected: false,
                    hand: vec![Card::King],
                },
            ],
            played_card_count: 0,
            players_turn: 0,
        };

        assert_eq!(state.active_players(), HashSet::from([1]));
    }

    #[test]
    fn other_players_should_return_ids_of_others() {
        let deck = &Card::deck();
        let state = GameState {
            deck,
            players: vec![
                PlayerState {
                    protected: false,
                    hand: vec![],
                },
                PlayerState {
                    protected: false,
                    hand: vec![],
                },
                PlayerState {
                    protected: false,
                    hand: vec![],
                },
            ],
            played_card_count: 0,
            players_turn: 1, // second player turn
        };

        assert_eq!(state.other_players(), HashSet::from([0, 2]));
    }

    #[test]
    fn all_protected_should_return_true_if_no_other_active_player_is_unprotected() {
        let deck = &Card::deck();
        let state = GameState {
            deck,
            players: vec![
                PlayerState {
                    protected: false,
                    hand: vec![],
                }, // inactive
                PlayerState {
                    protected: false,
                    hand: vec![Card::King],
                }, // players turn
                PlayerState {
                    protected: true,
                    hand: vec![Card::Countess],
                }, // protected
            ],
            played_card_count: 0,
            players_turn: 1, // second players turn
        };

        assert_eq!(state.all_protected(), true);
    }

    #[test]
    fn all_protected_should_return_false_if_at_least_one_other_active_player_is_unprotected() {
        let deck = &Card::deck();
        let state = GameState {
            deck,
            players: vec![
                PlayerState {
                    protected: false,
                    hand: vec![],
                }, // inactive
                PlayerState {
                    protected: false,
                    hand: vec![Card::King],
                }, // players turn
                PlayerState {
                    protected: true,
                    hand: vec![Card::Countess],
                }, // protected
                PlayerState {
                    protected: false,
                    hand: vec![Card::Guard],
                }, // unprotected
            ],
            played_card_count: 0,
            players_turn: 1, // second players turn
        };

        assert_eq!(state.all_protected(), false);
    }
}
