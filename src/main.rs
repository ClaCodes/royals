use rand::seq::SliceRandom; //can we do with std rust only
use std::{
    fmt::Debug,
    io::{self, BufRead, Write},
    str::FromStr,
};
use strum::{EnumMessage, IntoEnumIterator};
use strum_macros::{Display, EnumIter, EnumMessage, EnumString};

static RULES: &str = "
*** Royals ***
This is a simple terminal card game. The goal of the game is avoid dropping out until the end of the game and to have
the card with the highest value amongst the players who did not drop out at the end. Typically the less valueable a card is, the
more powerful it is. At the start of the game, every player gets a card that is hidden from other players.
Then the players, whos turn it is, picks up a second card and decide which of the two cards they want to play.
When the card is played an action might be performed based on the type of card it is. Press c to see what card does what.
At the beginning a card is put to the side, that is hidden an not used except for the special case, when the last card played is a Prince.
If all opponents are protected one may choose to not do anything.";

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Display, EnumIter, EnumString, EnumMessage)]
pub enum Card {
    #[strum(
        message = "If you play this card, you may choose an opponent and attempt to guess their card. If you guess right they drop out of the game. You may not guess the Guardian."
    )]
    Guardian,
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
    fn rule(&self) -> String {
        return format!(
            "{} [value = {}]: {}",
            self.to_string(),
            *self as usize + 1,
            self.get_message().unwrap_or("No rule")
        );
    }

    fn rules() -> String {
        Card::iter()
            .map(|c| c.rule().to_string())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    fn needs_guess(&self) -> bool {
        self == &Card::Guardian
    }

    fn needs_opponent(&self) -> bool {
        match self {
            Card::Guardian | Card::Priest | Card::Baron | Card::Prince | Card::King => true,
            _ => false,
        }
    }

    fn name(&self) -> String {
        self.to_string()
    }
}

pub trait PlayerInterface {
    fn notify(&self, game_log: &Vec<Event>, players: &Vec<Player>);
    fn obtain_action(
        &self,
        hand_cards: &Vec<Card>,
        players: &Vec<Player>,
        game_log: &Vec<Event>,
    ) -> Action;
}

struct RandomPlayingComputer {
    ind: usize,
}

impl PlayerInterface for RandomPlayingComputer {
    fn notify(&self, _game_log: &Vec<Event>, _players: &Vec<Player>) {}
    fn obtain_action(
        &self,
        hand_cards: &Vec<Card>,
        players: &Vec<Player>,
        _game_log: &Vec<Event>,
    ) -> Action {
        let mut hand = hand_cards.to_vec();
        hand.shuffle(&mut rand::thread_rng());
        let mut all_protected = true;
        for (ind, p) in players.iter().enumerate() {
            if !p.hand_cards.is_empty() && !p.protected && ind != self.ind {
                all_protected = false;
            }
        }
        let mut play = Play {
            card: hand[0],
            opponent: None,
            guess: None,
        };
        if play.card == Card::Princess {
            play = Play {
                card: hand[1],
                opponent: None,
                guess: None,
            };
        } else if hand[1] == Card::Contess && (play.card == Card::King || play.card == Card::Prince)
        {
            play = Play {
                card: hand[1],
                opponent: None,
                guess: None,
            };
        }
        let mut action = Action::Play(play);
        if let Action::Play(p) = &mut action {
            if p.card.needs_opponent() && !all_protected {
                let chosen = players.choose(&mut rand::thread_rng()).unwrap();
                let index = players.iter().position(|x| x.name == chosen.name).unwrap();
                p.opponent = Some(index);
            }
            if p.card.needs_guess() && !all_protected {
                let cards = vec![
                    Card::Priest,
                    Card::Baron,
                    Card::Maid,
                    Card::Prince,
                    Card::King,
                    Card::Contess,
                    Card::Princess,
                ];
                p.guess = Some(*cards.choose(&mut rand::thread_rng()).unwrap());
            }
        }
        action
    }
}

struct ConsolePlayer {
    ind: usize,
}

impl ConsolePlayer {
    fn query_user(
        &self,
        cmds: Vec<ConsoleAction>,
        prompt: &str,
        players: &Vec<Player>,
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
    fn prompt_card(&self, cards: &Vec<Card>, prompt: &str, players: &Vec<Player>) -> ConsoleAction {
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
    fn prompt_opponent(&self, players: &Vec<Player>) -> ConsoleAction {
        let mut queries = vec![
            ConsoleAction::Quit,
            ConsoleAction::Rules,
            ConsoleAction::CardEffects,
        ];
        let mut pl_ids = vec![];
        for (i, op) in players.iter().enumerate() {
            if !op.hand_cards.is_empty() && i != self.ind {
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
    fn print_event(&self, event: &Event, players: &Vec<Player>) {
        match &event {
            Event::Play(pl, p) => println!("~ PLay: {} played {}", players[*pl].name, p.info()),
            Event::DropOut(pl) => println!("~ DropOut: {}", players[*pl].name),
            Event::Fold(pl, c, reason) => println!(
                "~ Fold: {} folded {}, because {}",
                players[*pl].name,
                c.name(),
                reason
            ),
            Event::PickUp(pl, c, s) => {
                if let Some(card) = c {
                    println!(
                        "~ PickUp: {} picked up {} , {} cards remaining in deck",
                        players[*pl].name,
                        card.name(),
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
                        card.name()
                    );
                } else {
                    println!("~ LearnedCard: {} has card ***", players[*pl].name);
                }
            }
            Event::Winner(pl) => {
                let mut banner = "Winner is ".to_string();
                for p in pl {
                    banner = banner + &players[*p].name + ", ";
                }
                println!("{}", banner);
            }
        }
    }
}

impl PlayerInterface for ConsolePlayer {
    fn notify(&self, game_log: &Vec<Event>, players: &Vec<Player>) {
        println!("================================================");
        for entry in game_log {
            self.print_event(entry, players);
        }
    }
    fn obtain_action(
        &self,
        hand_cards: &Vec<Card>,
        players: &Vec<Player>,
        game_log: &Vec<Event>,
    ) -> Action {
        let mut all_protected = true;
        for (ind, p) in players.iter().enumerate() {
            if !p.hand_cards.is_empty() && !p.protected && ind != self.ind {
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

#[derive(Debug, PartialEq, Eq)]
pub struct ParsePlayError;

#[derive(Debug, Clone, PartialEq)]
pub struct Play {
    card: Card,
    opponent: Option<usize>,
    guess: Option<Card>,
}

impl Play {
    fn info(&self) -> String {
        let mut op_str = String::from("");
        let mut guess_str = String::from("");
        if let Some(op) = &self.opponent {
            op_str = "\n\tOpponent: ".to_string() + &op.to_string();
        }
        if let Some(g) = &self.guess {
            guess_str = "\n\tGuess: ".to_string() + &g.name().to_string();
        }
        format!("\n\t{}{}{}", self.card.name(), op_str, guess_str)
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

#[derive(Debug, PartialEq)]
pub enum Action {
    Quit,
    Play(Play),
}

#[derive(Debug, PartialEq)]
enum ConsoleAction {
    Quit,
    Rules,
    CardEffects,
    Card(Card),
    Player(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseActionError;

impl ConsoleAction {
    fn info(&self, players: &Vec<Player>) -> String {
        match self {
            ConsoleAction::Quit => "quit".to_string(),
            ConsoleAction::Rules => "display rules".to_string(),
            ConsoleAction::CardEffects => "display card effects".to_string(),
            ConsoleAction::Card(c) => c.rule().to_string(),
            ConsoleAction::Player(id) => players[*id].name.clone(),
        }
    }
    fn cmd_str(&self) -> String {
        match self {
            ConsoleAction::Quit => "q".to_string(),
            ConsoleAction::Rules => "r".to_string(),
            ConsoleAction::CardEffects => "c".to_string(),
            ConsoleAction::Card(c) => c.name().to_string(),
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

#[derive(Clone)]
pub enum Event {
    Play(usize, Play),
    Fold(usize, Card, String),
    PickUp(usize, Option<Card>, usize),
    DropOut(usize),
    LearnedCard(usize, Option<Card>),
    Winner(Vec<usize>),
}

#[derive(PartialEq)]
enum EventVisibility {
    Public,
    Private(usize),
}

pub struct EventEntry {
    visibility: EventVisibility,
    event: Event,
}

pub struct Player {
    name: String,
    interface: Box<dyn PlayerInterface>,
    hand_cards: Vec<Card>,
    protected: bool,
}

impl Player {
    fn new(name: String, interface: Box<dyn PlayerInterface>) -> Self {
        Self {
            name,
            interface,
            hand_cards: vec![],
            protected: false,
        }
    }
}

struct GameState {
    deck: Vec<Card>,
    players: Vec<Player>,
    game_log: Vec<EventEntry>,
    players_turn: usize,
    running: bool,
}

impl GameState {
    fn new() -> Self {
        let mut state = GameState {
            deck: vec![
                Card::Guardian,
                Card::Guardian,
                Card::Guardian,
                Card::Guardian,
                Card::Guardian,
                Card::Priest,
                Card::Priest,
                Card::Baron,
                Card::Baron,
                Card::Maid,
                Card::Maid,
                Card::Prince,
                Card::Prince,
                Card::King,
                Card::Contess,
                Card::Princess,
            ],
            players: vec![
                Player::new("You".to_string(), Box::new(ConsolePlayer { ind: 0 })),
                Player::new(
                    "Computer Alpha".to_string(),
                    Box::new(RandomPlayingComputer { ind: 1 }),
                ),
                Player::new(
                    "Computer Bravo".to_string(),
                    Box::new(RandomPlayingComputer { ind: 2 }),
                ),
                Player::new(
                    "Computer Charlie".to_string(),
                    Box::new(RandomPlayingComputer { ind: 3 }),
                ),
            ],
            game_log: vec![],
            players_turn: 0,
            running: true,
        };

        state.deck.shuffle(&mut rand::thread_rng());
        //state.players.shuffle(&mut rand::thread_rng()); todo
        state.pick_up_card(0);
        state.pick_up_card(1);
        state.pick_up_card(2);
        state.pick_up_card(3);

        state
    }

    fn run(&mut self) {
        let mut ok = true;
        while self.running {
            if ok {
                self.pick_up_card(self.players_turn);
            }
            let player_cards = &self.players[self.players_turn].hand_cards;
            let user_action = self.players[self.players_turn].interface.obtain_action(
                &player_cards,
                &self.players,
                &self.filter_event(),
            );

            match user_action {
                Action::Quit => self.running = false,
                Action::Play(p) => {
                    ok = self.is_valid(&p);
                    if ok {
                        self.handle_play(p);
                        self.next_player_turn();
                        // last card is ussually not used
                        self.running =
                            self.running && self.deck.len() > 1 && self.active_player_count() > 1;
                    }
                }
            }
        }
        for mut p in &mut self.game_log {
            p.visibility = EventVisibility::Public;
        }
        let mut best_players: Vec<usize> = vec![];
        let mut best_card: Option<Card> = None;
        for (i, p) in self.players.iter().enumerate() {
            if let Some(player_card) = p.hand_cards.get(0) {
                self.game_log.push(EventEntry {
                    visibility: EventVisibility::Public,
                    event: Event::Fold(i, player_card.clone(), "game is finished".to_string()),
                });
                if let Some(card) = best_card {
                    if card < *player_card {
                        best_players = vec![i];
                        best_card = Some(player_card.clone());
                    } else if card == *player_card {
                        best_players.push(i);
                    }
                } else {
                    best_players = vec![i];
                    best_card = Some(player_card.clone());
                }
            }
        }
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Winner(best_players),
        });
        for p in &self.players {
            p.interface.notify(&self.filter_event(), &self.players);
        }
    }
    fn filter_event(&self) -> Vec<Event> {
        let mut events = vec![];
        for e in &self.game_log {
            match e.visibility {
                EventVisibility::Public => events.push(e.event.clone()),
                EventVisibility::Private(player) => {
                    if player == self.players_turn {
                        events.push(e.event.clone())
                    } else {
                        match e.event {
                            Event::PickUp(p, _, s) => events.push(Event::PickUp(p, None, s)),
                            Event::LearnedCard(p, _) => events.push(Event::LearnedCard(p, None)),
                            _ => events.push(e.event.clone()),
                        }
                    }
                }
            }
        }
        events
    }
    fn pick_up_card(&mut self, p: usize) {
        let next_card = self.deck.pop().unwrap();
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Private(p),
            event: Event::PickUp(p, Some(next_card.clone()), self.deck.len()),
        });
        self.players[p].hand_cards.push(next_card);
    }
    fn drop_player(&mut self, p: usize, reason: String) {
        let op_card = self.players[p].hand_cards.pop().unwrap();
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Fold(p, op_card, reason),
        });
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::DropOut(p),
        });
    }
    fn active_player_count(&mut self) -> i32 {
        let mut count: i32 = 0;
        for p in &self.players {
            if !p.hand_cards.is_empty() {
                count += 1;
            }
        }
        count
    }
    fn next_player_turn(&mut self) {
        self.players_turn = (self.players_turn + 1) % self.players.len();
        while self.players[self.players_turn].hand_cards.len() == 0 {
            self.players_turn = (self.players_turn + 1) % self.players.len();
        }
    }
    fn is_valid(&self, play: &Play) -> bool {
        let mut all_protected = true;
        for (ind, p) in self.players.iter().enumerate() {
            if !p.hand_cards.is_empty() && !p.protected && ind != self.players_turn {
                all_protected = false;
            }
        }
        if play.card == Card::Princess {
            return false;
        }
        if self.players[self.players_turn].hand_cards[0] == Card::Contess
            || self.players[self.players_turn].hand_cards[1] == Card::Contess
        {
            if play.card == Card::Prince || play.card == Card::King {
                return false;
            }
        }
        if play.opponent.is_none() && play.card.needs_opponent() {
            if !all_protected {
                return false;
            }
        }
        if let Some(op) = play.opponent {
            if op == self.players_turn {
                return false;
            }
            if self.players[op].hand_cards.is_empty() {
                return false;
            }
        }
        true
    }
    fn handle_play(&mut self, p: Play) {
        let index = self.players[self.players_turn]
            .hand_cards
            .iter()
            .position(|x| *x == p.card)
            .unwrap();
        self.players[self.players_turn].hand_cards.remove(index);
        self.game_log.push(EventEntry {
            visibility: EventVisibility::Public,
            event: Event::Play(self.players_turn, p.clone()),
        });
        if let Some(opponent) = &p.opponent {
            let mut all_protected = true;
            for (ind, p) in self.players.iter().enumerate() {
                if !p.hand_cards.is_empty() && !p.protected && ind != self.players_turn {
                    all_protected = false;
                }
            }
            // do not attack protected player
            if self.players[*opponent].protected && !all_protected {
                self.drop_player(self.players_turn, "attacked a protected player".to_string());
                return;
            }
        }
        self.players[self.players_turn].protected = false;
        match p.card {
            Card::Guardian => {
                if let Some(op) = p.opponent {
                    let g = p.guess.unwrap();
                    if self.players[op].hand_cards[0] == g {
                        self.drop_player(op, "opponent guess the hand card".to_string())
                    }
                }
            }
            Card::Priest => {
                if let Some(op) = p.opponent {
                    self.game_log.push(EventEntry {
                        visibility: EventVisibility::Private(self.players_turn),
                        event: Event::LearnedCard(op, Some(self.players[op].hand_cards[0].clone())),
                    });
                }
            }
            Card::Baron => {
                if let Some(op) = p.opponent {
                    let op_card = self.players[op].hand_cards[0];
                    let player_card = self.players[self.players_turn].hand_cards[0];
                    if op_card < player_card {
                        self.drop_player(op, "smaller card then opponent".to_string());
                    } else if player_card < op_card {
                        self.drop_player(
                            self.players_turn,
                            "smaller card then opponent".to_string(),
                        );
                    }
                }
            }
            Card::Maid => {
                self.players[self.players_turn].protected = true;
            }
            Card::Prince => {
                if let Some(op) = p.opponent {
                    if self.players[op].hand_cards[0] == Card::Princess {
                        self.drop_player(op, "forced to play the princess".to_string());
                    } else {
                        let folded = self.players[op].hand_cards.pop().unwrap();
                        self.game_log.push(EventEntry {
                            visibility: EventVisibility::Public,
                            event: Event::Fold(
                                op,
                                folded,
                                "opponent has played prince to force it".to_string(),
                            ),
                        });
                        self.pick_up_card(op);
                    }
                }
            }
            Card::King => {
                if let Some(op) = p.opponent {
                    let op_card = self.players[op].hand_cards.pop().unwrap();
                    let player_card = self.players[self.players_turn].hand_cards.pop().unwrap();
                    self.players[op].hand_cards.push(player_card);
                    self.players[self.players_turn].hand_cards.push(op_card);
                }
            }
            Card::Contess => {}
            Card::Princess => self.drop_player(
                self.players_turn,
                "playing the princess is illegal".to_string(),
            ),
        }
    }
}

fn main() {
    let mut game = GameState::new();
    game.run()
}
