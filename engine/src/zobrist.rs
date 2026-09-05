// Board states:
// 91 Fields * 8 Figures * 2 Colors + 1 en_passant + 1 active player

//   field 0 | piece 0 | color 0
//   field 0 | piece 0 | color 1
//   field 0 | piece 1 | color 0
//   ...

use strum::EnumCount;

use crate::{
    Side,
    board::Board,
    coordinates::{NR_FIELDS, Position},
    piece::{Piece, PieceType},
};

const NR_FIELD_VARIANTS: usize = PieceType::COUNT * Side::COUNT;
const NR_FIELD_KEYS: usize = NR_FIELD_VARIANTS * NR_FIELDS;

// One key per (field, piece, color), plus the two state keys at the end.
const NR_KEYS: usize = NR_FIELD_KEYS + 2;
const BLACKS_TURN: usize = NR_FIELD_KEYS;
const EN_PASSANT_POSSIBLE: usize = NR_FIELD_KEYS + 1;

// Any fixed value works; this one is arbitrary. Changing it invalidates every
// hash that was ever written down.
const SEED: u64 = 0x0DDB_1A5E_5BAD_5EED;

// The keys are the same in every process and every run, so hashes stay
// comparable across games, across restarts and across anything persisted to
// disk — a transposition table, an opening book, recorded self-play. Rolling
// them at runtime would make all of that meaningless.
static KEYS: [u64; NR_KEYS] = zobrist_keys(SEED);

pub struct Zobrist {
    hash: u64,
}

// splitmix64: a fixed pseudo-random stream, computed at compile time. Its
// output is well distributed enough for hash keys, which is all that is asked
// of it here.
const fn zobrist_keys<const N: usize>(seed: u64) -> [u64; N] {
    let mut keys = [0u64; N];
    let mut state = seed;
    let mut i = 0;

    while i < N {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        keys[i] = z ^ (z >> 31);

        i += 1;
    }

    keys
}

fn field_key(pos: &Position, piece: &Piece) -> u64 {
    let idx = pos.id() * NR_FIELD_VARIANTS
        + (piece.piece_type as usize) * Side::COUNT
        + piece.side as usize;
    KEYS[idx]
}

impl Zobrist {
    pub fn from_board(board: &Board, blacks_turn: bool, en_passant: bool) -> Self {
        Self {
            hash: Self::init_hash(board, blacks_turn, en_passant),
        }
    }

    fn init_hash(board: &Board, blacks_turn: bool, en_passant: bool) -> u64 {
        let mut hash = 0u64;
        for (pos, piece) in &board.pieces {
            hash ^= field_key(pos, piece);
        }
        if blacks_turn {
            hash ^= KEYS[BLACKS_TURN];
        }
        if en_passant {
            hash ^= KEYS[EN_PASSANT_POSSIBLE];
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // Two keys that are equal cancel each other out when they are XORed
    // together, so distinct positions would share a hash.
    #[test]
    fn keys_are_distinct() {
        let unique: HashSet<u64> = KEYS.iter().copied().collect();
        assert_eq!(unique.len(), NR_KEYS, "two keys are equal");
        assert!(!unique.contains(&0), "a zero key folds away when XORed");
    }

    // The keys are baked in at compile time; a change to the generator or the
    // seed is a change to every hash the engine has ever produced.
    #[test]
    fn keys_are_stable() {
        assert_eq!(KEYS[0], 0xe39e_7cca_5374_7b99);
        assert_eq!(KEYS[BLACKS_TURN], 0x0266_7c02_b843_afe0);
        assert_eq!(KEYS[EN_PASSANT_POSSIBLE], 0x7908_38e1_92b1_9bf0);
    }
}
