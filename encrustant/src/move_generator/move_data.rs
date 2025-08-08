use crate::board::{piece::Piece, square::Square};
use core::fmt;

/// Extra move data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flag {
    /// A standard move.
    None,

    /// A pawn promoting to a queen.
    QueenPromotion,

    /// A pawn promoting to a rook.
    RookPromotion,

    /// A pawn promoting to a bishop.
    BishopPromotion,

    /// A pawn promoting to a knight.
    KnightPromotion,

    /// En passant.
    EnPassant,

    /// A pawn moving two squares up.
    PawnTwoUp,

    /// The king castling.
    Castle,
}

impl Flag {
    /// Every flag.
    pub const ALL: [Self; 8] = [
        Self::None,
        Self::QueenPromotion,
        Self::RookPromotion,
        Self::BishopPromotion,
        Self::KnightPromotion,
        Self::EnPassant,
        Self::PawnTwoUp,
        Self::Castle,
    ];

    /// Every promotion flag.
    pub const PROMOTIONS: [Self; 4] = [
        Self::QueenPromotion,
        Self::RookPromotion,
        Self::BishopPromotion,
        Self::KnightPromotion,
    ];

    /// Returns the piece being promoted into if it is a promotion flag, otherwise returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::{move_generator::move_data::Flag, board::piece::Piece};
    ///
    /// assert_eq!(Flag::QueenPromotion.get_promotion_piece(true), Some(Piece::WhiteQueen));
    /// assert_eq!(Flag::BishopPromotion.get_promotion_piece(false), Some(Piece::BlackBishop));
    /// assert_eq!(Flag::Castle.get_promotion_piece(true), None);
    /// ```
    #[must_use]
    pub const fn get_promotion_piece(&self, white: bool) -> Option<Piece> {
        if white {
            match self {
                Self::QueenPromotion => Some(Piece::WhiteQueen),
                Self::RookPromotion => Some(Piece::WhiteRook),
                Self::BishopPromotion => Some(Piece::WhiteBishop),
                Self::KnightPromotion => Some(Piece::WhiteKnight),
                _ => None,
            }
        } else {
            match self {
                Self::QueenPromotion => Some(Piece::BlackQueen),
                Self::RookPromotion => Some(Piece::BlackRook),
                Self::BishopPromotion => Some(Piece::BlackBishop),
                Self::KnightPromotion => Some(Piece::BlackKnight),
                _ => None,
            }
        }
    }
}

/// A piece move.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Move {
    /// Square the piece is moving from.
    pub from: Square,

    /// Square the piece is moving to.
    pub to: Square,

    /// Type of move.
    pub flag: Flag,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.flag == Flag::None {
            write!(f, "Move from {} to {}", self.from, self.to)
        } else {
            write!(
                f,
                "Move from {} to {}, flag = {:?}",
                self.from, self.to, self.flag
            )
        }
    }
}
