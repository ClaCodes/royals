use crate::{
    card::Card,
    event::Event,
    event::EventEntry,
    event::EventVisibility,
    game_state::GameState,
    play::ActionId,
    player::{Player, PlayerId},
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

    fn filter_event(log: &[EventEntry], visible_to: Option<PlayerId>) -> Vec<Event> {
        log.iter()
            .map(|e| match e.visibility {
                EventVisibility::Public => e.event.clone(),
                EventVisibility::Private(player) => {
                    if visible_to.is_none() || player == visible_to.unwrap() {
                        e.event.clone()
                    } else {
                        match e.event {
                            Event::PickUp(p, _, s) => Event::PickUp(p, None, s),
                            Event::LearnedCard(p, _) => Event::LearnedCard(p, None),
                            _ => e.event.clone(),
                        }
                    }
                }
            })
            .collect()
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
                &GameLobby::filter_event(&game_log, players_turn),
                &actions,
            );

            state.handle_action(chosen_action, &deck, &mut game_log);

            for (i, p) in self.players.iter().enumerate() {
                p.notify(
                    &GameLobby::filter_event(&game_log, Some(i)),
                    &self.player_names(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        event::Event,
        game_lobby::GameLobby,
        play::Action,
        player::{Player, PlayerData},
    };

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
