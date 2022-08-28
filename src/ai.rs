use std::collections::HashSet;

use crate::action::{Action, Source, Destination};
use crate::game_state::{Card, GameState, Facing, Rank, StateFingerprint};

pub struct Solver {
    fringe: Vec<Play>,
    dead: Vec<Play>,
    seen: HashSet<StateFingerprint>,
    try_harder: bool,
}

impl Solver {
    pub fn new(initial: GameState) -> Self {
        let mut seen = HashSet::new();
        seen.insert(initial.fingerprint());
        Self {
            fringe: vec![Play {
                moves: vec![],
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
        let mut best = 0;
        macro_rules! print_stats {
            () => {
                eprintln!("{}: best({}) fringe({}) dead({}) seen({}) stalled({})",
                    i, best, self.fringe.len(), self.dead.len(), self.seen.len(), stalled);
            }
        }
        while !self.fringe.is_empty() && stalled < stalled_rounds {
            for d in self.dead.iter().chain(self.fringe.iter()) {
                let s = d.state.score();
                if s > best {
                    best = s;
                    stalled = 0;
                    if self.try_harder {
                        self.try_harder = false;
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
                    .then_with(|| a.moves.len().cmp(&b.moves.len()))
            })
            .1
    }

    /// Sort the fringe by score, descending.
    pub fn sort(&mut self) {
        /*if self.try_harder*/ {
            // Number of moves descending, then score descending.
            self.fringe.sort_unstable_by(|a, b| {
                b.moves.len().cmp(&a.moves.len())
                    .then_with(|| b.state.score().cmp(&a.state.score()))
            })
        } /*else {
            //self.fringe.sort_unstable_by_key(|p| -p.state.score());
            // Score descending, then number of moves ascending.
            self.fringe.sort_unstable_by(|a, b| {
                b.state.score().cmp(&a.state.score())
                    .then_with(|| a.moves.len().cmp(&b.moves.len()))
            })
        }*/
    }

    fn iter(&mut self, expand: usize) {
        let mut new_fringe = vec![];
        self.sort();
        let num = if self.try_harder {
            //self.fringe.len() / 4
            // replace with usize::div_ceil when it's stable
            (self.fringe.len() as f64 / 4.).ceil() as usize
        } else {
            self.fringe.len().clamp(0, expand)
        };
        for play in self.fringe.drain(..num) {
            let mut any_novel = false;
            for new in play.next() {
                let is_novel = self.seen.insert(new.state.fingerprint());
                if is_novel {
                    //eprintln!("--------\n{:?}", new.state);
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
}

#[derive(Clone)]
pub struct Play {
    pub moves: Vec<Action>,
    pub state: GameState,
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
        self.moves.push(a);
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