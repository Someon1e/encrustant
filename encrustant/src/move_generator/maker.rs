use crate::board::{Board, game_state::GameState, piece::Piece, square::Square};

use super::move_data::{Flag, Move};

impl Board {
    /// # Panics
    ///
    /// Will panic if there is no friendly piece at `from`.
    /// Will panic if it is en passant and `self.game_state.en_passant_square` is `None`.
    pub fn make_move(&mut self, move_data: &Move) -> GameState {
        let old_state = self.game_state;

        let white_to_move = self.white_to_move;

        let piece = self.friendly_piece_at(move_data.from).unwrap();

        let flag = move_data.flag;

        if piece == Piece::WhiteKing {
            self.game_state.castling_rights.unset_white_king_side();
            self.game_state.castling_rights.unset_white_queen_side();
        } else if piece == Piece::BlackKing {
            self.game_state.castling_rights.unset_black_king_side();
            self.game_state.castling_rights.unset_black_queen_side();
        }
        if move_data.from == Square::from_index(0) || move_data.to == Square::from_index(0) {
            self.game_state.castling_rights.unset_white_queen_side();
        }
        if move_data.from == Square::from_index(7) || move_data.to == Square::from_index(7) {
            self.game_state.castling_rights.unset_white_king_side();
        }
        if move_data.from == Square::from_index(56) || move_data.to == Square::from_index(56) {
            self.game_state.castling_rights.unset_black_queen_side();
        }
        if move_data.from == Square::from_index(63) || move_data.to == Square::from_index(63) {
            self.game_state.castling_rights.unset_black_king_side();
        }

        let promotion_piece = flag.get_promotion_piece(white_to_move);

        let moving_bit_board = self.get_bit_board_mut(piece);

        if let Some(promotion_piece) = promotion_piece {
            moving_bit_board.toggle(&move_data.from);
            self.get_bit_board_mut(promotion_piece).set(&move_data.to);
        } else {
            moving_bit_board.toggle_two(&move_data.from, &move_data.to);
        }

        let en_passant_square = self.game_state.en_passant_square;

        self.game_state.en_passant_square = None;

        match flag {
            Flag::PawnTwoUp => {
                let en_passant_square = move_data.from.up(if self.white_to_move { 1 } else { -1 });
                self.game_state.en_passant_square = Some(en_passant_square);
                self.game_state.captured = None;
            }
            Flag::Castle => {
                let is_king_side = move_data.to.file() == 6;
                let rook_to_offset = if is_king_side { -1 } else { 1 };
                let rook_from_offset = if is_king_side { 1 } else { -2 };
                let rook = if white_to_move {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                };
                let rook_bit_board = self.get_bit_board_mut(rook);
                let rook_from = &move_data.to.offset(rook_from_offset);
                let rook_to = &move_data.to.offset(rook_to_offset);
                rook_bit_board.toggle_two(rook_from, rook_to);
            }
            Flag::EnPassant => {
                let capture_position =
                    en_passant_square
                        .unwrap()
                        .down(if self.white_to_move { 1 } else { -1 });
                let captured = if white_to_move {
                    Piece::BlackPawn
                } else {
                    Piece::WhitePawn
                };
                self.game_state.captured = Some(captured);

                let capturing_bit_board = self.get_bit_board_mut(captured);
                capturing_bit_board.toggle(&capture_position);
            }
            _ => {
                self.game_state.captured = self.enemy_piece_at(move_data.to);
                if let Some(captured) = self.game_state.captured {
                    let capturing_bit_board = self.get_bit_board_mut(captured);
                    capturing_bit_board.toggle(&move_data.to);
                }
            }
        }

        self.white_to_move = !white_to_move;

        old_state
    }

    /// # Panics
    ///
    /// Will panic if there is no friendly piece at `move_data.to`.
    /// Will panic if it is en passant and `self.game_state.captured` is `None`.
    pub fn unmake_move(&mut self, move_data: &Move, old_state: &GameState) {
        let capture = self.game_state.captured;
        self.game_state = *old_state;

        let white_to_move = !self.white_to_move;
        self.white_to_move = white_to_move;

        let flag = move_data.flag;
        match flag {
            Flag::None => {
                let moving_bit_board =
                    self.get_bit_board_mut(self.friendly_piece_at(move_data.to).unwrap());
                moving_bit_board.toggle_two(&move_data.from, &move_data.to);

                if let Some(capture) = capture {
                    let capturing_bit_board = self.get_bit_board_mut(capture);
                    capturing_bit_board.set(&move_data.to);
                }
            }

            Flag::PawnTwoUp => {
                let moving_bit_board = self.get_bit_board_mut(if white_to_move {
                    Piece::WhitePawn
                } else {
                    Piece::BlackPawn
                });
                moving_bit_board.toggle_two(&move_data.from, &move_data.to);
            }

            Flag::RookPromotion
            | Flag::BishopPromotion
            | Flag::KnightPromotion
            | Flag::QueenPromotion => {
                let moving_bit_board = self.get_bit_board_mut(if white_to_move {
                    Piece::WhitePawn
                } else {
                    Piece::BlackPawn
                });
                moving_bit_board.set(&move_data.from);
                self.get_bit_board_mut(flag.get_promotion_piece(white_to_move).unwrap())
                    .toggle(&move_data.to);

                if let Some(capture) = capture {
                    let capturing_bit_board = self.get_bit_board_mut(capture);
                    capturing_bit_board.set(&move_data.to);
                }
            }

            Flag::EnPassant => {
                let capture_position = {
                    self.game_state
                        .en_passant_square
                        .unwrap()
                        .down(if white_to_move { 1 } else { -1 })
                };
                let capturing_bit_board = self.get_bit_board_mut(capture.unwrap());
                capturing_bit_board.set(&capture_position);

                let moving_bit_board = self.get_bit_board_mut(if white_to_move {
                    Piece::WhitePawn
                } else {
                    Piece::BlackPawn
                });
                moving_bit_board.toggle_two(&move_data.from, &move_data.to);
            }

            Flag::Castle => {
                let is_king_side = move_data.to.file() == 6;
                let rook_to_offset = if is_king_side { -1 } else { 1 };
                let rook_from_offset = if is_king_side { 1 } else { -2 };
                let rook_bit_board = if white_to_move {
                    self.get_bit_board_mut(Piece::WhiteRook)
                } else {
                    self.get_bit_board_mut(Piece::BlackRook)
                };
                rook_bit_board.toggle_two(
                    &move_data.to.offset(rook_from_offset),
                    &move_data.to.offset(rook_to_offset),
                );

                let moving_bit_board = self.get_bit_board_mut(if white_to_move {
                    Piece::WhiteKing
                } else {
                    Piece::BlackKing
                });
                moving_bit_board.toggle_two(&move_data.from, &move_data.to);
            }
        }
    }
}
