use event::Event;
use game_state::GameState;
use player::{Player, PlayerId};

pub mod card;
pub mod event;
mod game_logic;
mod game_state;
pub mod play;
pub mod player;
mod random_playing_computer;
pub mod utils;

pub fn run_game<C, T>(player_constructor: C)
where
    C: Fn(PlayerId) -> T,
    T: Player + 'static,
{
    let mut game = GameState::new(player_constructor);
    game.run()
}
