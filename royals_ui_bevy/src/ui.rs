use itertools::Itertools;
use std::sync::mpsc::Sender;

use bevy::prelude::*;
use bevy_egui::{
    egui::{ScrollArea, SidePanel},
    EguiContexts,
};
use royals_core::{
    event,
    play::{Action, Play},
};

use crate::{sendable::Sendable, GameState};

pub fn ui_system(
    mut contexts: EguiContexts,
    game_state: Res<GameState>,
    sender: Res<Sendable<Sender<usize>>>,
) {
    let egui_context = contexts.ctx_mut();
    SidePanel::left("left_panel")
        .min_width(600.0)
        .default_width(600.0)
        .show(egui_context, |ui| {
            ui.vertical(|ui| {
                ui.label("Royals Bevy debug UI\n");

                ScrollArea::vertical()
                    .always_show_scroll(true)
                    .show(ui, |ui| {
                        if let Some(crate::events::GameEvent::ObtainAction(o)) =
                            &game_state.last_event
                        {
                            for (i, action) in o.valid_actions.iter().enumerate() {
                                if ui.button(action_to_string(action, &o.players)).clicked() {
                                    sender.send(i).unwrap();
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
                ScrollArea::vertical()
                    .always_show_scroll(true)
                    .show(ui, |ui| {
                        if let Some(last_event) = &game_state.last_event {
                            match last_event {
                                crate::events::GameEvent::Notify(n) => {
                                    for player in &n.players {
                                        ui.label(format!("Player: {}", player));
                                    }
                                    ui.label("----------------------------");
                                    for event in &n.game_log {
                                        ui.label(format!(
                                            "> {}",
                                            event_to_string(event, &n.players)
                                        ));
                                    }
                                }
                                crate::events::GameEvent::ObtainAction(o) => {
                                    for player in &o.players {
                                        ui.label(format!("Player: {}", player));
                                    }
                                    ui.label("----------------------------");
                                    for event in &o.game_log {
                                        ui.label(format!(
                                            "> {}",
                                            event_to_string(event, &o.players)
                                        ));
                                    }
                                }
                            }
                        }
                    });
            });
        });
}

fn event_to_string(event: &event::Event, players: &Vec<String>) -> String {
    match event {
        event::Event::Play(id, play) => format!("{} plays {:?}", players[*id], play),
        event::Event::Fold(id, card, _s) => format!("{} folds {:?}", players[*id], card),
        event::Event::PickUp(id, card_op, _i) => format!(
            "{} picks up {}",
            players[*id],
            card_op.map(|c| c.to_string()).unwrap_or("?".to_string())
        ),
        event::Event::DropOut(id) => format!("{} drops out", players[*id]),
        event::Event::LearnedCard(id, card_op) => {
            format!(
                "{} learns card {}",
                players[*id],
                card_op.map(|c| c.to_string()).unwrap_or("?".to_string())
            )
        }
        event::Event::Winner(ids) => {
            format!(
                "Winner(s): {}",
                ids.iter().map(|id| players[*id].clone()).join(" ")
            )
        }
    }
}

fn action_to_string(action: &Action, players: &Vec<String>) -> String {
    match action {
        Action::GiveUp => "Give up".to_string(),
        Action::Play(play) => play_to_string(play, players),
    }
}

fn play_to_string(play: &Play, players: &Vec<String>) -> String {
    [
        format!("Play card {}", play.card.to_string()),
        play.opponent
            .map(|oponent_id| format!(" targeting player {}", players[oponent_id].clone()))
            .unwrap_or("".to_string()),
        play.guess
            .map(|card| format!(" guessing card {}", card.to_string()))
            .unwrap_or("".to_string()),
    ]
    .concat()
}
