use rand::seq::SliceRandom;

use crate::{
    card::Card,
    event::Event,
    play::{Action, Play},
    player::{Player, PlayerData, PlayerId},
};

static COMPUTER_NAMES: &[&str] = &["Computer Alpha", "Computer Bravo", "Computer Charlie"];

pub struct RandomPlayingComputer {
    pub data: PlayerData,
}

impl RandomPlayingComputer {
    pub fn new(id: PlayerId) -> RandomPlayingComputer {
        let name = COMPUTER_NAMES[id % COMPUTER_NAMES.len()].to_string();
        RandomPlayingComputer {
            data: PlayerData::new(id, name),
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
        hand: &[Card],
        players: &[&String],
        _game_log: &[Event],
        all_protected: bool,
        _active_players: &[PlayerId],
    ) -> Action {
        let mut hand = hand.to_vec();
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
        } else if hand[1] == Card::Countess
            && (play.card == Card::King || play.card == Card::Prince)
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
                    Card::Countess,
                    Card::Princess,
                ];
                p.guess = Some(*cards.choose(&mut rand::thread_rng()).unwrap());
            }
        }
        action
    }
}
