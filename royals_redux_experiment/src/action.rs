// use crate::common::*;
// use crate::state::Move;
//
// #[derive(Debug, Clone)]
// pub enum Action {
//     Internal(InternalAction),
//     Incoming(IncomingAction),
//     Outgoing(OutgoingAction),
// }
//
// #[derive(Debug, Clone)]
// pub enum InternalAction {
//     PlayerAdd(PlayerSecret),
//     PlayerRemove(PlayerIndex),
//     PlayerRename(PlayerIndex, String),
//     PlayerSetConnected(PlayerIndex, bool),
//     PlayerSetActive(PlayerIndex, bool),
// }
//
// #[derive(Debug, Clone)]
// pub enum IncomingAction {
//     PlayerWantsToJoin(String),
//     PlayerWantsToPlay(PlayerIndex, PlayerSecret, usize),
//     PlayerWantsToRename(PlayerIndex, PlayerSecret, String),
// }
//
// #[derive(Debug, Clone)]
// pub enum OutgoingAction {
//     PublicGameStarted,
//     PublicGameFinished,
//     PublicPlayerUpdated(PlayerIndex, String),
//     PublicPlayerPlayed(PlayerIndex, Move),
//     PublicPlayerLost(PlayerIndex),
//     PublicPlayerWon(PlayerIndex),
//     PublicNextTurn(PlayerIndex),
//
//     PrivatePlayerJoined(PlayerIndex, PlayerSecret),
//     PrivateRequestMove(PlayerIndex, Vec<Move>),
// }
