#![deny(clippy::all)]
#![warn(clippy::nursery)]

mod evaluation;

use encrustant::board::Board;
use evaluation::{DataPoint, PARAMETER_COUNT, get_active, get_piece_counts, get_total_phase};
use rayon::prelude::*;
use std::io::BufRead;
use std::time::Instant;
use std::{fs::File, io::BufReader};

fn parse_data_set() -> Vec<DataPoint> {
    let file = File::open("dataset/positions.txt").expect("Failed to open file");
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
        let active = get_active(&board);
        let piece_counts = get_piece_counts(&board);
        parsed.push(DataPoint {
            active,
            result,
            piece_counts,
        });
    }
    parsed.shrink_to_fit();

    parsed
}

fn sigmoid(score: f64) -> f64 {
    1.0 / (1.0 + f64::exp(-score / 400.0))
}

fn mean_square_error(
    data_set: &[DataPoint],
    k: f64,
    parameters: &[(f64, f64)],
    phase_weights: &[f64; 5],
) -> f64 {
    let total_square_error: f64 = data_set
        .par_iter()
        .map(|data_point| {
            let phase = data_point.get_phase(phase_weights);
            let score = data_point.evaluate(parameters, phase);

            let sigmoid = sigmoid(k * score);

            let error = data_point.result - sigmoid;
            error * error
        })
        .sum();

    total_square_error / data_set.len() as f64
}

fn compute_gradients(
    data_set: &[DataPoint],
    k: f64,
    parameters: &[(f64, f64); PARAMETER_COUNT],
    phase_weights: &[f64; 5],
) -> ([(f64, f64); PARAMETER_COUNT], [f64; 5]) {
    let mut param_gradients = [(0.0, 0.0); PARAMETER_COUNT];
    let mut phase_gradients = [0.0; 5];
    let max_counts = [8.0, 2.0, 2.0, 2.0, 1.0];

    for data_point in data_set {
        let phase = data_point.get_phase(phase_weights);
        let score = data_point.evaluate(parameters, phase);
        let sigmoid_val = sigmoid(k * score);

        let term = 2.0 * (sigmoid_val - data_point.result) * sigmoid_val * (1.0 - sigmoid_val) * k;

        // Compute mid_total and end_total
        let white_mid: f64 = data_point.active[0]
            .iter()
            .map(|&i| parameters[i as usize].0)
            .sum();
        let white_end: f64 = data_point.active[0]
            .iter()
            .map(|&i| parameters[i as usize].1)
            .sum();
        let black_mid: f64 = data_point.active[1]
            .iter()
            .map(|&i| parameters[i as usize].0)
            .sum();
        let black_end: f64 = data_point.active[1]
            .iter()
            .map(|&i| parameters[i as usize].1)
            .sum();
        let mid_total = white_mid - black_mid;
        let end_total = white_end - black_end;
        let error_term = term * (mid_total - end_total);

        let current_phase: f64 = data_point
            .piece_counts
            .iter()
            .enumerate()
            .map(|(i, &count)| count * phase_weights[i])
            .sum();
        let max_phase = get_total_phase(phase_weights);

        // Phase gradients
        for i in 0..5 {
            let count_i = data_point.piece_counts[i];
            let max_count_i = max_counts[i];
            let derivative =
                count_i.mul_add(max_phase, -(current_phase * max_count_i)) / max_phase.powi(2);
            phase_gradients[i] += error_term * derivative;
        }

        // Parameter gradients
        let scores = (phase * term, (1.0 - phase) * term);
        for index in &data_point.active[0] {
            param_gradients[*index as usize].0 += scores.0;
            param_gradients[*index as usize].1 += scores.1;
        }
        for index in &data_point.active[1] {
            param_gradients[*index as usize].0 -= scores.0;
            param_gradients[*index as usize].1 -= scores.1;
        }
    }

    (param_gradients, phase_gradients)
}

fn compute_gradients_parallel(
    data_set: &[DataPoint],
    k: f64,
    parameters: &[(f64, f64); PARAMETER_COUNT],
    phase_weights: &[f64; 5],
) -> ([(f64, f64); PARAMETER_COUNT], [f64; 5]) {
    // Split the dataset into chunks for parallel processing
    data_set
        .par_chunks(262144)
        .map(|chunk| compute_gradients(chunk, k, parameters, phase_weights))
        .reduce(
            || ([(0.0, 0.0); PARAMETER_COUNT], [0.0; 5]),
            |mut a, b| {
                let (a_params, a_phases) = &mut a;
                let (b_params, b_phases) = b;

                // Sum parameter gradients
                for (i, param) in a_params.iter_mut().enumerate() {
                    param.0 += b_params[i].0;
                    param.1 += b_params[i].1;
                }

                // Sum phase gradients
                for (i, phase) in a_phases.iter_mut().enumerate() {
                    *phase += b_phases[i];
                }

                a
            },
        )
}

fn pretty_parameters(parameters: &[(f64, f64); PARAMETER_COUNT]) -> String {
    let mut output = String::new();
    output.push_str("[\n");
    for piece in 0..6 {
        for rank in 0..8 {
            for file in 0..8 {
                output.push_str(&format!(
                    "({:>4}, {:<4}), ",
                    parameters[piece * 64 + rank * 8 + file].0 as i16,
                    parameters[piece * 64 + rank * 8 + file].1 as i16,
                ));
            }
            output.push('\n');
        }
        if piece != 5 {
            output.push_str("\n\n");
        }
    }
    output.push(']');
    output
}

fn find_k(
    data_set: &[DataPoint],
    parameters: &[(f64, f64); PARAMETER_COUNT],
    phase_weights: &[f64; 5],
) -> f64 {
    let mut min = -10.0;
    let mut max = 10.0;
    let mut delta = 1.0;

    let mut best = 1.0;
    let mut best_error = 100.0;

    for _ in 0..10 {
        println!("Determining K: ({min} to {max}, {delta})");

        while min < max {
            let error = mean_square_error(data_set, min, parameters, phase_weights);
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

fn tune(
    data_set: &[DataPoint],
    k: f64,
    mut parameters: [(f64, f64); 384],
    mut phase_weights: [f64; 5],
) {
    const PARAM_LEARNING_RATE: f64 = 0.04;
    const PHASE_LEARNING_RATE: f64 = 0.001;
    const BETA1: f64 = 0.9;
    const BETA2: f64 = 0.999;

    let mut param_velocity = [(0.0, 0.0); 384];
    let mut param_momentum = [(0.0, 0.0); 384];

    let mut phase_velocity = [0.0; 384];
    let mut phase_momentum = [0.0; 384];

    let mut previous_error = f64::MAX;
    let log_params = |parameters: &[(f64, f64); 384], phase_weights: &[f64; 5]| {
        std::fs::write(
            "tuned.rs",
            format!(
                "#[rustfmt::skip]
pub const PIECE_SQUARE_TABLE: PieceSquareTable = {};

pub const PHASE_WEIGHTS: [i32; 5] = {:?};",
                pretty_parameters(parameters),
                phase_weights
                    .iter()
                    .map(|x| *x as i32)
                    .collect::<Vec<i32>>()
            ),
        )
        .unwrap();
    };
    log_params(&parameters, &phase_weights);

    let mut last_update = Instant::now();
    for iteration in 0..8000 {
        let (param_gradients, phase_gradients) =
            compute_gradients_parallel(data_set, k, &parameters, &phase_weights);

        // Update parameters
        for (i, gradient) in param_gradients.iter().enumerate() {
            param_momentum[i].0 = BETA1.mul_add(param_momentum[i].0, (1.0 - BETA1) * gradient.0);
            param_momentum[i].1 = BETA1.mul_add(param_momentum[i].1, (1.0 - BETA1) * gradient.1);

            param_velocity[i].0 =
                BETA2.mul_add(param_velocity[i].0, (1.0 - BETA2) * gradient.0 * gradient.0);
            param_velocity[i].1 =
                BETA2.mul_add(param_velocity[i].1, (1.0 - BETA2) * gradient.1 * gradient.1);

            parameters[i].0 -=
                PARAM_LEARNING_RATE * param_momentum[i].0 / (1e-8 + param_velocity[i].0.sqrt());
            parameters[i].1 -=
                PARAM_LEARNING_RATE * param_momentum[i].1 / (1e-8 + param_velocity[i].1.sqrt());
        }

        // Update phase weights
        for (i, gradient) in phase_gradients.iter().enumerate() {
            phase_momentum[i] = BETA1.mul_add(phase_momentum[i], (1.0 - BETA1) * gradient);

            phase_velocity[i] =
                BETA2.mul_add(phase_velocity[i], (1.0 - BETA2) * gradient * gradient);

            phase_weights[i] -=
                PHASE_LEARNING_RATE * phase_momentum[i] / (1e-8 + phase_velocity[i].sqrt());
        }

        let error = mean_square_error(data_set, k, &parameters, &phase_weights);
        println!("Iteration {iteration}: MSE = {error}");

        if error < previous_error && last_update.elapsed().as_millis() > 500 {
            log_params(&parameters, &phase_weights);
            last_update = Instant::now();
        }
        previous_error = error;
    }

    println!("Finished");
    log_params(&parameters, &phase_weights);
}

fn main() {
    let initial_phase_weights = [0.0, 100.0, 100.0, 200.0, 400.0];

    let initial_parameters = {
        let mut initial_parameters = [(0.0, 0.0); 384];

        let mut index = 0;

        // Pawn
        for pawn_index in 0..64 {
            if pawn_index > 7 && pawn_index < 56 {
                initial_parameters[index] = (50.0, 90.0);
            }
            index += 1;
        }

        // Knight
        for _ in 0..64 {
            initial_parameters[index] = (225.0, 270.0);
            index += 1;
        }

        // Bishop
        for _ in 0..64 {
            initial_parameters[index] = (250.0, 290.0);
            index += 1;
        }

        // Rook
        for _ in 0..64 {
            initial_parameters[index] = (320.0, 500.0);
            index += 1;
        }

        // Queen
        for _ in 0..64 {
            initial_parameters[index] = (710.0, 920.0);
            index += 1;
        }

        // King
        for _ in 0..64 {
            initial_parameters[index] = (-80.0, 15.0);
            index += 1;
        }

        initial_parameters
    };

    let data_set_start_time = Instant::now();
    let data_set = parse_data_set();
    println!(
        "Parsed dataset in {:.1} seconds",
        data_set_start_time.elapsed().as_secs_f64()
    );

    let k_start_time = Instant::now();
    // let k = find_k(&data_set, &parameters, &initial_phase_weights);
    let k = 4.0 * f64::ln(3.0);
    println!(
        "Found k: {k} in {:.1} seconds",
        k_start_time.elapsed().as_secs_f64()
    );

    let tune_start_time = Instant::now();
    tune(&data_set, k, initial_parameters, initial_phase_weights);
    println!(
        "Tuned in {:.1} seconds",
        tune_start_time.elapsed().as_secs_f64()
    );
}
