use crate::GameState;
use bevy::prelude::*;
use bevy_egui::{
    egui::{ScrollArea, SidePanel},
    EguiContexts,
};
use itertools::Itertools;
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant, SystemTime},
};
use bevy_renet::{
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport},
        ConnectionConfig, DefaultChannel, RenetClient,
    },
    transport::NetcodeClientPlugin,
    RenetClientPlugin,
};
use royals_core::{
    events,
    events::{Action, ClientEvent, Play},
};
use royals_core::{events::GameEvent, user_name::Username};

#[derive(Component)]
pub struct ClientEventComponent {
    pub e: ClientEvent,
}

pub fn ui_system(mut commands: Commands, mut contexts: EguiContexts, mut game_state:ResMut<GameState>,
    mut transport: Option<ResMut<NetcodeClientTransport>>) {
    let egui_context = contexts.ctx_mut();

    SidePanel::left("left_panel")
        .min_width(600.0)
        .default_width(600.0)
        .show(egui_context, |ui| {
            ui.vertical(|ui| {
                ui.label("Royals Bevy debug UI\n");

                ScrollArea::vertical().drag_to_scroll(true).show(ui, |ui| {
                    if let Some(t) = &mut transport
                    {
                        if ui.button("disconnect").clicked() {
                            t.disconnect();
                            commands.remove_resource::<NetcodeClientTransport>();
                            game_state.as_mut().last_event = None;
                        }
                    } else {
                        // Todo as component
                        let mut name = "bevy".to_owned();
                        let mut address = "127.0.0.1:6969".to_owned();
                        let name_label = ui.label("Your name: ");
                        ui.text_edit_singleline(&mut name).labelled_by(name_label.id);
                        let address_label = ui.label("Server address: ");
                        ui.text_edit_singleline(&mut address).labelled_by(address_label.id);
                        if ui.button("connect").clicked() {
                            let server_addr: SocketAddr = "127.0.0.1:6969".parse().unwrap();
                            let username = Username::from_string("bevy".to_owned());
                            let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
                            let current_time = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap();
                            let client_id = current_time.as_millis() as u64;
                            let authentication = ClientAuthentication::Unsecure {
                                server_addr,
                                client_id,
                                user_data: Some(username.to_netcode_user_data()),
                                protocol_id: 0,
                            };
                            let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
                            commands.insert_resource(transport)
                        }
                    }
                    if let Some(royals_core::events::GameEvent::ObtainAction(o)) =
                        &game_state.last_event
                    {
                        for (i, action) in o.valid_actions.iter().enumerate() {
                            if ui.button(action_to_string(action, &o.players)).clicked() {
                                commands.spawn(ClientEventComponent {
                                    e: ClientEvent { action_id: i },
                                });
                            }
                        }
                    }
                });
            });
        });

    SidePanel::right("right_panel")
        .min_width(600.0)
        .default_width(600.0)
        .show(egui_context, |ui| {
            ui.vertical(|ui| {
                ScrollArea::vertical().drag_to_scroll(true).show(ui, |ui| {
                    if let Some(last_event) = &game_state.last_event {
                        match last_event {
                            royals_core::events::GameEvent::Notify(n) => {
                                for player in &n.players {
                                    ui.label(format!("Player: {}", player));
                                }
                                ui.label("----------------------------");
                                for event in &n.game_log {
                                    ui.label(format!("> {}", event_to_string(event, &n.players)));
                                }
                            }
                            royals_core::events::GameEvent::ObtainAction(o) => {
                                for player in &o.players {
                                    ui.label(format!("Player: {}", player));
                                }
                                ui.label("----------------------------");
                                for event in &o.game_log {
                                    ui.label(format!("> {}", event_to_string(event, &o.players)));
                                }
                            }
                        }
                    }
                });
            });
        });
}

fn event_to_string(event: &events::Event, players: &[String]) -> String {
    match event {
        events::Event::Play(id, play) => format!("{} plays {:?}", players[*id], play),
        events::Event::Fold(id, card, _s) => format!("{} folds {:?}", players[*id], card),
        events::Event::PickUp(id, card_op, _i) => format!(
            "{} picks up {}",
            players[*id],
            card_op.map(|c| c.to_string()).unwrap_or("?".to_string())
        ),
        events::Event::DropOut(id) => format!("{} drops out", players[*id]),
        events::Event::LearnedCard(id, card_op) => {
            format!(
                "{} learns card {}",
                players[*id],
                card_op.map(|c| c.to_string()).unwrap_or("?".to_string())
            )
        }
        events::Event::Winner(ids) => {
            format!(
                "Winner(s): {}",
                ids.iter().map(|id| players[*id].clone()).join(" ")
            )
        }
    }
}

fn action_to_string(action: &Action, players: &[String]) -> String {
    match action {
        Action::GiveUp => "Give up".to_string(),
        Action::Play(play) => play_to_string(play, players),
    }
}

fn play_to_string(play: &Play, players: &[String]) -> String {
    [
        format!("Play card {}", play.card),
        play.opponent
            .map(|oponent_id| format!(" targeting player {}", players[oponent_id].clone()))
            .unwrap_or("".to_string()),
        play.guess
            .map(|card| format!(" guessing card {}", card))
            .unwrap_or("".to_string()),
    ]
    .concat()
}
