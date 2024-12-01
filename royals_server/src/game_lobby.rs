use crate::{game_logic::GameState, player::Player};
use royals_core::{
    card::Card,
    events::{ActionId, EventEntry},
};

use rand::seq::SliceRandom;

pub struct GameLobby {
    players: Vec<Box<dyn Player>>,
}

impl GameLobby {
    pub fn new() -> Self {
        GameLobby { players: vec![] }
    }

    pub fn add_player<C, T>(&mut self, player_constructor: C)
    where
        C: FnOnce() -> T,
        T: Player + 'static,
    {
        let player = player_constructor();
        self.players.push(Box::new(player));
    }

    pub fn player_names(&self) -> Vec<&String> {
        self.players.iter().map(|p| p.name()).collect::<Vec<_>>()
    }

    pub fn play_round(&mut self) {
        let mut game_log: Vec<EventEntry> = vec![];

        let mut deck_to_shuffle = Card::deck();
        deck_to_shuffle.shuffle(&mut rand::thread_rng());
        let deck = deck_to_shuffle;

        self.players.shuffle(&mut rand::thread_rng());

        let mut state = GameState::new(self.players.len(), &deck, &mut game_log);

        loop {
            let (players_turn, actions) = state.valid_actions();

            if players_turn.is_none() {
                break;
            }

            let chosen_action: ActionId = self.players[players_turn.unwrap()].obtain_action(
                &self.player_names(),
                &GameState::filter_event(&game_log, players_turn),
                &actions,
            );

            state.handle_action(chosen_action, &mut game_log);

            for (i, p) in self.players.iter().enumerate() {
                p.notify(
                    &GameState::filter_event(&game_log, Some(i)),
                    &self.player_names(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        game_lobby::GameLobby,
        player::{Player, PlayerData},
    };
    use royals_core::events::{Action, Event};

    #[test]
    fn player_names_should_return_list_of_names() {
        let lobby = GameLobby {
            players: vec![
                Box::new(TestPlayer::new("Foo")),
                Box::new(TestPlayer::new("Bar")),
            ],
        };

        assert_eq!(lobby.player_names(), vec!["Foo", "Bar"]);
    }

    // Infra ----------------------------------------------------------------

    pub struct TestPlayer {
        pub data: PlayerData,
    }

    impl TestPlayer {
        pub fn new(name: &str) -> Self {
            TestPlayer {
                data: PlayerData::new(name.to_string()),
            }
        }
    }

    impl Player for TestPlayer {
        fn data(&self) -> &PlayerData {
            &self.data
        }

        fn data_mut(&mut self) -> &mut PlayerData {
            &mut self.data
        }

        fn notify(&self, _game_log: &[Event], _players: &[&String]) {
            todo!()
        }

        fn obtain_action(
            &self,
            _players: &[&String],
            _game_log: &[Event],
            _actions: &[Action],
        ) -> usize {
            todo!()
        }
    }
}
