use itertools::Itertools;
use std::collections::HashSet;

use crate::{
    card::Card,
    event::{Event, EventEntry, EventVisibility},
    play::{Action, ActionId, Play},
    player::PlayerId,
    utils::VecExtensions,
};

struct PlayerState {
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
    fn protected(&self) -> bool {
        self.protected.clone()
    }

    fn set_protected(&mut self, value: bool) {
        self.protected = value;
    }

    fn hand(&self) -> &Vec<Card> {
        &self.hand
    }

    fn hand_mut(&mut self) -> &mut Vec<Card> {
        &mut self.hand
    }

    fn is_active(&self) -> bool {
        !&self.hand().is_empty()
    }
}

pub struct GameState<'a> {
    players: Vec<PlayerState>,
    deck_head: usize,
    players_turn: PlayerId,
    game_over: bool,
    deck: &'a[Card],
}

impl <'a>GameState<'a> {
    pub fn new(player_count: usize, deck: &'a[Card], log: &mut Vec<EventEntry>) -> Self {
        let mut state = GameState {
            players: vec![],
            deck_head: 0,
            players_turn: 0,
            game_over: false,
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
        if self.game_over {
            return (None, vec![]);
        }
        let mut actions = vec![Action::GiveUp];
        let mut first_card: Option<Card> = None;

        for card in self.players[self.players_turn].hand() {
            // avoid dublicate entries
            if first_card.is_none() {
                first_card = Some(card.clone());
            } else if first_card.unwrap() == *card {
                break;
            }

            actions.push(Action::Play(Play {
                card: *card,
                opponent: None,
                guess: None,
            }));
            for opponent in self.other_active_players() {
                actions.push(Action::Play(Play {
                    card: *card,
                    opponent: Some(opponent),
                    guess: None,
                }));
                for guess in Card::guessable() {
                    actions.push(Action::Play(Play {
                        card: *card,
                        opponent: Some(opponent),
                        guess: Some(*guess),
                    }));
                }
            }
        }
        (
            Some(self.players_turn),
            actions
                .into_iter()
                .filter(|a| self.is_valid(a))
                .collect_vec(),
        )
    }

    pub fn handle_action(&mut self, action: ActionId, log: &mut Vec<EventEntry>) {
        let (_, actions) = self.valid_actions();
        if action < actions.len() {
            match &actions[action] {
                Action::GiveUp => {
                    self.drop_player(self.players_turn, "Player gave up".to_string(), log);
                }
                Action::Play(p) => {
                    self.handle_play(p, log);
                }
            }
            self.next_player_turn(log);
        }
    }

    fn wrap_up_round(&mut self, log: &mut Vec<EventEntry>) {
        let mut best_players: Vec<PlayerId> = vec![];
        let mut best_card: Option<Card> = None;
        for (i, p) in self.players.iter().enumerate() {
            if let Some(player_card) = p.hand().get(0) {
                log.push(EventEntry {
                    visibility: EventVisibility::Public,
                    event: Event::Fold(i, player_card.clone(), "game is finished".to_string()),
                });
                if let Some(card) = best_card {
                    if card < *player_card {
                        best_players = vec![i];
                        best_card = Some(player_card.clone());
                    } else if card == *player_card {
                        best_players.push(i);
                    }
                } else {
                    best_players = vec![i];
                    best_card = Some(player_card.clone());
                }
            }
        }

        log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Winner(best_players),
        });

        for e in log {
            e.visibility = EventVisibility::Public;
        }
    }

    fn active_players(&self) -> HashSet<PlayerId> {
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

    fn other_active_players(&self) -> HashSet<PlayerId> {
        self.other_players()
            .intersection(&self.active_players())
            .cloned()
            .collect::<HashSet<_>>()
    }

    fn all_protected(&self) -> bool {
        self.other_active_players()
            .iter()
            .all(|&id| self.players[id].protected())
    }

    fn pick_up_card(&mut self, player_id: PlayerId, log: &mut Vec<EventEntry>) {
        let next_card = self.deck[self.deck_head];
        self.deck_head += 1;
        log.push(EventEntry {
            visibility: EventVisibility::Private(player_id),
            event: Event::PickUp(
                player_id,
                Some(next_card.clone()),
                self.deck.len() - self.deck_head,
            ),
        });
        self.players[player_id].hand_mut().push(next_card);
    }

    fn drop_player(&mut self, player_id: PlayerId, reason: String, log: &mut Vec<EventEntry>) {
        while let Some(op_card) = self.players[player_id].hand_mut().pop() {
            log.push(EventEntry {
                visibility: EventVisibility::Public,
                event: Event::Fold(player_id, op_card, reason.clone()),
            });
        }
        log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::DropOut(player_id),
        });
    }

    fn next_player_turn(&mut self, log: &mut Vec<EventEntry>) {
        self.players_turn = (self.players_turn + 1) % self.players.len();
        while !self.players[self.players_turn].is_active() {
            self.players_turn = (self.players_turn + 1) % self.players.len();
        }
        // last card is ussually not used
        self.game_over = !(self.deck.len() - self.deck_head > 1 && self.active_players().len() > 1);
        if !self.game_over {
            self.pick_up_card(self.players_turn, log);
        } else {
            self.wrap_up_round(log);
        }
    }

    fn is_valid(&self, action: &Action) -> bool {
        match action {
            Action::GiveUp => true,
            Action::Play(play) => {
                if !self.players[self.players_turn].hand().contains(&play.card) {
                    return false;
                }
                if self.players[self.players_turn]
                    .hand()
                    .contains(&Card::Countess)
                {
                    if play.card == Card::Prince || play.card == Card::King {
                        return false;
                    }
                }
                if !play.card.needs_opponent() {
                    if play.opponent.is_some() {
                        return false;
                    }
                } else if !self.all_protected() {
                    if play.opponent.is_none() {
                        return false;
                    }
                }

                if !play.card.needs_guess() {
                    if play.guess.is_some() {
                        return false;
                    }
                } else if !self.all_protected() {
                    if play.guess.is_none() {
                        return false;
                    }
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

    fn handle_play(&mut self, p: &Play, log: &mut Vec<EventEntry>) {
        let card = self.players[self.players_turn]
            .hand_mut()
            .remove_first_where(|&card| card == p.card)
            .unwrap();

        log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Play(self.players_turn, p.clone()),
        });
        if let Some(opponent) = p.opponent {
            // do not attack protected player
            if self.players[opponent].protected() && !self.all_protected() {
                self.drop_player(
                    self.players_turn,
                    "attacked a protected player".to_string(),
                    log,
                );
                return;
            }
        }
        self.players[self.players_turn].set_protected(false);
        match card {
            Card::Guard => {
                if let Some(op) = p.opponent {
                    let g = p.guess.unwrap();
                    if self.players[op].hand()[0] == g {
                        self.drop_player(op, "opponent guessed the hand card".to_string(), log)
                    }
                }
            }
            Card::Priest => {
                if let Some(op) = p.opponent {
                    log.push(EventEntry {
                        visibility: EventVisibility::Private(self.players_turn),
                        event: Event::LearnedCard(op, Some(self.players[op].hand()[0].clone())),
                    });
                }
            }
            Card::Baron => {
                if let Some(op) = p.opponent {
                    let op_card = self.players[op].hand()[0];
                    let player_card = self.players[self.players_turn].hand()[0];
                    if op_card < player_card {
                        self.drop_player(op, "smaller card then opponent".to_string(), log);
                    } else if player_card < op_card {
                        self.drop_player(
                            self.players_turn,
                            "smaller card then opponent".to_string(),
                            log,
                        );
                    }
                }
            }
            Card::Maid => {
                self.players[self.players_turn].set_protected(true);
            }
            Card::Prince => {
                if let Some(op) = p.opponent {
                    if self.players[op].hand()[0] == Card::Princess {
                        self.drop_player(op, "forced to play the princess".to_string(), log);
                    } else {
                        let folded = self.players[op].hand_mut().pop().unwrap();
                        log.push(EventEntry {
                            visibility: EventVisibility::Public,
                            event: Event::Fold(
                                op,
                                folded,
                                "opponent has played prince to force it".to_string(),
                            ),
                        });
                        self.pick_up_card(op, log);
                    }
                }
            }
            Card::King => {
                if let Some(op) = p.opponent {
                    let op_card = self.players[op].hand_mut().pop().unwrap();
                    let player_card = self.players[self.players_turn].hand_mut().pop().unwrap();
                    self.players[op].hand_mut().push(player_card);
                    self.players[self.players_turn].hand_mut().push(op_card);
                }
            }
            Card::Countess => {}
            Card::Princess => self.drop_player(
                self.players_turn,
                "playing the princess is equivalent to giving up".to_string(),
                log,
            ),
        }
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
            deck_head: 0,
            players_turn: 0,
            game_over: false,
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
            deck_head: 0,
            players_turn: 1, // second player turn
            game_over: false,
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
            deck_head: 0,
            players_turn: 1, // second players turn
            game_over: false,
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
            deck_head: 0,
            players_turn: 1, // second players turn
            game_over: false,
        };

        assert_eq!(state.all_protected(), false);
    }
}
