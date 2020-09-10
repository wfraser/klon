mod game_state;
mod ui;

use game_state::{Card, GameState, Rank, Suit};

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

        let action = match parse_action(&game, &input) {
            Ok(action) => action,
            Err(e) => {
                ui.write(e);
                continue;
            }
        };

        if let Action::Quit = action {
            break;
        }

        // FIXME
        if let Action::Move(Location::Waste, _) = action {
            game.take_waste_temp_hax();
        }

        ui.write(&format!("{:?}", action));
    }

    drop(ui);

    println!("Bye!");
}

#[derive(Debug)]
enum Location {
    Waste,
    Foundation(usize),
    Tableau { column: usize, row: Option<usize> },
}

#[derive(Debug)]
enum Action {
    Quit,
    Deal,
    Move(Location, Location),
    QuickMove(Location),
}

fn parse_location(game: &GameState, chars: &mut impl Iterator<Item=char>)
    -> Result<Location, &'static str>
{
    let c = chars.next().unwrap();
    match c {
        'W' => return Ok(Location::Waste),
        '0' => if let Some(idx) = get_int(chars, 'A', 'D') {
            return Ok(Location::Foundation(idx));
        }
        '1' | '2' | '3' | '4' | '5' | '6' | '7' => {
            let column = (c as u32 - '1' as u32) as usize;
            let next = chars.next();
            if next.is_none() {
                // No row specified: refers to the column as a whole. Only valid when used as a
                // destination.
                return Ok(Location::Tableau { column, row: None });
            }
            // Get the length of the column to figure out what the max valid letter is.
            let len = game.tableau(column).len();
            if len != 0 {
                let max = (b'A' + len as u8 - 1) as char;
                if let Some(row) = get_int(next.iter().cloned(), 'A', max) {
                    return Ok(Location::Tableau { column, row: Some(row) });
                }
            }
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

fn parse_action(game: &GameState, s: &str) -> Result<Action, &'static str> {
    match s.to_ascii_uppercase().as_str() {
        "" => return Err("enter 'quit' to exit"),
        "Q" | "QUIT" => return Ok(Action::Quit),
        "DD" => return Ok(Action::Deal),
        _ => (),
    }

    let mut chars = s.chars().map(|c| c.to_ascii_uppercase()).peekable();
    let source = parse_location(game, &mut chars)?;
    if let Location::Tableau { column: _, row } = source {
        if row.is_none() {
            return Err("unrecognized input");
        }
    }

    if chars.peek().is_none() {
        // let the game figure out if this is valid or not
        return Ok(Action::QuickMove(source));
    }

    let dest = parse_location(game, &mut chars)?;
    if let Location::Waste = dest {
        return Err("can't move a card to the waste pile");
    }

    Ok(Action::Move(source, dest))
}
