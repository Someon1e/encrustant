#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::io::{Write, stdout};
use std::{
    env,
    io::stdin,
    sync::{Arc, atomic::AtomicBool},
};

use core::cell::RefCell;
use encrustant::{
    board::Board,
    search::{Search, time_manager::TimeManager, transposition::megabytes_to_capacity},
    timer::Time,
    uci::{GoParameters, SpinU16, UCIProcessor},
};

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn print_string(output: *const u8, length: u32);
}

pub fn out(output: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        print_string(output.as_ptr(), output.len() as u32)
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("{output}");
        stdout().flush().unwrap();
    }
}

thread_local! {
    static UCI_PROCESSOR: RefCell<UCIProcessor> = RefCell::new(UCIProcessor::new(
        |output: &str| {
            out(output);
        },

        SpinU16::new(8..8193, 32),
    ));

    #[cfg(target_arch = "wasm32")]
    static INPUT: RefCell<String> = RefCell::new(String::new())
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "wasm32")]
pub extern "C" fn send_input(input: u8) {
    let character = input as char;
    if character == '\n' {
        INPUT.with(|input| {
            process_input(&input.borrow());
            input.borrow_mut().clear()
        })
    } else {
        INPUT.with(|input| input.borrow_mut().push(character))
    }
}

fn bench() {
    /// 512 randomly chosen positions and depths from lichess-big3-resolved
    #[rustfmt::skip]
    const SEARCH_POSITIONS: [(&str, u8); 512] = [
        ("r4rk1/8/p1n1p3/1pp1p3/4PpP1/2PP1P2/P5P1/R4RK1 b - - 0 4", 8),
        ("2kr3r/ppp2ppp/1Bp1b3/8/2P5/6PP/P1P3P1/1R2n1K1 w - - 0 1", 13),
        ("r4r2/p1pk2b1/1p1p3n/n3p1Bp/4N3/N1PQ1pPP/qP6/2KRR3 w - - 0 1", 1),
        ("q3b1k1/6p1/1p1p1p1p/p7/P2B4/1PP3PP/3RPP1K/R7 b - - 0 3", 1),
        ("r2q3r/pp3kp1/2p1p3/8/3n1b2/PP5P/R2N2P1/2BK3R w - - 0 1", 3),
        ("8/1p1N2n1/3K4/p3r3/5P2/1kP5/8/8 b - - 0 1", 1),
        ("8/8/4r1k1/p6p/2P4P/P5q1/8/7K w - - 0 1", 8),
        ("3q1nk1/2p3bp/pp1p2pn/3P4/P1P1QP2/2N3PP/1P4B1/R5K1 w - - 0 1", 2),
        ("3b2k1/p4ppp/2p1p3/2p5/4PB2/2P3P1/PP3PKP/8 b - - 0 1", 8),
        ("1k1rr3/1p3p1p/1ppP2p1/4n3/4N3/1P5P/PBP3BP/R2K4 w - - 0 2", 8),
        ("r4r2/p7/6k1/2R3p1/1p3nB1/2pP1P2/P1K5/3R4 b - - 0 1", 2),
        ("6k1/1pr1b1p1/p3p3/1q1nR2p/8/1N4P1/PPPQ3P/1K2R3 b - - 0 4", 4),
        ("r2r2k1/1p2bppp/p1n2n2/8/P1B3b1/5N2/RP3PPP/1NB2RK1 b - - 0 1", 2),
        ("r1b4k/1p3p2/p2r4/2pN2Pp/2Pb4/5P2/PPP3K1/R3R3 w - - 0 1", 12),
        ("4k2r/6RP/4N3/5K2/8/8/8/8 w - - 1 1", 8),
        ("2r1r3/5k2/p2p4/NpnPp3/1P5P/2P2P2/1PB5/1K3R2 b - - 0 3", 6),
        ("2bq1r2/1r2ppkp/p2p2p1/1npP4/R1P1P3/5N2/1P2BPPP/2Q1R1K1 b - - 0 2", 5),
        ("r2q2k1/pppb1ppp/3p4/4r3/5Q2/P1PB4/2P2PPP/2KR3R w - - 0 1", 6),
        ("8/4QBk1/4P2p/6pP/8/8/5q2/1K6 b - - 0 1", 3),
        ("2k5/1pp4p/p3p1p1/4qr2/2R5/1P2P2P/P3Q1P1/7K w - - 0 1", 3),
        ("2r5/7p/4kp2/p5p1/2PK4/5NPP/P4P2/8 b - - 0 1", 2),
        ("8/5k1K/5p2/1p2p2N/1pp1P3/7P/1P3P2/8 b - - 0 1", 1),
        ("8/3Q4/6k1/2P1qp2/2p2r2/r7/7K/8 w - - 0 1", 10),
        ("3k3r/ppp3p1/1bn2r2/3p4/3P1P2/2P2BK1/PP5P/R1BQ1R2 b - - 0 1", 10),
        ("8/3k4/8/pp2P2p/2n3pK/2B5/2P3bP/4r3 b - - 0 1", 10),
        ("q1rr2k1/5p1p/8/1pB4Q/1P1P1P2/8/7P/4R1K1 b - - 0 3", 7),
        ("3r4/2R2pkp/6p1/pp6/2Rp4/1P1PrP2/P4KPP/8 w - - 0 1", 3),
        ("r1rb2k1/1bqn4/p2p1p2/1p1Ppp2/4P2P/2PQ4/5PPN/R1B1R1K1 w - - 0 1", 8),
        ("8/2N3p1/3K2k1/5p1p/4pP2/6P1/7P/8 b - - 0 1", 6),
        ("8/8/8/3k4/5R2/6r1/5KP1/8 b - - 0 1", 5),
        ("1k5r/8/p1P2p1q/4bPn1/4p3/8/1PR4P/3B1RQK b - - 0 1", 10),
        ("r4rk1/p1q1bppp/b1p1p3/3pP3/3P4/2N2N2/PP1Q1PPP/R3K2R w KQ - 0 1", 11),
        ("7r/3rb1pp/k1pp4/4p3/5p2/5q2/P6K/R1R5 w - - 0 1", 5),
        ("2kq1rnr/2p1p1b1/ppb1Pp2/3p2pp/PP1P4/B1P1RNPP/5P2/RN1Q2K1 b - - 0 1", 1),
        ("rn1q1rk1/pp2bpp1/2p1bn1p/3p4/2PPp3/1PN3P1/P1N1PPBP/R1BQ1RK1 w - - 0 1", 9),
        ("3r1b1r/1pk3p1/1pp2pP1/4pP1p/4P2P/1P1B2N1/b1PK4/3R3R w - - 0 1", 7),
        ("4k3/7Q/5p2/1N6/2q1P3/2P1b1P1/6P1/4K3 w - - 0 1", 7),
        ("r1b2n1k/p6p/5qpB/2PN4/3P4/6P1/P3B1K1/1R1Q2R1 b - - 0 3", 2),
        ("8/k1K2p2/4p3/4P2p/P5pP/6P1/5P2/8 w - - 0 1", 9),
        ("3r1rk1/pp3pp1/1q2pbp1/2p5/1nPP3P/6P1/PP2QPB1/R1B1R1K1 w - - 0 1", 3),
        ("r2q1rk1/pp3p1p/3p2p1/2pP1b2/4PQ1P/2P5/P2N2PR/R1B2K2 b - - 0 1", 10),
        ("r3k2r/p1b3p1/2p5/2pp4/6Pp/2N1P1qP/PPPBQ3/R4K2 w kq - 0 1", 8),
        ("rn2k1r1/pp2npB1/2ppq3/1B5p/4PP2/5QP1/PPP4P/2KR3R w q - 0 1", 2),
        ("1R6/4pk1p/3pr1p1/6P1/5K1P/8/8/8 b - - 0 1", 13),
        ("1rr3k1/5ppp/b7/p2qPQ2/P2P4/5N2/6PP/R3R1K1 b - - 0 1", 10),
        ("7k/3r4/2p1p1R1/1pnp4/8/4PRP1/2P4P/r4B1K b - - 0 1", 12),
        ("r1bq1rk1/1pp2pp1/3pp2p/4n2P/p1P1P2N/PnN3Q1/1P3PP1/R1B1KB1R w KQ - 0 1", 9),
        ("5k2/5ppp/2P5/8/3p1P2/6qb/P2N2B1/6K1 w - - 0 1", 1),
        ("3R4/k2P4/p7/P1p1pBp1/2P3P1/7p/7r/5K2 w - - 0 1", 13),
        ("8/1ppn2k1/2n2ppp/1p2p3/1PN1P1N1/2P2P2/6PP/6K1 w - - 0 1", 10),
        ("r4bk1/3r2n1/1p1pq2p/1P2p1p1/P1N1P3/6PP/2QR2PN/3R2K1 b - - 0 1", 9),
        ("3q1rk1/r2b1pbp/3p2p1/1Pp5/1pB5/3P1N2/1P3PPP/1R1Q1RK1 w - - 0 1", 3),
        ("1r3rk1/p2p2p1/bqp1p3/2b2p2/2P5/5BP1/PPQBPPK1/5R1R w - - 0 1", 4),
        ("r2q1rk1/pp1nbpp1/4pn1p/1Bp5/8/1P3N1P/PBPPQ1P1/R4RK1 b - - 0 1", 2),
        ("8/2P5/b7/P1K3k1/7p/8/8/8 w - - 0 2", 2),
        ("8/7p/5pk1/8/5p2/PR6/r5PK/8 b - - 0 1", 2),
        ("3b2nr/8/1n1k1p2/4p1pp/P3P3/3N1N2/1P3PPP/R1B3K1 w - - 1 1", 3),
        ("3r3k/p2r3p/3N1p2/7P/3R4/8/bPP5/2KR4 w - - 0 1", 8),
        ("r3r1k1/p1Q2ppp/8/3q2b1/8/3B4/1P3P1P/1K1R3R w - - 0 4", 1),
        ("8/8/8/5p1p/1Q6/1Pp2pk1/K1P5/4q3 b - - 0 1", 7),
        ("1brq2k1/p4ppp/4rn2/2p1p3/3p4/Pp1P2P1/RB2PPBP/Q3R1K1 w - - 0 5", 7),
        ("8/1kp4b/p1p4B/8/4p2p/P1R4P/1P2KPP1/3r4 b - - 0 1", 8),
        ("r1bq1rk1/pp1n1ppn/3p3p/3Pp2N/1PP1P1P1/2N2P2/6BP/R1bQ1RK1 b - - 0 2", 11),
        ("4k3/5p1r/p3p3/2QpPp1p/2p5/P1P2P1P/3B1P2/5K2 b - - 0 1", 5),
        ("8/p6r/6N1/6p1/6k1/8/PP6/1R4K1 w - - 0 1", 9),
        ("rnbk4/3n2Q1/p2qp1B1/1p1p4/2pP4/P1P1P3/1P4PP/R1B2NK1 b - - 0 1", 11),
        ("1k2r3/1b4b1/3ppp1p/1N6/p3n3/6PB/PK5P/4RQ2 w - - 0 1", 3),
        ("rnb2rk1/pp2b1pp/4p3/4B3/3p4/8/PPP1NPPP/2KR1R2 b - - 0 2", 1),
        ("r2q3r/1ppk1ppp/p3p1b1/3pP3/3P4/B1P1P3/P1P1Q1PP/R4RK1 w - - 0 1", 1),
        ("4r2k/1b4q1/1b5p/3p3p/p1pN1P2/4P3/PP3QP1/3RR1K1 w - - 0 1", 3),
        ("r2q4/pp3rbk/2p3p1/3p3n/3P2Q1/P1PB3P/2PB2P1/R5K1 b - - 0 1", 9),
        ("5rk1/1q1b2bp/R2p4/1npPpP2/1p6/3BB2P/1PP1QPK1/5N2 w - - 0 1", 12),
        ("3r1rk1/1pp2ppp/pbpq1n2/4p3/1P2P1b1/2PP1N2/PN1B1PPP/1R1Q1RK1 b - - 0 1", 2),
        ("2N2k2/R5pp/4pp2/p1pp4/8/3P4/P1P2PPP/6K1 b - - 0 1", 1),
        ("4r1k1/1p5p/8/2r5/4p3/6P1/5PK1/R3R3 w - - 0 1", 12),
        ("1rn1kb1r/1p4pB/p2q1p2/2pPp3/2P5/1P3N1P/P1Q2PP1/R4RK1 b - - 0 1", 3),
        ("r1bqk2r/pp4p1/2p2p1p/8/2PBn3/5N2/P1P2PPP/R2QK2R b KQkq - 0 2", 12),
        ("8/8/1p3k2/1Pp5/2Kp4/3P4/8/8 w - - 0 1", 10),
        ("2R5/4r1p1/3k4/8/1p1n1N1P/4p3/6P1/3K4 w - - 0 1", 6),
        ("4r1kr/6pp/1R6/5pN1/P1P2n1B/8/3N1P2/5K2 w - - 1 1", 8),
        ("8/p7/1p2R3/6kp/3P1p2/2P4q/PP3R2/4K3 w - - 0 3", 10),
        ("r1q1k2r/1p1bnnbp/3p1pp1/pPp1pP2/2P1P1P1/2NP1N1P/P2B4/R2QKB1R b KQkq - 0 1", 12),
        ("r5k1/p4qbp/4pn2/nP2p1p1/P2pP3/3P2P1/1B2NPKP/R2QR3 b - - 0 1", 11),
        ("r1bq1rk1/pp1n1pp1/4p2p/3pN3/Pb1P1B2/2N5/1PP2PPP/R2QR1K1 w - - 0 1", 4),
        ("2Q5/8/5p2/1p2bPq1/3pP3/1k4P1/6K1/5N2 w - - 0 1", 10),
        ("8/4n3/K7/3bp3/8/3P2k1/8/2q5 b - - 0 1", 10),
        ("8/8/6B1/p3b3/1p6/1P4p1/P1Pk2K1/8 b - - 0 1", 4),
        ("rn2k1nr/pp3p1p/2Pp2p1/8/5P2/2P1BP2/q1PQ3P/2KR1B1R b kq - 0 2", 9),
        ("4r2r/pb1qp1k1/3p2p1/1p1n1pB1/3PB3/7P/PP3QP1/R4RK1 b - - 0 1", 9),
        ("8/8/5p2/r1p2p2/4pP2/1k2r3/6RP/2R2K2 b - - 0 1", 7),
        ("r3r1k1/pppq1ppp/3b1n2/3P4/3P4/1PN3P1/PB1Q1PKP/R4R2 b - - 0 1", 13),
        ("q2r2n1/6bk/pQ1P2pp/1P6/2n1P3/B5PB/P6P/3R1RK1 w - - 0 4", 1),
        ("r2q1rk1/pp1nbppp/2pp1nb1/8/3NP1P1/2N2P2/PPPQ1BBP/R4RK1 b - - 0 1", 11),
        ("r1b2rk1/pp4bp/4p1p1/2Bp1n2/5P2/2N5/PPP3PP/2KR1B1R b - - 0 1", 3),
        ("3r1rk1/1p1bqppp/2pp4/p1n1n3/P3PP2/2N3BP/1PP3P1/R2QRBK1 b - - 0 1", 9),
        ("r1b5/pp1p3k/4p3/2p1b3/8/2PP4/PP1NB3/R3K1N1 w - - 0 4", 9),
        ("1r3nk1/1q2rpp1/R1p4p/2PpPB1P/3P4/1pN2N2/1P1K4/R7 b - - 0 1", 4),
        ("3b2k1/p4pp1/1pp1p2p/4P3/2P1N3/5N2/PP3PPP/6K1 w - - 0 1", 12),
        ("r1bq1rk1/1p2bppp/p7/3np3/6P1/2P2P1N/P1Q2B1P/R3KB1R b KQ - 0 1", 6),
        ("2rr2n1/1Q4pk/p2b2qp/P1pP1p2/2P1p3/R3B1PP/1P2BP2/2R3K1 b - - 0 1", 13),
        ("8/1p6/1K3n1k/5P2/3N2pp/8/8/8 b - - 0 1", 2),
        ("5rk1/Qp2prb1/2p5/5p2/6qP/2PB4/1P2N3/1K6 b - - 0 1", 3),
        ("2r2rk1/pp1bnpbp/q2pp1p1/8/1B1pPP2/PP1P4/B1P1N1PP/2RQ1R1K b - - 0 1", 7),
        ("8/k6p/pp6/4N3/8/P7/1PP3PP/1K6 w - - 0 1", 6),
        ("8/p1bn4/1p2k3/2p1pp1p/2P4P/1NB2KP1/P4P2/8 w - - 0 1", 4),
        ("2b1r1k1/1p1n1pp1/2p2n1p/r3p3/4N1P1/p2B4/1PP1QP1P/1K1R3R w - - 0 1", 9),
        ("b7/5p2/1P1N1kpp/8/3r4/P1R4P/5PP1/6K1 w - - 0 1", 6),
        ("6k1/6pp/8/p1p2b2/2r5/8/6PP/4R1K1 w - - 0 1", 2),
        ("r3q1k1/pbp2rpp/1p1b1n2/4p3/4P3/1NP2QNP/PP3PP1/2KR3R w - - 0 1", 12),
        ("2r2qk1/R6p/2r1p1p1/3p2P1/2bP4/2P3P1/2Q2PB1/4R1K1 b - - 0 3", 10),
        ("r5k1/6pp/3p1n2/3P1p2/3RpPq1/2B1P1P1/6BP/6K1 b - - 0 2", 6),
        ("5r2/p1p2rkp/1p1bp1p1/6n1/8/2PB4/PP3PKP/R4R2 w - - 0 1", 12),
        ("8/8/p3bB2/P6P/5k1p/5P2/6K1/8 b - - 0 1", 2),
        ("3r2k1/p4p1p/q3p1p1/2p1b3/2P1P3/6P1/P1Q1NPKP/1R6 b - - 0 1", 8),
        ("8/8/3kp3/5p2/4R3/6KP/6P1/8 w - - 0 1", 4),
        ("8/6pk/5p1p/4nP2/5B2/6P1/2r5/5K2 w - - 0 1", 8),
        ("1k5r/pp1q1p2/2pb4/3p3p/1P6/P1N1PP2/6K1/R1B5 b - - 0 1", 12),
        ("8/8/4k3/6Rp/3K4/5bP1/8/8 b - - 0 2", 12),
        ("r1b1kb1r/pp5p/3p2n1/4p1NQ/4Pp2/2N5/RP3PPP/4K2R b K - 0 1", 3),
        ("5k2/1Q6/1K6/1PR5/8/4q3/8/8 b - - 0 1", 12),
        ("8/3k3p/p7/4Pp2/4b2P/8/4K3/8 b - - 0 2", 9),
        ("4rrk1/pppq1ppp/8/3N3Q/1P1P4/P7/2P3PP/5RK1 w - - 0 1", 2),
        ("6rk/6p1/pQ5p/5p2/3R1P2/1BB3P1/PPP1r2P/6K1 b - - 0 1", 1),
        ("r2q1rk1/1b2bpp1/p3pn1p/1p1pN3/3P1B2/1PP5/P1B2PPP/R2Q1RK1 b - - 0 1", 11),
        ("8/6k1/6p1/6R1/4N1p1/6r1/8/5K2 b - - 0 1", 5),
        ("2q3k1/1b3pp1/p1r4p/1p1R4/4RP2/6P1/PP5P/3Q2K1 b - - 0 1", 11),
        ("8/p7/kpp3r1/2p1P3/8/P4B2/1P3RPK/8 w - - 0 1", 4),
        ("8/5pk1/4p3/4n3/7p/6p1/8/6K1 w - - 0 1", 13),
        ("R3b3/5N1k/2n1pnpp/8/3P4/3BPN1P/5PP1/4K2R w K - 0 1", 3),
        ("6R1/Bpp1k1p1/3p3p/4n2b/7P/2P5/PPP3q1/2K1Qn2 b - - 0 3", 5),
        ("r2qkb1r/1p2pppp/p4nb1/2Pp2B1/8/2P2N2/PP1N1PPP/R2QR1K1 b kq - 0 1", 3),
        ("r4rk1/p4p1p/1p1bq3/1R2p3/2P1P3/2QPBBPb/P3K2P/2R5 w - - 0 1", 11),
        ("r2k3r/ppp2ppp/2n1b3/8/5n2/1P1B1P2/P1P4P/RN2K1NR w KQ - 0 1", 1),
        ("6k1/8/p2p1bp1/2p2pN1/2Pp4/P2P3P/4r1P1/5RK1 w - - 0 1", 2),
        ("1q4k1/6rp/p7/5P2/1B2p3/6RP/6PK/8 w - - 0 1", 4),
        ("r2r4/pp4p1/3kbpp1/3Nn3/4P3/3B4/PP3PPP/3R1RK1 w - - 0 1", 4),
        ("8/5p1p/p4kp1/1p1p4/1P1Pb1P1/1B2K2P/P7/8 b - - 0 1", 12),
        ("8/2n3p1/R7/2r1kp2/7P/4P1P1/5PK1/8 w - - 0 1", 3),
        ("6k1/5pb1/2r1p1pp/4P3/5P2/1P1BK3/P5PP/3R4 w - - 0 1", 11),
        ("8/8/1p1k1p1p/5B2/1P1K1P1P/8/6b1/8 w - - 0 1", 7),
        ("2QB4/p7/2p5/1k1p3p/1prP4/3N4/r5PP/5RK1 w - - 0 1", 8),
        ("7B/8/4k3/8/1N6/PP6/1K1n4/8 b - - 0 1", 10),
        ("r4rk1/1p2q1b1/2n1b1pp/p1p5/2PpPp2/PP1P3P/2NN1PB1/1R1QR1K1 w - - 0 2", 6),
        ("6k1/p7/7p/4K1p1/8/6P1/P7/8 b - - 0 1", 9),
        ("8/5pp1/7p/1k1N4/4n3/6P1/4KP1P/8 w - - 0 1", 13),
        ("5r2/1R6/p5k1/2Q2p2/PpP5/1P6/6KP/5N2 b - - 0 3", 3),
        ("r1b2rk1/2p2pp1/p4n1p/1p2p3/NP6/PP5P/3P1PP1/q1BQ1RK1 w - - 0 1", 12),
        ("8/5p2/5knR/1p2r1p1/6K1/8/6P1/8 w - - 0 1", 2),
        ("6k1/4bppp/p7/8/4Pp2/1bPR1P2/1R5P/1K3B2 b - - 0 1", 3),
        ("5b2/r4qk1/6pp/2p2p2/3p4/1P1P1NP1/2Q1PP1P/2R3K1 w - - 0 2", 2),
        ("r3kn1r/ppq2pp1/2pp1b1p/4pb2/1PP5/P2PPN2/1B2BPPP/R2Q1RK1 w kq - 0 2", 13),
        ("2kr3r/1p3pb1/4p2p/1N1p3P/P1p3n1/2P5/1P1P1P2/R1B1K2R w KQ - 0 1", 3),
        ("8/8/6QB/P3k3/4p3/3p4/3K4/8 b - - 0 1", 8),
        ("3r1n2/2r2p1k/p3P2p/1p1p4/3N4/2P1R1P1/PP6/4R2K w - - 0 1", 4),
        ("2rr2k1/ppqbbpp1/2n1pnp1/8/P1B1P2R/2B1QP2/2P1N2P/R3K3 b - - 0 2", 10),
        ("1rb1k2r/5pp1/p2pp3/1p6/3P1P2/6pq/PPP4P/R1BQR1K1 w - - 0 1", 8),
        ("rnb2rk1/1p3pp1/p3p3/3p2P1/6Q1/2N5/PPP2PP1/2K1R2R w - - 1 1", 6),
        ("8/8/2k5/8/5PK1/8/5R2/7q b - - 0 1", 5),
        ("3r2k1/2r2pp1/p1p4p/5b2/4nB2/3N3P/PPP2PP1/R3R1K1 b - - 0 1", 9),
        ("r7/3qn1k1/2p1pppp/pb2Q3/3PB3/P7/5PPP/2R1R1K1 w - - 0 1", 10),
        ("7k/pQ1b1Bpp/8/q4p2/3P4/8/P4P1P/2K4R b - - 0 1", 4),
        ("8/Q4pk1/4p3/P5p1/7p/4r3/5RK1/3r4 b - - 0 1", 8),
        ("4r1k1/pp4p1/5n1p/3pr1p1/2p3P1/P4P2/1PPB2P1/R2R2K1 b - - 0 1", 5),
        ("2r2rk1/1p1b1pp1/3Np2p/p2pPB1P/P6R/5P2/1P1Q1P2/R3K3 b - - 0 1", 4),
        ("1B6/8/2k5/pp4R1/7P/PP3K2/8/8 b - - 0 1", 8),
        ("4bbk1/7p/rp1qppp1/4p3/2PpP2N/r2P3P/1Q2BPP1/RR4K1 w - - 0 1", 4),
        ("1r6/p4kpp/2p5/3p4/P7/N1P2p2/1BKP1bq1/R4R2 w - - 0 1", 1),
        ("8/1q1k1p1p/3bpp2/1p2n3/5r2/1P4R1/P4PNP/3Q1RK1 b - - 0 1", 2),
        ("8/1p2k1pp/8/K1Pn4/1P6/1P3p2/7P/8 b - - 0 1", 7),
        ("2r2rk1/5ppp/2p1p3/ppRnP1q1/P2P4/4PQ1P/1P1B2P1/5RK1 b - - 0 1", 3),
        ("8/R2b3p/6p1/4k3/p1r3P1/5B1P/5PK1/8 b - - 0 1", 5),
        ("2r1q1k1/1p3p2/5bpB/2n4p/2PN4/pP4Q1/P4PPP/1K1R4 b - - 0 1", 5),
        ("r7/pp5k/2p2p1p/2B3p1/4R3/8/PP3PPP/R5K1 b - - 0 3", 4),
        ("3r2k1/5nbp/6p1/3q1p2/2pP4/2P3Q1/2B3PP/R4R1K w - - 0 1", 4),
        ("4k3/1p6/2p2p2/p2p4/P2P1PP1/1P6/8/1K6 w - - 0 1", 12),
        ("8/6k1/3p2P1/4p3/p1P4r/4qP1P/2Q3K1/3R4 b - - 0 1", 13),
        ("1k5r/ppp4p/6p1/1PP5/P1qPp3/4Q2P/5PP1/RR4K1 b - - 0 2", 5),
        ("1r6/1q3p1k/2p3p1/2Ppb3/p2N4/P5PP/1PRQ3K/8 b - - 0 1", 2),
        ("5rk1/5ppp/pq2p3/1p1pPn2/bNrP3P/P1P3P1/3B1P2/RQ3RK1 w - - 0 1", 13),
        ("8/R7/p4n1k/1p1r2p1/1P6/P3KP2/8/1B6 b - - 0 1", 8),
        ("r3r1k1/1bqn1pbp/p2pn1p1/1pp1p3/2P1P3/3PB1NP/PPBQ1PPN/RR4K1 b - - 0 1", 3),
        ("8/4R3/pBR3pk/2P4p/5K2/Pr5P/6P1/8 w - - 1 1", 4),
        ("5rk1/5ppp/3p4/8/PP2qP2/8/3B1KPP/1N3R2 w - - 0 3", 4),
        ("6k1/pp3bp1/2p1Nn1p/q4P2/2P1P3/1P5P/P2r1QBK/5R2 w - - 0 1", 4),
        ("3rr1k1/1p3ppp/p7/6q1/4p3/1PQ1P2P/P4PP1/R1B2RK1 b - - 0 1", 7),
        ("r6k/3R1p2/2p4B/3n3Q/1q6/6P1/1P3PK1/8 b - - 0 1", 1),
        ("1R6/5pk1/6pp/p7/n4N2/6PP/r7/6K1 b - - 0 1", 13),
        ("3r4/3P2kp/pN4p1/P1p2P2/4r3/7P/2R3K1/8 b - - 0 1", 8),
        ("1R6/5pk1/4p1p1/4P2p/1p3P1P/6P1/5K2/1r6 w - - 0 1", 10),
        ("r3kb1r/2pqn1p1/p1pp2b1/4p1Pp/4Pp1P/1PNP1N2/PBPQKP2/R6R w kq - 0 1", 3),
        ("2kr4/p2rbp1p/b1p2p1P/8/P1ppP3/1P3NP1/2PR1P2/1N1K3R b - - 0 1", 2),
        ("8/6p1/8/b4pp1/P3p1k1/2P5/2R2PP1/R5K1 b - - 0 1", 9),
        ("3rr1k1/p2q1pbp/b5p1/2PpP3/P2P4/2B1Q3/5RPP/R5K1 b - - 0 1", 7),
        ("8/1Q6/p2p4/2p1k3/P1p5/2P3R1/5P2/2K5 w - - 1 1", 10),
        ("8/5P2/6K1/8/1k6/8/8/6Q1 b - - 0 1", 8),
        ("8/6p1/4k3/p5P1/5P2/P1n2K2/8/8 w - - 0 1", 13),
        ("2rq1rk1/1p4p1/5p1p/p2Pp3/1b6/1PpQP1B1/P4PPP/R1R3K1 b - - 0 1", 11),
        ("r2q1rk1/1p3ppn/p1npb2p/4p3/4P1Pb/1NN1BP2/PPPQ4/2K1RB1R w - - 0 1", 12),
        ("rn1qk3/5p2/2p4b/1p1pP3/p2P4/P1Pb2P1/1P1N1P1P/R3R1K1 w q - 0 1", 4),
        ("N1q4k/pp1r1ppp/8/8/4Pb2/PN6/6PP/1R1Q3K w - - 0 2", 3),
        ("8/5p1p/7P/6K1/8/5k2/8/4q3 b - - 0 2", 7),
        ("2r1k3/3R1pp1/p1p4p/1p3P2/5p2/1Pn5/P1P4P/2KR4 w - - 0 1", 3),
        ("2rk1b1r/1Q2nppp/2nNp3/4P3/P4q2/5N1P/1P1P2P1/R1B1R1K1 b - - 0 1", 2),
        ("2k5/p7/1bp5/8/1P6/5r1p/5P1K/3R4 w - - 0 1", 9),
        ("8/1Q3p1k/5p2/1p3P2/p1q1PN1P/b5P1/6K1/8 w - - 0 1", 2),
        ("1r6/6k1/3p4/2pP2p1/p1P1P2p/P4RPP/2K5/8 b - - 0 1", 10),
        ("k1b2b1r/1p4pp/1B3p2/4p3/1p6/1P3P2/r5PK/8 w - - 0 1", 1),
        ("4r1k1/1q3pp1/5b1p/pN6/3P4/1Q4P1/1P3P1P/3R2K1 b - - 0 7", 6),
        ("8/5p1k/4pPpp/3pP3/1p4qP/1P6/2KQ4/8 w - - 0 1", 6),
        ("r2rn1k1/1Q3ppp/4b3/p3P3/2p1R3/P1N2NP1/5PBP/R5K1 b - - 0 3", 3),
        ("8/2p3pp/p7/6P1/2nk1K1P/8/5N2/8 w - - 0 1", 7),
        ("r3k1r1/1pp2p2/p3p3/3pP2p/P2n4/3KB1PB/1q1N1P1P/2R4R b q - 0 1", 5),
        ("r4rk1/ppp1bp1p/3q1np1/3P2Bb/2BN4/2N2P2/PP4PP/Q3R1K1 b - - 0 1", 5),
        ("r4rk1/p1q2pp1/2p2n1p/P2p4/1P1P4/2P4P/5PP1/R1BQ1RK1 w - - 0 1", 7),
        ("2r4r/1p1k1bpp/3n1p2/p7/4Pp2/5P2/PPP3PP/2KR1B1R w - - 0 1", 7),
        ("6k1/q4pp1/2p2b1p/2Qpp3/2nP4/P1B1PPPP/5K2/R7 b - - 0 1", 1),
        ("r5k1/p1Rp2pp/1p2p3/4p3/3nN3/P2r4/1P3P1P/4K3 b - - 0 1", 8),
        ("7B/4k3/2pR3K/2P5/8/4Pb1P/8/2r5 w - - 0 1", 6),
        ("8/1r6/pp6/3P4/1PP2k2/2b5/P4BK1/5R2 w - - 0 1", 2),
        ("2r5/4Qpk1/6pp/p2pPb2/q1pB4/P1P5/6PP/5RK1 w - - 0 1", 6),
        ("8/8/6pp/5p2/4pP1P/4P3/k1K2P2/8 w - - 0 1", 2),
        ("8/p1kn1p1p/3p2p1/2pP1b2/5B2/8/PPP1rPPP/R3N1K1 w - - 0 1", 9),
        ("7r/pk6/2p1r2p/P4p1P/2p2B2/8/1P3KP1/1N6 w - - 0 1", 2),
        ("8/4k3/1p1pp1p1/6P1/1p2PK1P/8/1P6/8 w - - 0 3", 12),
        ("5r2/1N2qppk/P3p1bp/3pn3/R7/1PQ2BP1/4PP1P/6K1 b - - 0 1", 1),
        ("8/8/8/p2Bb3/P1p3P1/1p2N1k1/1P5p/2K5 b - - 0 1", 6),
        ("5Q2/6pk/1p2pbp1/3B4/8/7r/1q3PPP/5RK1 b - - 0 1", 12),
        ("2kr1bnr/1pp2ppp/p7/4P3/3p2b1/2N1B3/PPP2PPP/R4RK1 w - - 0 1", 4),
        ("2kr4/1pp2pp1/p1p5/5P1p/8/1N2r3/PPP2KP1/R6R b - - 1 1", 4),
        ("r1bqk2r/1pp1bpp1/2n1p2p/2NpP3/p2P1PP1/2PB1R1Q/PP5P/R1B3K1 w q - 0 1", 12),
        ("3r4/pQ2ppkp/5bp1/2q5/8/2P1PBB1/PP1r1PPP/5RK1 w - - 0 1", 5),
        ("8/7k/4q1p1/6Pp/1Q5K/6P1/8/8 b - - 0 1", 6),
        ("1r4k1/4R3/3n3p/pp6/n4NPP/5P2/5K2/B7 w - - 0 1", 5),
        ("6k1/5bp1/7Q/p1q2P2/Br1pn3/1P5P/6PK/4R3 w - - 0 1", 11),
        ("6k1/6p1/5n1p/Q1p5/4pP1P/1q4P1/5P1K/8 b - - 0 1", 8),
        ("2r3k1/ppp1R1pp/3p2p1/8/6P1/P1K1R3/1PP4r/8 b - - 0 1", 5),
        ("7k/5p2/3p4/p2Pp1qp/1p2B3/1Pr2P1Q/P7/1K6 w - - 0 1", 8),
        ("rn4r1/2B5/5pkp/pp6/1b1p4/1q1P1QPN/5P1P/3R1K1R w - - 0 1", 12),
        ("8/2p2k2/4R3/2pN4/2P2b2/5P2/P3K3/7r b - - 0 2", 3),
        ("rnb2b1r/pp3k1p/6p1/2pN4/8/8/PP2BPPP/n1B2K1R w - - 0 1", 9),
        ("2q3k1/5pp1/p6p/1p5P/2p5/P3PBP1/1bP2P2/6K1 w - - 0 1", 8),
        ("2kr2r1/npp2p1p/p2bq3/3p1B2/3P3p/1NP1P3/PPQ2PP1/R3K1R1 b - - 0 1", 1),
        ("2kr3r/pp6/2n2n2/3p3p/4pq2/P1P2P2/1PQ1B2N/K1R4R b - - 0 1", 6),
        ("6k1/N6p/1p4p1/8/4pP2/1r5P/6P1/6K1 w - - 0 5", 9),
        ("1r2r1k1/p1p3pp/6b1/2q2pB1/2P5/7P/PP2QRP1/4R1K1 w - - 0 1", 10),
        ("2r4r/4kp1p/2pnp1p1/4Q3/3P3q/2PN4/PP3PPP/R3R1K1 w - - 0 1", 10),
        ("2r5/1pp2k1p/p1p2p2/3b4/P2B1bP1/1P5r/2PN4/2RR2K1 w - - 0 1", 2),
        ("5r2/pp1rp1kp/5pp1/2pNnP2/4P1P1/7P/PPP2R2/4R1K1 w - - 0 1", 2),
        ("8/1B5p/P4k2/8/2P3K1/8/r6P/8 w - - 0 1", 1),
        ("r2qk2r/pp1nbpp1/1n1p2b1/3Pp1Pp/2P4P/2N1BN2/PP2BP2/R2QK2R b KQkq - 0 1", 2),
        ("5k2/8/5pp1/4p2p/4P2P/4K1P1/2R2P2/8 b - - 0 1", 13),
        ("8/2p5/1r3k2/1P1P1p1p/2P4P/3K1pP1/8/8 w - - 0 1", 7),
        ("r2q1b1r/2p1k2p/4pp1n/pb1pN2Q/1p1P3B/2P5/PP3PPP/RN3RK1 w - - 0 1", 3),
        ("6k1/2p3p1/p3r2p/1p2P3/1Pq1NPp1/P2QR1P1/7P/5K2 b - - 0 1", 6),
        ("8/5p2/p6p/4Q3/4n1k1/8/5pP1/5K2 b - - 0 1", 5),
        ("r2q1r2/pb2n2k/1p2Npp1/2p1N2p/2BpPPP1/PPn4P/2P5/R3QRK1 b - - 0 1", 2),
        ("8/1P6/P5p1/8/5P2/q4R2/3K3k/8 b - - 0 1", 9),
        ("5k2/pb2bpp1/1p5p/3pP3/5N1P/1P4P1/P4PB1/6K1 b - - 0 2", 9),
        ("4Q3/8/1p6/2p5/pk3K2/8/1q6/8 w - - 0 1", 2),
        ("3r1k2/pp3q2/2nN1n1p/2P5/3P4/P2QP3/5PPP/R4RK1 b - - 0 1", 3),
        ("5Q2/8/8/8/P3kr2/7P/6K1/8 w - - 0 1", 8),
        ("r4rk1/3bppbp/p2p1np1/q2P4/4P3/1QN1BN2/PP3PPP/R4RK1 b - - 0 1", 1),
        ("5rk1/6p1/3bp3/3p1q2/1p1P1P2/2P1B1P1/1P3Q2/5R1K b - - 0 1", 10),
        ("1Q6/p4Qp1/1bp4p/1k6/2p3PP/2P5/P2r4/1K2R3 b - - 0 1", 5),
        ("R7/5k2/5p2/5p2/7P/5K2/5P2/8 b - - 0 1", 8),
        ("4rqk1/3n1p1p/B1r1b1p1/2p5/p3PQPP/8/1PP3K1/3R1R2 w - - 0 1", 9),
        ("5b2/5kpp/3p4/1P1N2P1/2B5/6r1/8/1KB4R b - - 0 1", 12),
        ("1k5r/pp2R1pp/1q2B3/4p3/8/7P/PP3PP1/3Q2K1 b - - 0 1", 5),
        ("8/8/3k1p2/p2n1Kp1/6P1/P7/8/8 w - - 0 1", 7),
        ("6k1/pp3p1n/2p1b3/4Pp1p/2P2P2/BP4P1/P2r2BP/R5K1 w - - 0 1", 13),
        ("8/ppk1r2p/5p2/2p2B2/P1N2b2/1P6/5PKP/5R2 b - - 0 1", 12),
        ("2r5/5pb1/3kp2p/p2p1p1P/P2P4/1P1K4/4BPP1/7R b - - 0 1", 8),
        ("3q1r1r/3k4/p1p4p/P1Qp1pp1/4p3/1B4PP/1PP3P1/2KR1R2 b - - 0 1", 7),
        ("r4rk1/pp1b4/2p1p2p/2Pp1qp1/3P1p2/2N1n2P/PP4P1/2K1RBR1 w - - 0 1", 7),
        ("4r3/2n2p2/2p5/p4k1p/Pp1PpPp1/6P1/1P3K1P/3R4 w - - 0 1", 6),
        ("8/p2k1p1p/6p1/2p1p3/2PbP3/6K1/P6P/8 b - - 0 2", 1),
        ("k1r4r/1p4Q1/p1p1Pn1p/3p4/5q2/1BPP3P/PP6/1K1R2R1 w - - 0 1", 6),
        ("8/2p5/2k5/2p3P1/2P5/7R/5PK1/r7 b - - 0 1", 7),
        ("r4rk1/4np1p/p2qp1p1/3p4/3P3Q/2P1RN2/P4PPP/3R2K1 b - - 0 1", 10),
        ("6k1/p1r2pp1/1pP1pb1p/3r4/P1RPQP2/2RN3P/1q4PK/8 b - - 0 1", 7),
        ("1k1r4/pppnq2p/3p1p2/4p1r1/4P3/2NP3P/PPP2PP1/R3R1K1 b - - 0 1", 4),
        ("3n4/pp3k1p/3P1pp1/3K4/8/8/PPB4P/8 b - - 0 1", 8),
        ("r2q1b2/p1nbnk2/1pp2p2/3p1Pp1/1P1N2P1/2N5/PBPPR1B1/2K1R3 b - - 0 1", 2),
        ("6k1/6p1/7p/3R1P1K/1rP5/8/1P1b3P/3N4 b - - 0 1", 9),
        ("3r4/5pk1/7p/p3p3/1p1b2P1/7P/PP3NQK/2r5 w - - 0 1", 11),
        ("5k2/2rbq1bp/B5p1/1p2pp2/1P6/P3P1P1/1B3P1P/3Q1RK1 w - - 0 1", 5),
        ("6k1/p2r2p1/B4p2/5P2/1KP2p1n/2B5/6P1/8 w - - 0 1", 11),
        ("r7/5pkp/bp1r4/p3N1p1/P7/1P5P/2P2PP1/2K4R w - - 0 1", 5),
        ("8/ppp2P2/3p1r2/4p3/4P1k1/3P4/PPP5/6K1 w - - 0 1", 6),
        ("r3k2r/pb1nppbp/1p3np1/1Bp3N1/1P2P3/2P1B3/P2N1PPP/3RK2R b K - 0 1", 2),
        ("3k4/8/3Pp3/1p2Ppp1/pPp5/P1PnR1P1/3K4/8 w - - 0 1", 8),
        ("8/p2b4/kp3p2/3P4/4B3/P4RP1/3K3P/8 b - - 0 1", 4),
        ("8/1p3ppk/p6p/8/1r3n1P/3R2K1/1P4P1/8 w - - 0 2", 7),
        ("2r1r2k/1bqnbpp1/p2ppn1p/1p5P/3NP3/P1N1BQP1/1PP2P2/1K1R1B1R b - - 0 1", 11),
        ("3rr1k1/2p2pp1/1p1p1b1p/p2P1q2/P1Pp4/1Q1P2P1/1P1B1P1P/4RRK1 w - - 0 2", 6),
        ("5r2/p4k1p/2P3p1/3p1p2/3P4/8/6PP/2R2RK1 w - - 0 2", 3),
        ("r1b2rk1/p3bppp/1p2p3/2pPq3/4N3/4P2P/PP2BPP1/R1BQ1RK1 w - - 0 2", 3),
        ("3r1rk1/ppq2pbp/1np2np1/4p3/P1P5/NP2P1PP/1B2QPB1/3R1RK1 b - - 0 1", 9),
        ("2kr4/ppp2ppp/2n5/2Pr4/6n1/3BB3/PQ3PPP/2RR2K1 w - - 0 1", 4),
        ("8/6r1/4p3/4R3/2k2P1K/8/8/8 b - - 0 1", 11),
        ("5k2/2p2q2/5p1p/2p1n1p1/2P1Q1P1/4N2P/r3PP2/3R2K1 w - - 0 1", 5),
        ("r5k1/5ppp/bQPrp3/p7/P2P4/6P1/5PBP/R1R3K1 b - - 0 3", 7),
        ("3rr1k1/1p2bppp/3p1p2/1P1N4/3Q2q1/6B1/2P2P1P/3RR1K1 b - - 0 1", 11),
        ("5k2/2p5/1p1p2p1/p1nP1b2/P3rPP1/1PR1B2p/4B2P/4K3 b - - 0 1", 3),
        ("r1b1r1k1/3n1pb1/1pqp1npp/4p3/1PP1P3/2N1B2P/3NBPP1/2RQ1RK1 w - - 0 1", 3),
        ("r1bq1r2/pp2b1k1/4p2p/3pNppn/2pP4/2P1PP1P/PPBQ1RPB/4R1K1 b - - 0 1", 8),
        ("r4rk1/1b3ppp/4pn2/p7/Pp1q4/1P1B4/2PNQPPP/R4RK1 w - - 0 1", 13),
        ("3r1r1k/pp2p2p/1n2R1pQ/2q5/2P3N1/1P1p4/P4PPP/3R2K1 b - - 0 1", 1),
        ("2kr3r/pp3p2/5p2/P7/4P1pP/B1Pp4/3P2B1/R5K1 w - - 0 2", 1),
        ("8/3n3p/2r1k1p1/5p2/1PN5/2P2PKP/4p3/R7 w - - 0 1", 3),
        ("rnb2rk1/ppp3pp/4p3/8/2PPN3/3QB2P/P1P2P1P/R3K2R b KQ - 0 1", 3),
        ("2q2k2/p4pp1/7p/2p5/7Q/1B2PP2/Pr3P1P/3R2K1 b - - 0 1", 7),
        ("rr4k1/4ppbp/q2p1np1/2pP4/P3P3/1PN1B1P1/2Q2PKP/R6R b - - 0 1", 9),
        ("8/pp3p2/2p3k1/8/5p1P/P1PKpP1P/2r5/4R3 b - - 0 1", 3),
        ("5rk1/p2q3n/1p1pp3/3P1pp1/2Q1P3/P4PP1/4N1K1/2R5 b - - 0 1", 8),
        ("r3k2r/ppp2ppp/n3p3/1b1n4/3P4/2P3P1/PP1B1PBP/R3K2R w KQkq - 0 1", 9),
        ("8/8/2K2k2/5p2/5Pnp/8/8/6N1 b - - 0 1", 9),
        ("r3kb1r/1b3pp1/pqp1pn1p/3pN3/Q2PPBP1/5P2/PP3N1P/R4RK1 w kq - 0 1", 3),
        ("2rr4/1p2p1k1/p1np1p2/4nP2/5N2/2Q4P/2P1B1P1/3R1RK1 b - - 0 1", 11),
        ("r4rk1/2p3pp/1pN1pn2/p4p2/2P5/2N3P1/PP2PPKP/R1B2R2 b - - 0 1", 7),
        ("1r2r1k1/2p3p1/5pP1/7Q/p2pq2P/P7/1PP2P2/1K1R3R w - - 0 1", 2),
        ("r3k2r/1p1b1pp1/1P2p3/2P4p/3pP3/3PqN1P/3QB3/5R1K w kq - 0 1", 3),
        ("2kr3r/1pp1bp2/p2q2p1/7p/RP6/2P2N1P/P4PP1/1QB3K1 b - - 0 2", 13),
        ("r2q1rk1/3b2pp/2p5/1p1pp3/1Q6/P3BRPP/N5B1/6K1 w - - 0 1", 2),
        ("5k2/R7/8/p7/2pKNr2/P7/8/8 b - - 0 1", 5),
        ("5q1r/1p1k3p/p3p3/3pQ3/3P4/1R6/5P2/1K5R b - - 0 3", 12),
        ("r7/2k5/7p/pRp5/2NbPP2/3P4/2K3PP/8 b - - 0 1", 11),
        ("r5k1/4r1pp/p1nBp3/1p1b4/3Pp1B1/6Q1/PP4PP/5R1K b - - 0 1", 7),
        ("3k4/2p2p2/1p5p/p3b3/P1P2p1N/1P4P1/5P1P/3b2K1 w - - 0 1", 3),
        ("6k1/8/4P3/6K1/8/p4r2/R7/8 b - - 0 1", 7),
        ("8/8/P5p1/8/1r2kp2/1PR3p1/6P1/4K3 b - - 0 1", 6),
        ("r5k1/5qp1/5n1p/p1Qp4/1p6/1P2PP2/P4P1P/3RK1R1 w - - 0 1", 11),
        ("3r1rk1/p5p1/1p1q3p/2p2p2/3b4/P1PP4/1P2Q1PP/2RN1R1K b - - 0 1", 7),
        ("b1q5/5k2/P4Pp1/1PB2pP1/N4P2/6KP/5Q2/8 b - - 0 1", 9),
        ("8/5kp1/6rp/R7/5P2/5P2/8/7K b - - 0 1", 7),
        ("r2r2k1/1p3pp1/p1pb3p/8/6q1/P1Q1B3/1P3PPP/3RR1K1 w - - 0 1", 4),
        ("8/1b5k/6pp/8/p1p4q/P1Q2P2/1P6/6K1 w - - 0 1", 13),
        ("1r3rk1/1p3ppp/3p1b2/2p2b2/8/1P1P1NPP/P4PB1/1K1R3R w - - 0 1", 8),
        ("2r3k1/1p2b1pp/1r6/p1pp1P2/6P1/2P1B2P/PP6/1R2R1K1 b - - 0 1", 9),
        ("6k1/4pr1p/2p3p1/2q1p3/6Q1/6BP/5PPK/8 b - - 0 1", 11),
        ("r4rk1/pp1n1ppp/4b3/8/2Pp4/1P2PPP1/P3K2P/3R1B1R b - - 0 1", 12),
        ("1r1q1rk1/1p1bpp1p/p2p2p1/2pPb3/4P3/3PB1P1/PP3PBP/2RQ1RK1 w - - 0 1", 5),
        ("8/6k1/7p/p1Pp4/P2P2K1/1P2p1P1/7P/8 b - - 0 1", 3),
        ("8/8/4k3/p2p2p1/B2p3P/1P3KP1/2P5/2b5 w - - 0 1", 2),
        ("8/4k3/7p/8/8/6PB/4K3/q7 b - - 0 1", 10),
        ("r7/p5k1/1p6/3b4/1P3P2/n4P1P/6BK/3r2B1 b - - 0 2", 10),
        ("4kb1r/1p2qpp1/p7/4n2p/P3R3/2P5/1P3PPP/RNB3K1 w - - 0 3", 8),
        ("2r3k1/4bp2/1r1pp2p/1p3q2/1P3Pp1/2P3P1/1Q2R1P1/5RK1 w - - 0 1", 10),
        ("5rk1/2p5/p5n1/1p1Pp1p1/1Pn1P1Pp/P1B5/B3K2P/2R5 b - - 0 1", 10),
        ("8/3R3p/2r2pk1/8/6p1/6P1/3K1P1P/8 b - - 0 1", 11),
        ("2rr2k1/4bppp/p2q1nb1/4p3/N2p1P2/1P1P4/PBPQN1PP/1K1R3R b - - 0 1", 2),
        ("5rk1/6q1/1p1r4/p6Q/5p2/P7/1P2R1P1/6K1 w - - 0 1", 2),
        ("r2q1rk1/6bp/1ppp2p1/4p3/P4p2/2NP3P/1PPQ1PP1/R4RK1 b - - 0 1", 3),
        ("4k3/p2r1pbp/1pR3p1/4p3/4P1P1/4B3/PP3P1P/5K2 b - - 0 1", 3),
        ("6k1/p1b3pp/8/1P2p3/4R3/1P5P/PBQ2PPK/3r4 b - - 0 1", 6),
        ("8/5kp1/3p2pp/2pP2P1/2P4P/8/p1K3P1/4r3 w - - 0 1", 1),
        ("k4b2/8/1Q1pR3/8/6n1/1PN5/P5P1/6K1 b - - 0 1", 2),
        ("2r2rk1/4bppp/p4nq1/P2p4/BP1Bp3/2P1Q1NP/5PP1/5RK1 b - - 0 2", 9),
        ("8/8/4rkp1/R3ppp1/6P1/5P1P/5K2/8 b - - 0 1", 13),
        ("8/8/4B1pp/1K3p2/3p3P/8/6k1/8 b - - 0 1", 8),
        ("1r4k1/5ppn/1b2p2p/8/p1P3PP/P1B5/1P1R1P2/6K1 w - - 0 1", 7),
        ("Q4b1r/1pkrqp2/4p3/3pP3/8/1Np3P1/7p/R1BQ1R1K b - - 0 1", 2),
        ("8/6pk/8/2B1p3/p7/P4P2/1K6/3q1b2 b - - 0 1", 4),
        ("r4nk1/1b3p1p/p5p1/Pp1p4/1P1Pp3/2P1N3/5RPP/3Q2K1 w - - 0 1", 12),
        ("4k3/4pp2/8/b2b3B/8/1P6/2K2B2/8 b - - 0 1", 13),
        ("6k1/1p3pp1/5n1p/p7/2PN4/1P1R3P/5PPK/4r3 b - - 0 1", 1),
        ("8/1R3pk1/6p1/3r2P1/1P6/8/6K1/8 w - - 0 1", 10),
        ("6k1/8/7p/p4p1P/P1Nr1p2/KPb5/2P5/5R2 w - - 0 1", 8),
        ("3r1rk1/1b1n2pp/p1np4/2p2p2/PpP5/1P1P1N1P/1R1NBPP1/R5K1 w - - 0 2", 11),
        ("7R/8/2k4p/1p3p1r/8/1PK3P1/5P2/8 w - - 0 1", 4),
        ("1r5k/1p5p/p2ppb2/7P/2P1BP2/4B3/n2K4/6R1 w - - 0 1", 1),
        ("r4r2/1Q1b3k/n2p1pp1/1NpP3p/2P1q3/P6P/1P2B1P1/1R3RK1 b - - 0 2", 8),
        ("8/pb1n1k1p/1p4p1/3p4/8/PP6/2rN1bBK/2B1R3 w - - 0 1", 9),
        ("2r3k1/3nbp2/6pQ/4p3/1p2q2p/1P1N1R1P/6P1/1K3R2 b - - 0 1", 4),
        ("8/6B1/p7/3k3P/4p1P1/r7/5K2/8 b - - 0 1", 9),
        ("2r1r1k1/1bp1qpbp/p1p1p1p1/Q7/3P1B2/2P2N1P/PP3PP1/4RRK1 w - - 0 1", 2),
        ("8/p7/1p1b4/5p1B/P1P4P/4k3/6K1/8 b - - 0 1", 13),
        ("r1bq1rk1/pp3ppp/4pn2/8/1b1n4/2NB3N/PP1B1PPP/R2QR1K1 b - - 0 1", 4),
        ("rnb2b2/1ppknr2/1P1p1p1p/pP5p/P2PNP2/4P3/3B2PP/R2Q1RK1 b - - 0 1", 11),
        ("8/7p/6p1/5pP1/4pP1P/1K2k3/1n6/8 b - - 0 1", 8),
        ("8/p5p1/2b2p1k/8/8/6KP/4R1P1/8 b - - 0 1", 8),
        ("3r4/p5pp/2pB1p2/1kP5/3R1P2/P7/3K1NPP/4r3 b - - 0 1", 4),
        ("4rr1k/5ppp/p7/1p1Q4/3pP1q1/PB6/1PP2P2/4RK2 b - - 0 1", 8),
        ("5rk1/4qpbp/n2N2p1/P1p1Pb2/8/4PN2/1r3PPP/R2R2K1 w - - 0 1", 1),
        ("2b5/3pR3/1ppP2p1/6k1/8/3P1K2/3r4/8 w - - 0 2", 11),
        ("2k1r3/ppp5/5p2/3p1p2/1P1P1P2/2n5/P3P1rP/2KR3R w - - 0 1", 3),
        ("8/3k4/5q1p/P3R1p1/3K4/2P4r/8/8 b - - 1 1", 12),
        ("8/5K2/5R2/8/8/1P2k1r1/8/8 b - - 0 2", 2),
        ("r2rnbk1/pp3p1p/2n3p1/3p4/2pP4/P1N4B/1PP1NPPP/R4RK1 w - - 0 1", 4),
        ("3b3k/p2n2rp/8/1p2PpPq/3R3P/B1P4Q/PP6/6RK b - - 0 1", 12),
        ("6k1/Q4p1p/1p4p1/2pp4/5q2/2P3RP/P1P2PPK/1r6 w - - 0 1", 5),
        ("1rb1nrk1/1p4bp/p2p4/2pPnp2/P6q/N1N1BP2/1P1QB1PP/1R3R1K b - - 0 1", 13),
        ("1rr3k1/p3bp2/6pP/4n3/7P/2N1P3/PPP1K3/R1BR4 w - - 0 2", 12),
        ("7r/7P/N1p3B1/1p6/bP4k1/P2PP3/6P1/6K1 b - - 0 1", 3),
        ("2k4r/ppp1qp2/7r/2n5/6nP/P3PNP1/5PB1/RNBQ1RK1 w - - 0 1", 9),
        ("3r2k1/pb1prp2/1p2pp1p/2n5/P1P1P3/2P2PP1/3N3P/R3KB1R b KQ - 0 1", 9),
        ("8/p6p/B1p1b1p1/2kp4/P3p1N1/1P5P/1bPK2P1/8 b - - 0 1", 2),
        ("3rr1k1/ppp2ppp/3b4/8/3P4/Pb2BN2/1P2BPPP/2R1K2R w K - 0 1", 3),
        ("r2q1rk1/pp1b1pbp/3p1np1/2p1p3/3nPP1P/2NP2P1/PPPQ1NB1/R1B2RK1 b - - 0 1", 11),
        ("8/2p5/6k1/2K3p1/6Rp/4P3/5P2/8 b - - 0 1", 9),
        ("r2q1rk1/4bppp/p3pn2/1p4B1/2pP4/P1P2N2/1P2QPPP/1R3RK1 b - - 0 1", 6),
        ("rnb1r1k1/pp2qnb1/2p2p1p/3p4/2PP3P/3B1PB1/PPQNN3/2R1K2R b K - 0 2", 13),
        ("4r1k1/pp2bppp/5p2/3q4/1P1P4/P3QP1P/2b3P1/R3K2R w KQ - 0 3", 5),
        ("8/R5pp/4k2b/3pP3/P2P4/5RPK/3r3P/8 w - - 0 1", 3),
        ("r5r1/p2kbQ1B/2p1p3/8/3P1B2/2P5/q3NPPP/5RK1 b - - 0 1", 10),
        ("4r3/nQ3ppk/p3pnp1/3p4/P1pP4/2b1PPP1/2RB1K1P/2R5 b - - 0 1", 6),
        ("2R5/8/4pk2/5p2/2P3p1/2r5/6K1/8 w - - 0 1", 3),
        ("8/2N5/5k1p/p1p5/P1p5/2Pbr3/1P4RK/8 w - - 0 1", 9),
        ("3k2q1/pp5R/3p1br1/2rP3Q/8/2P2N2/PPK2P2/8 w - - 0 1", 7),
        ("8/p4nk1/3p4/3P2p1/R7/P5P1/2B4P/3r1RK1 b - - 0 1", 4),
        ("2kr3r/2p2p2/bp1p1R2/p2Pp3/4P3/P1N1K1P1/1PP4P/R7 w - - 0 1", 1),
        ("1R6/Q2pn2p/b1p2k2/8/2qN4/4P3/5PPP/4KR2 w - - 0 1", 2),
        ("8/8/5pr1/4n3/8/7K/3Q2P1/k7 w - - 0 1", 11),
        ("R7/7Q/1p1p1k2/2r5/2q4P/8/5PP1/6K1 b - - 0 1", 2),
        ("5Q2/2p1P3/3n4/8/2kB1K2/2P5/R7/8 b - - 0 1", 4),
        ("r3r1k1/p4pb1/Q1pq3p/2Np3P/1P1Bp1p1/2P2bP1/P3RP2/4R1K1 b - - 0 1", 11),
        ("r1b2rk1/1pq2ppp/1n1p1n2/2p1p3/2P2P2/P3P1P1/1BQPN1BP/R4RK1 b - - 0 1", 13),
        ("Q7/4r3/R7/5k2/7P/1P3K2/2rp4/3R4 b - - 0 1", 5),
        ("3r2nr/2kp4/2pq3p/4p1p1/2P5/4P2P/PP2BPP1/R4RK1 w - - 0 1", 9),
        ("6rk/1R4bp/3pQP2/p3p3/n7/4q1P1/7P/1R5K b - - 1 1", 10),
        ("3r2k1/1pp3b1/p1n2ppp/4p3/4P1P1/2PrB3/PP2NPP1/R3RK2 w - - 0 1", 8),
        ("r3r1k1/2b1Bp1p/p1Rn2p1/8/pP1P4/P4N1P/5PP1/4R1K1 b - - 0 1", 12),
        ("1Q6/4pkq1/3p4/2p2p2/8/1P4P1/6P1/6K1 w - - 0 1", 1),
        ("6k1/6pp/p2br3/q2p4/1pn1nPb1/1Q4P1/PPP4R/1NK1R1B1 b - - 0 1", 4),
        ("8/5k2/R4p2/1P5p/2K5/5p2/P4n1P/8 b - - 0 1", 12),
        ("rnb2r2/pp1p1p1p/2p4k/2b1p2N/2B1P3/3P4/PPP2PPP/R3K1NR w KQ - 0 1", 13),
        ("4r3/R3N3/5k2/8/5p2/8/bPP2K2/8 w - - 0 1", 13),
        ("r2q1rk1/pp5p/2n1p3/1B3p1p/3Pp2b/4B2P/PPP3P1/R4RK1 w - - 0 1", 2),
        ("1R6/3b4/1p5k/3r2p1/5p2/5N1P/8/6K1 b - - 0 1", 13),
        ("r1r3k1/4q1bp/p3b1p1/1pp1p1P1/4P3/2Q5/1PPNBP2/1K1R3R w - - 0 1", 10),
        ("7R/4k3/5p2/8/1r4P1/6K1/8/8 b - - 0 1", 5),
        ("2kr1b1r/ppp2ppp/5n2/8/8/3q4/PP2QPPP/RNB3K1 w - - 0 1", 6),
        ("5rk1/rq3ppp/pNppb1n1/8/2PNP3/R2PB3/1P3PPP/Q4RK1 b - - 0 1", 9),
        ("r4rk1/pp3pbp/4b1p1/3q4/5B2/1P1B4/P5PP/1R1Q1R1K b - - 0 1", 5),
        ("k7/5q2/p4p2/1p6/6K1/8/1P3Q2/8 b - - 0 2", 2),
        ("4r1k1/4rp2/bpp3p1/p2p4/P5P1/2P3NP/1P3BB1/2R3K1 b - - 0 1", 1),
        ("5k2/1R6/2p5/4BP1p/3P4/2P4K/P7/8 b - - 0 1", 4),
        ("r6r/p1k1b3/2ppb2p/4p1p1/2P1n3/P4N2/1B2BPPP/2K3RR w - - 0 3", 8),
        ("5k2/1r3p2/1pR2K1P/1P2P3/p1P5/6P1/8/8 b - - 0 1", 13),
        ("8/pp2r1pp/2pR4/5k1K/1P3r2/P4R1P/8/8 w - - 0 1", 6),
        ("1n1r1k2/2R2p1p/1P6/p6p/3p4/6P1/5PK1/8 w - - 0 1", 12),
        ("5k2/3R2p1/5p1p/2P4P/1P3P2/P2Br1PK/8/8 b - - 0 1", 11),
        ("8/1p4pk/7p/3r1P2/1P4P1/8/2rpR3/1R4K1 w - - 0 1", 11),
        ("8/8/8/5k1p/2N4P/3n1KP1/8/8 w - - 0 1", 13),
        ("8/p4Bkp/1pQ3p1/5pN1/nP3B2/6P1/P1P2P1P/2KR4 b - - 0 1", 11),
        ("5n2/R7/5kpp/5p1P/5KP1/5P1B/8/7r w - - 0 1", 5),
        ("8/5p2/4p3/P6P/6k1/8/5K2/8 w - - 0 1", 7),
        ("1k6/p5pp/1p1b4/1P2pp2/P1R1p3/4P1P1/7P/6K1 w - - 0 1", 10),
        ("r3r3/5q2/2kN3p/2p2p2/8/5N1P/P2R1K2/8 b - - 0 1", 8),
        ("1r3k2/2R4R/5p2/1r2p1p1/2p1P3/2P3K1/6P1/8 b - - 0 1", 11),
        ("8/r4pkp/B1R1pnp1/8/1P3P2/p4K2/5P1P/8 b - - 0 1", 8),
        ("8/5p2/1p4k1/1Pq3pp/8/1Q4PP/5PK1/8 b - - 0 1", 4),
        ("4k3/8/3p3p/8/1p5n/1P1B4/8/1K6 w - - 0 1", 6),
        ("6k1/p1rB1p2/2P3pp/3p2q1/3P4/1r5P/5PP1/R3Q1K1 w - - 0 1", 12),
        ("1R6/8/5k1p/3B1n2/1B6/6P1/5n1K/8 w - - 0 1", 6),
        ("2bk2r1/3qn3/n1p1p2p/4P1p1/2QP1p2/P7/1P3PPP/R1B2RK1 b - - 0 2", 7),
        ("2kr4/1ppb1p1p/p1n5/4p2n/q1NpP1pP/3P2P1/2P1NPB1/1R1Q1RK1 w - - 0 1", 6),
        ("6k1/8/R7/1p1n2p1/8/6PK/P7/8 b - - 0 1", 8),
        ("r1b2rk1/ppp2p1p/3p1np1/R3p3/1PBnP3/2PP4/5PPP/1NB2RK1 b - - 0 2", 2),
        ("8/2kn2R1/P7/KPnP4/2P5/1r6/6P1/8 b - - 0 1", 6),
        ("8/6k1/1Q6/3p1P2/p2Pp3/7q/8/4K3 b - - 0 2", 8),
        ("1B4k1/1p3ppp/3p4/3p4/2n5/1RPb2P1/P6P/R1K5 b - - 0 1", 2),
        ("3rr1k1/1ppn2pp/p1bp4/P4p2/3RPN2/1P4PP/2P2PB1/4R1K1 w - - 0 1", 6),
        ("8/4bpk1/3P3p/3P2p1/n7/B2p3P/5PPK/3r4 b - - 0 1", 13),
        ("2r1qrk1/1b2b1pp/2p1pp2/1pn1nP2/4P3/2NBB3/1P2N1PP/1Q1R1RK1 w - - 0 1", 12),
        ("R7/6p1/6kp/P1b5/8/3P3P/6P1/5R1K b - - 0 1", 6),
        ("2rq1rk1/pp3pp1/3b1n1p/3p4/7B/5Q2/PPP1NPPP/2R1R1K1 b - - 0 1", 7),
        ("4b3/6kp/p4pp1/1p1P4/1Pn1P3/PB1K1P2/6PP/2B5 b - - 0 1", 5),
        ("r1b2kr1/ppq1bp2/2pp2np/4p3/2PP4/1P2PNPn/PBQ1N1BP/2R2R1K b - - 0 1", 11),
        ("7k/6p1/7p/4Q2P/8/6P1/4KP2/3r4 b - - 0 1", 10),
        ("6N1/2b5/8/2K4k/6pB/4n3/8/8 w - - 0 1", 11),
        ("1r6/5pk1/3b1p1p/pBp3pP/8/1Q6/1PK2PP1/1R4N1 b - - 0 1", 7),
        ("r2q1rk1/pp2nppp/2nb4/1N1p1b2/3P4/3B1N1P/PP3PP1/R1BQ1RK1 b - - 0 1", 4),
        ("6k1/5R2/4K2B/5N2/8/6P1/1P6/8 b - - 0 1", 4),
        ("5k2/pp1P3p/2p1r3/8/2P1pK2/1P3p2/P3r3/3R4 b - - 0 1", 10),
        ("8/8/p7/6k1/1R6/4KB2/r5P1/8 b - - 0 1", 4),
        ("8/8/1p2B2p/1P2p1p1/1p2P1Pk/2bP4/4K3/8 b - - 0 1", 4),
        ("r6k/ppp3p1/1b1p3p/4pq2/8/PBPp2Pb/1P2Q3/4R2K w - - 0 1", 11),
        ("r2k4/p3qp1p/2b5/3p4/4p3/2N1BN2/PPP2PrP/R3KR2 w - - 0 1", 12),
        ("r6k/2pbR1p1/3p2Bp/1p1P4/8/1Pq5/2P3PP/3Q3K b - - 0 1", 10),
        ("2r2rk1/1p3ppp/4p3/p5PP/1nqP1B2/8/P2Q1P2/1K5R b - - 0 1", 8),
        ("8/1p1rkppp/r1nNp3/2P1P3/2p2P2/2P1B3/1PK3PP/1R6 b - - 0 1", 4),
        ("3r1rk1/5p2/pq1p1n1p/1p3Q2/7p/8/PP1R1PPB/1B2R2K b - - 0 1", 3),
        ("r2q1rk1/1b1nbppp/p2p1n2/4p3/2B1P3/2PP1N2/1P3PPP/RNBQ1RK1 w - - 0 1", 6),
        ("r2r1k2/6pp/8/2p2p2/1p3P2/1P2R2P/7P/4R1K1 b - - 0 1", 3),
        ("5k2/8/8/6PB/3N4/P1PK2P1/8/8 b - - 0 1", 3),
        ("2rr2k1/pb2qp2/2n1p2B/2bn4/8/1P2PN2/4BPPP/Q1RR2K1 b - - 0 1", 11),
        ("8/4n1k1/4pR2/2Pp2PP/4rP2/8/6K1/8 b - - 0 1", 2),
        ("r3r1k1/pp1nbppp/2p2n2/q2p4/3P1B2/2NBPP1P/PPQ2P2/2KR3R w - - 0 1", 5),
        ("1r1Nk2r/p4ppp/n3p1b1/4P1N1/n5P1/7P/p1P2P2/2KR3R w - - 0 1", 2),
        ("6k1/R6p/6q1/2rP2p1/4np2/1P2P3/1Q4PP/R5K1 w - - 0 1", 7),
        ("r1N4k/pp3pb1/7p/2Q4p/1P2P3/2PP4/P1K2P2/RN4q1 b - - 0 1", 13),
        ("2qrk1r1/5p2/2pPp3/2P4p/1Q4pP/1P4B1/4bPP1/2R3K1 b - - 0 2", 8),
        ("5r2/1n2ppk1/2p1b1pp/2P5/1P4P1/5P2/4R2P/2KN1B2 b - - 0 1", 10),
        ("3r4/1bRP4/1k1B4/1p5p/pPp3p1/P2p4/1P1K1P1P/8 w - - 0 1", 8),
        ("6N1/1pk2p2/p6p/2P1p2P/1P3bP1/5P2/P1K5/8 w - - 0 1", 10),
        ("6k1/R4p1p/2p2p2/8/1KP2r2/P7/8/8 b - - 0 1", 3),
        ("6k1/p1r4p/1p1rP1pb/2p1Rp2/3p1P2/BP4P1/P1P3KP/4R3 w - - 0 1", 3),
        ("1k3r1r/ppp1q1b1/2n1p1Rp/3pPp2/3P1P2/2P2N2/PP3BPP/2RQ2K1 b - - 0 1", 5),
        ("2r2rk1/3q1p1p/p1p2p2/3p1b2/4pP2/1P2P3/P1QPN1PP/R4RK1 w - - 0 1", 10),
        ("r4rk1/1p1nppbp/1p3np1/2pp1b2/1P1P4/P1P1PNPP/3N1PB1/R1B2RK1 b - - 0 1", 9),
        ("7k/8/5P2/8/4K3/5n2/6R1/8 b - - 0 1", 7),
        ("k7/1bR5/pp6/8/6N1/8/PP3PPK/4r3 b - - 0 1", 11),
        ("8/5pk1/1N1Q4/3P4/8/P1b5/K4n2/6q1 b - - 0 1", 4),
        ("1r6/4kp2/5qp1/3Pp3/4P3/7P/1r3P2/4R1K1 w - - 0 3", 9),
        ("r1b2rk1/1p2bppp/p5q1/2pQ4/5B2/8/PP2BPPP/R4RK1 b - - 0 1", 11),
        ("2kq1r2/pp5p/2n5/2p5/2P1R2P/3p2N1/PP3BQ1/6K1 b - - 0 2", 4),
        ("r7/3k1Npp/ppnp4/5n2/8/2B5/PP3P1P/1K1R2R1 b - - 0 1", 4),
        ("6k1/p4pp1/1p2p2p/8/2P1P2b/4B3/PP4P1/3K4 w - - 0 1", 11),
        ("4rrk1/6pp/1R6/2pP1P2/p1q5/4PQPP/5K2/4R3 b - - 0 1", 6),
        ("r2q4/3n4/p5kP/1p2P3/2p5/5P2/1P3P2/2KR3R b - - 0 2", 1),
        ("r3k3/p2qnp2/2p2bpQ/3p4/3Pp3/BP2P1P1/P2N1PK1/R7 b q - 0 1", 12),
    ];

    let mut search = Search::new(
        Board::from_fen(Board::START_POSITION_FEN).unwrap(),
        megabytes_to_capacity(32),
        #[cfg(feature = "spsa")]
        UCI_PROCESSOR.with(|uci_processor| uci_processor.borrow().tunables),
    );

    let mut total_nodes: u64 = 0;
    let time = Time::now();
    for (position, depth) in SEARCH_POSITIONS {
        let board = Board::from_fen(position).unwrap();
        search.new_board(board);
        search.clear_cache_for_new_game();
        search.clear_for_new_search();

        #[cfg(not(target_arch = "wasm32"))]
        let time_manager = TimeManager::depth_limited(
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            None,
            depth,
        );
        #[cfg(target_arch = "wasm32")]
        let time_manager = TimeManager::depth_limited(false, false, None, depth);

        let result = search.iterative_deepening(&time_manager, &mut |_| {});
        out(&format!("{position} {depth} {}", search.node_count()));
        total_nodes += search.node_count();
    }
    out(&format!(
        "{total_nodes} nodes {nodes_per_second} nps",
        nodes_per_second = (total_nodes * 1000) / time.milliseconds()
    ));
}

fn process_input(input: &str) -> bool {
    let mut quit = false;
    let mut args = input.split_whitespace();
    UCI_PROCESSOR.with(|uci_processor| match args.next().expect("Empty input") {
        "isready" => uci_processor.borrow().isready(),
        "go" => {
            let mut parameters = GoParameters::empty();
            parameters.parse(&mut args);
            uci_processor.borrow_mut().go(parameters);
        }
        "position" => uci_processor.borrow_mut().position(&mut args),
        "ucinewgame" => uci_processor.borrow_mut().ucinewgame(),
        "setoption" => uci_processor.borrow_mut().setoption(input),

        #[cfg(not(target_arch = "wasm32"))]
        "ponderhit" => uci_processor.borrow().ponderhit(),

        "uci" => uci_processor.borrow().uci(),

        #[cfg(not(target_arch = "wasm32"))]
        "stop" => uci_processor.borrow().stop(),
        "quit" => quit = true,

        "bench" => {
            bench();
        }

        _ => panic!("Unrecognised command"),
    });
    quit
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let args: Vec<String> = env::args().collect();

        let target = args.get(1);
        if target.is_some_and(|arg| arg == "bench") {
            bench();
            return;
        }
    }

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let quit = process_input(&input);
        if quit {
            break;
        }
    }
}
