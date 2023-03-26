use royals_core::{event::Event, play::Action};

#[derive(Debug)]
pub enum GameEvent {
    Notify(NotifyEvent),
    ObtainAction(ObtainActionEvent),
}

#[derive(Default, Debug)]
pub struct NotifyEvent {
    pub players: Vec<String>,
    pub game_log: Vec<Event>,
}

#[derive(Default, Debug)]
pub struct ObtainActionEvent {
    pub players: Vec<String>,
    pub game_log: Vec<Event>,
    pub valid_actions: Vec<Action>,
}
