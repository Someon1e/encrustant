use encrustant::board::Board;
use encrustant::board::piece::Piece;
use encrustant::consume_bit_board;

pub const PARAMETER_COUNT: usize = 384;

pub struct DataPoint {
    /// Indices of evaluation parameters it used
    /// One for white and black (white = add, black = subtract from evaluation)
    pub active: [Vec<u16>; 2],

    /// Used to calculate game phase
    /// King not included
    pub piece_counts: [f64; 5],

    /// 0.0 -> black win;
    /// 0.5 -> draw;
    /// 1.0 -> white win;
    pub result: f64,
}

pub fn get_piece_counts(board: &Board) -> [f64; 5] {
    [
        (*board.get_bit_board(Piece::WhitePawn) | *board.get_bit_board(Piece::BlackPawn))
            .count()
            .into(),
        (*board.get_bit_board(Piece::WhiteKnight) | *board.get_bit_board(Piece::BlackKnight))
            .count()
            .into(),
        (*board.get_bit_board(Piece::WhiteBishop) | *board.get_bit_board(Piece::BlackBishop))
            .count()
            .into(),
        (*board.get_bit_board(Piece::WhiteRook) | *board.get_bit_board(Piece::BlackRook))
            .count()
            .into(),
        (*board.get_bit_board(Piece::WhiteQueen) | *board.get_bit_board(Piece::BlackQueen))
            .count()
            .into(),
    ]
}

pub fn get_total_phase(phase_weights: &[f64]) -> f64 {
    phase_weights[4].mul_add(
        2.0,
        phase_weights[3].mul_add(
            4.0,
            phase_weights[2].mul_add(4.0, phase_weights[0].mul_add(16.0, phase_weights[1] * 4.0)),
        ),
    )
}

pub fn get_active(board: &Board) -> [Vec<u16>; 2] {
    let mut white = Vec::new();
    let mut black = Vec::new();

    for piece in Piece::WHITE_PIECES {
        let mut bit_board = *board.get_bit_board(piece);
        consume_bit_board!(bit_board, square {
            let square_index = square.flip().usize();
            let piece_index = piece as usize;
            white.push((piece_index * 64 + square_index).try_into().unwrap());
        });
    }

    for piece in Piece::BLACK_PIECES {
        let mut bit_board = *board.get_bit_board(piece);
        consume_bit_board!(bit_board, square {
            let square_index = square.usize();
            let piece_index = piece as usize - 6;
            black.push((piece_index * 64 + square_index).try_into().unwrap());
        });
    }

    [white, black]
}

impl DataPoint {
    /// Returns in the range of 0..=1
    pub fn get_phase(&self, phase_weights: &[f64]) -> f64 {
        let total_phase = get_total_phase(phase_weights);

        let mut phase = 0.0;
        for (piece_index, count) in self.piece_counts.iter().enumerate() {
            phase += phase_weights[piece_index] * count;
        }

        phase.min(total_phase) / total_phase
    }

    pub fn evaluate(&self, parameters: &[(f64, f64)], phase: f64) -> f64 {
        let (mut mid_score, mut end_score) = (0.0, 0.0);

        for &used_index in &self.active[0] {
            mid_score += parameters[usize::from(used_index)].0;
            end_score += parameters[usize::from(used_index)].1;
        }

        for &used_index in &self.active[1] {
            mid_score -= parameters[usize::from(used_index)].0;
            end_score -= parameters[usize::from(used_index)].1;
        }

        phase.mul_add(mid_score, (1.0 - phase) * end_score)
    }
}

#[cfg(test)]
mod tests {
    use encrustant::{
        board::Board,
        evaluation::{
            Eval,
            eval_data::{PHASE_WEIGHTS, PIECE_SQUARE_TABLE},
        },
    };

    use crate::PARAMETER_COUNT;

    use super::{DataPoint, get_active, get_piece_counts};

    #[test]
    fn test_evaluation() {
        for test_position in [
            "rnbq1bnr/pppp1ppp/8/4k3/4P3/8/PPPPKPPP/RNBQ1B1R w - - 0 5",
            "8/8/8/3K4/8/8/8/7k w - - 0 1",
            "8/8/3k4/4r3/8/8/3Q4/2K5 b - - 0 1",
        ] {
            let board = Board::from_fen(test_position).unwrap();
            let true_eval = Eval::evaluate(&board) * if board.white_to_move { 1 } else { -1 };

            let data_point = DataPoint {
                active: get_active(&board),
                piece_counts: get_piece_counts(&board),
                result: 0.5, // placeholder, not used
            };

            let mut parameters = [(0.0, 0.0); PARAMETER_COUNT];
            for index in 0..PARAMETER_COUNT {
                parameters[index] = (
                    PIECE_SQUARE_TABLE[index].0.into(),
                    PIECE_SQUARE_TABLE[index].1.into(),
                );
            }
            let mut phase_weights = [0.0; 5];
            for (index, value) in PHASE_WEIGHTS.iter().enumerate() {
                phase_weights[index] = (*value).into();
            }
            let phase = data_point.get_phase(&phase_weights);
            assert_eq!(data_point.evaluate(&parameters, phase) as i32, true_eval);
        }
    }
}
