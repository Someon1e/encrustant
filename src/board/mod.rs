use core::fmt::Display;

pub mod bit_board;
pub mod piece;
pub mod square;

use bit_board::BitBoard;
use piece::{Piece, ALL_PIECES, BLACK_PIECES, WHITE_PIECES};
use square::Square;

use crate::move_generator::move_data::Move;

pub struct Board {
    bit_boards: [BitBoard; 12],

    pub white_to_move: bool,

    pub white_can_castle_king_side: bool,
    pub black_can_castle_queen_side: bool,

    pub black_can_castle_king_side: bool,
    pub white_can_castle_queen_side: bool,

    pub en_passant_square: Option<Square>,

    pub half_move_clock: u64,
    pub full_move_counter: u64,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl Board {
    pub fn from_fen(fen: &str) -> Self {
        let mut bit_boards = [BitBoard::empty(); 12];

        // rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR
        let (mut rank, mut file) = (7, 0);

        let mut characters = fen.chars().peekable();

        for character in characters.by_ref() {
            if character == '/' {
                continue;
            }
            if let Some(digit) = character.to_digit(10) {
                file += digit as i8;
            } else {
                let piece = piece::from_fen_char(&character).expect("{square} {character}");
                bit_boards[piece as usize].set(&Square::from_coords(rank, file));
                file += 1;
            }
            if file == 8 {
                if rank == 0 {
                    break
                }
                rank -= 1;
                file = 0;
            }
        }

        let state = characters.collect::<String>();
        let mut split = state.split_whitespace();

        let white_to_move = match split.next().expect("Missing w/b to move") {
            "w" => true,
            "b" => false,
            _ => panic!("No w/b to move"),
        };

        let castling_rights = split.next().expect("Missing castling rights");
        let (
            white_can_castle_king_side,
            white_can_castle_queen_side,
            black_can_castle_king_side,
            black_can_castle_queen_side,
        ) = if castling_rights == "-" {
            (false, false, false, false)
        } else {
            (
                castling_rights.contains('K'),
                castling_rights.contains('Q'),
                castling_rights.contains('k'),
                castling_rights.contains('q'),
            )
        };

        let mut en_passant = split.next().expect("Missing en passant").chars().peekable();
        let en_passant_square = if *en_passant.peek().expect("Missing en passant") == '-' {
            None
        } else {
            let file = en_passant.next().expect("Missing en passant") as u8 - b'a';
            let rank = en_passant
                .next()
                .expect("Missing en passant")
                .to_digit(10)
                .unwrap() as u8
                - 1;
            Some(Square::from_coords(rank as i8, file as i8))
        };
        let half_move_clock = split
            .next()
            .expect("No half move clock")
            .parse()
            .expect("No half move clock");
        let full_move_counter = split
            .next()
            .expect("No full move counter")
            .parse()
            .expect("No full move counter");

        Self {
            bit_boards,

            white_to_move,

            white_can_castle_king_side,
            black_can_castle_queen_side,

            black_can_castle_king_side,
            white_can_castle_queen_side,

            en_passant_square,

            half_move_clock,
            full_move_counter,
        }
    }
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        for piece in ALL_PIECES {
            let bit_board = self.bit_boards[piece as usize];
            if bit_board.get(&square) {
                return Some(piece);
            }
        }
        None
    }
    pub fn white_piece_at(&self, square: Square) -> Option<Piece> {
        for piece in WHITE_PIECES {
            let bit_board = self.bit_boards[piece as usize];
            if bit_board.get(&square) {
                return Some(piece);
            }
        }
        None
    }
    pub fn black_piece_at(&self, square: Square) -> Option<Piece> {
        for piece in BLACK_PIECES {
            let bit_board = self.bit_boards[piece as usize];
            if bit_board.get(&square) {
                return Some(piece);
            }
        }
        None
    }
    pub fn make_move(&mut self, move_data: &Move) {
        let moving_bit_board = &mut self.bit_boards[move_data.piece() as usize];
        moving_bit_board.unset(&move_data.from());
        moving_bit_board.set(&move_data.to());
        if let Some(captured) = move_data.capture() {
            let capturing_bit_board = &mut self.bit_boards[captured as usize];
            capturing_bit_board.unset(&move_data.to())
        }

        self.white_to_move = !self.white_to_move;
    }
    pub fn unmake_move(&mut self, move_data: &Move) {
        let bit_board = &mut self.bit_boards[move_data.piece() as usize];
        bit_board.unset(&move_data.to());
        bit_board.set(&move_data.from());
        if let Some(captured) = move_data.capture() {
            let capturing_bit_board = &mut self.bit_boards[captured as usize];
            capturing_bit_board.set(&move_data.to())
        }

        self.white_to_move = !self.white_to_move
    }
    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(87);

        let mut empty: u32 = 0;
        for rank in (0..8).rev() {
            for file in 0..8 {
                if let Some(piece) = self.piece_at(Square::from_coords(rank, file)) {
                    if empty != 0 {
                        fen.push(char::from_digit(empty, 10).unwrap());
                        empty = 0;
                    }
                    fen.push(piece.to_fen_char());
                } else {
                    empty += 1
                }
            }
            if empty != 0 {
                fen.push(char::from_digit(empty, 10).unwrap());
                empty = 0;
            }
            if rank != 0 {
                fen.push('/')
            }
        }

        if self.white_to_move {
            fen.push_str(" w ")
        } else {
            fen.push_str(" b ")
        }

        if self.white_can_castle_king_side
            || self.white_can_castle_queen_side
            || self.black_can_castle_king_side
            || self.black_can_castle_queen_side
        {
            if self.white_can_castle_king_side {
                fen.push('K')
            }
            if self.white_can_castle_queen_side {
                fen.push('Q')
            }
            if self.black_can_castle_king_side {
                fen.push('k')
            }
            if self.black_can_castle_queen_side {
                fen.push('q')
            }
        } else {
            fen.push('-')
        }
        fen.push(' ');

        if let Some(en_passant_square) = &self.en_passant_square {
            fen.push_str(&en_passant_square.to_notation())
        } else {
            fen.push('-')
        }

        fen.push(' ');
        fen.push_str(&self.half_move_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.full_move_counter.to_string());

        fen
    }
}
