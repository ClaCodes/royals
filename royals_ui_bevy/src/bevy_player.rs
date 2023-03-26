use std::sync::mpsc::{Receiver, Sender};

use royals_core::{
    event::Event,
    play::Action,
    player::{Player, PlayerData, PlayerId},
};

use crate::{
    events::{NotifyEvent, ObtainActionEvent},
    GameEvent,
};

pub struct BevyPlayer {
    pub data: PlayerData,
    pub sender: Sender<GameEvent>,
    pub receiver: Receiver<usize>,
}

impl BevyPlayer {
    pub fn new(id: PlayerId, sender: Sender<GameEvent>, receiver: Receiver<usize>) -> Self {
        BevyPlayer {
            data: PlayerData::new(id, "Bevy player".to_string()),
            sender,
            receiver,
        }
    }
}

impl Player for BevyPlayer {
    fn data(&self) -> &PlayerData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut PlayerData {
        &mut self.data
    }

    fn notify(&self, game_log: &[Event], players: &[&String]) {
        let players = players.iter().map(|&s| s.to_owned()).collect::<Vec<_>>();
        let game_log = game_log.iter().map(|e| e.to_owned()).collect::<Vec<_>>();

        self.sender
            .send(GameEvent::Notify(NotifyEvent { players, game_log }))
            .unwrap();
    }

    fn obtain_action(
        &self,
        players: &[&String],
        game_log: &[Event],
        valid_actions: &[Action],
    ) -> usize {
        let players = players.iter().map(|&s| s.to_owned()).collect::<Vec<_>>();
        let game_log = game_log.iter().map(|e| e.to_owned()).collect::<Vec<_>>();
        let valid_actions = valid_actions
            .iter()
            .map(|a| a.to_owned())
            .collect::<Vec<_>>();

        self.sender
            .send(GameEvent::ObtainAction(ObtainActionEvent {
                players,
                game_log,
                valid_actions,
            }))
            .unwrap();

        self.receiver.recv().unwrap()
    }
}
