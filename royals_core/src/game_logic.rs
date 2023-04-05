use crate::{
    card::Card,
    event::{Event, EventEntry, EventVisibility},
    game_state::GameState,
    play::{Action, ActionId, Play},
    player::PlayerId,
    utils::VecExtensions,
};

impl GameState<'_> {
    pub fn handle_action(&mut self, action: ActionId, log: &mut Vec<EventEntry>) {
        let (_, actions) = self.valid_actions();
        if action < actions.len() {
            match &actions[action] {
                Action::GiveUp => {
                    self.drop_player(self.players_turn, "Player gave up", log);
                }
                Action::Play(p) => {
                    self.handle_play(p, log);
                }
            }
            self.next_player_turn(log);
        }
    }

    pub fn filter_event(log: &[EventEntry], visible_to: Option<PlayerId>) -> Vec<Event> {
        log.iter()
            .map(|e| match e.visibility {
                EventVisibility::Public => e.event.clone(),
                EventVisibility::Private(player) => {
                    if visible_to.is_none() || player == visible_to.unwrap() {
                        e.event.clone()
                    } else {
                        match e.event {
                            Event::PickUp(p, _, s) => Event::PickUp(p, None, s),
                            Event::LearnedCard(p, _) => Event::LearnedCard(p, None),
                            _ => e.event.clone(),
                        }
                    }
                }
            })
            .collect()
    }

    pub fn pick_up_card(&mut self, player_id: PlayerId, log: &mut Vec<EventEntry>) {
        let next_card = self.deck[self.deck_head];
        self.deck_head += 1;
        log.push(EventEntry {
            visibility: EventVisibility::Private(player_id),
            event: Event::PickUp(player_id, Some(next_card), self.deck.len() - self.deck_head),
        });
        self.players[player_id].hand_mut().push(next_card);
    }

    pub fn drop_player(&mut self, player_id: PlayerId, reason: &str, log: &mut Vec<EventEntry>) {
        while let Some(op_card) = self.players[player_id].hand_mut().pop() {
            log.push(EventEntry {
                visibility: EventVisibility::Public,
                event: Event::Fold(player_id, op_card, reason.to_string()),
            });
        }
        log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::DropOut(player_id),
        });
    }

    pub fn next_player_turn(&mut self, log: &mut Vec<EventEntry>) {
        self.players_turn = (self.players_turn + 1) % self.players.len();
        while !self.players[self.players_turn].is_active() {
            self.players_turn = (self.players_turn + 1) % self.players.len();
        }
        if !self.game_over() {
            self.pick_up_card(self.players_turn, log);
        } else {
            self.wrap_up_round(log);
        }
    }

    pub fn wrap_up_round(&mut self, log: &mut Vec<EventEntry>) {
        let mut best_players: Vec<PlayerId> = vec![];
        let mut best_card: Option<Card> = None;
        for (i, p) in self.players.iter().enumerate() {
            if let Some(player_card) = p.hand().get(0) {
                log.push(EventEntry {
                    visibility: EventVisibility::Public,
                    event: Event::Fold(i, *player_card, "game is finished".to_string()),
                });
                if let Some(card) = best_card {
                    if card < *player_card {
                        best_players = vec![i];
                        best_card = Some(*player_card);
                    } else if card == *player_card {
                        best_players.push(i);
                    }
                } else {
                    best_players = vec![i];
                    best_card = Some(*player_card);
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

    pub fn handle_play(&mut self, p: &Play, log: &mut Vec<EventEntry>) {
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
                self.drop_player(self.players_turn, "attacked a protected player", log);
                return;
            }
        }
        self.players[self.players_turn].set_protected(false);
        match card {
            Card::Guard => {
                if let Some(op) = p.opponent {
                    let g = p.guess.unwrap();
                    if self.players[op].hand()[0] == g {
                        self.drop_player(op, "opponent guessed the hand card", log)
                    }
                }
            }
            Card::Priest => {
                if let Some(op) = p.opponent {
                    log.push(EventEntry {
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
                        self.drop_player(op, "smaller card then opponent", log);
                    } else if player_card < op_card {
                        self.drop_player(self.players_turn, "smaller card then opponent", log);
                    }
                }
            }
            Card::Maid => {
                self.players[self.players_turn].set_protected(true);
            }
            Card::Prince => {
                if let Some(op) = p.opponent {
                    if self.players[op].hand()[0] == Card::Princess {
                        self.drop_player(op, "forced to play the princess", log);
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
                "playing the princess is equivalent to giving up",
                log,
            ),
        }
    }
}
