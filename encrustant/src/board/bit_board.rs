use super::square::Square;
use core::fmt;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

/// 64 bit number where each bit represents a square.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BitBoard(u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                if self.get(&Square::from_coords(rank, file)) {
                    write!(f, "1")?;
                } else {
                    write!(f, "0")?;
                }
                if file != 7 {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Iterate a bit board by popping off squares.
#[macro_export]
macro_rules! consume_bit_board {
    ($bit_board:expr, $name:ident $code:block) => {
        while $bit_board.is_not_empty() {
            let $name = $bit_board.pop_square();
            $code
        }
    };
}

impl BitBoard {
    /// Bit board with all bits set except the A file.
    pub const NOT_A_FILE: Self = Self(!0x0101_0101_0101_0101);

    /// Bit board with all bits set except the H file.
    pub const NOT_H_FILE: Self = Self(!(0x0101_0101_0101_0101 << 7));

    /// Bit board with only the first rank set.
    pub const RANK_1: Self = Self(0xFF);

    // TODO: Rewrite as Self::RANK_X << 8;
    // currently cannot call non-const operator in constants.

    /// Bit board with only the second rank set.
    pub const RANK_2: Self = Self(0xFF << 8);

    /// Bit board with only the third rank set.
    pub const RANK_3: Self = Self(0xFF << 16);

    /// Bit board with only the fourth rank set.
    pub const RANK_4: Self = Self(0xFF << 24);

    /// Bit board with only the fifth rank set.
    pub const RANK_5: Self = Self(0xFF << 32);

    /// Bit board with only the sixth rank set.
    pub const RANK_6: Self = Self(0xFF << 40);

    /// Bit board with only the seventh rank set.
    pub const RANK_7: Self = Self(0xFF << 48);

    /// Bit board with only the eighth rank set.
    pub const RANK_8: Self = Self(0xFF << 56);

    /// Bit board with no bits set.
    pub const EMPTY: Self = Self(0);

    /// Bit board with all bits set.
    pub const FULL: Self = Self(!0);

    /// A bit board with only light squares set.
    pub const LIGHT_SQUARES: Self = Self(0x55AA_55AA_55AA_55AA);

    /// A bit board with only dark squares set.
    pub const DARK_SQUARES: Self = Self(0xAA55_AA55_AA55_AA55);

    /// Bit board from a 64 bit number.
    #[must_use]
    pub const fn new(bits: u64) -> Self {
        Self(bits)
    }

    /// Bit board with the square set.
    #[must_use]
    pub const fn from_square(square: &Square) -> Self {
        Self(1 << square.index())
    }

    /// Sets a square.
    pub fn set(&mut self, square: &Square) {
        *self |= square.bit_board();
    }

    /// Unsets a square.
    pub fn unset(&mut self, square: &Square) {
        *self &= !square.bit_board();
    }

    /// Toggles a square.
    pub fn toggle(&mut self, square: &Square) {
        *self ^= square.bit_board();
    }

    /// Toggles two squares.
    pub fn toggle_two(&mut self, a: &Square, b: &Square) {
        *self ^= a.bit_board() | b.bit_board();
    }

    /// Returns whether no bits are set.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::bit_board::BitBoard;
    ///
    /// assert!(BitBoard::EMPTY.is_empty());
    /// assert!(!BitBoard::FULL.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    /// Returns whether any bits are set.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::bit_board::BitBoard;
    ///
    /// assert!(BitBoard::FULL.is_not_empty());
    /// assert!(!BitBoard::EMPTY.is_not_empty());
    /// ```
    #[must_use]
    pub fn is_not_empty(&self) -> bool {
        *self != Self::EMPTY
    }

    /// Returns whether there is more than one bit set.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::bit_board::BitBoard;
    ///
    /// assert!(!BitBoard::new(0).more_than_one_bit_set());
    /// assert!(!BitBoard::new(1).more_than_one_bit_set());
    /// assert!(BitBoard::new(0b111).more_than_one_bit_set());
    /// ```
    #[must_use]
    pub fn more_than_one_bit_set(&self) -> bool {
        (*self & Self(self.0.wrapping_sub(1))).is_not_empty()
    }

    /// Returns whether there are any bits set in both bitboards.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::bit_board::BitBoard;
    ///
    /// assert!(BitBoard::RANK_1.overlaps(&BitBoard::NOT_A_FILE));
    /// ```
    #[must_use]
    pub fn overlaps(&self, bit_board: &Self) -> bool {
        (*self & *bit_board).is_not_empty()
    }

    /// Returns whether a square is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::{bit_board::BitBoard, square::Square};
    ///
    /// assert!(BitBoard::RANK_4.get(&Square::from_notation("a4").unwrap()));
    /// assert!(!BitBoard::RANK_4.get(&Square::from_notation("a8").unwrap()));
    /// ```
    #[must_use]
    pub fn get(&self, square: &Square) -> bool {
        self.overlaps(&square.bit_board())
    }

    /// Gets the last square.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::{bit_board::BitBoard, square::Square};
    ///
    /// assert_eq!(BitBoard::FULL.last_square(), Square::from_index(63));
    ///
    /// assert_eq!(BitBoard::EMPTY.last_square(), Square::from_index(-1));
    ///
    /// assert_eq!(BitBoard::new(1).last_square(), Square::from_index(0));
    /// ```
    #[must_use]
    pub const fn last_square(&self) -> Square {
        Square::from_index(63 - self.0.leading_zeros() as i8)
    }

    /// Gets the first square.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::{bit_board::BitBoard, square::Square};
    ///
    /// let mut bit_board = BitBoard::FULL;
    /// assert_eq!(bit_board.first_square(), Square::from_index(0));
    ///
    /// let mut bit_board = BitBoard::EMPTY;
    /// assert_eq!(bit_board.first_square(), Square::from_index(64));
    /// ```
    #[must_use]
    pub const fn first_square(&self) -> Square {
        Square::from_index(self.0.trailing_zeros() as i8)
    }

    /// Pops the first square.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::{bit_board::BitBoard, square::Square};
    ///
    /// let mut bit_board = BitBoard::FULL;
    /// assert_eq!(bit_board.pop_square(), Square::from_index(0));
    /// assert_eq!(bit_board.pop_square(), Square::from_index(1));
    /// assert_eq!(bit_board.pop_square(), Square::from_index(2));
    /// ```
    #[must_use]
    pub const fn pop_square(&mut self) -> Square {
        let index = self.first_square();
        self.0 &= self.0 - 1;
        index
    }

    /// Returns how many bits are set.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::bit_board::BitBoard;
    ///
    /// assert_eq!(BitBoard::EMPTY.count(), 0);
    /// assert_eq!(BitBoard::FULL.count(), 64);
    /// assert_eq!(BitBoard::RANK_5.count(), 8);
    /// ```
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.0.count_ones()
    }

    /// Used to traverse subsets of a set.
    /// This takes the current subset and finds the next highest subset after it.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::bit_board::BitBoard;
    ///
    /// let set = BitBoard::RANK_4;
    /// let mut subset = BitBoard::EMPTY;
    /// loop {
    ///     println!("{subset}");
    ///     subset = subset.carry_rippler(set);
    ///     if subset.is_empty() {
    ///         break;
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn carry_rippler(&self, d: Self) -> Self {
        Self(self.0.wrapping_sub(d.0) & d.0)
    }

    /// Multiplies by `magic`, then right shifts by `shift`.
    #[must_use]
    pub const fn magic_index(&self, magic: u64, shift: u8) -> usize {
        let hash = self.0.wrapping_mul(magic);
        (hash >> shift) as usize
    }
}

macro_rules! implement_op {
    ($op:ident, $name:ident, $operator:tt) => {
        impl $op<BitBoard> for BitBoard {
            type Output = BitBoard;

            fn $name(self, rhs: BitBoard) -> Self::Output {
               Self(self.0 $operator rhs.0)
            }
        }
    };
}
macro_rules! implement_assign_op {
    ($op:ident, $name:ident, $operator:tt) => {
        impl $op<BitBoard> for BitBoard {
            fn $name(&mut self, rhs: Self) {
                *self = Self(self.0 $operator rhs.0)
            }
        }
    };
}
implement_op!(BitOr, bitor, |);
implement_assign_op!(BitOrAssign, bitor_assign, |);

implement_op!(BitAnd, bitand, &);
implement_assign_op!(BitAndAssign, bitand_assign, &);

implement_op!(BitXor, bitxor, ^);
implement_assign_op!(BitXorAssign, bitxor_assign, ^);

macro_rules! shift {
    ($op:ident, $name:ident, $operator:tt) => {
        impl $op<u8> for BitBoard {
            type Output = BitBoard;

            fn $name(self, rhs: u8) -> Self::Output {
               Self(self.0 $operator rhs)
            }
        }
    };
}

shift!(Shl, shl, <<);
shift!(Shr, shr, >>);

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
