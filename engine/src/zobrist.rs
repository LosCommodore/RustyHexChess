// Board states:
// 91 Fields * 8 Figures * 2 Colors + 1 en_passant + 1 active player

//   field 0 | piece 0 | color 0
//   field 0 | piece 0 | color 1
//   field 0 | piece 1 | color 0
//   ...

use strum::EnumCount; // Import the Rng trait to access generation methods

use crate::{
    Side,
    board::Board,
    coordinates::{NR_FIELDS, Position},
    piece::{Piece, PieceType},
};

const NR_FIELD_VARIANTS: usize = PieceType::COUNT * Side::COUNT;

pub struct Zobrist {
    field_table: Box<[u64; NR_FIELD_VARIANTS * NR_FIELDS]>,
    blacks_turn: u64,
    en_passant_possible: u64,
    hash: u64,
}

impl Zobrist {
    pub fn from_board(board: &Board, blacks_turn: bool, en_passant: bool) -> Self {
        let mut field_table = Box::new([0u64; NR_FIELD_VARIANTS * NR_FIELDS]);
        for x in field_table.iter_mut() {
            *x = rand::random::<u64>();
        }

        let random_nr_blacks_turn = rand::random::<u64>();
        let random_nr_en_passant_possible = rand::random::<u64>();

        let me = Self {
            field_table,
            blacks_turn: random_nr_blacks_turn,
            en_passant_possible: random_nr_en_passant_possible,
            hash: 0,
        };

        me.init_hash(board, blacks_turn, en_passant);
        me
    }

    fn get_field_random_number(&self, pos: &Position, piece: &Piece) -> u64 {
        let idx = pos.id() * NR_FIELD_VARIANTS
            + (piece.piece_type as usize) * Side::COUNT
            + piece.side as usize;
        self.field_table[idx]
    }

    fn init_hash(&self, board: &Board, blacks_turn: bool, en_passant: bool) -> u64 {
        let mut hash = 0u64;
        for (pos, piece) in &board.pieces {
            hash ^= self.get_field_random_number(pos, piece);
        }
        if blacks_turn {
            hash ^= self.blacks_turn;
        }
        if en_passant {
            hash ^= self.en_passant_possible;
        }

        hash
    }

    pub fn update_piece(pos: &Position, piece: &Piece) {
        todo!()
    }

    pub fn update_active_player(side: Side) {
        todo!()
    }

    pub fn update_en_passant(side: Side) {
        todo!()
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}
