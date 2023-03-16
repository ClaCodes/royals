use rand::seq::SliceRandom;

use crate::{
    card::Card,
    console_player::ConsolePlayer,
    event::{Event, EventEntry, EventVisibility},
    play::{Action, Play},
    player::{PlayerInterface, PlayerId},
    random_playing_computer::RandomPlayingComputer,
};

struct Player {
    pub interface: Box<dyn PlayerInterface>,
    pub hand_cards: Vec<Card>,
    pub protected: bool,
}

impl Player {
    pub fn new(interface: Box<dyn PlayerInterface>) -> Self {
        Self {
            interface,
            hand_cards: vec![],
            protected: false,
        }
    }
}


pub struct GameState {
    deck: Vec<Card>,
    players: Vec<Player>,
    player_names: Vec<String>,
    game_log: Vec<EventEntry>,
    players_turn: PlayerId,
    running: bool,
}

impl GameState {
    pub fn new() -> Self {
        let mut players = vec![];
        let mut player_names = vec![];

        player_names.push( "You");
        player_names.push( "Computer Alpha");
        player_names.push( "Computer Bravo");
        player_names.push( "Computer Charlie");
        let player_names = player_names.iter().map(|x| x.to_string()).collect();
        players.push(Player::new( Box::new(ConsolePlayer { id: players.len() }),));
        players.push(Player::new( Box::new(RandomPlayingComputer { id: players.len() }),));
        players.push(Player::new( Box::new(RandomPlayingComputer { id: players.len() }),));
        players.push(Player::new( Box::new(RandomPlayingComputer { id: players.len() }),));
        //state.players.shuffle(&mut rand::thread_rng()); todo

        let mut state = GameState {
            deck: vec![
                Card::Guardian,
                Card::Guardian,
                Card::Guardian,
                Card::Guardian,
                Card::Guardian,
                Card::Priest,
                Card::Priest,
                Card::Baron,
                Card::Baron,
                Card::Maid,
                Card::Maid,
                Card::Prince,
                Card::Prince,
                Card::King,
                Card::Contess,
                Card::Princess,
            ],
            players,
            game_log: vec![],
            player_names,
            players_turn: 0,
            running: true,
        };

        state.deck.shuffle(&mut rand::thread_rng());

        for i in 0..state.players.len() {
            state.pick_up_card(i);
        }

        state
    }

    pub fn run(&mut self) {
        let mut ok = true;
        while self.running {
            if ok {
                self.pick_up_card(self.players_turn);
            }
            let user_action = self.players[self.players_turn].interface.obtain_action(
                &self.players[self.players_turn].hand_cards,
                &self.player_names,
                &self.filter_event(),
                self.all_protected(),
                &self.active_players()
            );

            match user_action {
                Action::GiveUp => {
                    self.drop_player(self.players_turn, "Player gave up".to_string());
                    self.next_player_turn();
                }
                Action::Play(p) => {
                    ok = self.is_valid(&p);
                    if ok {
                        self.handle_play(p);
                        self.next_player_turn();
                    }
                }
            }
        }
        for mut p in &mut self.game_log {
            p.visibility = EventVisibility::Public;
        }
        let mut best_players: Vec<PlayerId> = vec![];
        let mut best_card: Option<Card> = None;
        for (i, p) in self.players.iter().enumerate() {
            if let Some(player_card) = p.hand_cards.get(0) {
                self.game_log.push(EventEntry {
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
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Winner(best_players),
        });
        for p in &self.players {
            p.interface.notify(&self.filter_event(), &self.player_names);
        }
    }

    fn filter_event(&self) -> Vec<Event> {
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

    fn pick_up_card(&mut self, player_id: PlayerId) {
        let next_card = self.deck.pop().unwrap();
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Private(player_id),
            event: Event::PickUp(player_id, Some(next_card.clone()), self.deck.len()),
        });
        self.players[player_id].hand_cards.push(next_card);
    }

    fn drop_player(&mut self, player_id: PlayerId, reason: String) {
        while let Some(op_card) = self.players[player_id].hand_cards.pop() {
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

    fn active_players(&self) -> Vec<PlayerId> {
        self.players
            .iter().enumerate()
            .filter(|(_, p)| !p.hand_cards.is_empty())
            .map(|(i, _)| i)
            .collect()
    }

    fn next_player_turn(&mut self) {
        self.players_turn = (self.players_turn + 1) % self.players.len();
        while self.players[self.players_turn].hand_cards.is_empty() {
            self.players_turn = (self.players_turn + 1) % self.players.len();
        }
        // last card is ussually not used
        self.running = self.running && self.deck.len() > 1 && self.active_players().len() > 1;
    }

    fn is_valid(&self, play: &Play) -> bool {
        if play.card == Card::Princess {
            return false;
        }
        if self.players[self.players_turn].hand_cards.contains(&Card::Contess)
        {
            if play.card == Card::Prince || play.card == Card::King {
                return false;
            }
        }
        if play.opponent.is_none() && play.card.needs_opponent() {
            if !self.all_protected() {
                return false;
            }
        }
        if let Some(op) = play.opponent {
            if op == self.players_turn {
                return false;
            }
            if self.players[op].hand_cards.is_empty() {
                return false;
            }
        }
        true
    }

    fn all_protected(&self) -> bool {
        self.players
            .iter()
            .enumerate()
            .all(|(i, p)| p.hand_cards.is_empty() || p.protected || i == self.players_turn)
    }

    fn handle_play(&mut self, p: Play) {
        let index = self.players[self.players_turn]
            .hand_cards
            .iter()
            .position(|x| *x == p.card)
            .unwrap();
        self.players[self.players_turn].hand_cards.remove(index);
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Play(self.players_turn, p.clone()),
        });
        if let Some(opponent) = p.opponent {
            // do not attack protected player
            if self.players[opponent].protected && !self.all_protected() {
                self.drop_player(self.players_turn, "attacked a protected player".to_string());
                return;
            }
        }
        self.players[self.players_turn].protected = false;
        match p.card {
            Card::Guardian => {
                if let Some(op) = p.opponent {
                    let g = p.guess.unwrap();
                    if self.players[op].hand_cards[0] == g {
                        self.drop_player(op, "opponent guess the hand card".to_string())
                    }
                }
            }
            Card::Priest => {
                if let Some(op) = p.opponent {
                    self.game_log.push(EventEntry {
                        visibility: EventVisibility::Private(self.players_turn),
                        event: Event::LearnedCard(op, Some(self.players[op].hand_cards[0].clone())),
                    });
                }
            }
            Card::Baron => {
                if let Some(op) = p.opponent {
                    let op_card = self.players[op].hand_cards[0];
                    let player_card = self.players[self.players_turn].hand_cards[0];
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
                self.players[self.players_turn].protected = true;
            }
            Card::Prince => {
                if let Some(op) = p.opponent {
                    if self.players[op].hand_cards[0] == Card::Princess {
                        self.drop_player(op, "forced to play the princess".to_string());
                    } else {
                        let folded = self.players[op].hand_cards.pop().unwrap();
                        self.game_log.push(EventEntry {
                            visibility: EventVisibility::Public,
                            event: Event::Fold(
                                op,
                                folded,
                                "opponent has played prince to force it".to_string(),
                            ),
                        });
                        self.pick_up_card(op);
                    }
                }
            }
            Card::King => {
                if let Some(op) = p.opponent {
                    let op_card = self.players[op].hand_cards.pop().unwrap();
                    let player_card = self.players[self.players_turn].hand_cards.pop().unwrap();
                    self.players[op].hand_cards.push(player_card);
                    self.players[self.players_turn].hand_cards.push(op_card);
                }
            }
            Card::Contess => {}
            Card::Princess => self.drop_player(
                self.players_turn,
                "playing the princess is equivalent to giving up".to_string(),
            ),
        }
    }
}
