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

    fn name(&self) -> &String {
        &self.data().name
    }

    fn protected(&self) -> bool {
        self.data().protected.clone()
    }

    fn hand(&self) -> &Vec<Card> {
        &self.data().hand
    }

    fn hand_mut(&mut self) -> &mut Vec<Card> {
        &mut self.data_mut().hand
    }

    fn notify(&self, game_log: &[Event], players: &[&String]);

    fn obtain_action(
        &self,
        hand: &[Card],
        players: &[&String],
        game_log: &[Event],
        all_protected: bool,
        active_players: &[PlayerId],
    ) -> Action;
}
