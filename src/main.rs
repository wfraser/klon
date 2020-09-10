mod action;
mod game_state;
mod ui;

use action::{Action, Location};
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

        match action {
            Action::Quit => break,
            Action::Move(Location::Waste, _) => {
                if game.waste().is_empty() {
                    // FIXME this should be in the move logic
                    ui.write("waste is empty");
                } else {
                    // FIXME temp hax
                    game.take_waste_temp_hax();
                }
            }
            Action::Move(Location::Foundation(_), _)
                | Action::QuickMove(Location::Foundation(_))
                => {
                // TODO: some games do allow this
                ui.write("can't move from the foundation");
            }
            Action::Move(Location::Tableau { column, row }, dest) => {
                ui.write(&format!("move {}:{} to {:?}", column+1, row.unwrap()+1, dest));
            }
            Action::QuickMove(src) => {
                ui.write(&format!("quick move {:?}", src));
            }
            Action::Draw => {
                game.draw_three();
            }
        }
    }

    drop(ui);

    println!("Bye!");
}
