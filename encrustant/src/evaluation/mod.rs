//! Estimates how good a chess position is.

use crate::{
    board::{Board, piece::Piece},
    consume_bit_board,
};

pub mod eval_data;
use eval_data::{EvalNumber, PieceSquareTable};

/// Evaluation functions.
pub struct Eval;
impl Eval {
    /// Gets the phase.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // count() should never return more than 64
    pub fn get_phase(board: &Board, phases: &[EvalNumber; 5]) -> EvalNumber {
        let mut phase = 0;

        phase += phases[0]
            * ((*board.get_bit_board(Piece::WhitePawn) | *board.get_bit_board(Piece::BlackPawn))
                .count()) as EvalNumber;

        phase += phases[1]
            * ((*board.get_bit_board(Piece::WhiteKnight)
                | *board.get_bit_board(Piece::BlackKnight))
            .count()) as EvalNumber;

        phase += phases[2]
            * ((*board.get_bit_board(Piece::WhiteBishop)
                | *board.get_bit_board(Piece::BlackBishop))
            .count()) as EvalNumber;

        phase += phases[3]
            * ((*board.get_bit_board(Piece::WhiteRook) | *board.get_bit_board(Piece::BlackRook))
                .count()) as EvalNumber;

        phase += phases[4]
            * ((*board.get_bit_board(Piece::WhiteQueen) | *board.get_bit_board(Piece::BlackQueen))
                .count()) as EvalNumber;

        phase
    }

    /// Get the middlegame and endgame value of a piece on a square
    #[must_use]
    pub const fn get_piece_value(
        piece_square_tables: &PieceSquareTable,
        piece_index: usize,
        square_index: usize,
    ) -> (i16, i16) {
        let piece_score = piece_square_tables[piece_index * 64 + square_index];

        (piece_score.0, piece_score.1)
    }

    /// Uses the phase to interpolate between middlegame and endgame score
    #[must_use]
    pub fn calculate_score(
        phase: EvalNumber,
        total_phase: EvalNumber,
        middle_game_score: EvalNumber,
        end_game_score: EvalNumber,
    ) -> EvalNumber {
        let middle_game_phase = phase.min(total_phase);
        let end_game_phase = total_phase - middle_game_phase;
        (middle_game_score * middle_game_phase + end_game_score * end_game_phase) / total_phase
    }

    #[must_use]
    pub fn raw_evaluate_with_parameters(
        piece_square_tables: &PieceSquareTable,
        board: &Board,
    ) -> (EvalNumber, EvalNumber) {
        let mut total_middle_game_score = 0;
        let mut total_end_game_score = 0;

        for piece in Piece::WHITE_PIECES {
            let mut bit_board = *board.get_bit_board(piece);
            consume_bit_board!(bit_board, square {
                let (middle_game_value, end_game_value) = Self::get_piece_value(
                    piece_square_tables,
                    piece as usize,
                    square.flip().usize(),
                );

                total_middle_game_score += i32::from(middle_game_value);
                total_end_game_score += i32::from(end_game_value);
            });
        }

        for piece in Piece::BLACK_PIECES {
            let mut bit_board = *board.get_bit_board(piece);
            consume_bit_board!(bit_board, square {
                let (middle_game_value, end_game_value) = Self::get_piece_value(
                    piece_square_tables,
                    piece as usize - 6,
                    square.usize(),
                );

                total_middle_game_score -= i32::from(middle_game_value);
                total_end_game_score -= i32::from(end_game_value);
            });
        }

        (total_middle_game_score, total_end_game_score)
    }

    /// Returns an estimated score of the position for the side playing, using the provided evaluation parameters.
    #[must_use]
    pub fn evaluate_with_parameters(
        piece_square_tables: &PieceSquareTable,
        phases: &[EvalNumber; 5],
        board: &Board,
    ) -> EvalNumber {
        #[rustfmt::skip]
        let total_phase = {
            phases[0] * 16
            + phases[1] * 4
            + phases[2] * 4
            + phases[3] * 4
            + phases[4] * 2
        };

        let (total_middle_game_score, total_end_game_score) =
            Self::raw_evaluate_with_parameters(piece_square_tables, board);
        let phase = Self::get_phase(board, phases);
        Self::calculate_score(
            phase,
            total_phase,
            total_middle_game_score,
            total_end_game_score,
        ) * if board.white_to_move { 1 } else { -1 }
    }

    /// Returns an estimated score of the position for the side playing.
    #[must_use]
    pub fn evaluate(board: &Board) -> EvalNumber {
        Self::evaluate_with_parameters(
            &eval_data::PIECE_SQUARE_TABLE,
            &eval_data::PHASE_WEIGHTS,
            board,
        )
    }

    #[must_use]
    pub fn raw_evaluate(board: &Board) -> (EvalNumber, EvalNumber) {
        Self::raw_evaluate_with_parameters(&eval_data::PIECE_SQUARE_TABLE, board)
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::Board, evaluation::Eval};

    #[test]
    fn advanced_pawn_worth_more() {
        let starting_rank_pawn = Board::from_fen("7k/8/8/8/8/8/4P3/K7 w - - 0 1").unwrap();
        let one_step_from_promoting_pawn =
            Board::from_fen("7k/4P3/8/8/8/8/8/K7 w - - 0 1").unwrap();
        assert!(
            Eval::evaluate(&one_step_from_promoting_pawn) > Eval::evaluate(&starting_rank_pawn)
        );
    }

    #[test]
    fn centralised_knight_worth_more() {
        let centralised_knight = Board::from_fen("7k/8/8/4n3/8/8/8/K7 b - - 0 1").unwrap();
        let knight_on_the_edge = Board::from_fen("7k/8/8/8/7n/8/8/K7 b - - 0 1").unwrap();
        assert!(Eval::evaluate(&centralised_knight) > Eval::evaluate(&knight_on_the_edge));
    }
}
