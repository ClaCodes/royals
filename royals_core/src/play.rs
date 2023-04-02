use crate::{card::Card, player::PlayerId};

#[derive(Debug, Clone, PartialEq)]
pub struct Play {
    pub card: Card,
    pub opponent: Option<PlayerId>,
    pub guess: Option<Card>,
}

pub type ActionId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    GiveUp,
    Play(Play),
}
