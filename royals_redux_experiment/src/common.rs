pub type LobbyId = u128;
pub type PlayerId = u128;
pub type PlayerSecret = u128;

#[derive(Debug, Clone)]
pub struct PlayerCredentials {
    pub lobby_id: LobbyId,
    pub player_id: PlayerId,
    pub player_secret: PlayerSecret,
}
