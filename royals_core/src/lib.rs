use event::Event;
use game_lobby::GameLobby;
use player::Player;
use random_playing_computer::RandomPlayingComputer;

pub mod card;
pub mod event;
mod game_lobby;
mod game_state;
pub mod play;
pub mod player;
mod random_playing_computer;
pub mod utils;

pub fn run_game<C, T>(player_constructor: C)
where
    C: FnOnce() -> T,
    T: Player + 'static,
{
    let mut lobby = GameLobby::new();
    lobby.add_player(player_constructor);
    lobby.add_player(RandomPlayingComputer::new);
    lobby.add_player(RandomPlayingComputer::new);
    lobby.add_player(RandomPlayingComputer::new);
    lobby.play_round();
}
