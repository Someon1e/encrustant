//! Best sequence of move in a position.

use super::{Ply, encoded_move::EncodedMove};

pub type PvTable = Box<[[EncodedMove; Ply::MAX as usize]; Ply::MAX as usize]>;
pub type PvLength = [Ply; Ply::MAX as usize];

#[derive(Clone)]
pub struct Pv {
    pv_table: PvTable,
    pv_length: PvLength,
}

impl Pv {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pv_table: vec![[EncodedMove::NONE; Ply::MAX as usize]; Ply::MAX as usize]
                .try_into()
                .unwrap(),
            pv_length: [0; Ply::MAX as usize],
        }
    }

    /// Returns the best sequence of moves.
    pub fn best_line(&self) -> core::iter::Take<core::slice::Iter<'_, EncodedMove>> {
        self.pv_table[0].iter().take(self.pv_length[0] as usize)
    }

    /// Returns the best move at the first ply.
    #[must_use]
    pub const fn root_best_move(&self) -> EncodedMove {
        self.pv_table[0][0]
    }

    /// Returns the best reply to the best move at the first ply.
    #[must_use]
    pub const fn root_best_reply(&self) -> EncodedMove {
        if self.pv_length[0] >= 2 {
            self.pv_table[0][1]
        } else {
            EncodedMove::NONE
        }
    }

    pub const fn set_pv_length(&mut self, ply_from_root: Ply, length: Ply) {
        self.pv_length[ply_from_root as usize] = length;
    }

    /// Store a new best move.
    pub fn update_move(&mut self, ply_from_root: Ply, encoded_move_data: EncodedMove) {
        self.pv_table[ply_from_root as usize][ply_from_root as usize] = encoded_move_data;
        for next_ply in (ply_from_root + 1)..self.pv_length[ply_from_root as usize + 1] {
            self.pv_table[ply_from_root as usize][next_ply as usize] =
                self.pv_table[ply_from_root as usize + 1][next_ply as usize];
        }
        self.pv_length[ply_from_root as usize] = self.pv_length[ply_from_root as usize + 1];
    }
}

impl Default for Pv {
    fn default() -> Self {
        Self::new()
    }
}
