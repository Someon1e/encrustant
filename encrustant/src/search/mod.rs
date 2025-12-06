//! Finds the best outcome in a chess position.

pub mod encoded_move;
pub mod pv;
pub mod search_params;
pub mod time_manager;

use pv::Pv;
use time_manager::TimeManager;

use crate::{
    board::{Board, game_state::GameState, piece::Piece, square::Square},
    evaluation::{
        Eval,
        eval_data::{self, Score},
    },
    move_generator::{
        MoveGenerator,
        move_data::{Flag, Move},
    },
};

use self::encoded_move::EncodedMove;

pub type Ply = u8;

/// Score of having checkmated the opponent.
pub const IMMEDIATE_CHECKMATE_SCORE: Score = 70000;

const CHECKMATE_SCORE: Score = IMMEDIATE_CHECKMATE_SCORE - (Ply::MAX as Score);

#[cfg(not(feature = "spsa"))]
macro_rules! param {
    ($self:expr) => {
        crate::search::search_params::DEFAULT_TUNABLES
    };
}
#[cfg(feature = "spsa")]
macro_rules! param {
    ($self:expr) => {
        $self.tunable
    };
}

/// Search info at a depth.
#[derive(Clone)]
pub struct DepthSearchInfo<'a> {
    /// Depth searched at.
    pub depth: Ply,

    /// Highest number of moves looked ahead.
    pub highest_depth: Ply,

    /// The best move and evaluation.
    pub best: (&'a Pv, Score),

    /// How many times `make_move` was called in search
    pub node_count: u64,

    pub hash_full: u16,
}

/// Information used in search about the position.
#[derive(Clone, Copy, Debug)]
pub struct SearchState {
    total_middle_game_score: Score,
    total_end_game_score: Score,
}

/// A combination of `GameState` and `SearchState`.
pub struct ExtendedState {
    game_state: GameState,
    search_state: SearchState,
}

/// Looks for the best outcome in a position.
pub struct Search {
    board: Board,

    search_state: SearchState,

    pub pv: Pv,
    pub highest_depth: Ply,

    node_count: u64,

    #[cfg(feature = "spsa")]
    tunable: crate::search::search_params::Tunable,
}

impl Search {
    /// Create a new search.
    #[must_use]
    pub fn new(
        board: Board,
        transposition_capacity: usize,
        #[cfg(feature = "spsa")] tunable: crate::search::search_params::Tunable,
    ) -> Self {
        let (total_middle_game_score, total_end_game_score) = Eval::raw_evaluate(&board);

        Self {
            board,

            search_state: SearchState {
                total_middle_game_score,
                total_end_game_score,
            },

            pv: Pv::new(),
            highest_depth: 0,

            node_count: 0,

            #[cfg(feature = "spsa")]
            tunable,
        }
    }

    /// Sets an empty transposition table with the new capacity.
    pub fn resize_transposition_table(&mut self, transposition_capacity: usize) {}

    /// Returns the current board.
    #[must_use]
    pub const fn board(&self) -> &Board {
        &self.board
    }

    /// A new position.
    pub fn new_board(&mut self, board: Board) {
        self.board = board;

        let (total_middle_game_score, total_end_game_score) = Eval::raw_evaluate(&self.board);
        self.search_state.total_middle_game_score = total_middle_game_score;
        self.search_state.total_end_game_score = total_end_game_score;
    }

    /// Another search.
    pub fn clear_for_new_search(&mut self) {
        // Don't need to clear `eval_history` because each ply is overwritten before they can be read

        self.node_count = 0;
        self.highest_depth = 0;
    }

    /// A new match.
    pub fn clear_cache_for_new_game(&mut self) {}

    fn evaluation_remove_piece(&mut self, piece: Piece, square: Square) {
        let is_white = match piece {
            Piece::WhitePawn
            | Piece::WhiteKnight
            | Piece::WhiteBishop
            | Piece::WhiteRook
            | Piece::WhiteQueen
            | Piece::WhiteKing => true,
            Piece::BlackPawn
            | Piece::BlackKnight
            | Piece::BlackBishop
            | Piece::BlackRook
            | Piece::BlackQueen
            | Piece::BlackKing => false,
        };
        let piece_index = if is_white {
            piece as usize
        } else {
            piece as usize - 6
        };
        let actual_square = if is_white { square.flip() } else { square };
        let (middle_game_value, end_game_value) = Eval::get_piece_value(
            &eval_data::PIECE_SQUARE_TABLE,
            piece_index,
            actual_square.usize(),
        );

        if is_white {
            self.search_state.total_middle_game_score -= i32::from(middle_game_value);
            self.search_state.total_end_game_score -= i32::from(end_game_value);
        } else {
            self.search_state.total_middle_game_score += i32::from(middle_game_value);
            self.search_state.total_end_game_score += i32::from(end_game_value);
        }
    }
    fn evaluation_add_piece(&mut self, piece: Piece, square: Square) {
        let is_white = match piece {
            Piece::WhitePawn
            | Piece::WhiteKnight
            | Piece::WhiteBishop
            | Piece::WhiteRook
            | Piece::WhiteQueen
            | Piece::WhiteKing => true,
            Piece::BlackPawn
            | Piece::BlackKnight
            | Piece::BlackBishop
            | Piece::BlackRook
            | Piece::BlackQueen
            | Piece::BlackKing => false,
        };
        let piece_index = if is_white {
            piece as usize
        } else {
            piece as usize - 6
        };
        let actual_square = if is_white { square.flip() } else { square };
        let (middle_game_value, end_game_value) = Eval::get_piece_value(
            &eval_data::PIECE_SQUARE_TABLE,
            piece_index,
            actual_square.usize(),
        );

        if is_white {
            self.search_state.total_middle_game_score += i32::from(middle_game_value);
            self.search_state.total_end_game_score += i32::from(end_game_value);
        } else {
            self.search_state.total_middle_game_score -= i32::from(middle_game_value);
            self.search_state.total_end_game_score -= i32::from(end_game_value);
        }
    }

    #[must_use]
    pub fn static_evaluate(&self) -> Score {
        let phases = eval_data::PHASE_WEIGHTS;
        #[rustfmt::skip]
        let total_phase = {
            phases[0] * 16
            + phases[1] * 4
            + phases[2] * 4
            + phases[3] * 4
            + phases[4] * 2
        };
        let phase = Eval::get_phase(&self.board, &phases);

        let static_eval = Eval::calculate_score(
            phase,
            total_phase,
            self.search_state.total_middle_game_score,
            self.search_state.total_end_game_score,
        ) * if self.board.white_to_move { 1 } else { -1 };

        #[cfg(debug_assertions)]
        {
            assert_eq!(static_eval, Eval::evaluate(&self.board));
        };

        static_eval
    }

    /// Makes a move and updates the evaluation.
    pub fn make_move(&mut self, move_data: &Move) -> ExtendedState {
        let search_state = self.search_state;

        //self.search_state.position_zobrist_key.flip_side_to_move();

        let piece = self.board.friendly_piece_at(move_data.from).unwrap();

        //self.search_state
        //    .position_zobrist_key
        //    .xor_piece(piece as usize, move_data.from.usize());
        match piece {
            Piece::WhitePawn | Piece::BlackPawn => {
                //self.search_state
                //    .pawn_zobrist_key
                //    .xor_piece(piece as usize, move_data.from.usize());
            }

            Piece::BlackKnight
            | Piece::WhiteKnight
            | Piece::BlackBishop
            | Piece::WhiteBishop
            | Piece::WhiteKing
            | Piece::BlackKing => {
                //self.search_state
                //    .minor_piece_zobrist_key
                //    .xor_piece(piece as usize, move_data.from.usize());
            }

            _ => {}
        }
        self.evaluation_remove_piece(piece, move_data.from);

        let flag = move_data.flag;

        //self.search_state
        //    .position_zobrist_key
        //    .xor_castling_rights(&self.board.game_state.castling_rights);
        {
            let mut castling_rights = self.board.game_state.castling_rights;
            if piece == Piece::WhiteKing {
                castling_rights.unset_white_king_side();
                castling_rights.unset_white_queen_side();
            } else if piece == Piece::BlackKing {
                castling_rights.unset_black_king_side();
                castling_rights.unset_black_queen_side();
            }
            if move_data.from == Square::from_index(0) || move_data.to == Square::from_index(0) {
                castling_rights.unset_white_queen_side();
            }
            if move_data.from == Square::from_index(7) || move_data.to == Square::from_index(7) {
                castling_rights.unset_white_king_side();
            }
            if move_data.from == Square::from_index(56) || move_data.to == Square::from_index(56) {
                castling_rights.unset_black_queen_side();
            }
            if move_data.from == Square::from_index(63) || move_data.to == Square::from_index(63) {
                castling_rights.unset_black_king_side();
            }
            //self.search_state
            //    .position_zobrist_key
            //    .xor_castling_rights(&castling_rights);
        }

        let promotion_piece = flag.get_promotion_piece(self.board.white_to_move);

        if let Some(promotion_piece) = promotion_piece {
            self.evaluation_add_piece(promotion_piece, move_data.to);
            //self.search_state
            //    .position_zobrist_key
            //    .xor_piece(promotion_piece as usize, move_data.to.usize());

            //if matches!(
            //    promotion_piece,
            //    Piece::BlackKnight | Piece::WhiteKnight | Piece::BlackBishop | Piece::WhiteBishop
            //) {
            //    self.search_state
            //        .minor_piece_zobrist_key
            //        .xor_piece(promotion_piece as usize, move_data.to.usize());
            //}
        } else {
            self.evaluation_add_piece(piece, move_data.to);
            //self.search_state
            //    .position_zobrist_key
            //    .xor_piece(piece as usize, move_data.to.usize());

            match piece {
                Piece::WhitePawn | Piece::BlackPawn => {
                    //self.search_state
                    //    .pawn_zobrist_key
                    //    .xor_piece(piece as usize, move_data.to.usize());
                }

                //Piece::BlackKnight
                //| Piece::WhiteKnight
                //| Piece::BlackBishop
                //| Piece::WhiteBishop
                //| Piece::WhiteKing
                //| Piece::BlackKing => self
                //    .search_state
                //    .minor_piece_zobrist_key
                //    .xor_piece(piece as usize, move_data.to.usize()),
                _ => {}
            }
        }

        if let Some(en_passant_square) = self.board.game_state.en_passant_square {
            //self.search_state
            //    .position_zobrist_key
            //    .xor_en_passant(&en_passant_square);
        }
        match flag {
            Flag::PawnTwoUp => {
                let en_passant_square =
                    move_data
                        .from
                        .up(if self.board.white_to_move { 1 } else { -1 });
                //self.search_state
                //    .position_zobrist_key
                //    .xor_en_passant(&en_passant_square);
            }
            Flag::Castle => {
                let is_king_side = move_data.to.file() == 6;
                let rook_to_offset = if is_king_side { -1 } else { 1 };
                let rook_from_offset = if is_king_side { 1 } else { -2 };
                let rook = if self.board.white_to_move {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                };

                let rook_from = move_data.to.offset(rook_from_offset);
                let rook_to = move_data.to.offset(rook_to_offset);

                self.evaluation_remove_piece(rook, rook_from);
                self.evaluation_add_piece(rook, rook_to);

                //self.search_state
                //    .position_zobrist_key
                //    .xor_piece(rook as usize, rook_from.usize());
                //self.search_state
                //    .position_zobrist_key
                //    .xor_piece(rook as usize, rook_to.usize());
            }
            Flag::EnPassant => {
                let capture_position = self
                    .board
                    .game_state
                    .en_passant_square
                    .unwrap()
                    .down(if self.board.white_to_move { 1 } else { -1 });
                let captured = if self.board.white_to_move {
                    Piece::BlackPawn
                } else {
                    Piece::WhitePawn
                };

                self.evaluation_remove_piece(captured, capture_position);
                //self.search_state
                //    .position_zobrist_key
                //    .xor_piece(captured as usize, capture_position.usize());
                //self.search_state
                //    .pawn_zobrist_key
                //    .xor_piece(captured as usize, capture_position.usize());
            }
            _ => {
                if let Some(captured) = self.board.enemy_piece_at(move_data.to) {
                    self.evaluation_remove_piece(captured, move_data.to);
                    //self.search_state
                    //    .position_zobrist_key
                    //    .xor_piece(captured as usize, move_data.to.usize());

                    //match captured {
                    //    Piece::WhitePawn | Piece::BlackPawn => {
                    //        self.search_state
                    //            .pawn_zobrist_key
                    //            .xor_piece(captured as usize, move_data.to.usize());
                    //    }

                    //    Piece::BlackKnight
                    //    | Piece::WhiteKnight
                    //    | Piece::BlackBishop
                    //    | Piece::WhiteBishop
                    //    | Piece::WhiteKing
                    //    | Piece::BlackKing => {
                    //        self.search_state
                    //            .minor_piece_zobrist_key
                    //            .xor_piece(captured as usize, move_data.to.usize());
                    //    }

                    //    _ => {}
                    //}
                }
            }
        }

        //if PREFETCH {
        //    #[cfg(target_feature = "sse")]
        //    {
        //        use core::arch::x86_64::{_MM_HINT_NTA, _mm_prefetch};
        //        let index =
        //            self.position_zobrist_key()
        //                .distribute(self.transposition_table.len()) as usize;
        //        unsafe {
        //            _mm_prefetch::<{ _MM_HINT_NTA }>(
        //                self.transposition_table.as_ptr().add(index).cast::<i8>(),
        //            );
        //        }
        //    }
        //    #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
        //    {
        //        use core::arch::aarch64::{_PREFETCH_LOCALITY0, _PREFETCH_READ, _prefetch};
        //        let index =
        //            self.position_zobrist_key()
        //                .distribute(self.transposition_table.len()) as usize;
        //        unsafe {
        //            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>(
        //                self.transposition_table.as_ptr().add(index).cast::<i8>(),
        //            );
        //        }
        //    }
        //}

        let game_state = self.board.make_move(move_data);

        ExtendedState {
            game_state,
            search_state,
        }
    }

    /// Unmakes a move and updates the evaluation.
    pub fn unmake_move(&mut self, move_data: &Move, old_state: &ExtendedState) {
        self.search_state = old_state.search_state;
        self.board.unmake_move(move_data, &old_state.game_state);
    }

    fn negamax(
        &mut self,

        time_manager: &TimeManager,

        ply_remaining: Ply,
        ply_from_root: Ply,
    ) -> Score {
        if ply_from_root > self.highest_depth {
            self.highest_depth = ply_from_root;
        }

        self.pv.set_pv_length(ply_from_root, ply_from_root);

        if ply_remaining == 0 {
            return self.static_evaluate();
        }

        let move_generator = MoveGenerator::new(&self.board);
        let mut best_score = -Score::MAX;
        let mut move_count = 0;

        move_generator.generate(
            |move_data| {
                move_count += 1;

                if time_manager.hard_stop_inner_search(self.node_count) {
                    return;
                }

                self.node_count += 1;
                let old_state = self.make_move(&move_data);

                let score = -self.negamax(time_manager, ply_remaining - 1, ply_from_root + 1);

                self.unmake_move(&move_data, &old_state);

                if score > best_score {
                    self.pv
                        .update_move(ply_from_root, EncodedMove::new(move_data));
                    best_score = score;
                }
            },
            false,
        );

        if time_manager.hard_stop_inner_search(self.node_count) {
            return 0;
        }

        if move_count == 0 {
            // No moves
            let score = if move_generator.is_in_check() {
                // Checkmate
                -IMMEDIATE_CHECKMATE_SCORE + Score::from(ply_from_root)
            } else {
                // Stalemate
                0
            };
            return score;
        }

        best_score
    }

    /// Returns whether a score means forced checkmate.
    #[must_use]
    pub const fn score_is_checkmate(score: Score) -> bool {
        score.abs() >= CHECKMATE_SCORE
    }

    /// Repeatedly searches the board, increasing depth by one each time. Stops when `time_manager` returns `true`.
    #[must_use]
    pub fn iterative_deepening(
        &mut self,

        time_manager: &TimeManager,

        depth_completed: &mut dyn FnMut(DepthSearchInfo),
    ) -> (Ply, Score) {
        let mut depth = 0;
        let mut previous_best_score = -Score::MAX;

        let mut best_move_stability = 0;
        let mut previous_best_move = EncodedMove::NONE;

        loop {
            depth += 1;
            let best_score = self.negamax(time_manager, depth, 0);

            if time_manager.hard_stop_iterative_deepening(depth, self.node_count) {
                // Must stop now.
                break;
            }
            previous_best_score = best_score;

            if self.pv.root_best_move().is_none() {
                while time_manager.is_pondering() {}
                // No point searching more.

                break;
            }

            if self.pv.root_best_move() == previous_best_move {
                best_move_stability += 1;
            } else {
                best_move_stability = 0;
                previous_best_move = self.pv.root_best_move();
            }

            // Depth was completed
            // Report results of search iteration
            depth_completed(DepthSearchInfo {
                depth,
                best: (&self.pv, best_score),
                highest_depth: self.highest_depth,
                node_count: self.node_count,
                hash_full: self.hash_full(),
            });

            if depth == Ply::MAX {
                while time_manager.is_pondering() {}
                // Maximum depth, can not continue
                break;
            }

            if time_manager.soft_stop(
                self.node_count,
                best_score,
                best_move_stability,
                param!(self),
            ) {
                // It would probably be a waste of time to start another iteration
                break;
            }
        }

        (depth, previous_best_score)
    }

    /// Returns how many times `make_move` was called in search
    #[must_use]
    pub const fn node_count(&self) -> u64 {
        self.node_count
    }

    #[must_use]
    pub fn calculate_time(&self, clock_time: u64, increment: u64) -> (u64, u64) {
        let max_time = clock_time / 2;
        let hard_time_limit =
            (clock_time / param!(self).hard_time_divisor + increment * 2).min(max_time);
        let soft_time_limit =
            (clock_time / param!(self).soft_time_divisor + increment / 2).min(hard_time_limit);
        (hard_time_limit, soft_time_limit)
    }

    #[must_use]
    pub fn hash_full(&self) -> u16 {
        0
    }
}
