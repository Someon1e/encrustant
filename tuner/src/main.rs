#![deny(clippy::all)]
#![warn(clippy::nursery)]

use encrustant::board::Board;
use encrustant::evaluation::Eval;
use encrustant::evaluation::eval_data::{EvalNumber, PieceSquareTable};
use rayon::prelude::*;
use std::io::BufRead;
use std::time::Instant;
use std::{fs::File, io::BufReader};

fn parse_data_set() -> Vec<(Board, f64)> {
    let file = File::open("dataset/result.lfs").expect("Failed to open file");
    let data_set = BufReader::new(file);
    let mut parsed = Vec::with_capacity(2_000_000);

    for data in data_set.lines() {
        let Result::Ok(data) = data else {
            eprintln!("Failed to read data");
            continue;
        };

        let fen = &data[0..data.len() - 3];
        let result = &data[data.len() - 4..data.len() - 1];
        let result: f64 = match result {
            "0.0" => 0.0,
            "0.5" => 0.5,
            "1.0" => 1.0,
            _ => panic!("Unknown game result {result}"),
        };

        let board = Board::from_fen(fen).unwrap();
        parsed.push((board, result));
    }
    parsed.shrink_to_fit();

    parsed
}

fn mean_square_error(
    data_set: &[(Board, f64)],
    k: f64,
    middle_game_piece_square_tables: &PieceSquareTable,
    end_game_piece_square_tables: &PieceSquareTable,
    phases: &[i32; 5],
) -> f64 {
    let total_square_error: f64 = data_set
        .par_iter()
        .map(|(board, result)| {
            let score = f64::from(
                Eval::evaluate_with_parameters(
                    middle_game_piece_square_tables,
                    end_game_piece_square_tables,
                    phases,
                    board,
                ) * if board.white_to_move { 1 } else { -1 },
            );

            let sigmoid = 1.0 / (1.0 + f64::exp(-k * score / 400.0));

            let error = result - sigmoid;
            error * error
        })
        .sum();

    total_square_error / data_set.len() as f64
}

fn pretty_piece_square_tables(piece_square_tables: PieceSquareTable) -> String {
    let mut output = String::new();
    output.push_str("[\n");
    for piece in 0..6 {
        for rank in 0..8 {
            output.push('\n');
            for file in 0..8 {
                output.push_str(&format!(
                    "{:>4},",
                    piece_square_tables[piece * 64 + rank * 8 + file]
                ));
            }
        }
        output.push_str("\n\n");
    }
    output.push(']');
    output
}

fn tune(
    data_set: &[(Board, f64)],
    k: f64,
    middle_game_piece_square_tables: &PieceSquareTable,
    end_game_piece_square_tables: &PieceSquareTable,
    phases: &[i32; 5],
) {
    const PSQT_ADJUSTMENT_VALUE: i16 = 1;
    const PHASE_ADJUSTMENT_VALUE: i32 = 1;

    let mut best_error = mean_square_error(
        data_set,
        k,
        middle_game_piece_square_tables,
        end_game_piece_square_tables,
        phases,
    );
    println!("Currently {best_error}");

    let log_params = |psqt_1, psqt_2, new_phases| {
        std::fs::write(
            "tuned.rs",
            format!(
                "const MIDDLE_GAME_PIECE_SQUARE_TABLES: PieceSquareTable = {};
const END_GAME_PIECE_SQUARE_TABLES: PieceSquareTable = {};
const PHASES: [i32; 5] = {:#?};",
                pretty_piece_square_tables(psqt_1),
                pretty_piece_square_tables(psqt_2),
                new_phases
            ),
        )
        .unwrap();
    };
    log_params(
        *middle_game_piece_square_tables,
        *end_game_piece_square_tables,
        *phases,
    );

    let mut best_psqt = [
        *middle_game_piece_square_tables,
        *end_game_piece_square_tables,
    ];
    let mut best_phases = *phases;
    let mut improved = true;

    while improved {
        improved = false;

        for table_number in 0..2 {
            for index in 0..384 {
                let mut new_psqts: [PieceSquareTable; 2] = best_psqt;
                new_psqts[table_number][index] += PSQT_ADJUSTMENT_VALUE;

                let mut new_error =
                    mean_square_error(data_set, k, &new_psqts[0], &new_psqts[1], &best_phases);

                if new_error < best_error {
                    println!("{new_error} Found better params +");
                } else {
                    new_psqts[table_number][index] -= PSQT_ADJUSTMENT_VALUE * 2;
                    new_error =
                        mean_square_error(data_set, k, &new_psqts[0], &new_psqts[1], &best_phases);

                    if new_error < best_error {
                        println!("{new_error} Found better params -");
                    } else {
                        continue;
                    }
                }

                improved = true;
                best_error = new_error;
                best_psqt = new_psqts;
            }
        }
        for index in 0..5 {
            let mut new_phases = best_phases;
            new_phases[index] += PHASE_ADJUSTMENT_VALUE;

            let mut new_error =
                mean_square_error(data_set, k, &best_psqt[0], &best_psqt[1], &new_phases);

            if new_error < best_error {
                println!("{new_error} Found better params +");
            } else {
                new_phases[index] -= PHASE_ADJUSTMENT_VALUE * 2;
                new_error =
                    mean_square_error(data_set, k, &best_psqt[0], &best_psqt[1], &new_phases);

                if new_error < best_error {
                    println!("{new_error} Found better params -");
                } else {
                    continue;
                }
            }

            improved = true;
            best_error = new_error;
            best_phases = new_phases;
        }

        log_params(best_psqt[0], best_psqt[1], best_phases);
        println!("Finished one iteration");
    }
}

fn find_k(
    data_set: &[(Board, f64)],
    middle_game_piece_square_tables: &PieceSquareTable,
    end_game_piece_square_tables: &PieceSquareTable,
    phases: &[i32; 5],
) -> f64 {
    let mut min = -10.0;
    let mut max = 10.0;
    let mut delta = 1.0;

    let mut best = 1.0;
    let mut best_error = 100.0;

    for _ in 0..10 {
        println!("Determining K: ({min} to {max}, {delta})");

        while min < max {
            let error = mean_square_error(
                data_set,
                min,
                middle_game_piece_square_tables,
                end_game_piece_square_tables,
                phases,
            );
            if error < best_error {
                best_error = error;
                best = min;
                println!("New best K: {min}, Error: {best_error}");
            }
            min += delta;
        }

        min = best - delta;
        max = best + delta;
        delta /= 10.0;
    }

    best
}

fn main() {
    const MP: i16 = 85;
    const MN: i16 = 285;
    const MB: i16 = 313;
    const MR: i16 = 446;
    const MQ: i16 = 864;
    const MK: i16 = -28;

    const EP: i16 = 123;
    const EN: i16 = 256;
    const EB: i16 = 275;
    const ER: i16 = 475;
    const EQ: i16 = 893;
    const EK: i16 = 9;

    #[rustfmt::skip]
    let middle_game_piece_square_tables: PieceSquareTable = [
       0,   0,   0,   0,   0,   0,   0,   0,
      MP,  MP,  MP,  MP,  MP,  MP,  MP,  MP,
      MP,  MP,  MP,  MP,  MP,  MP,  MP,  MP,
      MP,  MP,  MP,  MP,  MP,  MP,  MP,  MP,
      MP,  MP,  MP,  MP,  MP,  MP,  MP,  MP,
      MP,  MP,  MP,  MP,  MP,  MP,  MP,  MP,
      MP,  MP,  MP,  MP,  MP,  MP,  MP,  MP,
       0,   0,   0,   0,   0,   0,   0,   0,


      MN,  MN,  MN,  MN,  MN,  MN,  MN,  MN,
      MN,  MN,  MN,  MN,  MN,  MN,  MN,  MN,
      MN,  MN,  MN,  MN,  MN,  MN,  MN,  MN,
      MN,  MN,  MN,  MN,  MN,  MN,  MN,  MN,
      MN,  MN,  MN,  MN,  MN,  MN,  MN,  MN,
      MN,  MN,  MN,  MN,  MN,  MN,  MN,  MN,
      MN,  MN,  MN,  MN,  MN,  MN,  MN,  MN,
      MN,  MN,  MN,  MN,  MN,  MN,  MN,  MN,

      MB,  MB,  MB,  MB,  MB,  MB,  MB,  MB,
      MB,  MB,  MB,  MB,  MB,  MB,  MB,  MB,
      MB,  MB,  MB,  MB,  MB,  MB,  MB,  MB,
      MB,  MB,  MB,  MB,  MB,  MB,  MB,  MB,
      MB,  MB,  MB,  MB,  MB,  MB,  MB,  MB,
      MB,  MB,  MB,  MB,  MB,  MB,  MB,  MB,
      MB,  MB,  MB,  MB,  MB,  MB,  MB,  MB,
      MB,  MB,  MB,  MB,  MB,  MB,  MB,  MB,


      MR,  MR,  MR,  MR,  MR,  MR,  MR,  MR,
      MR,  MR,  MR,  MR,  MR,  MR,  MR,  MR,
      MR,  MR,  MR,  MR,  MR,  MR,  MR,  MR,
      MR,  MR,  MR,  MR,  MR,  MR,  MR,  MR,
      MR,  MR,  MR,  MR,  MR,  MR,  MR,  MR,
      MR,  MR,  MR,  MR,  MR,  MR,  MR,  MR,
      MR,  MR,  MR,  MR,  MR,  MR,  MR,  MR,
      MR,  MR,  MR,  MR,  MR,  MR,  MR,  MR,


      MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,
      MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,
      MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,
      MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,
      MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,
      MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,
      MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,
      MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,  MQ,


      MK,  MK,  MK,  MK,  MK,  MK,  MK,  MK,
      MK,  MK,  MK,  MK,  MK,  MK,  MK,  MK,
      MK,  MK,  MK,  MK,  MK,  MK,  MK,  MK,
      MK,  MK,  MK,  MK,  MK,  MK,  MK,  MK,
      MK,  MK,  MK,  MK,  MK,  MK,  MK,  MK,
      MK,  MK,  MK,  MK,  MK,  MK,  MK,  MK,
      MK,  MK,  MK,  MK,  MK,  MK,  MK,  MK,
      MK,  MK,  MK,  MK,  MK,  MK,  MK,  MK,
    ];

    #[rustfmt::skip]
    let end_game_piece_square_tables: PieceSquareTable = [
      0,   0,   0,   0,   0,   0,   0,   0,
     EP,  EP,  EP,  EP,  EP,  EP,  EP,  EP,
     EP,  EP,  EP,  EP,  EP,  EP,  EP,  EP,
     EP,  EP,  EP,  EP,  EP,  EP,  EP,  EP,
     EP,  EP,  EP,  EP,  EP,  EP,  EP,  EP,
     EP,  EP,  EP,  EP,  EP,  EP,  EP,  EP,
     EP,  EP,  EP,  EP,  EP,  EP,  EP,  EP,
      0,   0,   0,   0,   0,   0,   0,   0,


     EN,  EN,  EN,  EN,  EN,  EN,  EN,  EN,
     EN,  EN,  EN,  EN,  EN,  EN,  EN,  EN,
     EN,  EN,  EN,  EN,  EN,  EN,  EN,  EN,
     EN,  EN,  EN,  EN,  EN,  EN,  EN,  EN,
     EN,  EN,  EN,  EN,  EN,  EN,  EN,  EN,
     EN,  EN,  EN,  EN,  EN,  EN,  EN,  EN,
     EN,  EN,  EN,  EN,  EN,  EN,  EN,  EN,
     EN,  EN,  EN,  EN,  EN,  EN,  EN,  EN,

     EB,  EB,  EB,  EB,  EB,  EB,  EB,  EB,
     EB,  EB,  EB,  EB,  EB,  EB,  EB,  EB,
     EB,  EB,  EB,  EB,  EB,  EB,  EB,  EB,
     EB,  EB,  EB,  EB,  EB,  EB,  EB,  EB,
     EB,  EB,  EB,  EB,  EB,  EB,  EB,  EB,
     EB,  EB,  EB,  EB,  EB,  EB,  EB,  EB,
     EB,  EB,  EB,  EB,  EB,  EB,  EB,  EB,
     EB,  EB,  EB,  EB,  EB,  EB,  EB,  EB,


     ER,  ER,  ER,  ER,  ER,  ER,  ER,  ER,
     ER,  ER,  ER,  ER,  ER,  ER,  ER,  ER,
     ER,  ER,  ER,  ER,  ER,  ER,  ER,  ER,
     ER,  ER,  ER,  ER,  ER,  ER,  ER,  ER,
     ER,  ER,  ER,  ER,  ER,  ER,  ER,  ER,
     ER,  ER,  ER,  ER,  ER,  ER,  ER,  ER,
     ER,  ER,  ER,  ER,  ER,  ER,  ER,  ER,
     ER,  ER,  ER,  ER,  ER,  ER,  ER,  ER,


     EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,
     EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,
     EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,
     EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,
     EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,
     EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,
     EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,
     EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,  EQ,


     EK,  EK,  EK,  EK,  EK,  EK,  EK,  EK,
     EK,  EK,  EK,  EK,  EK,  EK,  EK,  EK,
     EK,  EK,  EK,  EK,  EK,  EK,  EK,  EK,
     EK,  EK,  EK,  EK,  EK,  EK,  EK,  EK,
     EK,  EK,  EK,  EK,  EK,  EK,  EK,  EK,
     EK,  EK,  EK,  EK,  EK,  EK,  EK,  EK,
     EK,  EK,  EK,  EK,  EK,  EK,  EK,  EK,
     EK,  EK,  EK,  EK,  EK,  EK,  EK,  EK,
    ];

    let phases: [i32; 5] = [
        000, // Pawn
        100, // Knight
        100, // Bishop
        200, // Rook
        400, // Queen
    ];

    let data_set_start_time = Instant::now();
    let data_set = parse_data_set();
    println!(
        "Parsed dataset in {} seconds",
        data_set_start_time.elapsed().as_secs_f64()
    );

    let k_start_time = Instant::now();
    let k = 4.0 * f64::ln(3.0);
    dbg!(k);
    // let k = find_k(
    //     &data_set,
    //     &middle_game_piece_square_tables,
    //     &end_game_piece_square_tables,
    //     &phases,
    // );
    // println!(
    //     "Found k: {k} in {} seconds",
    //     k_start_time.elapsed().as_secs_f64()
    // );

    let tune_start_time = Instant::now();
    tune(
        &data_set,
        k,
        &middle_game_piece_square_tables,
        &end_game_piece_square_tables,
        &phases,
    );
    println!(
        "Tuned in {} seconds",
        tune_start_time.elapsed().as_secs_f64()
    );
}
