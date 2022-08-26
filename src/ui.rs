use crate::game_state::{Card, Color as CardColor, Facing, GameState};
use pancurses::*;

pub struct CursesUI {
    main_window: Window,
    draw_button: Window,
    waste: Window,
    tableau: [Window; 7],
    foundation: [Window; 4],
    text_window: Window,
}

const WHITE_ON_BLACK: i16 = 0;
const RED_ON_BLACK: i16 = 1;
const BLACK_ON_BLACK: i16 = 2;

#[derive(Debug, Copy, Clone)]
enum Color {
    Gray,
    Normal,
    Red,
}

trait WindowExt {
    fn color(&self, color: Color);
    fn underline(&self, enabled: bool);
}

impl WindowExt for Window {
    fn color(&self, color: Color) {
        use Color::*;
        self.attrset(
            match color {
                Gray => COLOR_PAIR(BLACK_ON_BLACK as chtype) | A_BOLD,
                Normal => COLOR_PAIR(WHITE_ON_BLACK as chtype),
                Red => COLOR_PAIR(RED_ON_BLACK as chtype),
                //White => COLOR_PAIR(WHITE_ON_BLACK as chtype) | A_BOLD,
            }
        );
    }

    #[cfg(unix)]
    fn underline(&self, enabled: bool) {
        if enabled {
            self.attron(A_UNDERLINE);
        } else {
            self.attroff(A_UNDERLINE);
        }
    }

    // Underline doesn't do what we want on pdcurses win32.
    #[cfg(windows)]
    fn underline(&self, _enabled: bool) {}
}

// Used to make an array of a type that needs explicit initialization.
macro_rules! init_array {
    ([$ty:ty; $n:literal], $init:expr) => {
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
        let main_window = initscr();

        curs_set(0); // hide the cursor
        start_color(); // set up color mode
        use_default_colors();
        nocbreak(); // enable simple terminal line editing, only yield input on newline.

        init_pair(WHITE_ON_BLACK, COLOR_WHITE, COLOR_BLACK);
        init_pair(RED_ON_BLACK, COLOR_RED, COLOR_BLACK);

        // This must be used with A_BOLD to be visible, as dark gray.
        #[cfg(unix)]
        init_pair(BLACK_ON_BLACK, COLOR_BLACK, COLOR_BLACK);

        // pdcurses win32 can't do gray, so do this dark blue instead.
        #[cfg(windows)]
        init_pair(BLACK_ON_BLACK, COLOR_BLUE, COLOR_BLACK);

        // The stock & waste draw area:
        //
        // 000000 00000000011
        // 123456 12345678901
        // __DD__ _W1 _W2 _W3
        // draw 3 10X 10Y 10Z
        let draw_button = newwin(2,  6, 1, 0);
        let waste       = newwin(2, 11, 1, 8);

        // Stacks of cards:
        let tableau = init_array!([Window; 7], |i| {
            newwin(21, 7, 6, 7 * i as i32)
        });

        // The foundation, where cards are stacked up by suit.
        // Just shows one card at a time.
        let foundation = init_array!([Window; 4], |i| {
            newwin(2, 5, 1, 29 + 5 * i as i32)
        });

        let text_window = newwin(2, 49, 4, 0);
        text_window.nodelay(false); // use blocking getch

        Self {
            main_window,
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
        let color = match card.suit.color() {
            CardColor::Red => Color::Red,
            CardColor::Black => Color::Normal,
        };
        win.color(color);
        win.addstr(&card_str);
    }

    pub fn render(&self, game: &GameState) {
        self.main_window.mvaddstr(0, 0, &format!("game #{}", game.game_number()));

        let points = format!("{}pts", game.score());
        self.main_window.clrtoeol();
        self.main_window.mvaddstr(0, 47 - points.len() as i32, &points);
        self.main_window.refresh();

        self.draw_button.mv(0, 0);
        self.draw_button.color(Color::Gray);
        self.draw_button.underline(true);
        self.draw_button.addstr("  DD  ");
        self.draw_button.underline(false);
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
                    self.waste.underline(true);
                    self.waste.addstr(" W ");
                    self.waste.underline(false);
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
            win.underline(true);
            win.addstr(&format!(" 0{} ", (b'A' + i as u8) as char));
            win.underline(false);

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
            win.underline(true);
            win.addstr(&format!("     {}\n", i + 1));
            win.underline(false);
            for (j, (card, facing)) in game.tableau(i).iter().enumerate() {
                win.addstr(&format!("{}{} ", i + 1, (b'A' + j as u8) as char));
                if matches!(facing, Facing::Down) {
                    win.addstr("---\n");
                } else {
                    Self::render_card(win, card);
                    win.color(Color::Gray);
                    win.addstr("\n");
                }
            }
            win.refresh();
        }
    }

    pub fn get_input(&self) -> Option<String> {
        let mut line = String::new();

        let prompt = "your move: ";
        self.text_window.mv(0, 0);
        self.text_window.clrtoeol();
        self.text_window.refresh();
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
        self.text_window.mv(0,0);
        self.text_window.refresh();
        Some(line)
    }

    pub fn write(&self, txt: &str) {
        self.text_window.mvaddstr(1, 0, txt);
    }

    pub fn halp(&self) {
        let win = newwin(13, 40, 2, 4);
        // Note the space before the line-continuation backslash; it makes room for the border.
        win.addstr("\n \
                    Move cards by typing the position of\n \
                    the card to be moved, followed by the\n \
                    destination. The columns of cards are\n \
                    numbered, and the rows are letters. To\n \
                    place at the bottom of a column, just\n \
                    specify the column number. Flip a\n \
                    face-down card over by just typing its\n \
                    position, without any destination. As\n \
                    a shortcut, moves to the foundation\n \
                    can omit the destination.\n \
                    Press any key to return to the game.");
        win.draw_box('|', '-');

        cbreak();
        win.getch();
        nocbreak();
        win.delwin();

        // Clear and redraw the screen because we drew in between windows.
        self.main_window.erase();
        self.main_window.refresh();
    }
}

impl Drop for CursesUI {
    fn drop(&mut self) {
        endwin();
    }
}
