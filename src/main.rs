mod action;
mod game_state;
mod ui;

use action::Action;
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

        if let Action::Quit = action {
            break;
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

    println!("Bye!");
}
