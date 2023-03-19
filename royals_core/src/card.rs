use itertools::Itertools;
use strum::{EnumMessage, IntoEnumIterator};
use strum_macros::{Display, EnumIter, EnumMessage, EnumString};

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Display, EnumIter, EnumString, EnumMessage)]
pub enum Card {
    #[strum(
        message = "If you play this card, you may choose an opponent and attempt to guess their card. If you guess right they drop out of the game. You may not guess the Guardian."
    )]
    Guard,
    #[strum(message = "If you play this card, you may choose an opponent and see their card.")]
    Priest,
    #[strum(
        message = "If you play this card, you may compare your other card against the card of an opponent. The one with the lower card is drops out of the game. If they are equal no one drops out."
    )]
    Baron,
    #[strum(
        message = "If you play this card, you are protected against all forms of attack for a single round. If the opponets forget and attempt to attack you, they drop out."
    )]
    Maid,
    #[strum(
        message = "If you play this card, you may force an opponent to fold their card and fetch a new one from the deck."
    )]
    Prince,
    #[strum(
        message = "If you play this card, you may choose an opponent and exchange you other card with theirs."
    )]
    King,
    #[strum(
        message = "If you in addition to this card hold either Prince or King, you must play it instead of the King or Prince."
    )]
    Contess,
    #[strum(
        message = "You must never play this card. If you are force to fold this card by any means (for example if you opponent plays the prince), you drop out."
    )]
    Princess,
}

impl Card {
    pub fn rules() -> String {
        Card::iter().map(|c| c.rule()).join("\n")
    }

    pub fn needs_guess(&self) -> bool {
        self == &Card::Guard
    }

    pub fn needs_opponent(&self) -> bool {
        match self {
            Card::Guard | Card::Priest | Card::Baron | Card::Prince | Card::King => true,
            _ => false,
        }
    }

    pub fn rule(&self) -> String {
        return format!(
            "{} [value = {}]: {}",
            self.to_string(),
            self.value(),
            self.get_message().unwrap_or("No rule")
        );
    }

    fn value(&self) -> u8 {
        *self as u8 + 1
    }
}
