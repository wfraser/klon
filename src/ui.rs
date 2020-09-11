use crate::game_state::{Card, Facing, GameState, Suit};
use crate::init_array;
use pancurses::*;

pub struct CursesUI {
    draw_button: Window,
    waste: Window,
    tableau: [Window; 7],
    foundation: [Window; 4],
    text_window: Window,
}

const WHITE_ON_BLACK: i16 = 0;
const RED_ON_BLACK: i16 = 1;
const BLACK_ON_BLACK: i16 = 2;

/*
#[cfg(windows)]
const A_DIM: chtype = 0x8000_0000;

#[cfg(unix)]
const A_DIM: chtype = 0x0010_0000;
*/

#[derive(Debug, Copy, Clone)]
enum Color {
    Gray,
    Normal,
    Red,
    White,
}

trait WindowExt {
    fn color(&self, color: Color);
}

impl WindowExt for Window {
    fn color(&self, color: Color) {
        use Color::*;
        self.attrset(
            match color {
                Gray => COLOR_PAIR(BLACK_ON_BLACK as chtype) | A_BOLD,
                Normal => COLOR_PAIR(WHITE_ON_BLACK as chtype),
                Red => COLOR_PAIR(RED_ON_BLACK as chtype),
                White => COLOR_PAIR(WHITE_ON_BLACK as chtype) | A_BOLD,
            }
        );
    }
}

impl CursesUI {
    #[cfg(windows)]
    fn platform_specific_init() {
        //unsafe { kernel32::FreeConsole(); }
    }

    #[cfg(unix)]
    fn platform_specific_init() {
        unsafe { libc::setlocale(libc::LC_ALL, b"\0" as *const _ as *const libc::c_char) };
    }

    pub fn new() -> Self {
        Self::platform_specific_init();
        initscr();

        curs_set(0); // hide the cursor
        start_color(); // set up color mode
        use_default_colors();
        nocbreak(); // enable simple terminal line editing, only yield input on newline.

        init_pair(WHITE_ON_BLACK, COLOR_WHITE, COLOR_BLACK);
        init_pair(RED_ON_BLACK, COLOR_RED, COLOR_BLACK);
        init_pair(BLACK_ON_BLACK, COLOR_BLACK, COLOR_BLACK); // must be used with intensifier

        // The stock & waste draw area:
        //
        // 000000 00000000011
        // 123456 12345678901
        // __DD__ _W1 _W2 _W3
        // draw 3 10X 10Y 10Z
        let draw_button = newwin(2,  6, 0, 0);
        let waste       = newwin(2, 11, 0, 8);

        // Stacks of cards:
        let tableau = init_array!(Window, 7, |i| {
            newwin(21, 7, 5, 7 * i as i32)
        });

        // The foundation, where cards are stacked up by suit.
        // Just shows one card at a time.
        let foundation = init_array!(Window, 4, |i| {
            newwin(2, 5, 0, 29 + 5 * i as i32)
        });

        let text_window = newwin(2, 49, 3, 0);
        text_window.nodelay(false); // use blocking getch

        Self {
            draw_button,
            waste,
            tableau,
            foundation,
            text_window,
        }
    }

    fn render_card(win: &Window, card: &Card) {
        let card_str = format!("{}{}", card.rank, card.suit);
        if card_str.len() == 4 { // UTF-8: 3 for suit, 1 for rank
            win.addstr(" "); // pad to two graphemes
        }
        let color = match card.suit {
            Suit::Hearts | Suit::Diamonds => Color::Red,
            Suit::Clubs | Suit::Spades => Color::Normal,
        };
        win.color(color);
        win.addstr(&card_str);
    }

    pub fn render(&self, game: &GameState) {
        self.draw_button.mv(0, 0);
        self.draw_button.color(Color::Gray);
        self.draw_button.attron(A_UNDERLINE);
        self.draw_button.addstr("  DD  ");
        self.draw_button.attroff(A_UNDERLINE);
        self.draw_button.color(Color::Normal);
        let stock_size = game.stock_size().min(3);
        if stock_size == 0 {
            if game.waste().is_empty() {
                self.draw_button.addstr(" empty");
            } else {
                self.draw_button.addstr("recycle");
            }
        } else {
            self.draw_button.addstr(&format!("draw {}", stock_size));
        }
        self.draw_button.refresh();

        let waste = game.waste();
        self.waste.erase();
        self.waste.mv(0, 0);
        if waste.is_empty() {
            self.waste.addstr("\n  empty");
        } else {
            self.waste.color(Color::Gray);
            for i in 0 .. waste.len() {
                if i == waste.len() - 1 {
                    self.waste.attron(A_UNDERLINE);
                    self.waste.addstr(&format!(" W "));
                    self.waste.attroff(A_UNDERLINE);
                    self.waste.mv(1, 0);
                } else {
                    self.waste.addstr("    ");
                }
            }
            for (i, card) in waste.iter().enumerate() {
                Self::render_card(&self.waste, card);
                if i != waste.len() - 1 {
                    self.waste.addstr(" ");
                }
            }
        }
        self.waste.refresh();

        for (i, win) in self.foundation.iter().enumerate() {
            win.mv(0, 0);
            win.color(Color::Gray);
            win.attron(A_UNDERLINE);
            win.addstr(&format!(" 0{} ", (b'A' + i as u8) as char));
            win.attroff(A_UNDERLINE);

            match game.foundation(i) {
                Some(card) => {
                    win.addstr(" ");
                    Self::render_card(win, card);
                }
                None => {
                    win.color(Color::Normal);
                    win.addstr("  () ");
                }
            }

            win.refresh();
        }

        for (i, win) in self.tableau.iter().enumerate() {
            win.erase();
            win.mv(0, 0);
            win.color(Color::Gray);
            //win.addstr("1234567");
            win.attron(A_UNDERLINE);
            win.addstr(&format!("     {}\n", i + 1));
            win.attroff(A_UNDERLINE);
            for (j, (card, facing)) in game.tableau(i).iter().enumerate() {
                win.addstr(&format!("{}{} ", i + 1, (b'A' + j as u8) as char));
                if matches!(facing, Facing::Down) {
                    win.addstr("---\n");
                } else {
                    Self::render_card(win, &card);
                    win.color(Color::Gray);
                    win.addstr("\n");
                }
            }
            win.refresh();
        }

    }

    /*
    pub fn refresh(&self) {
        self.draw_button.refresh();
        self.waste.refresh();
        for win in &self.tableau {
            win.refresh();
        }
        for win in &self.foundation {
            win.refresh();
        }
        self.text_window.refresh();
    }
    */

    pub fn get_input(&self) -> Option<String> {
        let mut line = String::new();

        let prompt = "your move: ";
        self.text_window.mv(0, 0);
        self.text_window.clrtoeol();
        self.text_window.addstr(prompt);
        self.text_window.refresh();
        curs_set(1); // turn on cursor while we're getting input

        loop {
            let input = match self.text_window.getch() {
                Some(input) => input,
                None => {
                    curs_set(0); // turn cursor back off
                    return None;
                }
            };

            if let Input::Character(c) = input {
                if c == '\n' {
                    break;
                }
                line.push(c);
            } else {
                eprintln!("unrecognized input {:?}", input);
            }
        }

        curs_set(0);

        // Clear the text line under the prompt before returning.
        self.text_window.mv(1, 0);
        self.text_window.deleteln();
        Some(line)
    }

    pub fn write(&self, txt: &str) {
        self.text_window.mvaddstr(1, 0, txt);
    }
}

impl Drop for CursesUI {
    fn drop(&mut self) {
        endwin();
    }
}
