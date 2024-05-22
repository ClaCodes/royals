use std::collections::HashMap;

use log::{info, Level, LevelFilter};
use redux_rs::{middlewares::logger::LoggerMiddleware, Store, StoreApi};

use crate::common::PlayerCredentials;
use crate::lobby_action::{InternalAction, LobbyAction, LobbyPlayerAction};
use crate::lobby_middleware::{MapToInternalMiddleware, ValidatorMiddleware};
use crate::lobby_state::{Lobby, LobbyConfig, LobbyPlayer, LobbyState};
use crate::logger::SimpleLogger;

mod action;
mod common;
mod lobby_action;
mod lobby_middleware;
mod lobby_state;
mod logger;
mod state;

fn reducer(mut state: LobbyState, action: LobbyAction) -> LobbyState {
    match action {
        // Action::PlayerAdd(secret) => State {
        //     players: {
        //         let index = state.players.len();
        //         state.players.push(Player {
        //             id:index,
        //             display_name: "".to_string(),
        //             secret,
        //             connected: false,
        //             active: false,
        //             hand: vec![],
        //         });
        //         state.players
        //     },
        //     ..state
        // },
        // Action::PlayerRemove(index) => State {
        //     players: {
        //         state.players.remove(index);
        //         state.players
        //     },
        //     ..state
        // },
        // Action::PlayerRename(index, new_name) => State {
        //     players: {
        //         if let Some(player) = state.players.get_mut(index) {
        //             player.display_name = new_name;
        //         }
        //         state.players
        //     },
        //     ..state
        // },
        // Action::PlayerSetConnected(index, connected) => State {
        //     players: {
        //         if let Some(player) = state.players.get_mut(index) {
        //             player.connected = connected;
        //         }
        //         state.players
        //     },
        //     ..state
        // },
        // Action::PlayerSetActive(index, active) => State {
        //     players: {
        //         if let Some(player) = state.players.get_mut(index) {
        //             player.active = active;
        //         }
        //         state.players
        //     },
        //     ..state
        // },
        // Action::Internal(_) => state,
        // Action::Incoming(_) => state,
        // Action::Outgoing(_) => state,
        LobbyAction::LobbyPlayer(_) => panic!("only internal actions should arrive at the reducer"),
        LobbyAction::Connection(_) => panic!("only internal actions should arrive at the reducer"),
        LobbyAction::Internal(i) => match i {
            InternalAction::LobbyCreated(id, creator, config) => LobbyState {
                lobbies: {
                    state.lobbies.insert(
                        id,
                        Lobby {
                            creator,
                            config,
                            start_time: None,
                            players: HashMap::new(),
                            chat: vec![],
                        },
                    );
                    state.lobbies
                },
            },
            InternalAction::LobbyClosed(_) => LobbyState { ..state },
            InternalAction::PlayerAdded(lobby_id, player_id, secret) => LobbyState {
                lobbies: {
                    if let Some(lobby) = state.lobbies.get_mut(&lobby_id) {
                        lobby.players.insert(
                            player_id,
                            LobbyPlayer {
                                id: player_id,
                                secret,
                                display_name: "?".to_string(),
                                is_ready: false,
                            },
                        );
                    }
                    state.lobbies
                },
            },

            // Action::PlayerSetConnected(index, connected) => State {
            //     players: {
            //         if let Some(player) = state.players.get_mut(index) {
            //             player.connected = connected;
            //         }
            //         state.players
            //     },
            //     ..state
            // },
            InternalAction::PlayerRemoved(lobby_id, player_id, _reason) => LobbyState {
                lobbies: {
                    if let Some(lobby) = state.lobbies.get_mut(&lobby_id) {
                        lobby.players.remove(&player_id);
                    }
                    state.lobbies
                },
            },
            InternalAction::PlayerRenamed(lobby_id, player_id, name) => LobbyState {
                lobbies: {
                    if let Some(lobby) = state.lobbies.get_mut(&lobby_id) {
                        if let Some(player) = lobby.players.get_mut(&player_id) {
                            player.display_name = name;
                        }
                    }
                    state.lobbies
                },
            },
            InternalAction::PlayerReady(lobby_id, player_id) => LobbyState {
                lobbies: {
                    if let Some(lobby) = state.lobbies.get_mut(&lobby_id) {
                        if let Some(player) = lobby.players.get_mut(&player_id) {
                            player.is_ready = true;
                        }
                    }
                    state.lobbies
                },
            },
            InternalAction::PlayerNotReady(lobby_id, player_id) => LobbyState {
                lobbies: {
                    if let Some(lobby) = state.lobbies.get_mut(&lobby_id) {
                        if let Some(player) = lobby.players.get_mut(&player_id) {
                            player.is_ready = false;
                        }
                    }
                    state.lobbies
                },
            },
            InternalAction::CountdownStarted(_) => LobbyState { ..state },
            InternalAction::GameStarted(_) => LobbyState { ..state },
        },
    }
}

// struct ValidatorMiddleware;
//
// #[async_trait]
// impl<Inner> MiddleWare<State, Action, Inner> for ValidatorMiddleware
// where
//     State: Send + Clone + 'static,
//     Inner: StoreApi<State, Action> + Send + Sync,
// {
//     async fn dispatch(&self, action: Action, inner: &Arc<Inner>) {
//         fn is_secret_valid(state: &State, id: PlayerId, secret: PlayerSecret) -> bool {
//             state
//                 .players
//                 .get(id)
//                 .map(|p| p.secret == secret)
//                 .unwrap_or(false)
//         }
//
//         fn is_move_valid(state: &State, id: PlayerId, move_index: usize) -> bool {
//             state
//                 .players
//                 .get(id)
//                 .map(|p| p.possible_moves.len() > move_index)
//                 .unwrap_or(false)
//         }
//
//         fn is_name_valid(name: &String) -> bool {
//             !name.trim().is_empty() && name.len() <= 64
//         }
//
//         let state = inner.state_cloned().await;
//
//         let valid = match &action {
//             Action::Internal(_) => true,
//             Action::Incoming(incoming) => match incoming {
//                 IncomingAction::PlayerWantsToJoin(name) => {
//                     !state.game_is_running && is_name_valid(name)
//                 }
//                 IncomingAction::PlayerWantsToPlay(id, secret, move_index) => {
//                     is_secret_valid(&state, *id, *secret) && is_move_valid(&state, *id, *move_index)
//                 }
//                 IncomingAction::PlayerWantsToRename(id, secret, name) => {
//                     !state.game_is_running
//                         && is_secret_valid(&state, *id, *secret)
//                         && is_name_valid(name)
//                 }
//             },
//             Action::Outgoing(_) => true,
//         };
//
//         if !valid {
//             // Ignore the action
//             warn!("Invalid action: {:?}", action);
//         } else {
//             // Continue dispatching the action
//             inner.dispatch(action).await;
//         }
//     }
// }

// struct ServerIncomingMiddleware;
//
// #[async_trait]
// impl<Inner> MiddleWare<State, Action, Inner> for ServerIncomingMiddleware
// where
//     State: Send + Clone + 'static,
//     Inner: StoreApi<State, Action> + Send + Sync,
// {
//     async fn dispatch(&self, action: Action, inner: &Arc<Inner>) {
//         let state = inner.state_cloned().await;
//
//         let valid = match &action {
//             Action::Internal(_) => true,
//             // Action::Incoming(incoming) => match incoming {
//             //     IncomingAction::PlayerWantsToJoin(name) => {
//             //         !state.game_is_running && is_name_valid(name)
//             //     }
//             //     IncomingAction::PlayerWantsToPlay(id, secret, move_index) => {
//             //         is_secret_valid(&state, *id, *secret) && is_move_valid(&state, *id, *move_index)
//             //     }
//             //     IncomingAction::PlayerWantsToRename(id, secret, name) => {
//             //         !state.game_is_running
//             //             && is_secret_valid(&state, *id, *secret)
//             //             && is_name_valid(name)
//             //     }
//             // },
//             _ => true,
//         };
//
//         if !valid {
//             // Ignore the action
//             warn!("Invalid action: {:?}", action);
//         } else {
//             // Continue dispatching the action
//             inner.dispatch(action).await;
//         }
//     }
// }

static LOGGER: SimpleLogger = SimpleLogger;
//
// #[tokio::main]
// async fn main() {
//     log::set_logger(&LOGGER)
//         .map(|()| log::set_max_level(LevelFilter::Info))
//         .unwrap();
//
//     info!("Starting...");
//
//     let mut rng = thread_rng();
//     // let player_1_secret = rng.gen();
//     // let player_2_id = rng.gen();
//
//     let logger_middleware = LoggerMiddleware::new(Level::Info);
//
//     let store = Store::new(reducer)
//         .wrap(ValidatorMiddleware)
//         .await
//         .wrap(logger_middleware)
//         .await;
//
//     // store.subscribe(|state: &State| println!("New state: {:?}", state)).await;
//
//     store
//         .dispatch(Action::Incoming(IncomingAction::PlayerWantsToJoin(
//             "Ferris".to_string(),
//         )))
//         .await;
//
//     // store.dispatch(Action::PlayerAdd(player_1_secret)).await;
//     // store.dispatch(Action::PlayerAdd(player_2_id)).await;
//     // // store.dispatch(Action::PlayerRemove(0)).await;
//     // store.dispatch(Action::PlayerSetConnected(0, true)).await;
//     // store
//     //     .dispatch(Action::PlayerRename(0, "Ferris".to_string()))
//     //     .await;
//     // store.dispatch(Action::PlayerSetActive(0, true)).await;
//     // store.dispatch(Action::PlayerSetConnected(0, true)).await;
// }

#[tokio::main]
async fn main() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();

    info!("Starting...");

    let logger_middleware = LoggerMiddleware::new(Level::Info);
    let logger_internal_middleware = LoggerMiddleware::new(Level::Info);

    let store = Store::new_with_state(
        reducer,
        LobbyState {
            lobbies: HashMap::new(),
        },
    )
    .wrap(logger_internal_middleware)
    .await
    .wrap(MapToInternalMiddleware)
    .await
    .wrap(ValidatorMiddleware)
    .await
    .wrap(logger_middleware)
    .await;

    store
        .subscribe(|state: &LobbyState| println!("New state: {:?}", state))
        .await;

    store
        .dispatch(LobbyAction::LobbyPlayer(LobbyPlayerAction::Create(
            LobbyConfig {
                min_players: 2,
                max_players: 2,
            },
        )))
        .await;

    let lobbies = store.state_cloned().await.lobbies;
    let first = lobbies.keys().next().unwrap();

    let creator = lobbies.get(first).unwrap().players.values().next().unwrap();

    store
        .dispatch(LobbyAction::LobbyPlayer(LobbyPlayerAction::Join(*first)))
        .await;

    store
        .dispatch(LobbyAction::LobbyPlayer(LobbyPlayerAction::Rename(
            PlayerCredentials {
                lobby_id: *first,
                player_id: creator.id,
                player_secret: creator.secret,
            },
            "Creator".to_string(),
        )))
        .await;

    store
        .dispatch(LobbyAction::LobbyPlayer(LobbyPlayerAction::Ready(
            PlayerCredentials {
                lobby_id: *first,
                player_id: creator.id,
                player_secret: creator.secret,
            },
        )))
        .await;

    // store.dispatch(Action::PlayerAdd(player_1_secret)).await;
    // store.dispatch(Action::PlayerAdd(player_2_id)).await;
    // // store.dispatch(Action::PlayerRemove(0)).await;
    // store.dispatch(Action::PlayerSetConnected(0, true)).await;
    // store
    //     .dispatch(Action::PlayerRename(0, "Ferris".to_string()))
    //     .await;
    // store.dispatch(Action::PlayerSetActive(0, true)).await;
    // store.dispatch(Action::PlayerSetConnected(0, true)).await;
}
