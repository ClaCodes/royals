use std::collections::HashSet;

use crate::{card::Card, event::EventEntry, player::PlayerId};

pub struct PlayerState {
    protected: bool,
    hand: Vec<Card>,
}
impl PlayerState {
    pub fn new() -> Self {
        PlayerState {
            protected: false,
            hand: vec![],
        }
    }
    pub fn protected(&self) -> bool {
        self.protected.clone()
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

pub struct GameState {
    pub players: Vec<PlayerState>,
    pub game_log: Vec<EventEntry>,
    pub deck_head: usize,
    pub players_turn: PlayerId,
}

impl GameState {
    pub fn new(player_count: usize) -> Self {
        let mut state = GameState {
            players: vec![],
            game_log: vec![],
            deck_head: 0,
            players_turn: 0,
        };
        for _i in 0..player_count {
            state.players.push(PlayerState::new());
        }
        state
    }

    pub fn active_players(&self) -> HashSet<PlayerId> {
        self.players
            .iter()
            .enumerate()
            .filter(|&(_, p)| p.is_active())
            .map(|(i, _)| i)
            .collect()
    }

    fn other_players(&self) -> HashSet<PlayerId> {
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{card::Card, game_state::GameState, game_state::PlayerState};

    #[test]
    fn active_players_should_return_player_ids_with_non_empty_hand() {
        let state = GameState {
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
            game_log: vec![],
            deck_head: 0,
            players_turn: 0,
        };

        assert_eq!(state.active_players(), HashSet::from([1]));
    }

    #[test]
    fn other_players_should_return_ids_of_others() {
        let state = GameState {
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
            game_log: vec![],
            deck_head: 0,
            players_turn: 1, // second player turn
        };

        assert_eq!(state.other_players(), HashSet::from([0, 2]));
    }

    #[test]
    fn all_protected_should_return_true_if_no_other_active_player_is_unprotected() {
        let state = GameState {
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
            game_log: vec![],
            deck_head: 0,
            players_turn: 1, // second players turn
        };

        assert_eq!(state.all_protected(), true);
    }

    #[test]
    fn all_protected_should_return_false_if_at_least_one_other_active_player_is_unprotected() {
        let state = GameState {
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
            game_log: vec![],
            deck_head: 0,
            players_turn: 1, // second players turn
        };

        assert_eq!(state.all_protected(), false);
    }
}
