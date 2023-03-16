use rand::seq::SliceRandom;

use crate::{
    card::Card,
    event::Event,
    play::{Action, Play},
    player::{Player, PlayerId, PlayerInterface},
};

pub struct RandomPlayingComputer {
    pub id: PlayerId,
}

impl PlayerInterface for RandomPlayingComputer {
    fn notify(&self, _game_log: &[Event], _players: &[Player]) {}

    fn obtain_action(
        &self,
        hand_cards: &[Card],
        players: &[Player],
        _game_log: &[Event],
    ) -> Action {
        let mut hand = hand_cards.to_vec();
        hand.shuffle(&mut rand::thread_rng());
        let mut all_protected = true;
        for (ind, p) in players.iter().enumerate() {
            if !p.hand_cards.is_empty() && !p.protected && ind != self.id {
                all_protected = false;
            }
        }
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
                let index = players.iter().position(|x| x.name == chosen.name).unwrap();
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
