use crate::{action::Action, card::Card, Event};

pub type PlayerId = usize;

pub struct Player {
    pub name: String,
    pub interface: Box<dyn PlayerInterface>,
    pub hand_cards: Vec<Card>,
    pub protected: bool,
}

impl Player {
    pub fn new(name: &str, interface: Box<dyn PlayerInterface>) -> Self {
        Self {
            name: name.to_string(),
            interface,
            hand_cards: vec![],
            protected: false,
        }
    }
}

pub trait PlayerInterface {
    fn notify(&self, game_log: &[Event], players: &[Player]);
    fn obtain_action(&self, hand_cards: &[Card], players: &[Player], game_log: &[Event]) -> Action;
}
