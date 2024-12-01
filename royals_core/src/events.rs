use crate::card::Card;
use serde::{Deserialize, Serialize};

pub type ActionId = usize;
pub type PlayerId = usize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Play {
    pub card: Card,
    pub opponent: Option<PlayerId>,
    pub guess: Option<Card>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    GiveUp,
    Play(Play),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum GameEvent {
    Notify(NotifyEvent),
    ObtainAction(ObtainActionEvent),
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct NotifyEvent {
    pub players: Vec<String>,
    pub game_log: Vec<Event>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ObtainActionEvent {
    pub players: Vec<String>,
    pub game_log: Vec<Event>,
    pub valid_actions: Vec<Action>,
}

#[derive(Deserialize, Serialize)]
pub struct ClientEvent {
    pub action_id: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
