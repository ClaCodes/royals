use crate::common::*;
use crate::lobby_state::LobbyConfig;

#[derive(Debug, Clone)]
pub enum LobbyAction {
    LobbyPlayer(LobbyPlayerAction),
    Connection(ConnectionAction),
    Internal(InternalAction),
}

#[derive(Debug, Clone)]
pub enum LobbyPlayerAction {
    Create(LobbyConfig),
    Join(LobbyId),
    Leave(PlayerCredentials),
    Rename(PlayerCredentials, String),
    Kick(PlayerCredentials, PlayerId),
    ChatMessage(PlayerCredentials, String),
    Ready(PlayerCredentials),
    NotReady(PlayerCredentials),
    Start(PlayerCredentials),
}

#[derive(Debug, Clone)]
pub enum ConnectionAction {
    PlayerDropped(LobbyId, PlayerId),
}

#[derive(Debug, Clone)]
pub enum InternalAction {
    LobbyCreated(LobbyId, PlayerId, LobbyConfig),
    LobbyClosed(LobbyId),
    PlayerAdded(LobbyId, PlayerId, PlayerSecret),
    PlayerRemoved(LobbyId, PlayerId, RemovalReason),
    PlayerRenamed(LobbyId, PlayerId, String),
    PlayerReady(LobbyId, PlayerId),
    PlayerNotReady(LobbyId, PlayerId),
    CountdownStarted(LobbyId),
    GameStarted(LobbyId),
}

#[derive(Debug, Clone)]
pub enum RemovalReason {
    Left,
    Kicked,
    Dropped,
}
