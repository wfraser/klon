mod action;
mod game_state;
mod ui;

use crate::action::Action;
use crate::game_state::{Card, GameState, Rank, Suit};
use crate::ui::CursesUI;
use getrandom::getrandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::exit;

#[macro_export]
macro_rules! init_array {
    ($ty:ty, $n:literal, $init:expr) => {
        {
            use std::mem::{self, MaybeUninit};

            let mut uninit: [MaybeUninit<$ty>; $n] = unsafe {
                // This is safe because it's an array of MaybeUninit, which do not require
                // initialization themselves.
                MaybeUninit::uninit().assume_init()
            };

            for i in 0 .. $n {
                uninit[i] = MaybeUninit::new($init(i));
            }

            // This is safe because the array is fully initialized now.
            unsafe { mem::transmute::<_, [$ty; $n]>(uninit) }
        }
    }
}

struct Game {
    state: GameState,
    undo: Vec<GameState>,
    moves: Vec<Action>,
    ui: CursesUI,
    input_file: Option<BufReader<File>>,
}

impl Game {
    pub fn new(game_number: u64) -> Self {
        let mut deck = vec![];
        for &rank in Rank::all() {
            for &suit in Suit::all() {
                let card = Card { suit, rank };
                deck.push(card);
            }
        }

        // Randomize the deck in a repeatable way by seeding a RNG with the given number and using that
        // to do swaps of cards in the deck.
        // The number of permutations of a 52-card deck is 52!, which is a 226-bit number, and we're
        // only using a 64-bit seed, and not doing this n a meticulous way, so obviously this can't
        // generate all possible decks, but it's proooooobably good enough.
        let mut rand = <Pcg32 as SeedableRng>::seed_from_u64(game_number);
        for i in 0 .. deck.len() {
            let j = rand.gen_range(i, deck.len());
            deck.swap(i, j);
        }

        let state = GameState::new(game_number, deck);
        let ui = CursesUI::new();

        Self {
            state,
            undo: vec![],
            moves: vec![],
            ui,
            input_file: None,
        }
    }

    fn get_input_text(&mut self) -> Result<Option<String>, String> {
        loop {
            let input = match self.input_file.as_mut() {
                Some(file) => {
                    let mut line = String::new();
                    file.read_line(&mut line)
                        .map_err(|e| e.to_string())?;
                    if line.is_empty() {
                        // EOF
                        self.input_file = None;
                        continue;
                    }
                    if line.starts_with('#') {
                        continue;
                    }
                    Some(line)
                }
                None => self.ui.get_input(),
            };

            if let Some(ref input) = input {
                let lc = input.trim().to_ascii_lowercase();
                if lc.starts_with("load ") {
                    let f = File::open(&input.trim()[5..])
                        .map_err(|e| e.to_string())?;
                    self.input_file = Some(BufReader::new(f));
                    continue;
                }
                if lc.starts_with("log ") {
                    let mut f = File::create(&input.trim()[4..])
                        .map_err(|e| e.to_string())?;
                    writeln!(f, "# game {}", self.state.game_number()).map_err(|_| "write error")?;
                    for action in &self.moves {
                        writeln!(f, "{}", action).map_err(|_| "write error")?;
                    }
                    self.ui.write("log file written");
                    continue;
                }
                if lc == "undo" {
                    if let Some(state) = self.undo.pop() {
                        self.moves.pop();
                        self.state = state;
                        self.ui.render(&self.state);
                    } else {
                        self.ui.write("no moves to undo");
                    }
                    continue;
                }
            }

            break Ok(input);
        }
    }

    fn main_loop(&mut self) {
        loop {
            self.ui.render(&self.state);

            let input = match self.get_input_text() {
                Err(e) => {
                    self.ui.write(&e);
                    continue;
                }
                Ok(Some(input)) => input,
                Ok(None) => return,
            };

            let action = match input.trim().parse::<Action>() {
                Ok(action) => action,
                Err(e) => {
                    self.ui.write(e);
                    continue;
                }
            };

            let prev_state = self.state.clone();
            if let Err(e) = self.state.apply_action(&action) {
                self.ui.write(e);
                continue;
            }

            if let Action::Quit = action {
                break;
            }

            if let Action::Help = action {
                self.ui.halp();
                continue;
            }

            self.undo.push(prev_state);
            self.moves.push(action.clone());

            if (0..4)
                .all(|i| self.state.foundation(i)
                    .map(|card| card.rank)
                        == Some(Rank::King))
            {
                self.ui.write("YOU'RE WINNER !"); // lol
            }
        }
    }

    pub fn end(self) -> GameState {
        self.state
    }
}

fn main() {
    let seed = match args().nth(1).as_deref() {
        Some("-h") | Some("--help") | Some("-V") | Some("--version") => {
            eprintln!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            eprintln!("usage: {} [<game number>]", args().next().unwrap());
            exit(1);
        }
        Some(n) => {
            match n.parse::<u64>() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("invalid game number: {}", e);
                    exit(2);
                }
            }
        }
        None => {
            let mut bytes = [0u8; 8];
            getrandom(&mut bytes).expect("unable to get random bytes");
            u64::from_le_bytes(bytes)
        }
    };

    let mut game = Game::new(seed);
    game.main_loop();
    let end_state = game.end();

    println!("That was game #{}.", seed);
    println!("You scored {} points.", end_state.score());
    println!("Bye!");
}
