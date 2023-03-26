use crate::{card::Card, play::Play, player::PlayerId};

#[derive(Clone, Debug)]
pub enum Event {
    Play(PlayerId, Play),
    Fold(PlayerId, Card, String),
    PickUp(PlayerId, Option<Card>, usize),
    DropOut(PlayerId),
    LearnedCard(PlayerId, Option<Card>),
    Winner(Vec<PlayerId>),
}

#[derive(PartialEq)]
pub enum EventVisibility {
    Public,
    Private(PlayerId),
}

pub struct EventEntry {
    pub visibility: EventVisibility,
    pub event: Event,
}
