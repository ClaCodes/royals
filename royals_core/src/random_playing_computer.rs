use rand::seq::SliceRandom;

use crate::{
    card::Card,
    event::Event,
    play::{Action, Play},
    player::{Player, PlayerData, PlayerId},
};

pub struct RandomPlayingComputer {
    pub data: PlayerData,
}

impl RandomPlayingComputer {
    pub fn new(data: PlayerData) -> RandomPlayingComputer {
        RandomPlayingComputer { data }
    }
}

impl Player for RandomPlayingComputer {
    fn get_data(&self) -> &PlayerData {
        &self.data
    }

    fn notify(&self, _game_log: &[Event], _players: &[String]) {}

    fn obtain_action(
        &self,
        hand_cards: &[Card],
        players: &[String],
        _game_log: &[Event],
        all_protected: bool,
        _active_players: &[PlayerId],
    ) -> Action {
        let mut hand = hand_cards.to_vec();
        hand.shuffle(&mut rand::thread_rng());
        let mut play = Play {
            card: hand[0],
            opponent: None,
            guess: None,
        };
        if play.card == Card::Princess {
            play = Play {
                card: hand[1],
                opponent: None,
                guess: None,
            };
        } else if hand[1] == Card::Contess && (play.card == Card::King || play.card == Card::Prince)
        {
            play = Play {
                card: hand[1],
                opponent: None,
                guess: None,
            };
        }
        let mut action = Action::Play(play);
        if let Action::Play(p) = &mut action {
            if p.card.needs_opponent() && !all_protected {
                let chosen = players.choose(&mut rand::thread_rng()).unwrap();
                let index = players.iter().position(|x| x == chosen).unwrap();
                p.opponent = Some(index);
            }
            if p.card.needs_guess() && !all_protected {
                let cards = vec![
                    Card::Priest,
                    Card::Baron,
                    Card::Maid,
                    Card::Prince,
                    Card::King,
                    Card::Contess,
                    Card::Princess,
                ];
                p.guess = Some(*cards.choose(&mut rand::thread_rng()).unwrap());
            }
        }
        action
    }
}
