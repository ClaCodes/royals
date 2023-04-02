use itertools::Itertools;

use crate::{
    card::Card,
    event::{Event, EventEntry, EventVisibility},
    game_state::GameState,
    play::{Action, Play},
    player::PlayerId,
    utils::VecExtensions,
};

impl GameState {
    pub fn valid_actions(&self) -> Vec<Action> {
        let mut actions = vec![Action::GiveUp];
        let mut first_card: Option<Card> = None;

        for card in self.players[self.players_turn].hand() {
            // avoid dublicate entries
            if first_card.is_none() {
                first_card = Some(*card);
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
        actions
            .into_iter()
            .filter(|a| self.is_valid(a))
            .collect_vec()
    }

    pub fn filter_event(&self) -> Vec<Event> {
        let mut events = vec![];
        for e in &self.game_log {
            match e.visibility {
                EventVisibility::Public => events.push(e.event.clone()),
                EventVisibility::Private(player) => {
                    if player == self.players_turn {
                        events.push(e.event.clone())
                    } else {
                        match e.event {
                            Event::PickUp(p, _, s) => events.push(Event::PickUp(p, None, s)),
                            Event::LearnedCard(p, _) => events.push(Event::LearnedCard(p, None)),
                            _ => events.push(e.event.clone()),
                        }
                    }
                }
            }
        }
        events
    }

    pub fn pick_up_card(&mut self, player_id: PlayerId, deck: &[Card]) {
        let next_card = deck[self.deck_head];
        self.deck_head += 1;
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Private(player_id),
            event: Event::PickUp(player_id, Some(next_card), deck.len() - self.deck_head),
        });
        self.players[player_id].hand_mut().push(next_card);
    }

    pub fn drop_player(&mut self, player_id: PlayerId, reason: String) {
        while let Some(op_card) = self.players[player_id].hand_mut().pop() {
            self.game_log.push(EventEntry {
                visibility: EventVisibility::Public,
                event: Event::Fold(player_id, op_card, reason.clone()),
            });
        }
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::DropOut(player_id),
        });
    }

    pub fn next_player_turn(&mut self, deck: &[Card]) -> bool {
        self.players_turn = (self.players_turn + 1) % self.players.len();
        while !self.players[self.players_turn].is_active() {
            self.players_turn = (self.players_turn + 1) % self.players.len();
        }
        // last card is ussually not used
        deck.len() - self.deck_head > 1 && self.active_players().len() > 1
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

    pub fn handle_play(&mut self, p: &Play, deck: &[Card]) {
        let card = self.players[self.players_turn]
            .hand_mut()
            .remove_first_where(|&card| card == p.card)
            .unwrap();

        self.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Play(self.players_turn, p.clone()),
        });
        if let Some(opponent) = p.opponent {
            // do not attack protected player
            if self.players[opponent].protected() && !self.all_protected() {
                self.drop_player(self.players_turn, "attacked a protected player".to_string());
                return;
            }
        }
        self.players[self.players_turn].set_protected(false);
        match card {
            Card::Guard => {
                if let Some(op) = p.opponent {
                    let g = p.guess.unwrap();
                    if self.players[op].hand()[0] == g {
                        self.drop_player(op, "opponent guessed the hand card".to_string())
                    }
                }
            }
            Card::Priest => {
                if let Some(op) = p.opponent {
                    self.game_log.push(EventEntry {
                        visibility: EventVisibility::Private(self.players_turn),
                        event: Event::LearnedCard(op, Some(self.players[op].hand()[0])),
                    });
                }
            }
            Card::Baron => {
                if let Some(op) = p.opponent {
                    let op_card = self.players[op].hand()[0];
                    let player_card = self.players[self.players_turn].hand()[0];
                    if op_card < player_card {
                        self.drop_player(op, "smaller card then opponent".to_string());
                    } else if player_card < op_card {
                        self.drop_player(
                            self.players_turn,
                            "smaller card then opponent".to_string(),
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
                        self.drop_player(op, "forced to play the princess".to_string());
                    } else {
                        let folded = self.players[op].hand_mut().pop().unwrap();
                        self.game_log.push(EventEntry {
                            visibility: EventVisibility::Public,
                            event: Event::Fold(
                                op,
                                folded,
                                "opponent has played prince to force it".to_string(),
                            ),
                        });
                        self.pick_up_card(op, deck);
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
            ),
        }
    }
}
