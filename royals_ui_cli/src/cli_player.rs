use std::{
    io::{self, BufRead, Write},
    str::FromStr,
};

use itertools::Itertools;

use royals_core::{
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

fn format_play(play: &Play, players: &[&String]) -> String {
    let op_str = play
        .opponent
        .map(|op| format!("\tOpponent: {}", players[op]));
    let guess_str = play.guess.map(|g| format!("\tGuess: {g}"));
    format!(
        "{}{}{}",
        play.card.to_string(),
        op_str.unwrap_or_default(),
        guess_str.unwrap_or_default()
    )
}

pub struct CliPlayer {
    pub data: PlayerData,
}

impl CliPlayer {
    fn query_user(&self, actions: &[Action], players: &[&String]) -> usize {
        loop {
            println!("- [{}]:\t{}", "r", "display rules");
            println!("- [{}]:\t{}", "c", "display card effects");
            for (i, a) in actions.iter().enumerate() {
                match a {
                    Action::GiveUp => println!("- [{}]:\t{}", i.to_string(), "give up"),
                    Action::Play(p) => {
                        println!("- [{}]:\t{}", i.to_string(), format_play(p, players))
                    }
                }
            }
            print!(">");
            io::stdout().flush().unwrap();
            if let Some(line) = io::stdin().lock().lines().next() {
                match line.unwrap_or_default().as_str() {
                    "r" => println!("{}", RULES),
                    "c" => println!("{}", Card::rules()),
                    s => {
                        if let Ok(p) = usize::from_str(&s) {
                            return p;
                        }
                    }
                }
            }
        }
    }

    fn print_event(&self, event: &Event, players: &[&String]) {
        match &event {
            Event::Play(pl, p) => println!(
                "~ Play:\t\t{} played\n\t\t{}\n",
                players[*pl],
                format_play(p, players)
            ),
            Event::DropOut(pl) => println!("~ DropOut:\t{}\n", players[*pl]),
            Event::Fold(pl, c, reason) => println!(
                "~ Fold:\t\t{} folded {}, because {}\n",
                players[*pl],
                c.to_string(),
                reason
            ),
            Event::PickUp(pl, c, s) => {
                if let Some(card) = c {
                    println!(
                        "~ PickUp:\t{} picked up {} , {} cards remaining in deck\n",
                        players[*pl],
                        card.to_string(),
                        s
                    );
                } else {
                    println!(
                        "~ PickUp:\t{} picked up *** , {} cards remaining in deck\n",
                        players[*pl], s
                    );
                }
            }
            Event::LearnedCard(pl, c) => {
                if let Some(card) = c {
                    println!(
                        "~ LearnedCard:\t{} has card {}\n",
                        players[*pl],
                        card.to_string()
                    );
                } else {
                    println!("~ LearnedCard:\t{} has card ***\n", players[*pl]);
                }
            }
            Event::Winner(pl) => {
                let banner = pl.iter().map(|&p| players[p].clone()).join(", ");
                println!("Winner is {}\n", banner);
            }
        }
    }
}

impl CliPlayer {
    pub fn new(id: PlayerId) -> CliPlayer {
        print!("Please Enter Name: ");
        io::stdout().flush().unwrap();

        let name = match io::stdin().lock().lines().next() {
            Some(Ok(line)) => line,
            _ => "You".to_string(),
        };

        CliPlayer {
            data: PlayerData::new(id, name),
        }
    }
}

impl Player for CliPlayer {
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
        players: &[&String],
        game_log: &[Event],
        valid_actions: &[Action],
    ) -> usize {
        self.notify(game_log, players);
        self.query_user(valid_actions, players)
    }
}
