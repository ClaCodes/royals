use std::{
    io::{self, BufRead, Write},
    str::FromStr,
};

use itertools::Itertools;

use crate::{
    action::{Action, ConsoleAction},
    card::Card,
    event::Event,
    play::Play,
    player::{Player, PlayerId, PlayerInterface},
};

static RULES: &str = "
*** Royals ***
This is a simple terminal card game. The goal of the game is avoid dropping out until the end of the game and to have
the card with the highest value amongst the players who did not drop out at the end. Typically the less valueable a card is, the
more powerful it is. At the start of the game, every player gets a card that is hidden from other players.
Then the players, whos turn it is, picks up a second card and decide which of the two cards they want to play.
When the card is played an action might be performed based on the type of card it is. Press c to see what card does what.
At the beginning a card is put to the side, that is hidden an not used except for the special case, when the last card played is a Prince.
If all opponents are protected one may choose to not do anything.";

pub struct ConsolePlayer {
    pub id: PlayerId,
}

impl ConsolePlayer {
    pub fn query_user(
        &self,
        cmds: Vec<ConsoleAction>,
        prompt: &str,
        players: &[Player],
    ) -> ConsoleAction {
        let mut op = None;
        print!("\n{}\n", prompt);
        while let None = op {
            for cmd in &cmds {
                println!("- [{}]: {}", cmd.cmd_str(), cmd.info(players));
            }
            print!(">");
            io::stdout().flush().unwrap();
            if let Some(line) = io::stdin().lock().lines().next() {
                if let Ok(s) = ConsoleAction::from_str(&line.unwrap()) {
                    op = Some(s);
                } else {
                    op = None;
                }
            }
        }
        op.unwrap()
    }

    fn prompt_card(&self, cards: &[Card], prompt: &str, players: &[Player]) -> ConsoleAction {
        let mut queries = vec![
            ConsoleAction::Quit,
            ConsoleAction::Rules,
            ConsoleAction::CardEffects,
        ];
        for c in cards {
            queries.push(ConsoleAction::Card(c.clone()));
        }
        self.query_user(queries, prompt, players)
    }

    fn prompt_opponent(&self, players: &[Player]) -> ConsoleAction {
        let mut queries = vec![
            ConsoleAction::Quit,
            ConsoleAction::Rules,
            ConsoleAction::CardEffects,
        ];
        let mut pl_ids = vec![];
        for (i, op) in players.iter().enumerate() {
            if !op.hand_cards.is_empty() && i != self.id {
                queries.push(ConsoleAction::Player(i));
                pl_ids.push(i);
            }
        }
        if queries.len() == 4 {
            return ConsoleAction::Player(pl_ids[0]);
        }
        self.query_user(
            queries,
            "Choose opponent against whom you want to play the card:",
            players,
        )
    }

    fn print_event(&self, event: &Event, players: &[Player]) {
        match &event {
            Event::Play(pl, p) => println!("~ PLay: {} played {}", players[*pl].name, p.info()),
            Event::DropOut(pl) => println!("~ DropOut: {}", players[*pl].name),
            Event::Fold(pl, c, reason) => println!(
                "~ Fold: {} folded {}, because {}",
                players[*pl].name,
                c.to_string(),
                reason
            ),
            Event::PickUp(pl, c, s) => {
                if let Some(card) = c {
                    println!(
                        "~ PickUp: {} picked up {} , {} cards remaining in deck",
                        players[*pl].name,
                        card.to_string(),
                        s
                    );
                } else {
                    println!(
                        "~ PickUp: {} picked up *** , {} cards remaining in deck",
                        players[*pl].name, s
                    );
                }
            }
            Event::LearnedCard(pl, c) => {
                if let Some(card) = c {
                    println!(
                        "~ LearnedCard: {} has card {}",
                        players[*pl].name,
                        card.to_string()
                    );
                } else {
                    println!("~ LearnedCard: {} has card ***", players[*pl].name);
                }
            }
            Event::Winner(pl) => {
                let banner = pl.iter().map(|&p| players[p].name.clone()).join(", ");
                println!("Winner is {}", banner);
            }
        }
    }
}

impl PlayerInterface for ConsolePlayer {
    fn notify(&self, game_log: &[Event], players: &[Player]) {
        println!("================================================");
        for entry in game_log {
            self.print_event(entry, players);
        }
    }

    fn obtain_action(&self, hand_cards: &[Card], players: &[Player], game_log: &[Event]) -> Action {
        let mut all_protected = true;
        for (ind, p) in players.iter().enumerate() {
            if !p.hand_cards.is_empty() && !p.protected && ind != self.id {
                all_protected = false;
            }
        }
        self.notify(game_log, players);

        let mut card = None;
        while card.is_none() {
            let action =
                self.prompt_card(&hand_cards, "Choose the card you want to play:", &players);
            match action {
                ConsoleAction::Quit => return Action::Quit,
                ConsoleAction::Rules => println!("{}", RULES),
                ConsoleAction::CardEffects => println!("{}", Card::rules()),
                ConsoleAction::Card(c) => card = Some(c),
                _ => {}
            }
        }

        let mut opponent = None;
        if card.unwrap().needs_opponent() && !all_protected {
            if players.len() == 1 {
                opponent = Some(0);
            }

            while opponent.is_none() {
                let action = self.prompt_opponent(&players);
                match action {
                    ConsoleAction::Quit => return Action::Quit,
                    ConsoleAction::Rules => println!("{}", RULES),
                    ConsoleAction::CardEffects => println!("{}", Card::rules()),
                    ConsoleAction::Player(c) => opponent = Some(c),
                    _ => {}
                }
            }
        }

        let mut guess = None;
        if card.unwrap().needs_guess() && !all_protected {
            while guess.is_none() {
                let action = self.prompt_card(
                    &vec![
                        Card::Priest,
                        Card::Baron,
                        Card::Maid,
                        Card::Prince,
                        Card::King,
                        Card::Contess,
                        Card::Princess,
                    ],
                    "Choose the card you want to guess the opponent has:",
                    &players,
                );
                match action {
                    ConsoleAction::Quit => return Action::Quit,
                    ConsoleAction::Rules => println!("{}", RULES),
                    ConsoleAction::CardEffects => println!("{}", Card::rules()),
                    ConsoleAction::Card(c) => guess = Some(c),
                    _ => {}
                }
            }
        }

        Action::Play(Play {
            card: card.unwrap(),
            opponent: opponent,
            guess: guess,
        })
    }
}
