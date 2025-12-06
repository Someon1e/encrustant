//! Parameters used in search.

#[derive(Clone, Copy)]
pub struct Tunable {
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
    best_move_stability_multiplier_0: 176,
    best_move_stability_multiplier_1: 133,
    best_move_stability_multiplier_2: 123,
    best_move_stability_multiplier_3: 110,
    best_move_stability_multiplier_4: 107,
    best_move_stability_multiplier_5: 117,
    best_move_stability_multiplier_6: 86,
    best_move_stability_multiplier_7: 83,
    hard_time_divisor: 6,
    soft_time_divisor: 25,
};
