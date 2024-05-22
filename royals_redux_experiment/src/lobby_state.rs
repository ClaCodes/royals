use std::collections::HashMap;

use crate::common::*;

// #[derive(Default, Debug, Clone)]
// pub struct LobbyState {
//     pub lobbies: Vec<Lobby>,
// }

#[derive(Default, Debug, Clone)]
pub struct LobbyState {
    pub lobbies: HashMap<LobbyId, Lobby>,
}

impl LobbyState {
    pub fn get_lobby_for_credentials(
        &self,
        player_credentials: &PlayerCredentials,
    ) -> Option<&Lobby> {
        match self.lobbies.get(&player_credentials.lobby_id) {
            None => None,
            Some(lobby) => match lobby.players.get(&player_credentials.player_id) {
                None => None,
                Some(player) => {
                    if player.secret == player_credentials.player_secret {
                        Some(lobby)
                    } else {
                        None
                    }
                }
            },
        }
    }

    pub fn lobby_matches<P>(&self, player_credentials: &PlayerCredentials, predicate: P) -> bool
    where
        P: FnOnce(&Lobby) -> bool,
    {
        self.get_lobby_for_credentials(player_credentials)
            .map(predicate)
            .unwrap_or(false)
    }
}

#[derive(Default, Debug, Clone)]
pub struct Lobby {
    pub creator: PlayerId,
    pub config: LobbyConfig,
    pub start_time: Option<u32>,
    pub players: HashMap<PlayerId, LobbyPlayer>,
    pub chat: Vec<(PlayerId, String)>,
}

impl Lobby {
    pub fn is_valid(&self) -> bool {
        self.players.len() <= self.config.max_players && !self.players.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.config.max_players
    }

    pub fn countdown_in_progress(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn players_can_join(&self) -> bool {
        self.is_valid() && !self.is_full() && !self.countdown_in_progress()
    }

    pub fn countdown_can_start(&self) -> bool {
        self.is_valid()
            && !self.countdown_in_progress()
            && self.players.len() >= self.config.min_players
    }

    pub fn can_be_kicked(&self, player_id: PlayerId) -> bool {
        !self.creator != player_id && self.players.get(&player_id).is_some()
    }
}

#[derive(Default, Debug, Clone)]
pub struct LobbyConfig {
    pub min_players: usize,
    pub max_players: usize,
}

#[derive(Debug, Clone)]
pub struct LobbyPlayer {
    pub id: PlayerId,
    pub secret: PlayerSecret,
    pub display_name: String,
    pub is_ready: bool,
}
