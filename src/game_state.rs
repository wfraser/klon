use crate::init_array;
use crate::action::{Action, Destination, Source};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Copy, Clone)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}{}>", self.rank, self.suit)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Facing {
    Up,
    Down,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct GameState {
    game_number: u64,
    stock: Stock,
    foundation: [Vec<Card>; 4],
    tableau: [Vec<(Card, Facing)>; 7],
}

impl GameState {
    pub fn new(game_number: u64, mut cards: Vec<Card>) -> Self {
        let mut tableau = init_array!(Vec<(Card, Facing)>, 7, |_| vec![]);

        for (i, column) in tableau.iter_mut().enumerate() {
            for j in 0 ..= i {
                let facing = if i == j { Facing::Up } else { Facing::Down };
                column.push((cards.pop().unwrap(), facing));
            }
        }

        Self {
            game_number,
            stock: Stock::new(cards),
            foundation: init_array!(Vec<Card>, 4, |_| vec![]),
            tableau,
        }
    }

    pub fn draw_three(&mut self) -> bool {
        self.stock.draw_three()
    }

    pub fn stock_size(&self) -> usize {
        self.stock.stock_size()
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

    pub fn apply_action(&mut self, action: Action) -> Result<(), &'static str> {
        match action {
            Action::Quit | Action::Help => (),
            Action::Draw => {
                self.draw_three();
            }
            Action::Move(src, dest) => {
                let (card_ref, tableau_position) = self.get_src_card_ref(&src)?;
                match dest {
                    Destination::Tableau(column) => {
                        self.can_stack_tableau(card_ref, column)?;
                    }
                    Destination::Foundation(column) => {
                        if let Some((src_col, src_row)) = tableau_position {
                            if !self.is_bottom_of_tableau(src_col, src_row) {
                                return Err("can only pop off the bottom card of a stack");
                            }
                        }
                        self.can_stack_foundation(card_ref, column)?;
                    }
                }

                match (src, dest) {
                    (Source::Waste, Destination::Foundation(column)) => {
                        self.foundation[column].push(self.stock.take().unwrap());
                    }
                    (Source::Waste, Destination::Tableau(column)) => {
                        self.tableau[column].push((self.stock.take().unwrap(), Facing::Up));
                    }
                    (Source::Tableau { column, row }, Destination::Foundation(idx)) => {
                        self.foundation[idx].push(self.tableau[column].remove(row).0);
                    }
                    (Source::Tableau { column: src_col, row: src_row },
                        Destination::Tableau(dst_col)) =>
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
                if let Source::Tableau { column, row } = src {
                    if let Some((_, Facing::Down)) = self.tableau.get(column)
                        .and_then(|cards| cards.get(row))
                    {
                        if self.is_bottom_of_tableau(column, row) {
                            // flip card
                            self.tableau[column][row].1 = Facing::Up;
                            return Ok(());
                        }
                    }
                }

                let (card_ref, tableau_position) = self.get_src_card_ref(&src)?;

                if let Some((col, row)) = tableau_position {
                    if !self.is_bottom_of_tableau(col, row) {
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
                        self.foundation[i].push(match src {
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

    fn get_src_card_ref(&self, location: &Source) -> Result<(&Card, Option<(usize, usize)>), &'static str> {
        match location {
            Source::Waste => match self.stock.showing().last() {
                Some(card) => Ok((card, None)),
                None => Err("waste is empty"),
            },
            Source::Tableau { column, row } => match self.tableau
                .get(*column)
                .and_then(|cards| cards.get(*row))
            {
                Some((card, Facing::Up)) => {
                    Ok((card, Some((*column, *row))))
                }
                Some((_, Facing::Down)) => Err("cannot move face-down card"),
                None => Err("no card there"),
            }
        }
    }

    fn is_bottom_of_tableau(&self, column: usize, row: usize) -> bool {
        self.tableau.get(column)
            .and_then(|cards| cards.get(row + 1))
            .is_none()
    }

    pub fn game_number(&self) -> u64 {
        self.game_number
    }
}
