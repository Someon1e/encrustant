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

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, square::Square},
        move_generator::move_data::{Flag, Move},
        search::{Search, transposition::megabytes_to_capacity},
    };

    #[test]
    fn repetition_table_works() {
        let board = Board::from_fen(Board::START_POSITION_FEN).unwrap();
        let mut search = Search::new(
            board,
            megabytes_to_capacity(8),
            #[cfg(feature = "spsa")]
            search_params::DEFAULT_TUNABLES,
        );
        fn is_repetition(search: &Search) -> bool {
            search.repetition_table.contains(
                search.position_zobrist_key(),
                search.board.game_state.half_move_clock,
            )
        }

        #[derive(Debug)]
        enum MoveType<'a> {
            Make(&'a str, &'a str, Flag, bool),
            MakeNull,
            UndoNull,
            UndoMove,
            UndoAll,
        }

        fn test_move_sequence(search: &mut Search, move_sequence: &[MoveType]) {
            let mut moves_and_states = Vec::new();

            for (i, move_type) in move_sequence.iter().enumerate() {
                println!("{:?}", move_type);
                match move_type {
                    MoveType::Make(from, to, flag, expected_repetition) => {
                        let move_data = Move {
                            from: Square::from_notation(from).unwrap(),
                            to: Square::from_notation(to).unwrap(),
                            flag: *flag,
                        };
                        let old_state = search.make_move_repetition::<false>(&move_data);
                        println!("{:?}", search.position_zobrist_key());

                        assert!(
                            is_repetition(search) == *expected_repetition,
                            "False {} repetition after move {}",
                            if *expected_repetition {
                                "negative"
                            } else {
                                "positive"
                            },
                            i + 1
                        );
                        moves_and_states.push((move_type, old_state));
                    }
                    MoveType::MakeNull => {
                        let old_state = search.make_null_move();
                        moves_and_states.push((&MoveType::MakeNull, old_state));
                    }
                    MoveType::UndoNull => {
                        let (_, old_state) = moves_and_states.pop().unwrap();
                        search.unmake_null_move(&old_state);
                        assert!(
                            !is_repetition(search),
                            "Unmaking a null move should not be a repetition"
                        );
                    }
                    MoveType::UndoMove => {
                        let (move_type, old_state) = moves_and_states.pop().unwrap();
                        match move_type {
                            MoveType::Make(from, to, flag, _) => {
                                let move_data = Move {
                                    from: Square::from_notation(from).unwrap(),
                                    to: Square::from_notation(to).unwrap(),
                                    flag: *flag,
                                };
                                search.unmake_move_repetition(&move_data, &old_state);
                                assert!(
                                    !is_repetition(search),
                                    "Unmaking a move should not be a repetition"
                                );
                            }
                            _ => panic!("Attempted to undo a move, but no move"),
                        }
                    }
                    MoveType::UndoAll => {
                        for (move_type, old_state) in moves_and_states.iter().rev() {
                            match move_type {
                                MoveType::Make(from, to, flag, _) => {
                                    let move_data = Move {
                                        from: Square::from_notation(from).unwrap(),
                                        to: Square::from_notation(to).unwrap(),
                                        flag: *flag,
                                    };
                                    search.unmake_move_repetition(&move_data, &old_state);
                                    assert!(
                                        !is_repetition(search),
                                        "Unmaking a move should not be a repetition"
                                    );
                                }
                                _ => break,
                            }
                        }
                        moves_and_states.clear();
                    }
                }
            }

            // Make sure all moves have been undone
            assert!(
                moves_and_states.is_empty(),
                "Not all moves were undone in the test sequence"
            );
        }

        // Test single move and unmake
        {
            let move_sequence = [
                MoveType::Make("g1", "f3", Flag::None, false),
                MoveType::UndoMove,
            ];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test single null move and unmake
        {
            let move_sequence = [MoveType::MakeNull, MoveType::UndoNull];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test sequence creating a repetition
        {
            let move_sequence = [
                MoveType::Make("g1", "f3", Flag::None, false), // White knight to f3
                MoveType::Make("g8", "f6", Flag::None, false), // Black knight to f6
                MoveType::Make("f3", "g1", Flag::None, false), // White knight back to g1
                MoveType::Make("f6", "g8", Flag::None, true), // Black knight back to g8 - should create repetition
                MoveType::UndoAll,
            ];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test null move in the middle of a sequence
        {
            let move_sequence = [
                MoveType::Make("e2", "e4", Flag::None, false), // e4
                MoveType::Make("e7", "e5", Flag::None, false), // e5
                MoveType::MakeNull,                            // null move (white)
                MoveType::UndoNull,                            // undo null
                MoveType::Make("g1", "f3", Flag::None, false), // Nf3
                MoveType::Make("b8", "c6", Flag::None, false), // Nc6
                MoveType::UndoAll,
            ];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test multiple null moves in sequence
        {
            let move_sequence = [
                MoveType::Make("d2", "d4", Flag::None, false), // d4
                MoveType::MakeNull,                            // null move (black)
                MoveType::MakeNull,                            // null move (white)
                MoveType::UndoNull,                            // undo null
                MoveType::UndoNull,                            // undo null
                MoveType::UndoMove,                            // undo d4
            ];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test repetition with null moves interspersed
        {
            let move_sequence = [
                MoveType::Make("g1", "f3", Flag::None, false), // Nf3
                MoveType::MakeNull,                            // null move (black)
                MoveType::UndoNull,                            // undo null
                MoveType::Make("g8", "f6", Flag::None, false), // Nf6
                MoveType::MakeNull,                            // null move (white)
                MoveType::UndoNull,                            // undo null
                MoveType::Make("f3", "g1", Flag::None, false), // Ng1
                MoveType::Make("f6", "g8", Flag::None, true),  // Ng8 - should be repetition
                MoveType::UndoAll,
            ];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test captures
        {
            let move_sequence = [
                MoveType::Make("e2", "e4", Flag::None, false), // e4
                MoveType::Make("d7", "d5", Flag::None, false), // d5
                MoveType::Make("e4", "d5", Flag::None, false), // exd5 (capture)
                MoveType::Make("d8", "d5", Flag::None, false), // Qxd5 (capture)
                MoveType::Make("b1", "c3", Flag::None, false), // Nc3
                MoveType::Make("d5", "d8", Flag::None, false), // Qd8
                MoveType::Make("c3", "b1", Flag::None, false), // Nb1
                MoveType::Make("d8", "d5", Flag::None, true),  // Qd5 - should be repetition
                MoveType::UndoAll,
            ];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test castling moves
        {
            let move_sequence = [
                // Setup for castling
                MoveType::Make("e2", "e4", Flag::None, false), // e4
                MoveType::Make("e7", "e5", Flag::None, false), // e5
                MoveType::Make("g1", "f3", Flag::None, false), // Nf3
                MoveType::Make("g8", "f6", Flag::None, false), // Nf6
                MoveType::Make("f1", "c4", Flag::None, false), // Bc4
                MoveType::Make("f8", "c5", Flag::None, false), // Bc5
                // Castling
                MoveType::Make("e1", "g1", Flag::Castle, false), // O-O
                // Test null move after castling
                MoveType::MakeNull,
                MoveType::UndoNull,
                MoveType::Make("e8", "g8", Flag::Castle, false), // O-O
                MoveType::UndoAll,
            ];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test en passant capture
        {
            let move_sequence = [
                MoveType::Make("e2", "e4", Flag::None, false), // e4
                MoveType::Make("a7", "a6", Flag::None, false), // a6
                MoveType::Make("e4", "e5", Flag::None, false), // e5
                MoveType::Make("d7", "d5", Flag::PawnTwoUp, false), // d5
                MoveType::Make("e5", "d6", Flag::EnPassant, false), // exd6 e.p.
                // Test null move after en passant
                MoveType::MakeNull,
                MoveType::UndoNull,
                MoveType::UndoAll,
            ];
            test_move_sequence(&mut search, &move_sequence);
        }

        // Test promotion
        {
            // Setup position where white can promote
            let promotion_board = Board::from_fen("8/P7/8/8/8/8/8/k1K5 w - - 0 1").unwrap();
            let mut promotion_search = Search::new(
                promotion_board,
                megabytes_to_capacity(8),
                #[cfg(feature = "spsa")]
                DEFAULT_TUNABLES,
            );

            let promotion_sequence = [
                MoveType::Make("a7", "a8", Flag::QueenPromotion, false), // a8=Q
                MoveType::MakeNull,
                MoveType::UndoNull,
                MoveType::UndoMove, // undo promotion
            ];
            test_move_sequence(&mut promotion_search, &promotion_sequence);
        }

        // Test nested null moves
        {
            let move_sequence = [
                MoveType::MakeNull,
                MoveType::Make("g8", "f6", Flag::None, false), // Nf6
                MoveType::Make("g1", "f3", Flag::None, false), // Nf3
                MoveType::Make("f6", "g8", Flag::None, false), // Ng8
                MoveType::MakeNull,
                MoveType::UndoNull,
                MoveType::Make("f3", "g1", Flag::None, true), // Ng1
                MoveType::UndoMove,
                MoveType::UndoMove,
                MoveType::UndoMove,
                MoveType::UndoMove,
                MoveType::UndoNull,
            ];
            test_move_sequence(&mut search, &move_sequence);
        }
    }
}
