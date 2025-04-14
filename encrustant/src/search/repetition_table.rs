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

    pub fn contains(&self, zobrist_key: Zobrist) -> bool {
        self.positions
            .iter()
            .rev()
            .any(|other_key| *other_key == zobrist_key)
    }

    pub fn clear(&mut self) {
        self.positions.clear();
    }
}
