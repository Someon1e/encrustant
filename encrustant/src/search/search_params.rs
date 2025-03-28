//! Parameters used in search.

#[derive(Clone, Copy)]
pub struct Tunable {
    pub iir_min_depth: u8,
    pub iir_depth_reduction: u8,

    pub futility_margin: i32,

    pub static_null_margin: i32,
    pub improving_static_null_margin: i32,
    pub static_null_min_depth: u8,

    pub lmr_min_index: usize,
    pub lmr_min_depth: u8,
    pub lmr_base: u32,
    pub lmr_ply_multiplier: u32,
    pub lmr_index_multiplier: u32,
    pub lmr_non_pv_reduction: u32,

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
}

pub(crate) const DEFAULT_TUNABLES: Tunable = Tunable {
    iir_min_depth: 5,
    iir_depth_reduction: 1,

    static_null_min_depth: 7,

    lmp_base: 2,

    nmp_min_depth: 2,
    nmp_base_reduction: 3,
    nmp_ply_divisor: 4,

    futility_margin: 116,
    static_null_margin: 58,

    lmr_min_depth: 3,
    lmr_min_index: 6,
    lmr_base: 1548,
    lmr_ply_multiplier: 130,
    lmr_index_multiplier: 100,
    lmr_non_pv_reduction: 500,

    aspiration_window_start: 12,
    aspiration_window_growth: 40,
    aspiration_window_count: 4,

    improving_static_null_margin: 41,

    pawn_correction_history_grain: 244,
    pawn_correction_history_weight: 1024,

    minor_piece_correction_history_grain: 256,
    minor_piece_correction_history_weight: 1024,

    quiet_history_multiplier_bonus: 297,
    quiet_history_subtraction_bonus: 149,
    quiet_history_multiplier_malus: 279,
    quiet_history_subtraction_malus: 136,
    history_decay: 9,

    capture_history_multiplier_bonus: 300,
    capture_history_subtraction_bonus: 150,
    capture_history_multiplier_malus: 290,
    capture_history_subtraction_malus: 140,
};
