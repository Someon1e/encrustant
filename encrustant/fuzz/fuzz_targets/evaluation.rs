#![no_main]

use encrustant::board::Board;
use encrustant::evaluation::Eval;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(string) = std::str::from_utf8(data) {
        let board = Board::from_fen(string);
        if let Ok(board) = board {
            Eval::evaluate(&board);
        }
    }
});
