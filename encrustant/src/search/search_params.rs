//! Parameters used in search.

#[derive(Clone, Copy)]
pub struct Tunable {
    pub iir_min_depth: u8,
    pub iir_depth_reduction: u8,

    pub futility_margin: i32,
    pub futility_max_depth: u8,

    pub static_null_margin: i32,
    pub improving_static_null_margin: i32,
    pub static_null_max_depth: u8,

    pub lmr_min_index: usize,
    pub lmr_min_depth: u8,
    pub lmr_base: u32,
    pub lmr_ply_multiplier: u32,
    pub lmr_index_multiplier: u32,

    pub lmp_base: u32,

    pub nmp_min_depth: u8,
    pub nmp_base_reduction: u8,
    pub nmp_ply_divisor: u8,

    pub aspiration_window_start: i32,
    pub aspiration_window_growth: i32,
    /// Maximum number of aspiration window attempts.
    pub aspiration_window_count: u32,

    pub pawn_correction_history_grain: i16,
    pub pawn_correction_history_weight: i32,

    pub minor_piece_correction_history_grain: i16,
    pub minor_piece_correction_history_weight: i32,

    pub quiet_history_multiplier_bonus: i32,
    pub quiet_history_subtraction_bonus: i32,
    pub quiet_history_multiplier_malus: i32,
    pub quiet_history_subtraction_malus: i32,
    pub history_decay: i16,

    pub capture_history_multiplier_bonus: i32,
    pub capture_history_subtraction_bonus: i32,
    pub capture_history_multiplier_malus: i32,
    pub capture_history_subtraction_malus: i32,

    pub best_move_stability_multiplier_0: u64,
    pub best_move_stability_multiplier_1: u64,
    pub best_move_stability_multiplier_2: u64,
    pub best_move_stability_multiplier_3: u64,
    pub best_move_stability_multiplier_4: u64,
    pub best_move_stability_multiplier_5: u64,
    pub best_move_stability_multiplier_6: u64,
    pub best_move_stability_multiplier_7: u64,

    pub hard_time_divisor: u64,
    pub soft_time_divisor: u64,
}

pub(crate) const DEFAULT_TUNABLES: Tunable = Tunable {
    iir_depth_reduction: 1,

    static_null_max_depth: 7,

    lmp_base: 2,

    nmp_min_depth: 2,
    nmp_base_reduction: 3,

    futility_max_depth: 11,

    history_decay: 9,
    iir_min_depth: 5,
    futility_margin: 115,
    static_null_margin: 56,
    lmr_min_index: 6,
    lmr_min_depth: 3,
    lmr_base: 1909,
    lmr_ply_multiplier: 135,
    lmr_index_multiplier: 94,
    nmp_ply_divisor: 4,
    aspiration_window_start: 12,
    aspiration_window_growth: 41,
    aspiration_window_count: 5,
    improving_static_null_margin: 39,
    pawn_correction_history_grain: 248,
    pawn_correction_history_weight: 1129,
    minor_piece_correction_history_grain: 256,
    minor_piece_correction_history_weight: 1093,
    quiet_history_multiplier_bonus: 294,
    quiet_history_subtraction_bonus: 151,
    quiet_history_multiplier_malus: 271,
    quiet_history_subtraction_malus: 129,
    capture_history_multiplier_bonus: 288,
    capture_history_subtraction_bonus: 144,
    capture_history_multiplier_malus: 291,
    capture_history_subtraction_malus: 139,
    best_move_stability_multiplier_0: 161,
    best_move_stability_multiplier_1: 133,
    best_move_stability_multiplier_2: 126,
    best_move_stability_multiplier_3: 103,
    best_move_stability_multiplier_4: 104,
    best_move_stability_multiplier_5: 105,
    best_move_stability_multiplier_6: 93,
    best_move_stability_multiplier_7: 71,
    hard_time_divisor: 6,
    soft_time_divisor: 24,
};
