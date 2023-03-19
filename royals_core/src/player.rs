use crate::{card::Card, play::Action, Event};

pub type PlayerId = usize;

pub trait Player {
    fn notify(&self, game_log: &[Event], players: &[String]);
    fn obtain_action(
        &self,
        hand_cards: &[Card],
        players: &[String],
        game_log: &[Event],
        all_protected: bool,
        active_players: &[PlayerId],
    ) -> Action;
}
