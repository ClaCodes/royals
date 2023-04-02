use crate::{
    card::Card,
    event::Event,
    event::EventEntry,
    event::EventVisibility,
    game_state::GameState,
    play::Action,
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

    pub fn play_round(&mut self) {
        let mut deck_to_shuffle = Card::deck();
        deck_to_shuffle.shuffle(&mut rand::thread_rng());
        let deck = deck_to_shuffle;
        self.players.shuffle(&mut rand::thread_rng());
        let mut state = GameState::new(self.players.len());

        for i in 0..self.players.len() {
            state.pick_up_card(i, &deck);
        }

        let mut ok = true;
        let mut running = true;

        while running {
            if ok {
                state.pick_up_card(state.players_turn, &deck);
            }
            let actions = state.valid_actions();
            let chosen_action_index = self.players[state.players_turn].obtain_action(
                &self.player_names(),
                &state.filter_event(),
                &actions,
            );
            ok = chosen_action_index < actions.len();
            if ok {
                match &actions[chosen_action_index] {
                    Action::GiveUp => {
                        state.drop_player(state.players_turn, "Player gave up".to_string());
                        running = state.next_player_turn(&deck);
                    }
                    Action::Play(p) => {
                        if ok {
                            state.handle_play(p, &deck);
                            running = state.next_player_turn(&deck);
                        }
                    }
                }
            }
        }
        for mut p in &mut state.game_log {
            p.visibility = EventVisibility::Public;
        }
        let mut best_players: Vec<PlayerId> = vec![];
        let mut best_card: Option<Card> = None;
        for (i, p) in state.players.iter().enumerate() {
            if let Some(player_card) = p.hand().get(0) {
                state.game_log.push(EventEntry {
                    visibility: EventVisibility::Public,
                    event: Event::Fold(i, player_card.clone(), "game is finished".to_string()),
                });
                if let Some(card) = best_card {
                    if card < *player_card {
                        best_players = vec![i];
                        best_card = Some(player_card.clone());
                    } else if card == *player_card {
                        best_players.push(i);
                    }
                } else {
                    best_players = vec![i];
                    best_card = Some(player_card.clone());
                }
            }
        }
        state.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Winner(best_players),
        });
        for p in &self.players {
            p.notify(&state.filter_event(), &self.player_names());
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
