mod action;
mod game_state;
mod ui;

use crate::action::{Action, Destination, Source};
use crate::game_state::{Card, GameState, Rank, Suit};
use getrandom::getrandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
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
    for &rank in Rank::all() {
        for &suit in Suit::all() {
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


    let mut input_file: Option<BufReader<File>> = None;
    'main: loop {
        ui.render(&game);

        let action = 'action: loop {
            let input = if let Some(file) = input_file.as_mut() {
                let mut line = String::new();
                file.read_line(&mut line).unwrap();
                line = line.trim().to_owned();
                if line.is_empty() {
                    input_file = None;
                    continue;
                }
                line
            } else {
                loop {
                    let input = match ui.get_input() {
                        None => break 'action Action::Quit,
                        Some(line) => line,
                    };

                    if input.to_ascii_lowercase().starts_with("load ") {
                        input_file = match File::open((&input[5..]).trim()) {
                            Ok(f) => Some(BufReader::new(f)),
                            Err(e) => {
                                ui.write(&e.to_string());
                                continue;
                            }
                        };
                        continue 'main;
                    }

                    break input;
                }
            };

            match input.parse::<Action>() {
                Ok(action) => break action,
                Err(e) => {
                    ui.write(e);
                }
            }
        };

        if let Err(e) = game.apply_action(&action) {
            ui.write(e);
            continue;
        }

        log_action(&action, io::stderr()).unwrap();

        if let Action::Quit = action {
            break;
        }

        if let Action::Help = action {
            ui.halp();
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

fn log_action(action: &Action, mut w: impl Write) -> io::Result<()> {
    match action {
        Action::Quit => writeln!(w, "QUIT")?,
        Action::Draw => writeln!(w, "DD")?,
        Action::Move(src, dst) => {
            match src {
                Source::Waste => write!(w, "W")?,
                Source::Tableau { column, row } =>
                    write!(w, "{}{}", column + 1, (b'A' + *row as u8) as char)?,
            }
            match dst {
                Destination::Foundation(idx) =>
                    write!(w, "0{}", (b'A' + *idx as u8) as char)?,
                Destination::Tableau(column) => write!(w, "{}", column + 1)?,
            }
            writeln!(w)?;
        }
        Action::QuickMove(src) => match src {
            Source::Waste => writeln!(w, "W")?,
            Source::Tableau { column, row } =>
                writeln!(w, "{}{}", column + 1, (b'A' + *row as u8) as char)?,
        }
        _ => (),
    }
    Ok(())
}
