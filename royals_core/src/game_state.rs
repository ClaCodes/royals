use std::collections::HashSet;

use rand::seq::SliceRandom;

use crate::{
    card::Card,
    event::EventEntry,
    player::{Player, PlayerId},
    random_playing_computer::RandomPlayingComputer,
};

pub struct GameState {
    pub deck: Vec<Card>,
    pub players: Vec<Box<dyn Player>>,
    pub game_log: Vec<EventEntry>,
    pub players_turn: PlayerId,
}

impl GameState {
    pub fn new<C, T>(player_constructor: C) -> Self
    where
        C: Fn(PlayerId) -> T,
        T: Player + 'static,
    {
        let mut state = GameState {
            deck: vec![
                Card::Guard,
                Card::Guard,
                Card::Guard,
                Card::Guard,
                Card::Guard,
                Card::Priest,
                Card::Priest,
                Card::Baron,
                Card::Baron,
                Card::Maid,
                Card::Maid,
                Card::Prince,
                Card::Prince,
                Card::King,
                Card::Countess,
                Card::Princess,
            ],
            players: vec![],
            game_log: vec![],
            players_turn: 0,
        };

        state.add_player(player_constructor);
        state.add_player(RandomPlayingComputer::new);
        state.add_player(RandomPlayingComputer::new);
        state.add_player(RandomPlayingComputer::new);

        //state.players.shuffle(&mut rand::thread_rng()); todo

        state.deck.shuffle(&mut rand::thread_rng());

        for i in 0..state.players.len() {
            state.pick_up_card(i);
        }

        state
    }

    fn add_player<C, T>(&mut self, player_constructor: C)
    where
        C: Fn(PlayerId) -> T,
        T: Player + 'static,
    {
        let id = self.players.len() as PlayerId;
        let player = player_constructor(id);
        self.players.push(Box::new(player));
    }

    pub fn player_names(&self) -> Vec<&String> {
        self.players.iter().map(|p| p.name()).collect::<Vec<_>>()
    }

    pub fn active_players(&self) -> HashSet<PlayerId> {
        self.players
            .iter()
            .filter(|p| p.is_active())
            .map(|p| p.id())
            .collect()
    }

    fn other_players(&self) -> HashSet<PlayerId> {
        self.players
            .iter()
            .map(|p| p.id())
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
