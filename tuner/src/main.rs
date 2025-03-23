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
    #[rustfmt::skip]
    let middle_game_piece_square_tables: PieceSquareTable = [
       0,   0,   0,   0,   0,   0,   0,   0,
     170, 170, 160, 180, 170, 150,  80,  80,
      80,  90, 120, 120, 120, 140, 120,  80,
      50,  70,  70,  70,  90,  90,  90,  70,
      40,  60,  60,  70,  70,  70,  80,  60,
      40,  60,  60,  60,  80,  70, 100,  60,
      40,  60,  60,  40,  60,  80, 100,  60,
       0,   0,   0,   0,   0,   0,   0,   0,

     150, 190, 250, 270, 290, 230, 250, 210,
     260, 280, 320, 320, 310, 360, 290, 310,
     280, 310, 330, 340, 370, 360, 330, 310,
     280, 290, 310, 330, 310, 340, 300, 310,
     280, 280, 300, 300, 310, 300, 300, 280,
     260, 280, 290, 290, 300, 290, 300, 270,
     250, 260, 280, 280, 280, 290, 270, 270,
     220, 260, 250, 260, 270, 280, 260, 240,

     280, 270, 280, 240, 250, 260, 300, 290,
     290, 300, 310, 300, 320, 320, 310, 300,
     300, 330, 330, 350, 340, 360, 340, 330,
     300, 310, 330, 340, 330, 330, 320, 300,
     300, 310, 320, 330, 330, 320, 310, 310,
     310, 320, 320, 320, 320, 320, 320, 320,
     320, 320, 320, 310, 310, 320, 330, 310,
     300, 310, 300, 290, 300, 300, 310, 300,

     450, 450, 450, 460, 470, 480, 480, 500,
     450, 450, 460, 480, 480, 500, 480, 510,
     430, 440, 450, 450, 470, 470, 490, 480,
     420, 430, 430, 440, 440, 440, 450, 450,
     410, 410, 420, 430, 430, 420, 440, 430,
     410, 420, 420, 420, 420, 430, 450, 430,
     400, 410, 420, 420, 420, 420, 440, 420,
     420, 420, 430, 430, 440, 430, 440, 430,

     830, 840, 860, 890, 890, 910, 890, 890,
     860, 840, 860, 860, 860, 900, 880, 910,
     860, 870, 870, 880, 900, 920, 920, 920,
     860, 860, 860, 860, 860, 880, 880, 880,
     860, 860, 860, 860, 860, 870, 880, 880,
     860, 860, 860, 860, 870, 870, 880, 870,
     860, 860, 870, 870, 870, 870, 880, 880,
     860, 850, 860, 870, 860, 850, 850, 850,

     -40, -40, -40,-100, -70, -20,  10,  20,
     -50, -50, -80, -50, -60, -40,   0,  10,
     -80, -40, -80, -80, -60, -10, -10, -30,
     -90, -80, -90,-120,-110, -80, -80, -70,
     -80, -80,-100,-130,-120,-100, -80,-100,
     -50, -50, -90,-100,-100, -80, -50, -60,
       0, -20, -40, -60, -70, -40, -10,   0,
     -10,  20,   0, -80, -30, -50,   0,   0,
    ];

    #[rustfmt::skip]
    let end_game_piece_square_tables: PieceSquareTable = [
      50,  20,  10, -10,  10,   0,  10,  10,
     240, 240, 230, 190, 180, 190, 230, 240,
     190, 200, 180, 150, 140, 130, 170, 170,
     130, 120, 100, 100,  90,  90, 110, 110,
     100, 100,  90,  90,  80,  80,  90,  90,
     100, 100,  90, 100,  90,  80,  90,  80,
     100, 100,  90, 100, 100,  90,  90,  80,
      40,  10,   0,   0,   0,  10,   0,   0,

     220, 270, 270, 270, 280, 260, 260, 200,
     260, 280, 280, 280, 280, 270, 270, 250,
     270, 280, 300, 300, 290, 290, 280, 260,
     270, 300, 300, 310, 310, 300, 290, 280,
     280, 290, 300, 300, 300, 300, 290, 270,
     270, 280, 290, 300, 300, 290, 280, 260,
     260, 270, 280, 280, 280, 280, 270, 260,
     250, 240, 270, 270, 270, 260, 250, 240,

     290, 290, 290, 300, 300, 300, 290, 290,
     280, 300, 300, 300, 300, 290, 300, 270,
     300, 300, 310, 300, 300, 300, 300, 300,
     300, 310, 310, 320, 310, 310, 310, 300,
     290, 310, 310, 310, 310, 310, 300, 280,
     290, 300, 310, 310, 310, 310, 300, 290,
     290, 290, 290, 300, 300, 290, 300, 280,
     280, 290, 280, 290, 290, 290, 280, 270,

     510, 520, 520, 520, 510, 500, 500, 500,
     510, 520, 520, 510, 510, 500, 500, 490,
     510, 510, 510, 510, 500, 500, 490, 480,
     510, 510, 510, 510, 500, 500, 490, 490,
     500, 500, 510, 500, 500, 500, 480, 480,
     500, 500, 500, 500, 490, 490, 470, 480,
     490, 500, 500, 500, 490, 480, 480, 480,
     480, 490, 500, 500, 490, 480, 480, 470,

     980,1000,1000, 980, 980, 950, 960, 940,
     930, 970,1000,1010,1030, 980, 950, 920,
     940, 950, 990, 990, 990, 980, 930, 920,
     940, 970, 990,1010,1010,1000, 970, 950,
     950, 970, 980, 990,1000, 980, 960, 950,
     940, 950, 970, 970, 970, 960, 940, 930,
     940, 940, 940, 940, 940, 930, 900, 890,
     920, 940, 940, 930, 940, 930, 920, 920,

     -40, -10,   0,  40,  20,  30,  20, -30,
      10,  30,  50,  40,  50,  60,  50,  30,
      20,  40,  50,  60,  60,  60,  60,  30,
      20,  40,  50,  60,  60,  60,  60,  30,
      10,  30,  40,  50,  50,  50,  40,  20,
       0,  20,  30,  40,  40,  30,  20,  10,
     -20,   0,  10,  20,  20,  20,  10, -10,
     -30, -20, -10,   0, -20,   0, -10, -40,
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
