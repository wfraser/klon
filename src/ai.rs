use std::collections::HashSet;

use crate::action::{Action, Source, Destination};
use crate::game_state::{Card, GameState, Facing, Rank, StateFingerprint};

pub struct Solver {
    fringe: Vec<Play>,
    dead: Vec<Play>,
    seen: HashSet<StateFingerprint>,
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
        }
    }

    pub fn solve(&mut self) {
        let mut i = 0;
        while !self.fringe.is_empty() {
            let mut best = 0;
            for d in self.dead.iter().chain(self.fringe.iter()) {
                let s = d.state.score();
                if s > best {
                    best = s;
                }
            }
            eprintln!("{}: best({}) fringe({}) dead({}) seen({})",
                i, best, self.fringe.len(), self.dead.len(), self.seen.len());
            self.iter();
            i += 1;
        }
    }

    pub fn best(&mut self) -> &Play {
        let (_, x, _) = self.dead.select_nth_unstable_by(0, |a, b| {
            b.state.score().cmp(&a.state.score())
        });
        x
    }

    fn iter(&mut self) {
        let mut new_fringe = vec![];
        for play in self.fringe.drain(..) {
            let mut any_novel = false;
            for new in play.next() {
                let is_novel = self.seen.insert(new.state.fingerprint());
                if is_novel {
                    //eprintln!("--------\n{:?}", new.state);

                    new_fringe.push(new);
                    any_novel = true;
                }
            }
            if !any_novel {
                self.dead.push(play);
            }
        }
        std::mem::swap(&mut self.fringe, &mut new_fringe);
    }
}

pub struct Play {
    pub moves: Vec<Action>,
    pub state: GameState,
}

impl Play {
    pub fn next(&self) -> Vec<Self> {
        let mut res = vec![];
        let mut moves = find_moves(&self.state);
        if !self.state.stock_is_empty() {
            moves.push(Action::Draw);
        }
        for a in moves {
            let mut state = self.state.clone();
            let mut moves = self.moves.clone();
            if let Err(e) = state.apply_action(&a) {
                for m in &moves {
                    println!("{}", m);
                }
                println!("-> {}", a);
                println!("{}", e);
                panic!("illegal move?!?!");
            }
            moves.push(a);
            res.push(Play { moves, state });
        }
        res
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