use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    event::Event,
    play::Action,
    player::{Player, PlayerData},
};

static COMPUTER_NAMES: &[&str] = &["Computer Alpha", "Computer Bravo", "Computer Charlie"];
static COMPUTER_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct RandomPlayingComputer {
    pub data: PlayerData,
}

impl RandomPlayingComputer {
    pub fn new() -> RandomPlayingComputer {
        let my_id = COMPUTER_COUNT.fetch_add(1, Ordering::Relaxed);
        let name = COMPUTER_NAMES[my_id % COMPUTER_NAMES.len()].to_string();
        RandomPlayingComputer {
            data: PlayerData::new(name),
        }
    }
}

impl Player for RandomPlayingComputer {
    fn data(&self) -> &PlayerData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut PlayerData {
        &mut self.data
    }

    fn notify(&self, _game_log: &[Event], _players: &[&String]) {}

    fn obtain_action(
        &self,
        _players: &[&String],
        _game_log: &[Event],
        valid_action: &[Action],
    ) -> usize {
        let mut rng = rand::thread_rng();
        let len = valid_action.len();
        let mut action_index = rng.gen_range(0, len);
        while len > 1 && valid_action[action_index] == Action::GiveUp {
            action_index = rng.gen_range(0, len);
        }
        action_index
    }
}
