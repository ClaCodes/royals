use std::str::FromStr;

use crate::{card::Card, Play, PlayerId, player::Player};

#[derive(Debug, PartialEq)]
pub enum Action {
    Quit,
    Play(Play),
}

#[derive(Debug, PartialEq)]
pub enum ConsoleAction {
    Quit,
    Rules,
    CardEffects,
    Card(Card),
    Player(PlayerId),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseActionError;

impl ConsoleAction {
    pub fn info(&self, players: &[Player]) -> String {
        match self {
            ConsoleAction::Quit => "quit".to_string(),
            ConsoleAction::Rules => "display rules".to_string(),
            ConsoleAction::CardEffects => "display card effects".to_string(),
            ConsoleAction::Card(c) => c.rule().to_string(),
            ConsoleAction::Player(id) => players[*id].name.clone(),
        }
    }

    pub fn cmd_str(&self) -> String {
        match self {
            ConsoleAction::Quit => "q".to_string(),
            ConsoleAction::Rules => "r".to_string(),
            ConsoleAction::CardEffects => "c".to_string(),
            ConsoleAction::Card(c) => c.to_string(),
            ConsoleAction::Player(id) => "".to_string() + &id.to_string(),
        }
    }
}

impl FromStr for ConsoleAction {
    type Err = ParseActionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "q" => Ok(ConsoleAction::Quit),
            "r" => Ok(ConsoleAction::Rules),
            "c" => Ok(ConsoleAction::CardEffects),
            _ => {
                if let Ok(p) = Card::from_str(s) {
                    Ok(ConsoleAction::Card(p))
                } else {
                    if let Ok(p) = usize::from_str(s) {
                        Ok(ConsoleAction::Player(p))
                    } else {
                        Err(ParseActionError)
                    }
                }
            }
        }
    }
}
