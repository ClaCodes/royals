use std::{
    io::{self, BufRead, Write},
    str::FromStr,
};

use itertools::Itertools;

use crate::{
    card::Card,
    event::Event,
    play::{Action, Play},
    player::{Player, PlayerData, PlayerId},
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

#[derive(Debug, PartialEq)]
enum ConsoleAction {
    Quit,
    Rules,
    CardEffects,
    Card(Card),
    Player(PlayerId),
}

#[derive(Debug, PartialEq, Eq)]
struct ParseActionError;

impl ConsoleAction {
    fn info(&self, players: &[&String]) -> String {
        match self {
            ConsoleAction::Quit => "quit".to_string(),
            ConsoleAction::Rules => "display rules".to_string(),
            ConsoleAction::CardEffects => "display card effects".to_string(),
            ConsoleAction::Card(c) => c.rule().to_string(),
            ConsoleAction::Player(id) => players[*id].clone(),
        }
    }

    fn cmd_str(&self) -> String {
        match self {
            ConsoleAction::Quit => "q".to_string(),
            ConsoleAction::Rules => "r".to_string(),
            ConsoleAction::CardEffects => "c".to_string(),
            ConsoleAction::Card(c) => c.to_string(),
            ConsoleAction::Player(id) => id.to_string(),
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

pub struct ConsolePlayer {
    pub data: PlayerData,
}

impl ConsolePlayer {
    fn query_user(
        &self,
        cmds: Vec<ConsoleAction>,
        prompt: &str,
        players: &[&String],
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

    fn prompt_card(&self, cards: &[Card], prompt: &str, players: &[&String]) -> ConsoleAction {
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

    fn prompt_opponent(&self, players: &[&String], active_players: &[PlayerId]) -> ConsoleAction {
        let mut queries = vec![
            ConsoleAction::Quit,
            ConsoleAction::Rules,
            ConsoleAction::CardEffects,
        ];
        let mut pl_ids = vec![];
        for &i in active_players.iter() {
            if i != self.id() {
                queries.push(ConsoleAction::Player(i));
                pl_ids.push(i);
            }
        }
        if pl_ids.len() == 1 {
            return ConsoleAction::Player(pl_ids.pop().unwrap());
        }
        self.query_user(
            queries,
            "Choose opponent against whom you want to play the card:",
            players,
        )
    }

    pub fn format_play(&self, play: &Play, players: &[&String]) -> String {
        let op_str = play
            .opponent
            .map(|op| format!("\n\tOpponent: {}", players[op]));
        let guess_str = play.guess.map(|g| format!("\n\tGuess: {g}"));
        format!(
            "\n\t{}{}{}",
            play.card.to_string(),
            op_str.unwrap_or_default(),
            guess_str.unwrap_or_default()
        )
    }

    fn print_event(&self, event: &Event, players: &[&String]) {
        match &event {
            Event::Play(pl, p) => println!(
                "~ Play: {} played {}",
                players[*pl],
                self.format_play(p, players)
            ),
            Event::DropOut(pl) => println!("~ DropOut: {}", players[*pl]),
            Event::Fold(pl, c, reason) => println!(
                "~ Fold: {} folded {}, because {}",
                players[*pl],
                c.to_string(),
                reason
            ),
            Event::PickUp(pl, c, s) => {
                if let Some(card) = c {
                    println!(
                        "~ PickUp: {} picked up {} , {} cards remaining in deck",
                        players[*pl],
                        card.to_string(),
                        s
                    );
                } else {
                    println!(
                        "~ PickUp: {} picked up *** , {} cards remaining in deck",
                        players[*pl], s
                    );
                }
            }
            Event::LearnedCard(pl, c) => {
                if let Some(card) = c {
                    println!(
                        "~ LearnedCard: {} has card {}",
                        players[*pl],
                        card.to_string()
                    );
                } else {
                    println!("~ LearnedCard: {} has card ***", players[*pl]);
                }
            }
            Event::Winner(pl) => {
                let banner = pl.iter().map(|&p| players[p].clone()).join(", ");
                println!("Winner is {}", banner);
            }
        }
    }
}

impl ConsolePlayer {
    pub fn new(id: PlayerId) -> ConsolePlayer {
        print!("Please Enter Name: ");
        io::stdout().flush().unwrap();

        let name = match io::stdin().lock().lines().next() {
            Some(Ok(line)) => line,
            _ => "You".to_string(),
        };

        ConsolePlayer {
            data: PlayerData::new(id, name),
        }
    }
}

impl Player for ConsolePlayer {
    fn data(&self) -> &PlayerData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut PlayerData {
        &mut self.data
    }

    fn notify(&self, game_log: &[Event], players: &[&String]) {
        println!("================================================");
        for entry in game_log {
            self.print_event(entry, players);
        }
    }

    fn obtain_action(
        &self,
        hand: &[Card],
        players: &[&String],
        game_log: &[Event],
        all_protected: bool,
        active_players: &[PlayerId],
    ) -> Action {
        self.notify(game_log, players);

        let mut card = None;
        while card.is_none() {
            let action = self.prompt_card(&hand, "Choose the card you want to play:", &players);
            match action {
                ConsoleAction::Quit => return Action::GiveUp,
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
                let action = self.prompt_opponent(&players, &active_players);
                match action {
                    ConsoleAction::Quit => return Action::GiveUp,
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
                        Card::Countess,
                        Card::Princess,
                    ],
                    "Choose the card you want to guess the opponent has:",
                    &players,
                );
                match action {
                    ConsoleAction::Quit => return Action::GiveUp,
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
