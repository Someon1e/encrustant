//! Universal Chess Protocol

use core::ops::Range;
use core::str::SplitWhitespace;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod go_params;
mod move_encoding;
mod search_controller;

use go_params::SearchType;
pub use move_encoding::{decode_move, encode_move};
use search_controller::SearchController;

use crate::{
    board::{Board, square::Square},
    move_generator::move_data::Flag,
    perft::perft_root,
    search::transposition::megabytes_to_capacity,
    timer::Time,
};

pub use self::go_params::GoParameters;

/// An value within a range.
pub struct SpinU16 {
    range: Range<u16>,
    default: u16,
}
impl SpinU16 {
    /// # Panics
    ///
    /// Will panic if the default value is not contained with the range.
    #[must_use]
    pub fn new(range: Range<u16>, default: u16) -> Self {
        assert!(range.contains(&default));

        Self { range, default }
    }
}

#[cfg(target_arch = "wasm32")]
type Bool = bool;

#[cfg(not(target_arch = "wasm32"))]
type Bool = Arc<AtomicBool>;

/// Handles UCI input and output.
pub struct UCIProcessor {
    /// FEN to be used.
    fen: Option<String>,

    /// Moves to be played after FEN.
    moves: Vec<(Square, Square, Flag)>,

    /// Called with UCI output.
    out: fn(&str),

    /// Range and default size of the transposition table in megabytes.
    hash_option: SpinU16,

    /// Maximum entry count of the transposition table.
    transposition_capacity: usize,

    stopped: Bool,

    ponder_info: PonderInfo,

    search_controller: Option<SearchController>,

    #[cfg(feature = "spsa")]
    pub tunables: crate::search::search_params::Tunable,
}

#[derive(Clone)]
pub struct PonderInfo {
    ponder_allowed: bool,
    is_pondering: Bool,
}

#[cfg(feature = "spsa")]
use std::ops::RangeInclusive;

#[cfg(feature = "spsa")]
struct TunableRange {
    pub iir_min_depth: RangeInclusive<u8>,
    //pub iir_depth_reduction: RangeInclusive<u8>,
    pub futility_margin: RangeInclusive<i32>,

    pub static_null_margin: RangeInclusive<i32>,
    pub improving_static_null_margin: RangeInclusive<i32>,
    //pub static_null_min_depth: RangeInclusive<u8>,
    pub lmr_min_index: RangeInclusive<usize>,
    pub lmr_min_depth: RangeInclusive<u8>,
    pub lmr_base: RangeInclusive<u32>,
    pub lmr_ply_multiplier: RangeInclusive<u32>,
    pub lmr_index_multiplier: RangeInclusive<u32>,

    //pub lmp_base: RangeInclusive<u32>,

    //pub nmp_min_depth: RangeInclusive<u8>,
    //pub nmp_base_reduction: RangeInclusive<u8>,
    pub nmp_ply_divisor: RangeInclusive<u8>,
    pub aspiration_window_start: RangeInclusive<i32>,
    pub aspiration_window_growth: RangeInclusive<i32>,
    pub aspiration_window_count: RangeInclusive<u32>,

    pub pawn_correction_history_grain: RangeInclusive<i16>,
    pub pawn_correction_history_weight: RangeInclusive<i32>,

    pub minor_piece_correction_history_grain: RangeInclusive<i16>,
    pub minor_piece_correction_history_weight: RangeInclusive<i32>,

    pub quiet_history_multiplier_bonus: RangeInclusive<i32>,
    pub quiet_history_subtraction_bonus: RangeInclusive<i32>,
    pub quiet_history_multiplier_malus: RangeInclusive<i32>,
    pub quiet_history_subtraction_malus: RangeInclusive<i32>,
    pub history_decay: RangeInclusive<i16>,

    pub capture_history_multiplier_bonus: RangeInclusive<i32>,
    pub capture_history_subtraction_bonus: RangeInclusive<i32>,
    pub capture_history_multiplier_malus: RangeInclusive<i32>,
    pub capture_history_subtraction_malus: RangeInclusive<i32>,

    pub best_move_stability_multiplier_0: RangeInclusive<u64>,
    pub best_move_stability_multiplier_1: RangeInclusive<u64>,
    pub best_move_stability_multiplier_2: RangeInclusive<u64>,
    pub best_move_stability_multiplier_3: RangeInclusive<u64>,
    pub best_move_stability_multiplier_4: RangeInclusive<u64>,
    pub best_move_stability_multiplier_5: RangeInclusive<u64>,
    pub best_move_stability_multiplier_6: RangeInclusive<u64>,
    pub best_move_stability_multiplier_7: RangeInclusive<u64>,

    pub hard_time_divisor: RangeInclusive<u64>,
    pub soft_time_divisor: RangeInclusive<u64>,
}

#[cfg(feature = "spsa")]
const TUNABLE_RANGES: TunableRange = TunableRange {
    history_decay: 4..=18,
    iir_min_depth: 3..=7,
    //iir_depth_reduction: 0..=3,
    futility_margin: 70..=170,
    static_null_margin: 25..=100,
    //static_null_min_depth: 3..=9,
    lmr_min_index: 3..=8,
    lmr_min_depth: 2..=5,
    lmr_base: 1000..=3000,
    lmr_ply_multiplier: 70..=200,
    lmr_index_multiplier: 70..=200,

    //lmp_base: 2..=5,
    //nmp_min_depth: 1..=5,
    //nmp_base_reduction: 1..=6,
    nmp_ply_divisor: 2..=7,
    aspiration_window_start: 10..=60,
    aspiration_window_growth: 20..=70,
    aspiration_window_count: 2..=8,

    improving_static_null_margin: 25..=80,

    pawn_correction_history_grain: 130..=400,
    pawn_correction_history_weight: 700..=1500,

    minor_piece_correction_history_grain: 130..=400,
    minor_piece_correction_history_weight: 700..=1500,

    quiet_history_multiplier_bonus: 140..=550,
    quiet_history_subtraction_bonus: 70..=300,
    quiet_history_multiplier_malus: 120..=490,
    quiet_history_subtraction_malus: 70..=260,

    capture_history_multiplier_bonus: 140..=550,
    capture_history_subtraction_bonus: 70..=300,
    capture_history_multiplier_malus: 120..=490,
    capture_history_subtraction_malus: 70..=260,

    best_move_stability_multiplier_0: 75..=300,
    best_move_stability_multiplier_1: 65..=260,
    best_move_stability_multiplier_2: 60..=240,
    best_move_stability_multiplier_3: 55..=220,
    best_move_stability_multiplier_4: 50..=200,
    best_move_stability_multiplier_5: 47..=190,
    best_move_stability_multiplier_6: 45..=180,
    best_move_stability_multiplier_7: 42..=170,

    hard_time_divisor: 3..=11,
    soft_time_divisor: 13..=46,
};

impl UCIProcessor {
    pub fn new(out: fn(&str), hash_option: SpinU16) -> Self {
        let megabytes = hash_option.default as usize;
        let transposition_capacity = megabytes_to_capacity(megabytes);

        Self {
            fen: None,
            moves: Vec::new(),
            out,

            #[cfg(not(target_arch = "wasm32"))]
            stopped: Arc::new(AtomicBool::new(false)),

            #[cfg(target_arch = "wasm32")]
            stopped: false,

            hash_option,
            ponder_info: PonderInfo {
                ponder_allowed: false,

                #[cfg(not(target_arch = "wasm32"))]
                is_pondering: Arc::new(AtomicBool::new(false)),

                #[cfg(target_arch = "wasm32")]
                is_pondering: false,
            },
            transposition_capacity,
            search_controller: None,
            #[cfg(feature = "spsa")]
            tunables: crate::search::search_params::DEFAULT_TUNABLES,
        }
    }
    fn set_transposition_capacity(&mut self, transposition_capacity: usize) {
        self.transposition_capacity = transposition_capacity;
        if let Some(search_controller) = &mut self.search_controller {
            search_controller.set_transposition_capacity(transposition_capacity);
        }
    }
}

impl UCIProcessor {
    /// Outputs `id` command, `option` commands, and `uciok`
    pub fn uci(&self) {
        let min_hash = self.hash_option.range.start;
        let default_hash = self.hash_option.default;
        let max_hash = self.hash_option.range.end - 1;
        let mut options = format!(
            "option name Hash type spin default {default_hash} min {min_hash} max {max_hash}
option name Ponder type check default false
option name Threads type spin default 1 min 1 max 1"
        );

        #[cfg(feature = "spsa")]
        {
            macro_rules! spin {
                ($name:expr, $default:expr, $min:expr, $max:expr) => {
                    options.push_str(&format!(
                        "\noption name {} type spin default {} min {} max {}",
                        $name, $default, $min, $max
                    ));
                };
            }
            macro_rules! define_spins {
                ($($field:ident),*) => {
                    $(
                        spin!(
                            stringify!($field),
                            crate::search::search_params::DEFAULT_TUNABLES.$field,
                            TUNABLE_RANGES.$field.start(),
                            TUNABLE_RANGES.$field.end()
                        );
                    )*
                };
            }

            define_spins!(
                history_decay,
                iir_min_depth,
                //iir_depth_reduction,
                futility_margin,
                static_null_margin,
                //static_null_min_depth,
                lmr_min_index,
                lmr_min_depth,
                lmr_base,
                lmr_ply_multiplier,
                lmr_index_multiplier,
                //lmp_base,
                //nmp_min_depth,
                //nmp_base_reduction,
                nmp_ply_divisor,
                aspiration_window_start,
                aspiration_window_growth,
                aspiration_window_count,
                improving_static_null_margin,
                pawn_correction_history_grain,
                pawn_correction_history_weight,
                minor_piece_correction_history_grain,
                minor_piece_correction_history_weight,
                quiet_history_multiplier_bonus,
                quiet_history_subtraction_bonus,
                quiet_history_multiplier_malus,
                quiet_history_subtraction_malus,
                capture_history_multiplier_bonus,
                capture_history_subtraction_bonus,
                capture_history_multiplier_malus,
                capture_history_subtraction_malus,
                best_move_stability_multiplier_0,
                best_move_stability_multiplier_1,
                best_move_stability_multiplier_2,
                best_move_stability_multiplier_3,
                best_move_stability_multiplier_4,
                best_move_stability_multiplier_5,
                best_move_stability_multiplier_6,
                best_move_stability_multiplier_7,
                hard_time_divisor,
                soft_time_divisor
            );
        }

        (self.out)(&format!(
            "id name {} {}
id author someone
{}
uciok",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            options
        ));
    }

    /// This should output `readyok`.
    pub fn isready(&self) {
        (self.out)("readyok");
    }

    pub fn setoption(&mut self, args: &str) {
        let trimmed = args.trim();

        let name_index = trimmed.find("name ").expect("Did not find name");
        let value_index = trimmed.find(" value ");

        let (name, value) = if let Some(value_index) = value_index {
            let name = &trimmed[(name_index + 5)..value_index];
            let value = &trimmed[(value_index + 7)..];
            (name, Some(value))
        } else {
            let name = &trimmed[(name_index + 5)..];
            (name, None)
        };

        macro_rules! handle_option {
            ($option_name:expr, $value:expr, $self:expr, { $($field:ident),* $(,)? }) => {
                match $option_name {
                    $(
                        #[cfg(feature = "spsa")]
                        stringify!($field) => {
                            let parsed_value = $value.expect("Missing value").parse().unwrap();
                            assert!(TUNABLE_RANGES.$field.contains(&parsed_value));
                            $self.tunables.$field = parsed_value;
                        }
                    )*
                    _ => {
                        if $value.is_none() {
                            panic!("Unknown option name (or missing value label)");
                        } else {
                            panic!("Unknown option name");
                        }
                    }
                }
            };
        }

        match name.trim().to_lowercase().as_str() {
            "hash" => {
                let megabytes = value.expect("Missing value").parse().unwrap();
                assert!(self.hash_option.range.contains(&megabytes));

                self.set_transposition_capacity(megabytes_to_capacity(megabytes.into()));
            }
            "threads" => {
                let threads: u16 = value.expect("Missing value").parse().unwrap();
                assert!(threads == 1, "Only supports single thread");
            }
            "ponder" => {
                let ponder_allowed: bool = value.expect("Missing value").parse().unwrap();
                self.ponder_info.ponder_allowed = ponder_allowed;
            }

            option_name => handle_option!(
                option_name,
                value,
                self,
                {
                    history_decay,
                    iir_min_depth,
                    //iir_depth_reduction,
                    futility_margin,
                    static_null_margin,
                    //static_null_min_depth,
                    lmr_min_index,
                    lmr_min_depth,
                    lmr_base,
                    lmr_ply_multiplier,
                    lmr_index_multiplier,
                    //lmp_base,
                    //nmp_min_depth,
                    //nmp_base_reduction,
                    nmp_ply_divisor,
                    aspiration_window_start,
                    aspiration_window_growth,
                    aspiration_window_count,
                    improving_static_null_margin,
                    pawn_correction_history_grain,
                    pawn_correction_history_weight,
                    minor_piece_correction_history_grain,
                    minor_piece_correction_history_weight,
                    quiet_history_multiplier_bonus,
                    quiet_history_subtraction_bonus,
                    quiet_history_multiplier_malus,
                    quiet_history_subtraction_malus,
                    capture_history_multiplier_bonus,
                    capture_history_subtraction_bonus,
                    capture_history_multiplier_malus,
                    capture_history_subtraction_malus,
                    best_move_stability_multiplier_0,
                    best_move_stability_multiplier_1,
                    best_move_stability_multiplier_2,
                    best_move_stability_multiplier_3,
                    best_move_stability_multiplier_4,
                    best_move_stability_multiplier_5,
                    best_move_stability_multiplier_6,
                    best_move_stability_multiplier_7,
                    hard_time_divisor,
                    soft_time_divisor
                }
            ),
        }
    }

    /// # Panics
    ///
    /// Will panic if there are invalid moves.
    pub fn position(&mut self, args: &mut SplitWhitespace) {
        self.moves.clear();

        let mut startpos = true;
        let mut building_fen = String::new();

        while let Some(label) = args.next() {
            match label {
                "startpos" => startpos = true,
                "fen" => startpos = false,
                "moves" => {
                    for uci_move in args.by_ref() {
                        let (from, to) = (&uci_move[0..2], &uci_move[2..4]);
                        let (from, to) = (
                            Square::from_notation(from).unwrap(),
                            Square::from_notation(to).unwrap(),
                        );
                        let promotion = match uci_move.chars().nth(4) {
                            None => Flag::None,
                            Some('q') => Flag::QueenPromotion,
                            Some('r') => Flag::RookPromotion,
                            Some('n') => Flag::KnightPromotion,
                            Some('b') => Flag::BishopPromotion,
                            _ => {
                                panic!("Invalid promotion notation in {uci_move}")
                            }
                        };
                        self.moves.push((from, to, promotion));
                    }
                }
                _ => {
                    if !startpos {
                        building_fen.push_str(label);
                        building_fen.push(' ');
                    }
                }
            }
        }

        self.fen = Some(if startpos {
            Board::START_POSITION_FEN.to_owned()
        } else {
            building_fen
        });
    }

    /// # Panics
    ///
    /// Will panic if there are missing parameters.
    pub fn go(&mut self, parameters: GoParameters) {
        let mut board = Board::from_fen(self.fen.as_ref().unwrap()).unwrap();

        match parameters.search_type().unwrap() {
            SearchType::Perft(depth) => {
                for (from, to, promotion) in &self.moves {
                    board.make_move(&decode_move(&board, *from, *to, *promotion));
                }

                let search_start = Time::now();
                let nodes = perft_root(&mut board, depth, self.out);
                let time = search_start.milliseconds();
                let nodes_per_second = if time == 0 { 0 } else { (nodes * 1000) / time };
                (self.out)(&format!(
                    "Searched {nodes} nodes in {time} milliseconds ({nodes_per_second} nodes per second)",
                ));
            }
            SearchType::Normal(search_time) => {
                #[cfg(target_arch = "wasm32")]
                {
                    self.stopped = false;
                    self.ponder_info.is_pondering = false;
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.stopped.store(false, Ordering::SeqCst);
                    self.ponder_info.is_pondering.store(
                        self.ponder_info.ponder_allowed && search_time.pondering().unwrap_or(false),
                        Ordering::SeqCst,
                    );
                }

                if self.search_controller.is_none() {
                    self.search_controller =
                        Some(SearchController::new(self.out, self.transposition_capacity));
                }
                let search_controller = self.search_controller.as_mut().unwrap();
                search_controller.set_position(board, self.moves.clone());

                let mated_in_plies = search_time
                    .mate_in_moves()
                    .map(|mate_in_moves| 2 * mate_in_moves - 1);

                search_controller.search(
                    self.stopped.clone(),
                    search_time,
                    self.ponder_info.clone(),
                    mated_in_plies,
                    #[cfg(feature = "spsa")]
                    self.tunables,
                );
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Stop calculating as soon as possible.
    pub fn stop(&self) {
        self.ponder_info.is_pondering.store(false, Ordering::SeqCst);
        self.stopped.store(true, Ordering::SeqCst);
    }

    /// The user has played the expected move.
    /// This will be sent if the engine was told to ponder on the same move the user has played.
    /// The engine should continue searching but switch from pondering to normal search.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn ponderhit(&self) {
        self.ponder_info.is_pondering.store(false, Ordering::SeqCst);
    }

    /// This is sent to the engine when the next search (started with "position" and "go") will be from
    /// a different game. This can be a new game the engine should play or a new game it should analyse but
    /// also the next position from a testsuite with positions only.
    pub fn ucinewgame(&mut self) {
        // New game, so old data like the transposition table will not help
        if let Some(search_controller) = &mut self.search_controller {
            search_controller.clear_cache_for_new_game();
        }
    }
}
