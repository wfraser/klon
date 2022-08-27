use std::fmt::Debug;

use crate::action::{Action, Destination, Source};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Suit::*;
        f.write_str(match self {
            Spades   => "♠",
            Clubs    => "♣",
            Hearts   => "♥",
            Diamonds => "♦",
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

impl Suit {
    pub fn color(self) -> Color {
        use Suit::*;
        match self {
            Spades | Clubs => Color::Black,
            Hearts | Diamonds => Color::Red,
        }
    }

    pub fn all() -> &'static [Suit] {
        use Suit::*;
        &[Spades, Clubs, Hearts, Diamonds]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    Ace = 1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N10,
    Jack,
    Queen,
    King,
}

impl Rank {
    pub fn value(self) -> u8 {
        self as u8
    }

    pub fn all() -> &'static [Rank] {
        use Rank::*;
        &[Ace, N2, N3, N4, N5, N6, N7, N8, N9, N10, Jack, Queen, King]
    }
}

impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Rank::*;
        match self {
            Ace => f.write_str("A"),
            Jack => f.write_str("J"),
            Queen => f.write_str("Q"),
            King => f.write_str("K"),
            _ => write!(f, "{}", *self as u8),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}{}>", self.rank, self.suit)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Facing {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stock {
    stock: Vec<Card>,
    waste: Vec<Card>,
}

impl Stock {
    pub fn new(cards: Vec<Card>) -> Self {
        Self {
            stock: cards,
            waste: vec![],
        }
    }

    pub fn draw_three(&mut self) -> bool {
        if self.stock.is_empty() {
            self.stock.extend(self.waste.drain(..).rev());
            true
        } else {
            let end = self.stock.len().saturating_sub(3);
            self.waste.extend(self.stock.drain(end..).rev());
            false
        }
    }

    pub fn stock_size(&self) -> usize {
        self.stock.len()
    }

    pub fn showing(&self) -> &[Card] {
        let end = self.waste.len().saturating_sub(3);
        &self.waste[end..]
    }

    pub fn take(&mut self) -> Option<Card> {
        self.waste.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.stock.is_empty() && self.waste.is_empty()
    }
}

#[cfg(test)]
mod test_stock {
    use super::*;

    fn waste(stock: &Stock) -> Vec<u8> {
        stock.showing()
            .iter()
            .map(|card| card.rank as u8)
            .collect()
    }

    #[test]
    fn test_stock() {
        use Rank::*;
        use Suit::*;
        let mut stock = Stock::new(vec![
            Card { rank: Ace, suit: Clubs },
            Card { rank: N2,  suit: Clubs },
            Card { rank: N3,  suit: Clubs },
            Card { rank: N4,  suit: Clubs },
            Card { rank: N5,  suit: Clubs },
        ]);
        assert!(stock.showing().is_empty());
        assert_eq!(5, stock.stock_size());

        assert_eq!(false, stock.draw_three());
        assert_eq!(&[5, 4, 3][..], waste(&stock));

        assert_eq!(false, stock.draw_three());
        assert_eq!(&[3, 2, 1][..], waste(&stock));

        assert_eq!(true, stock.draw_three());
        assert!(stock.showing().is_empty());

        assert_eq!(false, stock.draw_three());
        assert_eq!(&[5, 4, 3][..], waste(&stock));

        assert_eq!(Some(3), stock.take().map(|card| card.rank as u8));
        assert_eq!(&[5, 4][..], waste(&stock));

        assert_eq!(false, stock.draw_three());
        assert_eq!(&[4, 2, 1][..], waste(&stock));
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GameState {
    game_number: u64,
    stock: Stock,
    foundation: [Vec<Card>; 4],
    tableau: [Vec<(Card, Facing)>; 7],
    score: i32,
}

impl Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("stock: ")?;
        for c in &self.stock.waste {
            write!(f, "{:?} ", c)?;
        }
        f.write_str("/ ")?;
        for c in &self.stock.stock {
            write!(f, "{:?} ", c)?;
        }
        f.write_str("\nfoundation: ")?;
        for stack in &self.foundation {
            if let Some(last) = stack.last() {
                write!(f, "{:?} ", last)?;
            } else {
                f.write_str("<xx> ")?;
            }
        }
        for i in 0 .. 7 {
            write!(f, "\ntableau {}: ", i)?;
            for (c, facing) in &self.tableau[i] {
                if *facing == Facing::Up {
                    write!(f, "{:?} ", c)?;
                } else {
                    f.write_str("<xx> ")?;
                }
            }
        }
        f.write_str("\n")?;
        Ok(())
    }
}

impl GameState {
    pub fn new(game_number: u64, mut cards: Vec<Card>) -> Self {
        let mut tableau = <[Vec<(Card, Facing)>; 7]>::default();

        for (i, column) in tableau.iter_mut().enumerate() {
            for j in 0 ..= i {
                let facing = if i == j { Facing::Up } else { Facing::Down };
                column.push((cards.pop().unwrap(), facing));
            }
        }

        Self {
            game_number,
            stock: Stock::new(cards),
            foundation: Default::default(),
            tableau,
            score: 0,
        }
    }

    pub fn draw_three(&mut self) -> bool {
        self.stock.draw_three()
    }

    pub fn stock_size(&self) -> usize {
        self.stock.stock_size()
    }

    pub fn stock_is_empty(&self) -> bool {
        self.stock.is_empty()
    }

    pub fn waste(&self) -> &[Card] {
        self.stock.showing()
    }

    pub fn tableau(&self, idx: usize) -> &[(Card, Facing)] {
        &self.tableau[idx]
    }

    pub fn foundation(&self, idx: usize) -> Option<&Card> {
        self.foundation[idx].last()
    }

    pub fn can_stack(&self, card: &Card, dest: Destination) -> bool {
        match dest {
            Destination::Foundation(column) => self.can_stack_foundation(card, column).is_ok(),
            Destination::Tableau(column) => self.can_stack_tableau(card, column).is_ok()
        }
    }

    fn can_stack_tableau(&self, card: &Card, column: usize) -> Result<(), &'static str> {
        match self.tableau.get(column).ok_or("no such column")?.last() {
            None => {
                if card.rank == Rank::King {
                    Ok(())
                } else {
                    Err("only King can go on empty tableau space")
                }
            }
            Some((_, Facing::Down)) => Err("cannot place on face-down card"),
            Some((parent, Facing::Up)) => {
                if parent.suit.color() == card.suit.color() {
                    Err("cards must differ in color")
                } else if parent.rank.value() != card.rank.value() + 1 {
                    Err("card value is not one higher than that being placed")
                } else {
                    Ok(())
                }
            }
        }
    }

    fn can_stack_foundation(&self, card: &Card, column: usize) -> Result<(), &'static str> {
        match self.foundation.get(column).ok_or("no such column")?.last() {
            None => {
                if card.rank == Rank::Ace {
                    Ok(())
                } else {
                    Err("only Ace can go on empty foundation space")
                }
            }
            Some(parent) => {
                if parent.suit != card.suit {
                    Err("cards must match in suit")
                } else if parent.rank.value() + 1 != card.rank.value() {
                    Err("card value is not one lower than that being placed")
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn apply_action(&mut self, action: &Action) -> Result<(), &'static str> {
        match action {
            Action::Quit | Action::Help => (),
            Action::Draw => {
                self.draw_three();
            }
            Action::Move(src, dest) => {
                let card_ref = self.get_src_card_ref(src)?;
                match *dest {
                    Destination::Tableau(column) => {
                        self.can_stack_tableau(card_ref, column)?;
                    }
                    Destination::Foundation(column) => {
                        if let Source::Tableau { column: src_col, row: src_row } = *src {
                            if !self.is_bottom_of_tableau(src_col, src_row) {
                                return Err("can only pop off the bottom card of a stack");
                            }
                        }
                        self.can_stack_foundation(card_ref, column)?;
                    }
                }

                match (src, dest) {
                    (Source::Waste, &Destination::Foundation(column)) => {
                        self.score += 10;
                        self.foundation[column].push(self.stock.take().unwrap());
                    }
                    (Source::Waste, &Destination::Tableau(column)) => {
                        self.score += 5;
                        self.tableau[column].push((self.stock.take().unwrap(), Facing::Up));
                    }
                    (&Source::Tableau { column, row }, &Destination::Foundation(idx)) => {
                        self.score += 10;
                        self.foundation[idx].push(self.tableau[column].remove(row).0);
                    }
                    (&Source::Tableau { column: src_col, row: src_row },
                        &Destination::Tableau(dst_col)) =>
                    {
                        while self.tableau[src_col].get(src_row).is_some() {
                            let (card, facing) = self.tableau[src_col].remove(src_row);
                            self.tableau[dst_col].push((card, facing));
                        }
                    }
                };
            }
            Action::QuickMove(src) => {
                // Unfortunately a big duplication of get_src_card_ref...
                if let Source::Tableau { column, row } = *src {
                    if let Some((_, Facing::Down)) = self.tableau.get(column)
                        .and_then(|cards| cards.get(row))
                    {
                        if self.is_bottom_of_tableau(column, row) {
                            // flip card
                            self.score += 5;
                            self.tableau[column][row].1 = Facing::Up;
                            return Ok(());
                        }
                    }
                }

                let card_ref = self.get_src_card_ref(src)?;

                if let Source::Tableau { column, row } = *src {
                    if !self.is_bottom_of_tableau(column, row) {
                        return Err("can only pop off the bottom card of a stack");
                    }
                }

                let mut foundation_idx = None;
                for i in 0 .. 4 {
                    if self.can_stack_foundation(card_ref, i).is_ok() {
                        foundation_idx = Some(i);
                        break;
                    }
                }

                match foundation_idx {
                    Some(i) => {
                        self.score += 10;
                        self.foundation[i].push(match *src {
                            Source::Waste => self.stock.take().unwrap(),
                            Source::Tableau { column, row } => {
                                self.tableau[column].remove(row).0
                            }
                        });
                    }
                    None => return Err("can't put that on any of the foundation stacks"),
                }
            }
        }
        Ok(())
    }

    fn get_src_card_ref(&self, location: &Source) -> Result<&Card, &'static str> {
        match location {
            Source::Waste => match self.stock.showing().last() {
                Some(card) => Ok(card),
                None => Err("waste is empty"),
            },
            Source::Tableau { column, row } => match self.tableau
                .get(*column)
                .and_then(|cards| cards.get(*row))
            {
                Some((card, Facing::Up)) => {
                    Ok(card)
                }
                Some((_, Facing::Down)) => Err("cannot move face-down card"),
                None => Err("no card there"),
            }
        }
    }

    pub fn is_bottom_of_tableau(&self, column: usize, row: usize) -> bool {
        self.tableau.get(column)
            .and_then(|cards| cards.get(row + 1))
            .is_none()
    }

    pub fn game_number(&self) -> u64 {
        self.game_number
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn is_win(&self) -> bool {
        self.foundation.iter()
            .map(|f| f.last().map(|c| c.rank))
            .all(|r| r == Some(Rank::King))
    }

    pub fn fingerprint(&self) -> StateFingerprint {
        let mut f = StateFingerprint {
            stock: self.stock.stock.clone(),
            waste: self.stock.waste.clone(),
            foundation: self.foundation.clone().map(|mut s| s.pop()),
            tableau: self.tableau.clone(),
            score: self.score,
        };
        f.tableau.sort_unstable_by(|a, b| {
            a.len().cmp(&b.len())
                .then_with(|| {
                    if let (Some((ac, af)), Some((bc, bf))) = (a.first(), b.first()) {
                        af.cmp(bf).then_with(|| ac.cmp(bc))
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
            });
        f
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct StateFingerprint {
    stock: Vec<Card>,
    waste: Vec<Card>,
    foundation: [Option<Card>; 4],
    tableau: [Vec<(Card, Facing)>; 7],
    score: i32,
}
