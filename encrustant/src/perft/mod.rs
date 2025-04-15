//! Perft testing.

use crate::{board::Board, move_generator::MoveGenerator, uci};

fn perft(board: &mut Board, depth: u16) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut move_count = 0;
    MoveGenerator::new(board).generate(
        &mut |move_data| {
            if !cfg!(test) && depth == 1 {
                move_count += 1;
                return;
            }

            let old_state = board.make_move(&move_data);

            move_count += perft(board, depth - 1);
            board.unmake_move(&move_data, &old_state);
        },
        false,
    );

    move_count
}

/// Starts a perft test.
pub fn perft_root(board: &mut Board, depth: u16, log: fn(&str)) -> u64 {
    let mut move_count = 0;
    MoveGenerator::new(board).generate(
        &mut |move_data| {
            if !cfg!(test) && depth == 1 {
                log(&format!("{}: 1", uci::encode_move(move_data)));
                move_count += 1;
                return;
            }

            let old_state = board.make_move(&move_data);

            let inner = perft(board, depth - 1);
            move_count += inner;
            log(&format!("{}: {}", uci::encode_move(move_data), inner));

            board.unmake_move(&move_data, &old_state);
        },
        false,
    );
    move_count
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{board::Board, perft::perft_root, tests::TEST_FENS};

    fn debug_perft(board: &mut Board, depth: u16, expected_move_count: u64) {
        let start = Instant::now();

        let move_count = perft_root(board, depth, |out| println!("{out}"));

        let seconds_elapsed = start.elapsed().as_secs_f32();
        println!(
            "Done in {} seconds, {} nodes per second",
            seconds_elapsed,
            (move_count as f32) / seconds_elapsed
        );
        if move_count == expected_move_count {
            println!("Nodes searched: {move_count}");
        } else {
            panic!("Expected {expected_move_count} got {move_count}")
        }
    }

    #[test]
    fn test_perft() {
        let mut fens = TEST_FENS;
        fens.sort_by_key(|v| v.1);
        for (depth, expected_move_count, fen) in fens {
            let mut board = Board::from_fen(fen).unwrap();

            println!("{fen}");
            debug_perft(&mut board, depth, expected_move_count);
            println!();
        }
    }
}
