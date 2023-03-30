use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_player::BevyPlayer;
use events::GameEvent;
use royals_core::run_game;
use sendable::Sendable;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use ui::ui_system;

mod bevy_player;
mod events;
mod sendable;
mod ui;

fn main() {
    let (sender1, receiver1) = channel();
    let (sender2, receiver2) = channel();

    start_game_thread(sender1, receiver2);

    App::new()
        // -----------------------------------------------------
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        // -----------------------------------------------------
        .insert_resource(GameState { last_event: None })
        .insert_resource(Sendable::new(receiver1))
        .insert_resource(Sendable::new(sender2))
        // -----------------------------------------------------
        .add_system(receiver_system)
        .add_system(ui_system)
        // -----------------------------------------------------
        .run();
}

#[derive(Resource)]
pub struct GameState {
    pub last_event: Option<GameEvent>,
}

fn start_game_thread(sender: Sender<GameEvent>, receiver: Receiver<usize>) {
    thread::spawn(move || {
        run_game(move |id| BevyPlayer::new(id, sender, receiver));
    });
}

fn receiver_system(
    receiver: Res<Sendable<Receiver<GameEvent>>>,
    mut game_state: ResMut<GameState>,
) {
    while let Ok(event) = receiver.try_recv() {
        game_state.as_mut().last_event = Some(event);
    }
}
