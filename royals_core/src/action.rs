use crate::Play;

#[derive(Debug, PartialEq)]
pub enum Action {
    GiveUp,
    Play(Play),
}

