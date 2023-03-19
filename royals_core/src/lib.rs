use event::Event;
use game_state::GameState;

mod card;
mod console_player;
mod event;
mod game_state;
mod play;
mod player;
mod random_playing_computer;
mod utils;

pub fn run_game() {
    let mut game = GameState::new();
    game.run()
}
