#![allow(unused)]

// Board states:
// 91 Fields * 6 Figures * 2 Colors + 91 en_passant + 1 active player

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

// One key per (field, piece, color), then one per en-passant target field, then
// the single side-to-move key. A boolean "en passant is possible" would not do:
// two positions whose only difference is *which* field can be captured on would
// then share a hash, and a repetition would be claimed that never happened.
const EN_PASSANT: usize = NR_FIELD_KEYS;
const BLACKS_TURN: usize = EN_PASSANT + NR_FIELDS;
const NR_KEYS: usize = BLACKS_TURN + 1;

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

fn en_passant_key(target: &Position) -> u64 {
    KEYS[EN_PASSANT + target.id()]
}

impl Zobrist {
    /// `en_passant` is the field a pawn may be captured *on* this turn — the one
    /// the double-stepping pawn skipped over — and it must be `None` unless a
    /// pawn of the side to move can really capture there. A double step nobody
    /// can answer leaves the position identical to the same placement reached
    /// any other way, so folding its field in regardless would hide repetitions.
    pub fn from_board(board: &Board, blacks_turn: bool, en_passant: Option<Position>) -> Self {
        Self {
            hash: Self::init_hash(board, blacks_turn, en_passant),
        }
    }

    fn init_hash(board: &Board, blacks_turn: bool, en_passant: Option<Position>) -> u64 {
        let mut hash = 0u64;
        for (pos, piece) in &board.pieces {
            hash ^= field_key(pos, piece);
        }
        if blacks_turn {
            hash ^= KEYS[BLACKS_TURN];
        }
        if let Some(target) = en_passant {
            hash ^= en_passant_key(&target);
        }

        hash
    }

    /// Puts a piece on a field, or takes it off again — the same call does both,
    /// because XOR is its own inverse. A piece that moves is two calls: one on
    /// the field it leaves, one on the field it arrives at. A capture is a third
    /// one for the piece that comes off, on *its* field, which for en passant is
    /// not the field the capturing pawn ends up on. A promotion is the pawn off
    /// the field and the new piece onto it.
    ///
    /// Only the caller knows whether a field is being vacated or filled, so a
    /// call in the wrong place goes unnoticed here and shows up as a hash that
    /// drifts away from the position.
    pub fn update_piece(&mut self, pos: &Position, piece: &Piece) {
        self.hash ^= field_key(pos, piece);
    }

    /// Hands the turn to the other side. There is one key for this, folded in
    /// while Black is to move and out again when it is White's turn, so this
    /// takes no side: calling it is the change of turn itself.
    pub fn update_active_player(&mut self) {
        self.hash ^= KEYS[BLACKS_TURN];
    }

    /// Takes the previously available en-passant field back out of the hash and
    /// folds the new one in. Both are `None` when no capture is available; see
    /// [`Zobrist::from_board`] for what counts as available.
    ///
    /// Undo needs no separate path: XOR is its own inverse, so replaying the
    /// same two fields the other way round restores the old hash exactly.
    pub fn update_en_passant(&mut self, old: Option<Position>, new: Option<Position>) {
        if old == new {
            return;
        }
        if let Some(old) = old {
            self.hash ^= en_passant_key(&old);
        }
        if let Some(new) = new {
            self.hash ^= en_passant_key(&new);
        }
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
        assert_eq!(KEYS[EN_PASSANT], 0x0266_7c02_b843_afe0);
        assert_eq!(KEYS[BLACKS_TURN], 0x1283_1467_3344_4768);
    }

    fn two_kings() -> Board {
        let mut board = Board::default();
        board.pieces.insert(
            Position::from_human(('F', 5)).unwrap(),
            Piece::new(PieceType::King, Side::White),
        );
        board.pieces.insert(
            Position::from_human(('A', 11)).unwrap(),
            Piece::new(PieceType::King, Side::Black),
        );
        board
    }

    // The whole point of a key per field: which field the capture is available
    // on is part of the position, so those two positions must not collide.
    #[test]
    fn en_passant_field_changes_the_hash() {
        let board = two_kings();
        let one = Position::from_human(('J', 2)).unwrap();
        let other = Position::from_human(('B', 6)).unwrap();

        let plain = Zobrist::from_board(&board, false, None);
        let on_one = Zobrist::from_board(&board, false, Some(one));
        let on_other = Zobrist::from_board(&board, false, Some(other));

        assert_ne!(plain.hash(), on_one.hash());
        assert_ne!(plain.hash(), on_other.hash());
        assert_ne!(on_one.hash(), on_other.hash());
    }

    // Moving the availability from one field to another, and back off again,
    // has to land on exactly the hash the position started with.
    #[test]
    fn en_passant_update_matches_a_fresh_hash() {
        let board = two_kings();
        let one = Position::from_human(('J', 2)).unwrap();
        let other = Position::from_human(('B', 6)).unwrap();

        let mut hash = Zobrist::from_board(&board, false, None);
        let plain = hash.hash();

        hash.update_en_passant(None, Some(one));
        assert_eq!(
            hash.hash(),
            Zobrist::from_board(&board, false, Some(one)).hash()
        );

        hash.update_en_passant(Some(one), Some(other));
        assert_eq!(
            hash.hash(),
            Zobrist::from_board(&board, false, Some(other)).hash()
        );

        hash.update_en_passant(Some(other), None);
        assert_eq!(hash.hash(), plain, "the field did not come back out");
    }

    // A move updated piece by piece has to arrive at the hash the resulting
    // position gets when it is hashed from scratch, and playing the same calls
    // a second time has to undo it.
    #[test]
    fn a_move_updates_to_the_same_hash_as_a_fresh_one() {
        let before = two_kings();
        let origin = Position::from_human(('F', 5)).unwrap();
        let destination = Position::from_human(('F', 6)).unwrap();
        let king = Piece::new(PieceType::King, Side::White);

        let mut after = before.clone();
        let moved = after.pieces.remove(&origin).expect("no king at the origin");
        after.pieces.insert(destination, moved);

        let mut hash = Zobrist::from_board(&before, false, None);
        let white_to_move = hash.hash();

        hash.update_piece(&origin, &king);
        hash.update_piece(&destination, &king);
        hash.update_active_player();

        assert_eq!(hash.hash(), Zobrist::from_board(&after, true, None).hash());

        // The same three calls again, in any order, take the move back.
        hash.update_piece(&destination, &king);
        hash.update_piece(&origin, &king);
        hash.update_active_player();

        assert_eq!(hash.hash(), white_to_move, "the move did not come back out");
    }

    // The side to move is part of the position: the same placement with the
    // other player on turn is a different position, not a repetition of it.
    #[test]
    fn the_side_to_move_changes_the_hash() {
        let board = two_kings();

        let white = Zobrist::from_board(&board, false, None);
        let black = Zobrist::from_board(&board, true, None);

        assert_ne!(white.hash(), black.hash());
    }
}
