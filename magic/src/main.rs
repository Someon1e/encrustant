use std::{
    cmp::Reverse,
    sync::{Arc, Mutex},
    thread::{self, available_parallelism},
};

use bitvec::prelude::*;
use encrustant::{
    board::{bit_board::BitBoard, square::Square},
    move_generator::slider_lookup::{
        RELEVANT_BISHOP_BLOCKERS, RELEVANT_ROOK_BLOCKERS, get_bishop_moves, get_rook_moves,
        iterate_combinations,
    },
};

use rand_chacha::{
    ChaCha20Rng,
    rand_core::{RngCore, SeedableRng},
};

fn fill_magic_table(
    table: &mut [BitBoard],
    square: Square,
    blockers: BitBoard,
    magic: u64,
    index_bits: u8,
    direction_offset: usize,
) -> Result<usize, usize> {
    let mut highest_used_index = None;
    for blocker_combination in iterate_combinations(blockers) {
        let moves = if direction_offset == 0 {
            get_rook_moves(square, blocker_combination)
        } else {
            get_bishop_moves(square, blocker_combination)
        }; //gen_rook_or_bishop(square, &blocker_combination, direction_offset);
        //assert_eq!(
        //    if direction_offset == 0 {
        //        get_rook_moves(square, blocker_combination)
        //    } else {
        //        get_bishop_moves(square, blocker_combination)
        //    },
        //    moves
        //);

        let magic_index = blocker_combination.magic_index(magic, 64 - index_bits);
        let table_entry = &mut table[magic_index];
        if highest_used_index.is_none() || highest_used_index.unwrap() < magic_index {
            highest_used_index = Some(magic_index)
        }
        if table_entry.is_empty() {
            *table_entry = moves;
        } else if *table_entry != moves {
            return Err(highest_used_index.unwrap());
        }
    }
    Ok(highest_used_index.unwrap())
}

fn out(output: &mut String, best_magics: &[u64; 64], offsets: &[u32; 64]) {
    for (magic, offset) in best_magics.iter().zip(offsets) {
        let magic_hex = format!("{magic:#018X}");
        let mut magic_hex_underscores =
            String::with_capacity(magic_hex.len() + magic_hex.len() / 2);
        for (i, character) in magic_hex.chars().enumerate() {
            if i != 2 && (i + 2).is_multiple_of(4) {
                magic_hex_underscores.push('_');
            }
            magic_hex_underscores.push(character);
        }
        output.push_str(&format!(
            "Key {{magic: {magic_hex_underscores}, offset: {offset}}},\n"
        ));
    }
}

#[derive(Clone, Copy)]
enum RookOrBishop {
    Rook,
    Bishop,
}

fn find_magics(
    random: &mut ChaCha20Rng,
    relevant_blockers: &[BitBoard; 64],
    direction_offset: usize,
    index_bits: u8,
) -> [u64; 64] {
    let mut magics = [0; 64];
    let mut table = vec![BitBoard::EMPTY; 1 << index_bits];

    for square_index in 0..64 {
        let square = Square::from_index(square_index as i8);
        let blockers = relevant_blockers[square_index];
        let mut best_magic = 0;
        let mut best_highest_used_index = usize::MAX;

        // Try multiple magics to find the most compressible one
        let mut retries = 0;
        while retries < 9000 {
            let magic = random.next_u64() & random.next_u64() & random.next_u64();
            let filled = fill_magic_table(
                &mut table,
                square,
                blockers,
                magic,
                index_bits,
                direction_offset,
            );
            if let Ok(highest_used_index) = filled {
                table[..=highest_used_index].fill(BitBoard::EMPTY);

                retries += 1;
                // Prefer magics that cluster used entries at lower indices
                if highest_used_index < best_highest_used_index {
                    best_highest_used_index = highest_used_index;
                    best_magic = magic;
                }
            } else if let Err(highest_used_index) = filled {
                table[..=highest_used_index].fill(BitBoard::EMPTY);
            }
        }
        magics[square_index] = best_magic;
    }

    magics
}

const BISHOP_INDEX_BITS: u8 = 9;
const ROOK_INDEX_BITS: u8 = 12;

fn find_closest_zero_range(occupied: &BitSlice, size: usize) -> Option<usize> {
    occupied
        .windows(size)
        .enumerate()
        .find(|(_, window)| window.not_any())
        .map(|(i, _)| i)
}

fn process_tables(
    all_tables: &mut Vec<(usize, usize, RookOrBishop)>,
    piece_type: RookOrBishop,
    magics: &[u64; 64],
) {
    let relevant_blockers = match piece_type {
        RookOrBishop::Rook => RELEVANT_ROOK_BLOCKERS,
        RookOrBishop::Bishop => RELEVANT_BISHOP_BLOCKERS,
    };
    let index_bits = match piece_type {
        RookOrBishop::Rook => ROOK_INDEX_BITS,
        RookOrBishop::Bishop => BISHOP_INDEX_BITS,
    };
    let direction_offset = match piece_type {
        RookOrBishop::Rook => 0,
        RookOrBishop::Bishop => 4,
    };
    for square_index in 0..64 {
        let square = Square::from_index(square_index as i8);
        let blockers = relevant_blockers[square_index];
        let magic = magics[square_index];
        let mut table = vec![BitBoard::EMPTY; 1 << index_bits];
        let highest_used_index = fill_magic_table(
            &mut table,
            square,
            blockers,
            magic,
            index_bits,
            direction_offset,
        )
        .unwrap();
        all_tables.push((highest_used_index + 1, square_index, piece_type));
    }
}

fn main() {
    let num_threads = available_parallelism().unwrap().into();
    let best_length = Arc::new(Mutex::new(None));

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let best_length = Arc::clone(&best_length);
            thread::spawn(move || {
                let mut random = ChaCha20Rng::seed_from_u64(i as u64 + 1);
                let mut occupied = bitvec![0; 300_000]; // Preallocated bitmap

                loop {
                    let bishop_magics =
                        find_magics(&mut random, &RELEVANT_BISHOP_BLOCKERS, 4, BISHOP_INDEX_BITS);
                    let rook_magics =
                        find_magics(&mut random, &RELEVANT_ROOK_BLOCKERS, 0, ROOK_INDEX_BITS);

                    let mut rook_offsets = [0; 64];
                    let mut bishop_offsets = [0; 64];

                    // Collect all tables with effective lengths
                    let mut all_tables = Vec::with_capacity(128);

                    process_tables(&mut all_tables, RookOrBishop::Rook, &rook_magics);
                    process_tables(&mut all_tables, RookOrBishop::Bishop, &bishop_magics);

                    // Sort by effective length descending
                    all_tables.sort_by_key(|(len, ..)| Reverse(*len));

                    // Reset occupancy tracking
                    occupied.fill(false);
                    occupied.resize(300_000, false);
                    let mut max_offset = 0;

                    // Place tables in optimal order
                    for (required_size, square_index, piece_type) in all_tables {
                        if let Some(index) = find_closest_zero_range(&occupied, required_size) {
                            // Mark space as occupied
                            occupied[index..index + required_size].fill(true);
                            match piece_type {
                                RookOrBishop::Rook => {
                                    rook_offsets[square_index] = index as u32;
                                }
                                RookOrBishop::Bishop => {
                                    bishop_offsets[square_index] = index as u32;
                                }
                            }
                            max_offset = max_offset.max(index + required_size);
                        } else {
                            // Append to end
                            let start = occupied.len();
                            occupied.resize(start + required_size, false);
                            occupied[start..].fill(true);
                            match piece_type {
                                RookOrBishop::Rook => {
                                    rook_offsets[square_index] = start as u32;
                                }
                                RookOrBishop::Bishop => {
                                    bishop_offsets[square_index] = start as u32;
                                }
                            }
                            max_offset = start + required_size;
                        }
                    }

                    // Update best length
                    let mut best = best_length.lock().unwrap();
                    if best.is_none() || max_offset < best.unwrap() {
                        *best = Some(max_offset);
                        let mut output = String::new();
                        out(&mut output, &bishop_magics, &bishop_offsets);
                        output.push_str("\n----------\n");
                        out(&mut output, &rook_magics, &rook_offsets);
                        println!("{output}\nNew best length found: {max_offset}");
                    }
                }
            })
        })
        .collect();

    if let Some(handle) = handles.into_iter().next() {
        handle.join().unwrap();
    }
}
