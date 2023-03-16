use std::str::FromStr;

use crate::{card::Card, player::PlayerId};

#[derive(Debug, Clone, PartialEq)]
pub struct Play {
    pub card: Card,
    pub opponent: Option<PlayerId>,
    pub guess: Option<Card>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsePlayError;

impl Play {
    pub fn info(&self) -> String {
        let op_str = self.opponent.map(|op| format!("\n\tOpponent: {op}"));
        let guess_str = self.guess.map(|g| format!("\n\tGuess: {g}"));
        format!(
            "\n\t{}{}{}",
            self.card.to_string(),
            op_str.unwrap_or_default(),
            guess_str.unwrap_or_default()
        )
    }
}

impl FromStr for Play {
    type Err = ParsePlayError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(card) = Card::from_str(s) {
            Ok(Play {
                card: card,
                opponent: None,
                guess: None,
            })
        } else {
            Err(ParsePlayError)
        }
    }
}
