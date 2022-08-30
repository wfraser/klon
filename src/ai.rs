use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use crate::action::{Action, Source, Destination};
use crate::game_state::{Card, GameState, Facing, Rank, StateFingerprint};

pub struct Solver {
    fringe: Vec<Play>,
    dead: Vec<Play>,
    seen: HashMap<StateFingerprint, Rc<RefCell<Vec<Action>>>>,
    try_harder: bool,
}

impl Solver {
    pub fn new(initial: GameState) -> Self {
        let mut seen = HashMap::new();
        let moves = Rc::new(RefCell::new(vec![]));
        seen.insert(initial.fingerprint(), Rc::clone(&moves));
        Self {
            fringe: vec![Play {
                moves,
                state: initial,
            }],
            dead: vec![],
            seen,
            try_harder: false,
        }
    }

    pub fn solve(&mut self, expand: usize, stalled_rounds: usize) {
        let mut i = 0;
        let mut stalled = 0;
        let mut best_score = 0;
        let mut best_score_len = usize::MAX;
        macro_rules! print_stats {
            () => {
                eprintln!("{}: best({}/{}) fringe({}) dead({}) seen({}) stalled({})",
                    i, best_score, best_score_len, self.fringe.len(),
                    self.dead.len(), self.seen.len(), stalled);
            }
        }
        while !self.fringe.is_empty() && stalled < stalled_rounds {
            for d in self.dead.iter().chain(self.fringe.iter()) {
                let s = d.state.score();
                if s > best_score {
                    best_score = s;
                    best_score_len = d.moves.borrow().len();
                    stalled = 0;
                    if self.try_harder {
                        self.try_harder = false;
                    }
                } else if s == best_score {
                    let len = d.moves.borrow().len();
                    if len < best_score_len {
                        best_score_len = len;
                    }
                }
            }
            print_stats!();
            self.iter(expand);
            i += 1;
            stalled += 1;

            if stalled == stalled_rounds {
                if !self.try_harder {
                    self.try_harder = true;
                    stalled = 0;
                }
            }
        }
        print_stats!();
    }

    /// Return the best "dead" (either win or failure) gameplay by score and move count.
    pub fn best(&mut self) -> &Play {
        self.dead
            .select_nth_unstable_by(0, |a, b| {
                b.state.score().cmp(&a.state.score())
                    .then_with(|| a.moves.borrow().len().cmp(&b.moves.borrow().len()))
            })
            .1
    }

    /// Sort the fringe, placing next states to explore at the end.
    pub fn sort(&mut self) {
        /*self.fringe.sort_unstable_by(|a, b| {
            b.moves.borrow().len().cmp(&a.moves.borrow().len())
                .then_with(|| a.state.score().cmp(&b.state.score()))
        })*/

        /*if self.try_harder*/ {
            // Number of moves ascending, then score ascending.
            self.fringe.sort_unstable_by(|a, b| {
                a.moves.borrow().len().cmp(&b.moves.borrow().len())
                    .then_with(|| a.state.score().cmp(&b.state.score()))
            })
        } /*else {
            //self.fringe.sort_unstable_by_key(|p| p.state.score());
            // Score ascending, then number of moves descending.
            self.fringe.sort_unstable_by(|a, b| {
                a.state.score().cmp(&b.state.score())
                    .then_with(|| b.moves.len().cmp(&a.moves.len()))
            })
        }*/
    }

    fn iter(&mut self, expand: usize) {
        let mut new_fringe = vec![];
        self.sort();
        let split_idx = if self.try_harder {
            // replace with usize::div_ceil when it's stable
            self.fringe.len() - (self.fringe.len() as f64 / 4.).ceil() as usize
        } else {
            self.fringe.len().saturating_sub(expand)
        };
        let to_explore = self.fringe.split_off(split_idx);
        for play in to_explore {
            let mut any_novel = false;
            for mut new in play.next() {
                if self.is_novel(&mut new) {
                    if new.state.is_win() {
                        // Win states can be moved to dead immediately.
                        self.dead.push(new);
                        self.try_harder = true;
                    } else {
                        new_fringe.push(new);
                        any_novel = true;
                    }
                }
            }
            if !any_novel {
                // This state had no moves that yielded novel states: it is a dead end.
                self.dead.push(play);
            }
        }
        self.fringe.extend(new_fringe);
    }

    fn is_novel(&mut self, play: &mut Play) -> bool {
        match self.seen.entry(play.state.fingerprint()) {
            Entry::Occupied(mut e) => {
                let cur_len = play.moves.borrow().len();
                let prev_len = e.get().borrow().len();
                match prev_len.cmp(&cur_len) {
                    Ordering::Less => {
                        // Update the play's moveset with the shorter stored one.
                        play.moves = Rc::clone(e.get());
                    },
                    Ordering::Greater => {
                        // Update the stored moveset with this play's shorter one, and make this
                        // play refer to the stored one.
                        let moves_rc = std::mem::replace(&mut play.moves, Rc::clone(e.get()));
                        let moves = Rc::try_unwrap(moves_rc)
                            .unwrap_or_else(|rc| (*rc).clone())
                            .into_inner();
                        e.get_mut().replace(moves);
                    },
                    Ordering::Equal => {
                        // Make the play refer to the stored moveset.
                        play.moves = Rc::clone(e.get());
                    }
                }
                false
            }
            Entry::Vacant(e) => {
                e.insert(Rc::clone(&play.moves));
                true
            }
        }
    }
}

pub struct Play {
    pub moves: Rc<RefCell<Vec<Action>>>,
    pub state: GameState,
}

impl Clone for Play {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            moves: Rc::new(RefCell::new(self.moves.borrow().clone())),
        }
    }
}

impl Play {
    pub fn next(&self) -> Vec<Self> {
        let mut res = vec![];
        let mut base = self.clone();
        let mut moves = find_moves(&self.state);
        if let Some(Action::QuickMove(_)) = moves.last() {
            // There's no reason to defer a flip of a tableau card, and doing so doesn't invalidate
            // any subsequent moves, so apply it to the base state immediately.
            base.apply(moves.pop().unwrap());
        }
        for m in &moves {
            if let Action::Move(Source::Tableau { .. }, Destination::Foundation(f)) = m {
                if base.state.foundation(*f).is_none() {
                    // Moving an ace from the tableau to the foundation is always the best move.
                    // There's no reason to ever defer it.
                    base.apply(m.clone());
                    return vec![base];
                }
            }
        }
        if !self.state.stock_is_empty() {
            moves.push(Action::Draw);
        }
        for a in moves {
            let mut g = base.clone();
            g.apply(a);
            res.push(g);
        }
        res
    }

    fn apply(&mut self, a: Action) {
        self.state.apply_action(&a).expect("illegal move");
        self.moves.borrow_mut().push(a);
    }
}

pub fn find_moves(gs: &GameState) -> Vec<Action> {
    let mut actions = vec![];
    for (src, scard) in all_sources(gs) {
        for (dst, _dcard) in all_dests(gs) {
            if let (Source::Tableau { column, row }, Destination::Foundation(_)) = (src, dst) {
                if !gs.is_bottom_of_tableau(column, row) {
                    continue;
                }
            }
            if gs.can_stack(scard, dst) {
                actions.push(Action::Move(src, dst));
            }
        }
        if scard.rank == Rank::King {
            for i in 0 .. 7 {
                if gs.tableau(i).is_empty() {
                    actions.push(Action::Move(src, Destination::Tableau(i)));
                }
            }
        } else if scard.rank == Rank::Ace {
            // to cut down on state duplication, make each suit always go to one foundation column
            actions.push(Action::Move(src, Destination::Foundation(scard.suit as _)));
        }
    }
    for column in 0 .. 7 {
        let cards = gs.tableau(column);
        if let Some((_, Facing::Down)) = cards.last() {
            actions.push(Action::QuickMove(Source::Tableau { column, row: cards.len() - 1 }));
        }
    }
    actions
}

/// All cards that can be moved.
fn all_sources(gs: &GameState) -> Vec<(Source, &Card)> {
    let mut sources = vec![];
    for column in 0 .. 7 {
        for (row, (card, facing)) in gs.tableau(column).iter().enumerate() {
            if *facing == Facing::Up {
                sources.push((Source::Tableau { column, row }, card));
            }
        }
    }
    if let Some(card) = gs.waste().last() {
        sources.push((Source::Waste, card));
    }
    sources
}

/// All cards that can be placed upon. Does not include empty tableau or foundation spots.
fn all_dests(gs: &GameState) -> Vec<(Destination, &Card)> {
    let mut dests = vec![];
    for column in 0 .. 7 {
        if let Some((card, facing)) = gs.tableau(column).last() {
            if *facing == Facing::Up {
                dests.push((Destination::Tableau(column), card));
            }
        }
    }
    for i in 0 .. 4 {
        if let Some(card) = gs.foundation(i) {
            dests.push((Destination::Foundation(i), card));
        }
    }
    dests
}
