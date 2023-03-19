use crate::{card::Card, play::Action, Event};

pub type PlayerId = usize;

pub struct PlayerData {
    pub id: PlayerId,
    pub name: String,
    pub protected: bool,
    pub hand: Vec<Card>,
}

pub trait Player {
    fn data(&self) -> &PlayerData;

    fn data_mut(&mut self) -> &mut PlayerData;

    fn name(&self) -> String {
        self.data().name.clone()
    }

    fn protected(&self) -> bool {
        self.data().protected.clone()
    }

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
