use cli_player::CliPlayer;
use royals_core::run_game;

mod cli_player;

fn main() {
    run_game(CliPlayer::new);
}
