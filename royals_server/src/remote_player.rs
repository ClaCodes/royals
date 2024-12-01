use crate::player::{Player, PlayerData};
use royals_core::events::{Action, Event, GameEvent, NotifyEvent, ObtainActionEvent};
use std::sync::mpsc::{Receiver, Sender};

pub struct RemotePlayer {
    pub data: PlayerData,
    pub sender: Sender<GameEvent>,
    pub receiver: Receiver<usize>,
}

impl RemotePlayer {
    pub fn new(name: String, sender: Sender<GameEvent>, receiver: Receiver<usize>) -> Self {
        RemotePlayer {
            data: PlayerData::new(name),
            sender,
            receiver,
        }
    }
}

impl Player for RemotePlayer {
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
