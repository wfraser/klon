mod game_state;
mod ui;

use game_state::{Card, GameState, Location, Rank, Suit};

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

    let mut game = GameState::new(deck);

    game.draw_three();

    let ui = ui::CursesUI::new();

    loop {
        ui.render(&game);
        let input = match ui.get_input() {
            None => break,
            Some(line) => line,
        };

        let loc = match parse_location(&game, &input) {
            Ok(Some(loc)) => loc,
            Ok(None) => break,
            Err(e) => {
                ui.write(e);
                continue;
            }
        };

        ui.write(&format!("{:?}", loc));
    }

    drop(ui);

    println!("Bye!");
}

fn parse_location(game: &GameState, s: &str) -> Result<Option<Location>, &'static str> {
    if s == "" {
        return Err("enter 'quit' to exit");
    }
    if s == "q" || s == "quit" {
        return Ok(None);
    }
    let mut chars = s.chars().map(|c| c.to_ascii_uppercase());
    let c = chars.next().unwrap();
    match c {
        'W' => if let Some(idx) = get_int(chars, '1', '3') {
            return Ok(Some(Location::Waste(idx)));
        }
        '0' => if let Some(idx) = get_int(chars, 'A', 'D') {
            return Ok(Some(Location::Foundation(idx)));
        }
        '1' | '2' | '3' | '4' | '5' | '6' | '7' => {
            let column = (c as u32 - '1' as u32) as usize;
            let next = chars.next();
            if next.is_none() {
                // No row specified: refers to the column as a whole. Only valid when used as a
                // destination.
                return Ok(Some(Location::Tableau { column, row: None }));
            }
            // Get the length of the column to figure out what the max valid letter is.
            let len = game.tableau(column).len();
            if len != 0 {
                let max = (b'A' + len as u8 - 1) as char;
                if let Some(row) = get_int(next.iter().cloned(), 'A', max) {
                    return Ok(Some(Location::Tableau { column, row: Some(row) }));
                }
            }
        }
        'D' => if chars.next() == Some('D') {
            return Ok(Some(Location::Deal));
        }
        _ => (),
    }
    Err("unrecognized input")
}

fn get_int(mut chars: impl Iterator<Item = char>, min: char, max: char) -> Option<usize> {
    if let Some(c) = chars.next() {
        if c as u32 >= min as u32 && c as u32 <= (max as u32) {
            return Some((c as u32 - min as u32) as usize);
        }
    }
    None
}
