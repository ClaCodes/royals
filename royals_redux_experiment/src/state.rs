use crate::common::*;

#[derive(Default, Debug, Clone)]
pub struct State {
    pub game_is_running: bool,
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub players_turn: Option<PlayerId>,
}

#[derive(Debug, Clone)]
pub enum Card {
    Guard,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub display_name: String,
    pub secret: PlayerSecret,
    pub connected: bool,
    pub active: bool,
    pub hand: Vec<Card>,
    pub possible_moves: Vec<Move>,
}

#[derive(Debug, Clone)]
pub enum Move {
    A,
}
