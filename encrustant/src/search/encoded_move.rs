//! Compressed representation of moves.

use crate::{
    board::square::Square,
    move_generator::move_data::{Flag, Move},
};
use core::fmt;

/// A piece move, bit-packed.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct EncodedMove(u16);

impl EncodedMove {
    /// Packs a move into 16 bits.
    #[must_use]
    pub const fn new(move_data: Move) -> Self {
        let mut data: u16 = 0;

        // Squares are 6 bits each
        data |= move_data.from.index() as u16;
        data |= (move_data.to.index() as u16) * 64;

        data |= (move_data.flag as u16) << 12;

        Self(data)
    }

    /// Decodes from, to, and flag.
    #[must_use]
    pub const fn decode(self) -> Move {
        Move {
            from: self.from(),
            to: self.to(),
            flag: *self.flag(),
        }
    }

    /// No move.
    pub const NONE: Self = Self(0);

    /// Returns `true` if the move is none
    #[must_use]
    pub fn is_none(self) -> bool {
        self == Self::NONE
    }

    #[must_use]
    #[allow(clippy::unreadable_literal)]
    pub const fn from(self) -> Square {
        Square::from_index((self.0 & 0b111111) as i8)
    }

    #[must_use]
    #[allow(clippy::unreadable_literal)]
    pub const fn to(self) -> Square {
        Square::from_index(((self.0 >> 6) & 0b111111) as i8)
    }

    #[must_use]
    pub const fn flag(&self) -> &Flag {
        &Flag::ALL[((self.0 >> 12) & 0b1111) as usize]
    }

    /// Returns the bit-packed move data without the flag.
    #[must_use]
    pub const fn without_flag(self) -> u16 {
        self.0 & 0b111111_111111
    }
}

impl fmt::Display for EncodedMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.decode().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::EncodedMove;
    use crate::{
        board::square::Square,
        move_generator::move_data::{Flag, Move},
    };

    // TODO: add more test cases
    const TEST_MOVES: [(Square, Square, Flag); 2] = [
        (
            Square::from_coords(2, 2),
            Square::from_coords(3, 2),
            Flag::None,
        ),
        (
            Square::from_coords(5, 5),
            Square::from_coords(7, 7),
            Flag::None,
        ),
    ];
    #[test]
    fn move_encoded_correctly() {
        for test_move in TEST_MOVES {
            let (from, to, flag) = test_move;
            let encoded = EncodedMove::new(Move { from, to, flag });
            assert_eq!(encoded.from(), from);
            assert_eq!(encoded.to(), to);
            assert_eq!(*encoded.flag(), flag);
        }
    }
}
