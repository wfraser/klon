use crate::init_array;

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)] // only constructed via the primitive, using transmute
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
    stock: Stock,
    foundation: [Vec<Card>; 4],
    tableau: [Vec<(Card, Facing)>; 7],
}

impl GameState {
    pub fn new(mut cards: Vec<Card>) -> Self {
        let mut tableau = init_array!(Vec<(Card, Facing)>, 7, |_| vec![]);

        for (i, column) in tableau.iter_mut().enumerate() {
            for j in 0 ..= i {
                let facing = if i == j { Facing::Up } else { Facing::Down };
                column.push((cards.pop().unwrap(), facing));
            }
        }

        Self {
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
        //let rank = unsafe { std::mem::transmute::<_, Rank>((10 - idx) as u8) };
        //Some(Card { suit: Suit::Hearts, rank })
    }

    pub fn take_waste_temp_hax(&mut self) -> Option<Card> {
        self.stock.take()
    }
}
