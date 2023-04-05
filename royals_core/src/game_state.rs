use std::collections::HashSet;

use rand::seq::SliceRandom;

use crate::{
    card::Card,
    event::EventEntry,
    player::{Player, PlayerId},
    random_playing_computer::RandomPlayingComputer,
};

pub struct GameState {
    pub deck: Vec<Card>,
    pub players: Vec<Box<dyn Player>>,
    pub game_log: Vec<EventEntry>,
    pub players_turn: PlayerId,
}

impl GameState {
    pub fn new<C, T>(player_constructor: C) -> Self
    where
        C: FnOnce() -> T,
        T: Player + 'static,
    {
        let mut state = GameState {
            deck: vec![
                Card::Guard,
                Card::Guard,
                Card::Guard,
                Card::Guard,
                Card::Guard,
                Card::Priest,
                Card::Priest,
                Card::Baron,
                Card::Baron,
                Card::Maid,
                Card::Maid,
                Card::Prince,
                Card::Prince,
                Card::King,
                Card::Countess,
                Card::Princess,
            ],
            players: vec![],
            game_log: vec![],
            players_turn: 0,
        };

        state.add_player(player_constructor);
        state.add_player(RandomPlayingComputer::new);
        state.add_player(RandomPlayingComputer::new);
        state.add_player(RandomPlayingComputer::new);

        //state.players.shuffle(&mut rand::thread_rng()); todo

        state.deck.shuffle(&mut rand::thread_rng());

        for i in 0..state.players.len() {
            state.pick_up_card(i);
        }

        state
    }

    fn add_player<C, T>(&mut self, player_constructor: C)
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

    pub fn active_players(&self) -> HashSet<PlayerId> {
        self.players
            .iter()
            .enumerate()
            .filter(|&(_, p)| p.is_active())
            .map(|(i, _)| i)
            .collect()
    }

    fn other_players(&self) -> HashSet<PlayerId> {
        self.players
            .iter()
            .enumerate()
            .map(|(i, _)| i)
            .filter(|&id| id != self.players_turn)
            .collect()
    }

    pub fn other_active_players(&self) -> HashSet<PlayerId> {
        self.other_players()
            .intersection(&self.active_players())
            .cloned()
            .collect::<HashSet<_>>()
    }

    pub fn all_protected(&self) -> bool {
        self.other_active_players()
            .iter()
            .all(|&id| self.players[id].protected())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        card::Card,
        event::Event,
        game_state::GameState,
        play::Action,
        player::{Player, PlayerData, PlayerId},
    };

    #[test]
    fn player_names_should_return_list_of_names() {
        let state = GameState {
            deck: vec![],
            players: vec![
                Box::new(TestPlayer::new(0, "Foo", false, vec![])),
                Box::new(TestPlayer::new(1, "Bar", false, vec![])),
            ],
            game_log: vec![],
            players_turn: 0,
        };

        assert_eq!(state.player_names(), vec!["Foo", "Bar"]);
    }

    #[test]
    fn active_players_should_return_player_ids_with_non_empty_hand() {
        let state = GameState {
            deck: vec![],
            players: vec![
                Box::new(TestPlayer::new(0, "Foo", false, vec![])),
                Box::new(TestPlayer::new(1, "Bar", false, vec![Card::King])),
            ],
            game_log: vec![],
            players_turn: 0,
        };

        assert_eq!(state.active_players(), HashSet::from([1]));
    }

    #[test]
    fn other_players_should_return_ids_of_others() {
        let state = GameState {
            deck: vec![],
            players: vec![
                Box::new(TestPlayer::new(0, "Foo", false, vec![])),
                Box::new(TestPlayer::new(1, "Bar", false, vec![])),
                Box::new(TestPlayer::new(2, "Baz", false, vec![])),
            ],
            game_log: vec![],
            players_turn: 1, // Baz's turn
        };

        assert_eq!(state.other_players(), HashSet::from([0, 2]));
    }

    #[test]
    fn all_protected_should_return_true_if_no_other_active_player_is_unprotected() {
        let state = GameState {
            deck: vec![],
            players: vec![
                Box::new(TestPlayer::new(0, "Foo", false, vec![])), // inactive
                Box::new(TestPlayer::new(1, "Baz", false, vec![Card::King])), // Baz' turn
                Box::new(TestPlayer::new(2, "Bar", true, vec![])),  // protected
            ],
            game_log: vec![],
            players_turn: 1, // Baz' turn
        };

        assert_eq!(state.all_protected(), true);
    }

    #[test]
    fn all_protected_should_return_false_if_at_least_one_other_active_player_is_unprotected() {
        let state = GameState {
            deck: vec![],
            players: vec![
                Box::new(TestPlayer::new(0, "Foo", false, vec![])), // inactive
                Box::new(TestPlayer::new(1, "Baz", false, vec![Card::King])), // Baz' turn
                Box::new(TestPlayer::new(2, "Bar", true, vec![])),  // protected
                Box::new(TestPlayer::new(3, "Qux", false, vec![Card::Guard])), // unprotected
            ],
            game_log: vec![],
            players_turn: 1, // Baz' turn
        };

        assert_eq!(state.all_protected(), false);
    }

    // Infra ----------------------------------------------------------------

    pub struct TestPlayer {
        pub data: PlayerData,
    }

    impl TestPlayer {
        pub fn new(id: PlayerId, name: &str, protected: bool, hand: Vec<Card>) -> Self {
            let mut player = TestPlayer {
                data: PlayerData::new(name.to_string()),
            };
            player.set_protected(protected);
            *player.hand_mut() = hand;
            player
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
