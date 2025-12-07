//! Detect two fold repetition

use super::zobrist::Zobrist;

pub struct RepetitionTable {
    positions: Vec<Zobrist>,
}

impl RepetitionTable {
    pub const fn new() -> Self {
        Self {
            positions: Vec::new(),
        }
    }

    pub fn push(&mut self, zobrist_key: Zobrist) {
        self.positions.push(zobrist_key);
    }

    pub fn pop(&mut self) -> Zobrist {
        self.positions.pop().unwrap()
    }

    pub fn contains(&self, zobrist_key: Zobrist, half_move_clock: u32) -> bool {
        if half_move_clock < 4 {
            return false;
        }
        self.positions
            .iter()
            .rev()
            .take(half_move_clock as usize)
            .skip(3)
            .step_by(2)
            .any(|other| *other == zobrist_key)
    }

    pub fn clear(&mut self) {
        self.positions.clear();
    }
}
