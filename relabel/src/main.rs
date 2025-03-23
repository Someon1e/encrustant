#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use clap::Parser;
use encrustant::evaluation::eval_data::EvalNumber;

use rayon::prelude::*;

use std::io::{BufRead, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use std::{fs::File, io::BufReader};

use encrustant::board::Board;
use encrustant::search::Search;
use encrustant::search::time_manager::{NodeLimit, TimeManager};
use encrustant::search::transposition::megabytes_to_capacity;

const CHUNK_SIZE: usize = 1 << 19;
const HARD_NODE_LIMIT: u64 = 400_000;
const SOFT_NODE_LIMIT: u64 = 150_000;

const WIN_THRESHOLD: EvalNumber = 100;

fn parse_data_set(path: &Path) -> Vec<Board> {
    let file = File::open(path).expect("Failed to open file");
    let data_set = BufReader::new(file);
    let mut parsed = Vec::with_capacity(2_000_000);

    for data in data_set.lines() {
        let Ok(data) = data else {
            eprintln!("Failed to read data");
            continue;
        };

        let fen = &data[0..data.len() - 3];

        let board = Board::from_fen(fen).unwrap();
        parsed.push(board);
    }
    parsed.shrink_to_fit();

    parsed
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    dataset: String,

    #[arg(short, long)]
    output: String,
}

enum WDL {
    WhiteWin,
    BlackWin,
    Draw,
}

fn main() {
    let args = Args::parse();

    let data_set_start_time = Instant::now();
    let data_set = parse_data_set(&Path::new(&args.dataset));
    let mut output_file = File::create(args.output).unwrap();
    println!(
        "Parsed dataset in {} seconds",
        data_set_start_time.elapsed().as_secs_f64()
    );

    const NODE_LIMIT: NodeLimit = NodeLimit::new(HARD_NODE_LIMIT, SOFT_NODE_LIMIT);

    let data_set_len = data_set.len();
    let progress_counter = Arc::new(AtomicUsize::new(0));

    dbg!(data_set_len / CHUNK_SIZE);
    let searching_start_time = Instant::now();
    let results: Vec<(usize, (String, WDL))> = data_set
        .par_chunks(CHUNK_SIZE)
        .enumerate()
        .flat_map_iter(|(chunk_idx, chunk)| {
            let progress_counter = Arc::clone(&progress_counter);
            let data_set_len = data_set_len;

            let mut search = Search::new(
                Board::from_fen(Board::START_POSITION_FEN).unwrap(),
                megabytes_to_capacity(1_000),
            );
            chunk
                .iter()
                .map(move |board| {
                    search.new_board(board.clone());
                    search.clear_for_new_search();

                    let (_depth, best_score) = search.iterative_deepening(
                        &TimeManager::node_limited(
                            Arc::new(AtomicBool::new(false)),
                            Arc::new(AtomicBool::new(false)),
                            None,
                            NODE_LIMIT,
                        ),
                        &mut |_| {},
                    );
                    let normalised_score = if search.board().white_to_move {
                        best_score
                    } else {
                        -best_score
                    };

                    let wdl = if normalised_score > WIN_THRESHOLD {
                        WDL::WhiteWin
                    } else if normalised_score < -WIN_THRESHOLD {
                        WDL::BlackWin
                    } else {
                        WDL::Draw
                    };

                    let prev = progress_counter.fetch_add(1, Ordering::Relaxed);
                    let current = prev + 1;

                    if current % 256 == 0 {
                        let percent = (current as f64 / data_set_len as f64) * 100.0;
                        let elapsed = searching_start_time.elapsed();
                        let elapsed_secs = elapsed.as_secs_f64();

                        let estimated_total_time =
                            elapsed_secs / (current as f64 / data_set_len as f64);
                        let remaining_time = estimated_total_time - elapsed_secs;

                        eprintln!("Progress: {:.3}%", percent);
                        eprintln!("Estimated time remaining: {:.1} seconds", remaining_time);
                    }

                    (chunk_idx, (board.to_fen(), wdl))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Sort by chunk index to maintain original order
    let mut sorted_results = results;
    sorted_results.sort_unstable_by_key(|&(idx, _)| idx);

    for (_, (fen, wdl)) in sorted_results {
        match wdl {
            WDL::WhiteWin => writeln!(output_file, "{} [1.0]", fen).unwrap(),
            WDL::BlackWin => writeln!(output_file, "{} [0.0]", fen).unwrap(),
            WDL::Draw => writeln!(output_file, "{} [0.5]", fen).unwrap(),
        }
    }

    println!("Done");
}
