use std::sync::mpsc::Sender;

use bevy::prelude::*;
use bevy_egui::{egui::SidePanel, EguiContexts};

use crate::{sendable::Sendable, GameState};

pub fn ui_system(
    mut contexts: EguiContexts,
    game_state: Res<GameState>,
    mut sender: ResMut<Sendable<Sender<usize>>>,
) {
    let egui_context = contexts.ctx_mut();
    SidePanel::left("left_panel").show(egui_context, |ui| {
        ui.vertical(|ui| {
            ui.label("Royals Bevy debug UI\n");

            if let Some(crate::events::GameEvent::ObtainAction(o)) = &game_state.last_event {
                for (i, action) in o.valid_actions.iter().enumerate() {
                    if ui.button(format!("Action: {:?}", action)).clicked() {
                        sender.send(i).unwrap();
                    }
                }
            }
        });
    });

    SidePanel::right("right_panel").show(egui_context, |ui| {
        ui.vertical(|ui| {
            if let Some(last_event) = &game_state.last_event {
                match last_event {
                    crate::events::GameEvent::Notify(n) => {
                        for player in &n.players {
                            ui.label(format!("Player: {:?}", player));
                        }
                        ui.label("----------------------------");
                        for log in &n.game_log {
                            ui.label(format!("Event: {:?}", log));
                        }
                    }
                    crate::events::GameEvent::ObtainAction(o) => {
                        for player in &o.players {
                            ui.label(format!("Player: {:?}", player));
                        }
                        ui.label("----------------------------");
                        for log in &o.game_log {
                            ui.label(format!("Event: {:?}", log));
                        }
                    }
                }
            }
        });
    });
    // TopBottomPanel::bottom("bottom_panel").show(egui_context, |ui| {
    //     ui.horizontal(|ui| {
    //         ui.vertical(|ui| {
    //             ui.label("Text\n");

    //             ui.checkbox(&mut ui_state.some_value, "set ting");

    //             if ui.button("Button").clicked() {}
    //         });

    //         ui.separator();

    //         ScrollArea::horizontal()
    //             .always_show_scroll(true)
    //             .show(ui, |ui| {
    //                 // for (image, texture, name) in images.iter() {
    //                 //     ui.vertical(|ui| {
    //                 //         if ui
    //                 //             .add(ImageButton::new(texture.texture_id, [180.0, 180.0]))
    //                 //             .clicked()
    //                 //         {
    //                 //             for (_, mat) in materials.iter_mut() {
    //                 //                 mat.base_color_texture = Some(image.0.clone());
    //                 //             }
    //                 //         }
    //                 //         if let Some(n) = name {
    //                 //             ui.label(n.as_str());
    //                 //         }
    //                 //     });
    //                 // }
    //             });
    //     });
    // });
}
