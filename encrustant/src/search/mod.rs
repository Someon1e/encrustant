//! Finds the best outcome in a chess position.

pub mod encoded_move;
mod move_ordering;
pub mod pv;
mod repetition_table;
pub mod search_params;
pub mod time_manager;
pub mod transposition;

/// Zobrist key.
pub mod zobrist;

use pv::Pv;
use time_manager::TimeManager;
use zobrist::Zobrist;

use crate::{
    board::{Board, game_state::GameState, piece::Piece, square::Square},
    evaluation::{
        Eval,
        eval_data::{self, EvalNumber},
    },
    move_generator::{
        MoveGenerator,
        move_data::{Flag, Move},
    },
};

use self::{
    encoded_move::EncodedMove,
    move_ordering::MoveOrderer,
    repetition_table::RepetitionTable,
    transposition::{NodeType, NodeValue},
};

pub type Ply = u8;

/// Score of having checkmated the opponent.
pub const IMMEDIATE_CHECKMATE_SCORE: EvalNumber = 70000;

const CHECKMATE_SCORE: EvalNumber = IMMEDIATE_CHECKMATE_SCORE - (Ply::MAX as EvalNumber);

const USE_STATIC_NULL_MOVE_PRUNING: bool = true;
const USE_NULL_MOVE_PRUNING: bool = true;
const USE_LATE_MOVE_REDUCTION: bool = true;
const USE_INTERNAL_ITERATIVE_REDUCTION: bool = true;
const USE_PVS: bool = true;
const USE_KILLER_MOVE: bool = true;
const USE_ASPIRATION_WINDOWS: bool = true;
const USE_FUTILITY_PRUNING: bool = true;

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
    pub best: (&'a Pv, EvalNumber),

    /// How many times `make_move` was called in search
    pub node_count: u64,

    pub hash_full: u16,
}

const PAWN_CORRECTION_HISTORY_LENGTH: usize = 8192;
const MINOR_PIECE_CORRECTION_HISTORY_LENGTH: usize = 8192;

/// Information used in search about the position.
#[derive(Clone, Copy, Debug)]
pub struct SearchState {
    total_middle_game_score: EvalNumber,
    total_end_game_score: EvalNumber,

    /// Position zobrist key.
    pub position_zobrist_key: Zobrist,

    /// Pawn zobrist key.
    pub pawn_zobrist_key: Zobrist,

    /// Minor piece (knight, bishop, king) zobrist key.
    pub minor_piece_zobrist_key: Zobrist,
}

/// A combination of `GameState` and `SearchState`.
pub struct ExtendedState {
    game_state: GameState,
    search_state: SearchState,
}

/// Looks for the best outcome in a position.
pub struct Search {
    board: Board,

    repetition_table: RepetitionTable,

    transposition_table: Vec<Option<NodeValue>>,

    quiet_history: Box<[[i16; 64 * 64]; 2]>,
    capture_history: Box<[[[i16; 6]; 64]; 12]>, // Inner table length is 6 because outer table already gives information about the piece colour

    pawn_correction_history: Box<[[i16; PAWN_CORRECTION_HISTORY_LENGTH]; 2]>,
    minor_piece_correction_history: Box<[[i16; MINOR_PIECE_CORRECTION_HISTORY_LENGTH]; 2]>,

    eval_history: [EvalNumber; 256],

    killer_moves: [EncodedMove; 64],

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
        let position_zobrist_key = Zobrist::compute(&board);
        let pawn_zobrist_key = Zobrist::pawn_key(&board);
        let minor_piece_zobrist_key = Zobrist::minor_piece_key(&board);

        Self {
            board,

            repetition_table: RepetitionTable::new(),

            transposition_table: vec![None; transposition_capacity],

            killer_moves: [EncodedMove::NONE; 64],
            quiet_history: vec![[0; 64 * 64]; 2].try_into().unwrap(),
            capture_history: vec![[[0; 6]; 64]; 12].try_into().unwrap(),

            pawn_correction_history: vec![[0; PAWN_CORRECTION_HISTORY_LENGTH]; 2]
                .try_into()
                .unwrap(),
            minor_piece_correction_history: vec![[0; MINOR_PIECE_CORRECTION_HISTORY_LENGTH]; 2]
                .try_into()
                .unwrap(),

            eval_history: [0; 256],

            search_state: SearchState {
                total_middle_game_score,
                total_end_game_score,
                position_zobrist_key,
                pawn_zobrist_key,
                minor_piece_zobrist_key,
            },

            pv: Pv::new(),
            highest_depth: 0,

            node_count: 0,

            #[cfg(feature = "spsa")]
            tunable,
        }
    }

    /// Skips the turn
    pub fn make_null_move(&mut self) -> ExtendedState {
        self.repetition_table.push(self.position_zobrist_key());

        let old_search_state = self.search_state;
        let old_game_state = self.board.game_state;

        self.board.white_to_move = !self.board.white_to_move;
        self.search_state.position_zobrist_key.flip_side_to_move();

        self.board.game_state.half_move_clock = 0;

        let en_passant_square = self.board.game_state.en_passant_square;
        if let Some(en_passant_square) = en_passant_square {
            self.search_state
                .position_zobrist_key
                .xor_en_passant(&en_passant_square);
        }
        self.board.game_state.en_passant_square = None;

        ExtendedState {
            search_state: old_search_state,
            game_state: old_game_state,
        }
    }

    /// Unskips the turn
    pub fn unmake_null_move(&mut self, old_state: &ExtendedState) {
        self.search_state = old_state.search_state;
        self.board.game_state = old_state.game_state;
        self.board.white_to_move = !self.board.white_to_move;
        assert_eq!(self.repetition_table.pop(), self.position_zobrist_key());
    }

    /// Sets an empty transposition table with the new capacity.
    pub fn resize_transposition_table(&mut self, transposition_capacity: usize) {
        self.transposition_table = vec![None; transposition_capacity];
    }

    /// Returns the current board.
    #[must_use]
    pub const fn board(&self) -> &Board {
        &self.board
    }

    /// A new position.
    pub fn new_board(&mut self, board: Board) {
        self.board = board;
        self.repetition_table.clear();

        let position_zobrist_key = Zobrist::compute(&self.board);
        let pawn_zobrist_key = Zobrist::pawn_key(&self.board);
        let minor_piece_zobrist_key = Zobrist::minor_piece_key(&self.board);
        let (total_middle_game_score, total_end_game_score) = Eval::raw_evaluate(&self.board);
        self.search_state.total_middle_game_score = total_middle_game_score;
        self.search_state.total_end_game_score = total_end_game_score;
        self.search_state.position_zobrist_key = position_zobrist_key;
        self.search_state.pawn_zobrist_key = pawn_zobrist_key;
        self.search_state.minor_piece_zobrist_key = minor_piece_zobrist_key;
    }

    /// Another search.
    pub fn clear_for_new_search(&mut self) {
        // Don't need to clear `eval_history` because each ply is overwritten before they can be read

        self.node_count = 0;
        self.highest_depth = 0;
        self.killer_moves.fill(EncodedMove::NONE);

        for value in &mut self.quiet_history[0] {
            *value /= param!(self).history_decay;
        }
        for value in &mut self.quiet_history[1] {
            *value /= param!(self).history_decay;
        }
    }

    /// A new match.
    pub fn clear_cache_for_new_game(&mut self) {
        self.pawn_correction_history[0].fill(0);
        self.pawn_correction_history[1].fill(0);
        self.minor_piece_correction_history[0].fill(0);
        self.minor_piece_correction_history[1].fill(0);

        for x in self.capture_history.iter_mut() {
            for y in x.iter_mut() {
                y.fill(0);
            }
        }

        self.quiet_history[0].fill(0);
        self.quiet_history[1].fill(0);

        self.transposition_table.fill(None);
    }

    #[must_use]
    fn quiescence_search(&mut self, mut alpha: EvalNumber, beta: EvalNumber) -> EvalNumber {
        let pawn_index = self
            .pawn_zobrist_key()
            .modulo(PAWN_CORRECTION_HISTORY_LENGTH as u64);
        let minor_piece_index = self
            .minor_piece_zobrist_key()
            .modulo(MINOR_PIECE_CORRECTION_HISTORY_LENGTH as u64);

        let mut best_score =
            self.get_correction(self.static_evaluate(), pawn_index, minor_piece_index);

        if best_score > alpha {
            alpha = best_score;

            if best_score >= beta {
                return best_score;
            }
        }

        let move_generator = MoveGenerator::new(&self.board);
        let (mut move_guesses, move_count) =
            MoveOrderer::get_move_guesses_captures_only(self, &move_generator);
        let mut index = 0;
        while index != move_count {
            let move_data = unsafe {
                // SAFETY: `get_move_guesses_captures_only` guarantees that `move_guesses[0..move_count]` are initialised.
                // `index` can not be higher than `move_count`, due to the loop condition.

                MoveOrderer::put_highest_guessed_move(&mut move_guesses, index, move_count)
            }
            .move_data
            .decode();

            let old_state = self.make_move::<false>(&move_data);
            self.node_count += 1;
            let score = -self.quiescence_search(-beta, -alpha);
            self.unmake_move(&move_data, &old_state);

            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;

                    if score >= beta {
                        break;
                    }
                }
            }

            index += 1;
        }
        best_score
    }

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

    /// Returns the current position zobrist key
    #[must_use]
    pub const fn position_zobrist_key(&self) -> Zobrist {
        self.search_state.position_zobrist_key
    }

    /// Returns the current pawn zobrist key
    #[must_use]
    pub const fn pawn_zobrist_key(&self) -> Zobrist {
        self.search_state.pawn_zobrist_key
    }

    /// Returns the current minor piece (knight, bishop, king) zobrist key
    #[must_use]
    pub fn minor_piece_zobrist_key(&self) -> Zobrist {
        self.search_state.minor_piece_zobrist_key
    }

    #[must_use]
    pub fn static_evaluate(&self) -> EvalNumber {
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
    pub fn make_move<const PREFETCH: bool>(&mut self, move_data: &Move) -> ExtendedState {
        debug_assert!(Zobrist::pawn_key(&self.board) == self.pawn_zobrist_key());
        debug_assert!(Zobrist::minor_piece_key(&self.board) == self.minor_piece_zobrist_key());
        debug_assert!(Zobrist::compute(&self.board) == self.position_zobrist_key());

        let search_state = self.search_state;

        self.search_state.position_zobrist_key.flip_side_to_move();

        let piece = self.board.friendly_piece_at(move_data.from).unwrap();

        self.search_state
            .position_zobrist_key
            .xor_piece(piece as usize, move_data.from.usize());
        match piece {
            Piece::WhitePawn | Piece::BlackPawn => {
                self.search_state
                    .pawn_zobrist_key
                    .xor_piece(piece as usize, move_data.from.usize());
            }

            Piece::BlackKnight
            | Piece::WhiteKnight
            | Piece::BlackBishop
            | Piece::WhiteBishop
            | Piece::WhiteKing
            | Piece::BlackKing => {
                self.search_state
                    .minor_piece_zobrist_key
                    .xor_piece(piece as usize, move_data.from.usize());
            }

            _ => {}
        }
        self.evaluation_remove_piece(piece, move_data.from);

        let flag = move_data.flag;

        self.search_state
            .position_zobrist_key
            .xor_castling_rights(&self.board.game_state.castling_rights);
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
            self.search_state
                .position_zobrist_key
                .xor_castling_rights(&castling_rights);
        }

        let promotion_piece = flag.get_promotion_piece(self.board.white_to_move);

        if let Some(promotion_piece) = promotion_piece {
            self.evaluation_add_piece(promotion_piece, move_data.to);
            self.search_state
                .position_zobrist_key
                .xor_piece(promotion_piece as usize, move_data.to.usize());

            if matches!(
                promotion_piece,
                Piece::BlackKnight | Piece::WhiteKnight | Piece::BlackBishop | Piece::WhiteBishop
            ) {
                self.search_state
                    .minor_piece_zobrist_key
                    .xor_piece(promotion_piece as usize, move_data.to.usize())
            }
        } else {
            self.evaluation_add_piece(piece, move_data.to);
            self.search_state
                .position_zobrist_key
                .xor_piece(piece as usize, move_data.to.usize());

            match piece {
                Piece::WhitePawn | Piece::BlackPawn => {
                    self.search_state
                        .pawn_zobrist_key
                        .xor_piece(piece as usize, move_data.to.usize());
                }

                Piece::BlackKnight
                | Piece::WhiteKnight
                | Piece::BlackBishop
                | Piece::WhiteBishop
                | Piece::WhiteKing
                | Piece::BlackKing => self
                    .search_state
                    .minor_piece_zobrist_key
                    .xor_piece(piece as usize, move_data.to.usize()),

                _ => {}
            }
        }

        if let Some(en_passant_square) = self.board.game_state.en_passant_square {
            self.search_state
                .position_zobrist_key
                .xor_en_passant(&en_passant_square);
        }
        match flag {
            Flag::PawnTwoUp => {
                let en_passant_square =
                    move_data
                        .from
                        .up(if self.board.white_to_move { 1 } else { -1 });
                self.search_state
                    .position_zobrist_key
                    .xor_en_passant(&en_passant_square);
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

                self.search_state
                    .position_zobrist_key
                    .xor_piece(rook as usize, rook_from.usize());
                self.search_state
                    .position_zobrist_key
                    .xor_piece(rook as usize, rook_to.usize());
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
                self.search_state
                    .position_zobrist_key
                    .xor_piece(captured as usize, capture_position.usize());
                self.search_state
                    .pawn_zobrist_key
                    .xor_piece(captured as usize, capture_position.usize());
            }
            _ => {
                if let Some(captured) = self.board.enemy_piece_at(move_data.to) {
                    self.evaluation_remove_piece(captured, move_data.to);
                    self.search_state
                        .position_zobrist_key
                        .xor_piece(captured as usize, move_data.to.usize());

                    match captured {
                        Piece::WhitePawn | Piece::BlackPawn => {
                            self.search_state
                                .pawn_zobrist_key
                                .xor_piece(captured as usize, move_data.to.usize());
                        }

                        Piece::BlackKnight
                        | Piece::WhiteKnight
                        | Piece::BlackBishop
                        | Piece::WhiteBishop
                        | Piece::WhiteKing
                        | Piece::BlackKing => {
                            self.search_state
                                .minor_piece_zobrist_key
                                .xor_piece(captured as usize, move_data.to.usize());
                        }

                        _ => {}
                    }
                }
            }
        }

        if PREFETCH {
            #[cfg(target_feature = "sse")]
            {
                use core::arch::x86_64::{_MM_HINT_NTA, _mm_prefetch};
                let index =
                    self.position_zobrist_key()
                        .distribute(self.transposition_table.len()) as usize;
                unsafe {
                    _mm_prefetch::<{ _MM_HINT_NTA }>(
                        self.transposition_table.as_ptr().add(index).cast::<i8>(),
                    );
                }
            }
            #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
            {
                use core::arch::aarch64::{_PREFETCH_LOCALITY0, _PREFETCH_READ, _prefetch};
                let index =
                    self.position_zobrist_key()
                        .distribute(self.transposition_table.len()) as usize;
                unsafe {
                    _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>(
                        self.transposition_table.as_ptr().add(index).cast::<i8>(),
                    );
                }
            }
        }

        let game_state = self.board.make_move(move_data);

        debug_assert!(Zobrist::pawn_key(&self.board) == self.pawn_zobrist_key());
        debug_assert!(Zobrist::minor_piece_key(&self.board) == self.minor_piece_zobrist_key());
        debug_assert!(Zobrist::compute(&self.board) == self.position_zobrist_key());

        ExtendedState {
            game_state,
            search_state,
        }
    }

    /// Adds the position into the repetition table then calls `self.make_move`.
    pub fn make_move_repetition<const PREFETCH: bool>(
        &mut self,
        move_data: &Move,
    ) -> ExtendedState {
        self.repetition_table.push(self.position_zobrist_key());

        self.make_move::<PREFETCH>(move_data)
    }

    /// Calls `unmake_move`, then removes the position from the repetition table.
    ///
    /// # Panics
    ///
    /// Will panic if the zobrist key after playing the move does not match the previous position's.
    pub fn unmake_move_repetition(&mut self, move_data: &Move, old_state: &ExtendedState) {
        self.unmake_move(move_data, old_state);

        assert_eq!(self.repetition_table.pop(), self.position_zobrist_key());
    }

    /// Unmakes a move and updates the evaluation.
    pub fn unmake_move(&mut self, move_data: &Move, old_state: &ExtendedState) {
        self.search_state = old_state.search_state;
        self.board.unmake_move(move_data, &old_state.game_state);

        debug_assert!(Zobrist::compute(&self.board) == self.position_zobrist_key());
        debug_assert!(Zobrist::pawn_key(&self.board) == self.pawn_zobrist_key());
        debug_assert!(Zobrist::minor_piece_key(&self.board) == self.minor_piece_zobrist_key());
    }

    fn negamax(
        &mut self,

        time_manager: &TimeManager,

        mut ply_remaining: Ply,
        ply_from_root: Ply,

        allow_null_move: bool,

        mut alpha: EvalNumber,
        beta: EvalNumber,
    ) -> EvalNumber {
        if ply_from_root > self.highest_depth {
            self.highest_depth = ply_from_root;
        }

        self.pv.set_pv_length(ply_from_root, ply_from_root);

        // Get the zobrist key
        let zobrist_key = self.position_zobrist_key();

        // Check for repetition
        if ply_from_root != 0 {
            if self
                .repetition_table
                .contains(zobrist_key, self.board.game_state.half_move_clock)
            {
                return 0;
            }
            if self.board.is_insufficient_material() {
                return 0;
            }
        }

        // Turn zobrist key into an index into the transposition table
        let zobrist_index = zobrist_key.distribute(self.transposition_table.len()) as usize;

        // This is the best move in this position according to previous searches
        let mut hash_move = EncodedMove::NONE;

        // Check if this is a pv node
        let is_not_pv_node = alpha + 1 == beta;

        // Get value from transposition table
        let mut saved = None;
        if let Some(entry) = self.transposition_table[zobrist_index] {
            // Check if it's actually the same position
            if entry.zobrist_key_32 == zobrist_key.lower_u32() {
                let value = transposition::retrieve_mate_score(entry.value, ply_from_root);

                // Check if the saved depth is as high as the depth now
                if entry.ply_remaining >= ply_remaining {
                    let node_type = &entry.node_type;
                    if match node_type {
                        NodeType::Exact => is_not_pv_node,
                        NodeType::Beta => value >= beta,
                        NodeType::Alpha => value <= alpha,
                    } {
                        self.pv.update_move(ply_from_root, entry.transposition_move);

                        return value;
                    }
                }

                hash_move = entry.transposition_move;

                saved = Some((value, entry.node_type));
            }
        }

        if ply_from_root == 0 {
            // Use iterative deepening move as hash move
            hash_move = self.pv.root_best_move();
        }
        if USE_INTERNAL_ITERATIVE_REDUCTION
            && hash_move.is_none()
            && ply_remaining > param!(self).iir_min_depth
        {
            // Internal iterative reduction
            ply_remaining = ply_remaining.saturating_sub(param!(self).iir_depth_reduction);
        }

        if ply_remaining == 0 {
            // Enter quiescence search
            return self.quiescence_search(alpha, beta);
        }

        let move_generator = MoveGenerator::new(&self.board);

        let pawn_index = self
            .pawn_zobrist_key()
            .modulo(PAWN_CORRECTION_HISTORY_LENGTH as u64);
        let minor_piece_index = self
            .minor_piece_zobrist_key()
            .modulo(MINOR_PIECE_CORRECTION_HISTORY_LENGTH as u64);

        let static_eval = {
            let mut static_eval = self.static_evaluate();
            if let Some((saved_value, saved_node_type)) = saved {
                // Use saved value as better static evaluation
                if !Self::score_is_checkmate(saved_value)
                    && match saved_node_type {
                        NodeType::Exact => true,
                        NodeType::Beta => saved_value > static_eval,
                        NodeType::Alpha => saved_value < static_eval,
                    }
                {
                    static_eval = saved_value;
                }
            }

            self.get_correction(static_eval, pawn_index, minor_piece_index)
        };

        let improving = if move_generator.is_in_check() {
            if ply_from_root >= 2 {
                self.eval_history[ply_from_root as usize] =
                    self.eval_history[ply_from_root as usize - 2];
            }
            false
        } else {
            self.eval_history[ply_from_root as usize] = static_eval;
            ply_from_root >= 2 && static_eval > self.eval_history[ply_from_root as usize - 2]
        };

        if is_not_pv_node && !move_generator.is_in_check() {
            // Static null move pruning (also known as reverse futility pruning)
            if USE_STATIC_NULL_MOVE_PRUNING {
                let static_null_margin = if improving {
                    param!(self).improving_static_null_margin
                } else {
                    param!(self).static_null_margin
                };
                if ply_remaining < param!(self).static_null_max_depth
                    && static_eval - i32::from(ply_remaining) * static_null_margin > beta
                {
                    return static_eval;
                }
            }

            // Null move pruning
            if USE_NULL_MOVE_PRUNING
            && allow_null_move
            && ply_remaining > param!(self).nmp_min_depth

            && static_eval >= beta

            // Do not do it if we only have pawns and a king
            && move_generator.friendly_pawns().count() + 1
                != move_generator.friendly_pieces().count()
            {
                let old_state = self.make_null_move();

                let score = -self.negamax(
                    time_manager,
                    ply_remaining.saturating_sub(
                        param!(self).nmp_base_reduction
                            + ply_remaining / param!(self).nmp_ply_divisor,
                    ),
                    ply_from_root + 1,
                    false,
                    -beta,
                    -beta + 1,
                );
                self.unmake_null_move(&old_state);

                if score >= beta {
                    return score;
                }
            }
        }

        // Get legal moves and their estimated value
        let (mut move_guesses, move_count) = MoveOrderer::get_move_guesses(
            self,
            &move_generator,
            hash_move,
            if USE_KILLER_MOVE && (ply_from_root as usize) < self.killer_moves.len() {
                self.killer_moves[ply_from_root as usize]
            } else {
                EncodedMove::NONE
            },
        );

        if move_count == 0 {
            // No moves
            let score = if move_generator.is_in_check() {
                // Checkmate
                -IMMEDIATE_CHECKMATE_SCORE + EvalNumber::from(ply_from_root)
            } else {
                // Stalemate
                0
            };
            return score;
        }

        let mut node_type = NodeType::Alpha;
        let (mut best_move, mut best_score) = (EncodedMove::NONE, -EvalNumber::MAX);

        let mut quiets_evaluated: Vec<EncodedMove> = Vec::new();
        let mut captures_evaluated: Vec<EncodedMove> = Vec::new();
        let mut index = 0;
        loop {
            let encoded_move_data = unsafe {
                // SAFETY: `get_move_guesses` guarantees that `move_guesses[0..move_count]` are initialised.
                // `index` can not be higher than `move_count`, due to the loop condition.

                MoveOrderer::put_highest_guessed_move(&mut move_guesses, index, move_count)
            }
            .move_data;
            let move_data = encoded_move_data.decode();

            // This won't consider en passant
            let is_capture = move_generator.enemy_piece_bit_board().get(&move_data.to);

            let old_state = self.make_move_repetition::<true>(&move_data);
            self.node_count += 1;

            // Search deeper when in check
            let check_extension = MoveGenerator::calculate_is_in_check(&self.board);

            let mut normal_search = check_extension // Do not reduce if extending
                || is_capture // Do not reduce if it's a capture
                || index < param!(self).lmr_min_index // Do not reduce if it's not a late move
                || (ply_remaining) < param!(self).lmr_min_depth // Do not reduce if there is little depth remaining
                || !USE_LATE_MOVE_REDUCTION; // Do not reduce if turned off
            let mut score = 0;

            if !normal_search {
                // Late move reduction
                let r = {
                    let mut r = param!(self).lmr_base;
                    r += u32::from(ply_remaining) * param!(self).lmr_ply_multiplier;
                    r += (index as u32) * param!(self).lmr_index_multiplier;
                    (r / 1024) as u8
                };
                score = -self.negamax(
                    time_manager,
                    ply_remaining.saturating_sub(r),
                    ply_from_root + 1,
                    true,
                    -alpha - 1,
                    -alpha,
                );
                if score > alpha {
                    // Need to search again without reduction
                    normal_search = true;
                }
            }

            if USE_PVS && normal_search && index != 0 {
                score = -self.negamax(
                    time_manager,
                    ply_remaining - 1 + Ply::from(check_extension),
                    ply_from_root + 1,
                    true,
                    -alpha - 1,
                    -alpha,
                );
                normal_search = alpha < score && score < beta;
            }
            if normal_search {
                score = -self.negamax(
                    time_manager,
                    ply_remaining - 1 + Ply::from(check_extension),
                    ply_from_root + 1,
                    true,
                    -beta,
                    -alpha,
                );
            }

            self.unmake_move_repetition(&move_data, &old_state);

            if ply_remaining > 1 && time_manager.hard_stop_inner_search(self.node_count) {
                return 0;
            }

            if score > best_score {
                best_score = score;

                if score > alpha {
                    alpha = score;
                    best_move = encoded_move_data;

                    self.pv.update_move(ply_from_root, encoded_move_data);

                    node_type = NodeType::Exact;

                    if score >= beta {
                        fn get_capture_entry(
                            search: &mut Search,
                            from: Square,
                            to: Square,
                        ) -> &mut i16 {
                            let moving_piece = search.board.friendly_piece_at(from).unwrap();
                            let captured = search.board.enemy_piece_at(to).unwrap();
                            &mut search.capture_history[moving_piece as usize][to.usize()][if search
                                .board
                                .white_to_move
                            {
                                captured as usize - 6
                            } else {
                                captured as usize
                            }]
                        }

                        const MAX_HISTORY: i32 = 16384;
                        fn history_gravity(current_value: i16, history_bonus: i32) -> i16 {
                            (history_bonus
                                - (i32::from(current_value) * history_bonus.abs() / MAX_HISTORY))
                                as i16
                        }

                        if is_capture {
                            let history_bonus = (param!(self).capture_history_multiplier_bonus
                                * i32::from(ply_remaining)
                                - param!(self).capture_history_subtraction_bonus)
                                .min(MAX_HISTORY);
                            let entry = get_capture_entry(self, move_data.from, move_data.to);
                            *entry += history_gravity(*entry, history_bonus);
                        } else {
                            // Not a capture but still caused beta cutoff, sort this higher later

                            if (ply_from_root as usize) < self.killer_moves.len() {
                                self.killer_moves[usize::from(ply_from_root)] = encoded_move_data;
                            }

                            let history_bonus = (param!(self).quiet_history_multiplier_bonus
                                * i32::from(ply_remaining)
                                - param!(self).quiet_history_subtraction_bonus)
                                .min(MAX_HISTORY);

                            let history_side =
                                &mut self.quiet_history[usize::from(self.board.white_to_move)];

                            let history =
                                &mut history_side[encoded_move_data.without_flag() as usize];
                            *history += history_gravity(*history, history_bonus);

                            let quiet_history_malus = -(param!(self)
                                .quiet_history_multiplier_malus
                                * i32::from(ply_remaining)
                                - param!(self).quiet_history_subtraction_malus)
                                .min(MAX_HISTORY);
                            for previous_quiet in quiets_evaluated {
                                let history =
                                    &mut history_side[previous_quiet.without_flag() as usize];
                                *history += history_gravity(*history, quiet_history_malus);
                            }
                        }

                        let capture_history_malus = -(param!(self)
                            .capture_history_multiplier_malus
                            * i32::from(ply_remaining)
                            - param!(self).capture_history_subtraction_malus)
                            .min(MAX_HISTORY);
                        for previous_capture in captures_evaluated {
                            let previous_entry = get_capture_entry(
                                self,
                                previous_capture.from(),
                                previous_capture.to(),
                            );
                            *previous_entry +=
                                history_gravity(*previous_entry, capture_history_malus);
                        }

                        node_type = NodeType::Beta;
                        break;
                    }
                }
            }
            if is_capture {
                captures_evaluated.push(encoded_move_data);
            } else {
                if is_not_pv_node && !move_generator.is_in_check() {
                    if USE_FUTILITY_PRUNING
                        && ply_remaining < param!(self).futility_max_depth
                        && best_score > -CHECKMATE_SCORE // Do not prune if we might find a move to avoid getting checkmated
                        && static_eval + param!(self).futility_margin * i32::from(ply_remaining)
                            < alpha
                    {
                        // Futility pruning
                        break;
                    }
                    if quiets_evaluated.len() as u32 + 1
                        > (param!(self).lmp_base
                            + u32::from(ply_remaining) * u32::from(ply_remaining))
                            / (2 - u32::from(improving))
                    {
                        // Late move pruning
                        break;
                    }
                }
                quiets_evaluated.push(encoded_move_data);
            }

            index += 1;
            if index == move_count {
                break;
            }
        }

        if !move_generator.is_in_check() {
            let not_loud_move = {
                if best_move.is_none() {
                    true
                } else {
                    // Not promotion and not capture
                    !matches!(
                        best_move.flag(),
                        Flag::QueenPromotion
                            | Flag::RookPromotion
                            | Flag::BishopPromotion
                            | Flag::KnightPromotion
                            | Flag::EnPassant
                    ) && !move_generator.enemy_piece_bit_board().get(&best_move.to())
                }
            };

            if not_loud_move
                && match node_type {
                    NodeType::Beta => best_score > static_eval,
                    NodeType::Alpha => best_score < static_eval,
                    NodeType::Exact => true,
                }
            {
                let error = best_score - static_eval;

                Self::update_correction_history::<PAWN_CORRECTION_HISTORY_LENGTH>(
                    &mut self.pawn_correction_history,
                    ply_remaining,
                    self.board.white_to_move,
                    pawn_index,
                    error,
                    param!(self).pawn_correction_history_grain,
                );

                Self::update_correction_history::<MINOR_PIECE_CORRECTION_HISTORY_LENGTH>(
                    &mut self.minor_piece_correction_history,
                    ply_remaining,
                    self.board.white_to_move,
                    minor_piece_index,
                    error,
                    param!(self).minor_piece_correction_history_grain,
                );
            }
        }

        // Save to transposition table
        self.transposition_table[zobrist_index] = Some(NodeValue {
            zobrist_key_32: zobrist_key.lower_u32(),
            ply_remaining,
            node_type,
            value: transposition::normalise_mate_score(best_score, ply_from_root),
            transposition_move: if best_move.is_none() {
                hash_move
            } else {
                best_move
            },
        });

        best_score
    }

    /// Returns whether a score means forced checkmate.
    #[must_use]
    pub const fn score_is_checkmate(score: EvalNumber) -> bool {
        score.abs() >= CHECKMATE_SCORE
    }

    #[must_use]
    fn aspiration_search(
        &mut self,
        time_manager: &TimeManager,
        mut best_score: EvalNumber,
        depth: Ply,
    ) -> EvalNumber {
        if USE_ASPIRATION_WINDOWS && depth > 2 {
            let mut alpha = best_score
                .saturating_sub(param!(self).aspiration_window_start)
                .max(-EvalNumber::MAX);
            let mut beta = best_score.saturating_add(param!(self).aspiration_window_start);
            for _ in 0..param!(self).aspiration_window_count {
                best_score = self.negamax(time_manager, depth, 0, false, alpha, beta);
                if best_score <= alpha {
                    alpha = alpha
                        .saturating_sub(param!(self).aspiration_window_growth)
                        .max(-EvalNumber::MAX);
                    // -EvalNumber::MAX = -2147483647
                    // EvalNumber::MIN = -2147483648

                    beta = ((i64::from(alpha) + i64::from(beta)) / 2) as i32;
                } else if best_score >= beta {
                    beta = beta.saturating_add(param!(self).aspiration_window_growth);
                } else {
                    return best_score;
                }
            }
        }
        self.negamax(
            time_manager,
            depth,
            0,
            false,
            -EvalNumber::MAX,
            EvalNumber::MAX,
        )
    }

    /// Repeatedly searches the board, increasing depth by one each time. Stops when `time_manager` returns `true`.
    #[must_use]
    pub fn iterative_deepening(
        &mut self,

        time_manager: &TimeManager,

        depth_completed: &mut dyn FnMut(DepthSearchInfo),
    ) -> (Ply, EvalNumber) {
        let mut depth = 0;
        let mut previous_best_score = -EvalNumber::MAX;

        let mut best_move_stability = 0;
        let mut previous_best_move = EncodedMove::NONE;

        loop {
            depth += 1;
            let best_score = self.aspiration_search(time_manager, previous_best_score, depth);

            if time_manager.hard_stop_iterative_deepening(depth, self.node_count) {
                // Must stop now.
                break;
            }
            previous_best_score = best_score;

            if self.pv.root_best_move().is_none() || Self::score_is_checkmate(best_score) {
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
    fn get_correction(
        &self,
        evaluation: EvalNumber,
        pawn_index: u64,
        minor_piece_index: u64,
    ) -> EvalNumber {
        let pawn_correction = self.pawn_correction_history[usize::from(self.board.white_to_move)]
            [pawn_index as usize]
            / param!(self).pawn_correction_history_grain;

        let minor_piece_correction = self.minor_piece_correction_history
            [usize::from(self.board.white_to_move)][minor_piece_index as usize]
            / param!(self).minor_piece_correction_history_grain;

        let correction = ((i32::from(pawn_correction)
            * param!(self).pawn_correction_history_weight)
            + (i32::from(minor_piece_correction)
                * param!(self).minor_piece_correction_history_weight))
            / 1024;
        evaluation + correction
    }

    fn update_correction_history<const CORRECTION_HISTORY_LENGTH: usize>(
        correction_history: &mut [[i16; CORRECTION_HISTORY_LENGTH]; 2],
        ply_remaining: Ply,
        white_to_move: bool,
        index: u64,
        error: EvalNumber,
        grain: i16,
    ) {
        const CORRECTION_HISTORY_WEIGHT_SCALE: i16 = 1024;
        const CORRECTION_HISTORY_MAX: i16 = 16384;

        let mut entry = i32::from(correction_history[usize::from(white_to_move)][index as usize]);
        let scaled_error = error * i32::from(grain);
        let new_weight = i32::min(
            i32::from(ply_remaining) * i32::from(ply_remaining) + 2 * i32::from(ply_remaining) + 1,
            128,
        );
        assert!(new_weight <= i32::from(CORRECTION_HISTORY_WEIGHT_SCALE));

        entry = (entry * (i32::from(CORRECTION_HISTORY_WEIGHT_SCALE) - new_weight)
            + scaled_error * new_weight)
            / i32::from(CORRECTION_HISTORY_WEIGHT_SCALE);
        entry = i32::clamp(
            entry,
            i32::from(-CORRECTION_HISTORY_MAX),
            i32::from(CORRECTION_HISTORY_MAX),
        );

        correction_history[usize::from(white_to_move)][index as usize] = entry as i16;
    }

    #[must_use]
    pub fn hash_full(&self) -> u16 {
        const SAMPLES: usize = 10000;

        let mut count = 0;
        for entry in self.transposition_table.iter().take(SAMPLES) {
            if entry.is_some() {
                count += 1;
            }
        }
        (count * 1000 / SAMPLES as u32) as u16
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        evaluation::{Eval, eval_data::EvalNumber},
        search::{Search, transposition::megabytes_to_capacity},
    };

    #[test]
    fn quiescence_search_works() {
        let board =
            Board::from_fen("rnbqkb1r/pppp1ppp/5n2/4p2Q/4P3/8/PPPPBPPP/RNB1K1NR b KQkq - 3 3")
                .unwrap();
        let quiet =
            Board::from_fen("rnbqkb1r/pppp1ppp/8/4p2B/4P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4")
                .unwrap();
        assert_eq!(
            Search::new(
                board,
                megabytes_to_capacity(8),
                #[cfg(feature = "spsa")]
                crate::search::search_params::DEFAULT_TUNABLES,
            )
            .quiescence_search(-EvalNumber::MAX, EvalNumber::MAX),
            Eval::evaluate(&quiet)
        );
    }
}
