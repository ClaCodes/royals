use std::sync::Arc;

use async_trait::async_trait;
use log::warn;
use redux_rs::{MiddleWare, StoreApi};

use crate::lobby_action::{InternalAction, LobbyAction, LobbyPlayerAction, RemovalReason};
use crate::lobby_state::LobbyState;

pub struct ValidatorMiddleware;

#[async_trait]
impl<Inner> MiddleWare<LobbyState, LobbyAction, Inner> for ValidatorMiddleware
where
    LobbyState: Send + Clone + 'static,
    Inner: StoreApi<LobbyState, LobbyAction> + Send + Sync,
{
    async fn dispatch(&self, action: LobbyAction, inner: &Arc<Inner>) {
        fn is_name_valid(name: &String) -> bool {
            !name.trim().is_empty() && name.len() <= 64
        }

        fn is_chat_message_valid(name: &String) -> bool {
            !name.trim().is_empty() && name.len() <= 512
        }

        let state = inner.state_cloned().await;

        let valid = match &action {
            LobbyAction::LobbyPlayer(lobby_player_action) => match lobby_player_action {
                LobbyPlayerAction::Create(_config) => true,
                LobbyPlayerAction::Join(lobby_id) => state
                    .lobbies
                    .get(lobby_id)
                    .map(|l| l.players_can_join())
                    .unwrap_or(false),
                LobbyPlayerAction::Leave(cred) => state.get_lobby_for_credentials(cred).is_some(),
                LobbyPlayerAction::Rename(cred, name) => {
                    state.lobby_matches(cred, |_| is_name_valid(name))
                }
                LobbyPlayerAction::Kick(cred, id_to_kick) => state.lobby_matches(cred, |lobby| {
                    !lobby.countdown_in_progress() && lobby.can_be_kicked(*id_to_kick)
                }),
                LobbyPlayerAction::ChatMessage(cred, message) => {
                    state.lobby_matches(cred, |_| is_chat_message_valid(message))
                }
                LobbyPlayerAction::Ready(cred) => state.get_lobby_for_credentials(cred).is_some(),
                LobbyPlayerAction::NotReady(cred) => {
                    state.get_lobby_for_credentials(cred).is_some()
                }
                LobbyPlayerAction::Start(cred) => state.lobby_matches(cred, |lobby| {
                    lobby.creator == cred.player_id && lobby.countdown_can_start()
                }),
            },
            LobbyAction::Connection(_) => true,
            LobbyAction::Internal(_) => true,
        };

        if !valid {
            // Ignore the action
            warn!("Ignoring invalid action: {:?}", action);
        } else {
            // Continue dispatching the action
            inner.dispatch(action).await;
        }
    }
}

pub struct MapToInternalMiddleware;

#[async_trait]
impl<Inner> MiddleWare<LobbyState, LobbyAction, Inner> for MapToInternalMiddleware
where
    LobbyState: Send + Clone + 'static,
    Inner: StoreApi<LobbyState, LobbyAction> + Send + Sync,
{
    async fn dispatch(&self, action: LobbyAction, inner: &Arc<Inner>) {
        let state = inner.state_cloned().await;

        match action {
            LobbyAction::LobbyPlayer(a) => match a {
                LobbyPlayerAction::Create(config) => {
                    let lobby_id = rand::random();
                    let player_id = rand::random();

                    inner
                        .dispatch(LobbyAction::Internal(InternalAction::LobbyCreated(
                            lobby_id, player_id, config,
                        )))
                        .await;

                    inner
                        .dispatch(LobbyAction::Internal(InternalAction::PlayerAdded(
                            lobby_id,
                            player_id,
                            rand::random(),
                        )))
                        .await;
                }
                LobbyPlayerAction::Join(lobby_id) => {
                    inner
                        .dispatch(LobbyAction::Internal(InternalAction::PlayerAdded(
                            lobby_id,
                            rand::random(),
                            rand::random(),
                        )))
                        .await;
                }
                LobbyPlayerAction::Leave(cred) => {
                    let lobby = state.get_lobby_for_credentials(&cred).unwrap();
                    inner
                        .dispatch(LobbyAction::Internal(InternalAction::PlayerRemoved(
                            cred.lobby_id,
                            cred.player_id,
                            RemovalReason::Left,
                        )))
                        .await;

                    if cred.player_id == lobby.creator {
                        // creator has left: close the lobby
                        inner
                            .dispatch(LobbyAction::Internal(InternalAction::LobbyClosed(
                                cred.lobby_id,
                            )))
                            .await;
                    }
                }
                LobbyPlayerAction::Rename(cred, name) => {
                    inner
                        .dispatch(LobbyAction::Internal(InternalAction::PlayerRenamed(
                            cred.lobby_id,
                            cred.player_id,
                            name,
                        )))
                        .await;
                }
                LobbyPlayerAction::Kick(cred, id_to_kick) => {
                    inner
                        .dispatch(LobbyAction::Internal(InternalAction::PlayerRemoved(
                            cred.lobby_id,
                            id_to_kick,
                            RemovalReason::Kicked,
                        )))
                        .await;
                }
                LobbyPlayerAction::ChatMessage(_, _) => {}
                LobbyPlayerAction::Ready(cred) => {
                    inner
                        .dispatch(LobbyAction::Internal(InternalAction::PlayerReady(
                            cred.lobby_id,
                            cred.player_id,
                        )))
                        .await;
                }
                LobbyPlayerAction::NotReady(cred) => {
                    inner
                        .dispatch(LobbyAction::Internal(InternalAction::PlayerNotReady(
                            cred.lobby_id,
                            cred.player_id,
                        )))
                        .await;
                }
                LobbyPlayerAction::Start(_) => {}
            },
            LobbyAction::Connection(_) => {}
            LobbyAction::Internal(_) => {}
        };
    }
}
