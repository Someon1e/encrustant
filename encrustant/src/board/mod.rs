//! Represents a chess position.

use core::fmt::Display;
use std::num::NonZeroU32;

/// Implements bit boards.
pub mod bit_board;

/// Implements FEN notation.
pub mod fen;

/// Game state.
pub mod game_state;

/// Handles pieces.
pub mod piece;

/// Abstraction for squares.
pub mod square;

use bit_board::BitBoard;
use piece::Piece;
use square::Square;

use self::game_state::GameState;

/// Represents a chess position.
pub struct Board {
    /// Whether it is white's turn to move.
    pub white_to_move: bool,

    /// Bit boards, one for every piece type.
    pub bit_boards: [BitBoard; 12],

    /// The number of full moves.
    pub full_move_counter: NonZeroU32,

    /// State of the game.
    pub game_state: GameState,
}

impl Display for Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl Board {
    /// Returns a piece at a square.
    #[must_use]
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let square_bit_board = square.bit_board();
        for piece in Piece::ALL_PIECES {
            let piece_bit_board = self.get_bit_board(piece);
            if piece_bit_board.overlaps(&square_bit_board) {
                return Some(piece);
            }
        }
        None
    }

    /// Returns a white piece at a square.
    #[must_use]
    pub fn white_piece_at(&self, square: Square) -> Option<Piece> {
        let square_bit_board = square.bit_board();
        for piece in Piece::WHITE_PIECES {
            let piece_bit_board = self.get_bit_board(piece);
            if piece_bit_board.overlaps(&square_bit_board) {
                return Some(piece);
            }
        }
        None
    }

    /// Returns a black piece at a square.
    #[must_use]
    pub fn black_piece_at(&self, square: Square) -> Option<Piece> {
        let square_bit_board = square.bit_board();
        for piece in Piece::BLACK_PIECES {
            let piece_bit_board = self.get_bit_board(piece);
            if piece_bit_board.overlaps(&square_bit_board) {
                return Some(piece);
            }
        }
        None
    }

    /// Returns a piece on the side-to-move at a square.
    #[must_use]
    pub fn friendly_piece_at(&self, square: Square) -> Option<Piece> {
        if self.white_to_move {
            self.white_piece_at(square)
        } else {
            self.black_piece_at(square)
        }
    }

    /// Returns an opponent's piece at a square.
    #[must_use]
    pub fn enemy_piece_at(&self, square: Square) -> Option<Piece> {
        if self.white_to_move {
            self.black_piece_at(square)
        } else {
            self.white_piece_at(square)
        }
    }

    /// Returns a reference to a piece type's bit board.
    #[must_use]
    pub const fn get_bit_board(&self, piece: Piece) -> &BitBoard {
        &self.bit_boards[piece as usize]
    }

    /// Returns a mutable reference to a piece type's bit board.
    #[must_use]
    pub const fn get_bit_board_mut(&mut self, piece: Piece) -> &mut BitBoard {
        &mut self.bit_boards[piece as usize]
    }

    /// Returns true if any of the below are true:
    /// - Both sides have a bare King
    /// - King and a Minor Piece versus a bare King
    /// - Both sides have a King and a Bishop, the Bishops being the same Color
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::Board;
    ///
    /// // Both sides have a bare King
    /// assert!(Board::from_fen("8/8/8/6k1/1K6/8/8/8 w - - 0 1").unwrap().is_insufficient_material());
    /// assert!(Board::from_fen("8/3K4/8/8/8/8/3k4/8 w - - 0 1").unwrap().is_insufficient_material());
    ///
    /// // One side has a King and a Minor Piece against a bare King
    /// assert!(Board::from_fen("8/4bk2/8/8/8/8/3K4/8 w - - 0 1").unwrap().is_insufficient_material());
    /// assert!(Board::from_fen("8/5k2/8/8/8/3B4/3K4/8 w - - 0 1").unwrap().is_insufficient_material());
    ///
    /// // Both sides have a King and a Bishop, the Bishops being the same Color
    /// assert!(Board::from_fen("8/3KB3/8/8/8/4b3/3k4/8 w - - 0 1").unwrap().is_insufficient_material());
    ///
    /// // Normal position
    /// assert!(!Board::from_fen(Board::START_POSITION_FEN).unwrap().is_insufficient_material());
    /// ```
    #[must_use]
    pub fn is_insufficient_material(&self) -> bool {
        let white_nonking = *self.get_bit_board(Piece::WhitePawn)
            | *self.get_bit_board(Piece::WhiteKnight)
            | *self.get_bit_board(Piece::WhiteBishop)
            | *self.get_bit_board(Piece::WhiteRook)
            | *self.get_bit_board(Piece::WhiteQueen);
        let black_nonking = *self.get_bit_board(Piece::BlackPawn)
            | *self.get_bit_board(Piece::BlackKnight)
            | *self.get_bit_board(Piece::BlackBishop)
            | *self.get_bit_board(Piece::BlackRook)
            | *self.get_bit_board(Piece::BlackQueen);

        // Both sides have a bare King
        if white_nonking.is_empty() && black_nonking.is_empty() {
            return true;
        }

        let kings = *self.get_bit_board(Piece::WhiteKing) | *self.get_bit_board(Piece::BlackKing);

        let bishops =
            *self.get_bit_board(Piece::WhiteBishop) | *self.get_bit_board(Piece::BlackBishop);
        let all_pieces = white_nonking | black_nonking | kings;

        // King and a Minor Piece versus a bare King
        let minor_pieces = *self.get_bit_board(Piece::BlackKnight)
            | *self.get_bit_board(Piece::WhiteKnight)
            | bishops;
        if all_pieces == (kings | minor_pieces) && minor_pieces.count() == 1 {
            return true;
        }

        // Both sides have a King and a Bishop, the Bishops being the same Color
        if (all_pieces == kings | (bishops & BitBoard::LIGHT_SQUARES))
            || (all_pieces == kings | (bishops & BitBoard::DARK_SQUARES))
        {
            return true;
        }

        false
    }
}
