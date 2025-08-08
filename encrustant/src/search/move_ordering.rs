use core::mem::MaybeUninit;

use crate::{
    evaluation::eval_data::Score,
    move_generator::{
        MoveGenerator,
        move_data::{Flag, Move},
    },
    search::Ply,
};

use super::{Search, encoded_move::EncodedMove};

pub struct ContinuationHistory(
    // [previous_piece][previous_to][current_piece][current_to]
    Box<[[[[i16; 64]; 6]; 64]; 12]>,
);
impl ContinuationHistory {
    pub fn new() -> Self {
        Self(vec![[[[0; 64]; 6]; 64]; 12].try_into().unwrap())
    }

    pub fn fill(&mut self, value: i16) {
        for x in self.0.iter_mut() {
            for y in x {
                for z in y {
                    z.fill(value);
                }
            }
        }
    }

    pub fn get(
        &self,
        previous_piece: usize,
        previous_to: usize,
        current_piece: usize,
        current_to: usize,
    ) -> i16 {
        self.0[previous_piece][previous_to][current_piece][current_to]
    }

    pub fn get_mut(
        &mut self,
        previous_piece: usize,
        previous_to: usize,
        current_piece: usize,
        current_to: usize,
    ) -> &mut i16 {
        &mut self.0[previous_piece][previous_to][current_piece][current_to]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CorrectionHistoryEntry(pub i16);

impl CorrectionHistoryEntry {
    const CORRECTION_HISTORY_WEIGHT_SCALE: i16 = 1024;
    const CORRECTION_HISTORY_MAX: i16 = 16384;

    pub fn update(&mut self, ply_remaining: Ply, scaled_error: Score) {
        let new_weight = i32::min(
            i32::from(ply_remaining) * i32::from(ply_remaining) + 2 * i32::from(ply_remaining) + 1,
            128,
        );
        assert!(new_weight <= i32::from(Self::CORRECTION_HISTORY_WEIGHT_SCALE));

        let new_value = (i32::from(self.0)
            * (i32::from(Self::CORRECTION_HISTORY_WEIGHT_SCALE) - new_weight)
            + scaled_error * new_weight)
            / i32::from(Self::CORRECTION_HISTORY_WEIGHT_SCALE);
        let clamped = i32::clamp(
            new_value,
            i32::from(-Self::CORRECTION_HISTORY_MAX),
            i32::from(Self::CORRECTION_HISTORY_MAX),
        );

        *self = Self(clamped as i16);
    }
}

pub struct CorrectionHistory<const LEN: usize>([[CorrectionHistoryEntry; LEN]; 2]);

impl<const LEN: usize> CorrectionHistory<LEN> {
    pub fn new() -> Self {
        Self(
            vec![[CorrectionHistoryEntry(0); LEN]; 2]
                .try_into()
                .unwrap(),
        )
    }

    pub fn get(&self, white_to_move: bool, index: usize) -> CorrectionHistoryEntry {
        self.0[usize::from(white_to_move)][index]
    }

    pub fn get_mut(&mut self, white_to_move: bool, index: usize) -> &mut CorrectionHistoryEntry {
        &mut self.0[usize::from(white_to_move)][index]
    }

    pub fn fill(&mut self, value: i16) {
        self.0[0].fill(CorrectionHistoryEntry(value));
        self.0[1].fill(CorrectionHistoryEntry(value));
    }
}

pub type MoveGuessNum = i32;

#[derive(Clone, Copy)]
pub struct MoveGuess {
    guess: MoveGuessNum,
    pub move_data: EncodedMove,
}

const MAX_LEGAL_MOVES: usize = 218;
const MAX_CAPTURES: usize = 74;

const HASH_MOVE_BONUS: MoveGuessNum = MoveGuessNum::MAX;
const CAPTURE_BONUS: MoveGuessNum = 50_000_000;
const KILLER_MOVE_BONUS: MoveGuessNum = 40_000_000;
const QUEEN_PROMOTION_BONUS: MoveGuessNum = 30_000_000;
const KNIGHT_PROMOTION_BONUS: MoveGuessNum = 20_000_000;
const ROOK_PROMOTION_BONUS: MoveGuessNum = 0;
const BISHOP_PROMOTION_BONUS: MoveGuessNum = 0;

const CAPTURING_SCORE: [i32; 12] = {
    const SCALE: i32 = 500;

    const PAWN: i32 = 10 * SCALE;
    const KNIGHT: i32 = 30 * SCALE;
    const BISHOP: i32 = 31 * SCALE;
    const ROOK: i32 = 50 * SCALE;
    const QUEEN: i32 = 90 * SCALE;

    // Should not be possible
    const KING: i32 = 0;

    [
        PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING,
    ]
};

pub struct MoveOrderer;
impl MoveOrderer {
    fn guess_move_value(search: &Search, move_data: Move, ply_from_root: Ply) -> MoveGuessNum {
        let moving_from = move_data.from;
        let moving_to = move_data.to;

        match move_data.flag {
            Flag::EnPassant | Flag::Castle | Flag::PawnTwoUp => {
                return MoveGuessNum::from(
                    search.quiet_history[usize::from(search.board.white_to_move)]
                        [moving_from.usize() + moving_to.usize() * 64],
                );
            }

            Flag::BishopPromotion => return BISHOP_PROMOTION_BONUS,
            Flag::KnightPromotion => return KNIGHT_PROMOTION_BONUS,
            Flag::RookPromotion => return ROOK_PROMOTION_BONUS,
            Flag::QueenPromotion => return QUEEN_PROMOTION_BONUS,

            Flag::None => {}
        }

        let mut score = 0;
        let moving_piece = search.board.friendly_piece_at(moving_from).unwrap();

        // This won't consider en passant
        if let Some(capturing) = search.board.enemy_piece_at(moving_to) {
            score += CAPTURE_BONUS;
            score += MoveGuessNum::from(CAPTURING_SCORE[capturing as usize]);

            score += i32::from(
                search.capture_history[moving_piece as usize][moving_to.usize()][if search
                    .board
                    .white_to_move
                {
                    capturing as usize - 6
                } else {
                    capturing as usize
                }],
            );
        } else {
            score += MoveGuessNum::from(
                search.quiet_history[usize::from(search.board.white_to_move)]
                    [moving_from.usize() + moving_to.usize() * 64],
            );

            if ply_from_root != 0 {
                let previous_to = search.continuation_indices[(ply_from_root - 1) as usize]
                    .1
                    .usize();
                let previous_piece =
                    search.continuation_indices[(ply_from_root - 1) as usize].0 as usize;

                score += MoveGuessNum::from(search.continuation_history.get(
                    previous_piece,
                    previous_to,
                    if search.board.white_to_move {
                        moving_piece as usize
                    } else {
                        moving_piece as usize - 6
                    },
                    moving_to.usize(),
                ));
            }
        }
        score
    }

    /// # Safety
    /// It is up to the caller to guarantee that `move_guesses[unsorted_index..move_count]` are initialised.
    pub unsafe fn put_highest_guessed_move(
        move_guesses: &mut [MaybeUninit<MoveGuess>],
        unsorted_index: usize,
        move_count: usize,
    ) -> MoveGuess {
        let (mut index_of_highest_move, mut highest_guess) = (
            unsorted_index,
            unsafe { move_guesses[unsorted_index].assume_init() }.guess,
        );

        // Find highest guessed unsorted move
        for (index, item) in move_guesses
            .iter()
            .enumerate()
            .take(move_count)
            .skip(unsorted_index)
        {
            // Iterate part of the array that is unsorted
            let guess = unsafe { item.assume_init() }.guess;
            if guess > highest_guess {
                // New highest guess
                highest_guess = guess;
                index_of_highest_move = index;
            }
        }

        if index_of_highest_move != unsorted_index {
            // Swap highest with first unsorted
            move_guesses.swap(index_of_highest_move, unsorted_index);
        }

        unsafe { move_guesses[unsorted_index].assume_init() }
    }

    fn guess_capture_value(search: &Search, move_data: Move) -> MoveGuessNum {
        let mut score = match move_data.flag {
            Flag::EnPassant => return 0,
            Flag::BishopPromotion => return -1,
            Flag::RookPromotion => return -1,

            Flag::KnightPromotion => 1300,
            Flag::QueenPromotion => 1900,

            Flag::None => 0,

            _ => unreachable!(),
        };

        let capturing = search.board.enemy_piece_at(move_data.to).unwrap();
        let moving_piece = search.board.friendly_piece_at(move_data.from).unwrap();

        macro_rules! repeat_array {
            // Base case: When no elements are left to process
            (@internal [$($acc:expr),*] []) => {
                [$($acc),*]
            };

            // Recursive case: Process the first element and recurse on the rest
            (@internal [$($acc:expr),*] [$head:expr $(, $tail:expr)*]) => {
                repeat_array!(@internal [$($acc,)* $head] [$($tail),*])
            };

            // Entry point: Duplicate the array by calling the internal rule twice
            ([$($arr:expr),*]) => {
                repeat_array!(@internal [$($arr),*] [$($arr),*])
            };
        }

        const MVV_LVA_PAWN: [u8; 12] = repeat_array!([15, 14, 13, 12, 11, 10]); // Victim P > Attacker P, N, B, R, Q, K
        const MVV_LVA_KNIGHT: [u8; 12] = repeat_array!([25, 24, 23, 22, 21, 20]); // Victim N > Attacker P, N, B, R, Q, K
        const MVV_LVA_BISHOP: [u8; 12] = repeat_array!([35, 34, 33, 32, 31, 30]); // Victim B > Attacker P, N, B, R, Q, K
        const MVV_LVA_ROOK: [u8; 12] = repeat_array!([45, 44, 43, 42, 41, 40]); // Victim R > Attacker P, N, B, R, Q, K
        const MVV_LVA_QUEEN: [u8; 12] = repeat_array!([55, 54, 53, 52, 51, 50]); // Victim Q > Attacker P, N, B, R, Q, K
        const MVV_LVA_KING: [u8; 12] = repeat_array!([0, 0, 0, 0, 0, 0]); // Victim K > Attacker P, N, B, R, Q, K
        const MVV_LVA: [[u8; 12]; 12] = [
            MVV_LVA_PAWN,
            MVV_LVA_KNIGHT,
            MVV_LVA_BISHOP,
            MVV_LVA_ROOK,
            MVV_LVA_QUEEN,
            MVV_LVA_KING,
            MVV_LVA_PAWN,
            MVV_LVA_KNIGHT,
            MVV_LVA_BISHOP,
            MVV_LVA_ROOK,
            MVV_LVA_QUEEN,
            MVV_LVA_KING,
        ];
        score += MoveGuessNum::from(MVV_LVA[capturing as usize][moving_piece as usize]);

        score
    }

    pub fn get_move_guesses_captures_only(
        search: &Search,
        move_generator: &MoveGenerator,
    ) -> ([MaybeUninit<MoveGuess>; MAX_CAPTURES], usize) {
        let mut move_guesses = [MaybeUninit::uninit(); MAX_CAPTURES];

        let mut index = 0;
        move_generator.generate(
            &mut |move_data| {
                let encoded = EncodedMove::new(move_data);
                move_guesses[index].write(MoveGuess {
                    move_data: encoded,
                    guess: Self::guess_capture_value(search, move_data),
                });
                index += 1;
            },
            true,
        );

        (move_guesses, index)
    }

    pub fn get_move_guesses(
        search: &Search,
        move_generator: &MoveGenerator,
        hash_move: EncodedMove,
        killer_move: EncodedMove,
        ply_from_root: Ply,
    ) -> ([MaybeUninit<MoveGuess>; MAX_LEGAL_MOVES], usize) {
        let mut move_guesses = [MaybeUninit::uninit(); MAX_LEGAL_MOVES];

        let mut index = 0;
        move_generator.generate(
            &mut |move_data| {
                let encoded = EncodedMove::new(move_data);

                let guess = if encoded == hash_move {
                    HASH_MOVE_BONUS
                } else if encoded == killer_move {
                    KILLER_MOVE_BONUS
                } else {
                    Self::guess_move_value(search, move_data, ply_from_root)
                };

                move_guesses[index].write(MoveGuess {
                    move_data: encoded,
                    guess,
                });
                index += 1;
            },
            false,
        );

        (move_guesses, index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, square::Square},
        move_generator::{
            MoveGenerator,
            move_data::{Flag, Move},
        },
        search::{
            Search, encoded_move::EncodedMove, move_ordering::MoveOrderer,
            transposition::megabytes_to_capacity,
        },
    };

    #[test]
    fn move_ordering_works() {
        let board = Board::from_fen("8/P6p/6r1/1q1n4/2P3R1/8/2K2k2/8 w - - 0 1").unwrap();
        let move_generator = MoveGenerator::new(&board);

        let (mut move_guesses, move_count) = MoveOrderer::get_move_guesses(
            &Search::new(
                board,
                megabytes_to_capacity(8),
                #[cfg(feature = "spsa")]
                crate::search::search_params::DEFAULT_TUNABLES,
            ),
            &move_generator,
            EncodedMove::NONE,
            EncodedMove::NONE,
            0,
        );

        let mut index = 0;
        let mut next_move = || {
            let move_guess = unsafe {
                MoveOrderer::put_highest_guessed_move(&mut move_guesses, index, move_count)
            };
            println!("{index} {} {}", move_guess.move_data, move_guess.guess);
            index += 1;
            (move_guess.move_data, index != move_count)
        };

        assert!(
            next_move().0.decode()
                == Move {
                    from: Square::from_notation("c4").unwrap(),
                    to: Square::from_notation("b5").unwrap(),
                    flag: Flag::None
                }
        );
        while next_move().1 {}
    }
}
