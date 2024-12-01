use royals_core::events::{Action, Event};

pub struct PlayerData {
    name: String,
}

impl PlayerData {
    pub fn new(name: String) -> Self {
        PlayerData { name }
    }
}

pub trait Player {
    fn data(&self) -> &PlayerData;

    fn data_mut(&mut self) -> &mut PlayerData;

    fn name(&self) -> &String {
        &self.data().name
    }

    fn notify(&self, game_log: &[Event], players: &[&String]);

    fn obtain_action(
        &self,
        players: &[&String],
        game_log: &[Event],
        valid_actions: &[Action],
    ) -> usize;
}
