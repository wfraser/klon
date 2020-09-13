mod action;
mod game_state;
mod ui;

use crate::action::Action;
use crate::game_state::{Card, GameState, Rank, Suit};
use getrandom::getrandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use std::env::args;
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

fn main() {
    let mut deck = vec![];
    for rank in 1u8 ..= 13u8 {
        for suit in [Suit::Spades, Suit::Clubs, Suit::Hearts, Suit::Diamonds].iter().cloned() {
            let rank: Rank = unsafe { std::mem::transmute(rank) };
            let card = Card { suit, rank };
            deck.push(card);
        }
    }

    let seed = match args().nth(1).as_deref() {
        Some("-h") | Some("--help") => {
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

    // Randomize the deck in a repeatable way by seeding a RNG with the given number and using that
    // to do swaps of cards in the deck.
    // The number of permutations of a 52-card deck is 52!, which is a 226-bit number, and we're
    // only using a 64-bit seed, and not doing this n a meticulous way, so obviously this can't
    // generate all possible decks, but it's proooooobably good enough.
    let mut rand = <Pcg32 as SeedableRng>::seed_from_u64(seed);
    for i in 0 .. deck.len() {
        let j = rand.gen_range(i, deck.len());
        deck.swap(i, j);
    }

    let mut game = GameState::new(seed, deck);
    let ui = ui::CursesUI::new();

    loop {
        ui.render(&game);

        let action = loop {
            let input = match ui.get_input() {
                None => break Action::Quit,
                Some(line) => line,
            };

            match input.parse::<Action>() {
                Ok(action) => break action,
                Err(e) => {
                    ui.write(e);
                }
            }
        };

        if let Action::Quit = action {
            break;
        }

        if let Action::Help = action {
            ui.halp();
            continue;
        }

        if let Err(e) = game.apply_action(action) {
            ui.write(e);
            continue;
        }

        if (0..4)
            .all(|i| game.foundation(i)
                .map(|card| card.rank)
                    == Some(Rank::King))
        {
            ui.write("YOU'RE WINNER !"); // lol
        }
    }

    drop(ui);

    println!("That was game #{}.", seed);
    println!("Bye!");
}
