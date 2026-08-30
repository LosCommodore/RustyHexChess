#![allow(unused)]

use crate::{Side, piece::PieceType};
type Offset = (isize, isize);

// Name of Directions like in the pictures from:  doc/coordinates.drawio (first picture)

// Main axis:
const UP: Offset = (0, 1);
const DOWN: Offset = (0, -1);

// 1st diag:
const DOWN_RIGHT: Offset = (1, -1);
const UP_LEFT: Offset = (-1, 1);

// 2nd diag
const UP_RIGHT: Offset = (1, 0);
const DOWN_LEFT: Offset = (-1, 0);

// The edges
const EDGE_1: Offset = (1, 1);
const EDGE_2: Offset = (-1, -1);

const EDGE_3: Offset = (-1, 2);
const EDGE_4: Offset = (1, -2);

const EDGE_5: Offset = (2, -1);
const EDGE_6: Offset = (-2, 1);

#[derive(Clone)]
pub enum MovementPattern {
    Walk {
        direction: Offset,
        limit: Option<usize>,
    }, // Offset
    Step(&'static [Offset]),
    Pawn,
}

const ROOK_MOVEMENTS: &[MovementPattern] = &[
    MovementPattern::Walk {
        direction: UP,
        limit: None,
    },
    MovementPattern::Walk {
        direction: DOWN,
        limit: None,
    },
    MovementPattern::Walk {
        direction: UP_LEFT,
        limit: None,
    },
    MovementPattern::Walk {
        direction: DOWN_RIGHT,
        limit: None,
    },
    MovementPattern::Walk {
        direction: UP_RIGHT,
        limit: None,
    },
    MovementPattern::Walk {
        direction: DOWN_LEFT,
        limit: None,
    },
];

const KING_MOVEMENTS: &[MovementPattern] = &[MovementPattern::Step(&[
    UP, DOWN, UP_LEFT, DOWN_RIGHT, UP_RIGHT, DOWN_LEFT, EDGE_1, EDGE_2, EDGE_3, EDGE_4, EDGE_5,
    EDGE_6,
])];

const BISHOP_MOVEMENTS: &[MovementPattern] = &[
    MovementPattern::Walk {
        direction: EDGE_1,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_2,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_3,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_4,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_5,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_6,
        limit: None,
    },
];

const QUEEN_MOVEMENTS: &[MovementPattern] = &[
    MovementPattern::Walk {
        direction: UP,
        limit: None,
    },
    MovementPattern::Walk {
        direction: DOWN,
        limit: None,
    },
    MovementPattern::Walk {
        direction: UP_LEFT,
        limit: None,
    },
    MovementPattern::Walk {
        direction: DOWN_RIGHT,
        limit: None,
    },
    MovementPattern::Walk {
        direction: UP_RIGHT,
        limit: None,
    },
    MovementPattern::Walk {
        direction: DOWN_LEFT,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_1,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_2,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_3,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_4,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_5,
        limit: None,
    },
    MovementPattern::Walk {
        direction: EDGE_6,
        limit: None,
    },
];

const KNIGHT_MOVEMENTS: &[MovementPattern] = &[MovementPattern::Step(&[
    (2, 1),
    (1, 2),
    (3, -1),
    (3, -2),
    (1, -3),
    (2, -3),
    (-2, -1),
    (-1, -2),
    (-3, 1),
    (-3, 2),
    (-1, 3),
    (-2, 3),
])];

pub fn get_movement_patterns(piece_type: PieceType) -> &'static [MovementPattern] {
    use PieceType::*;

    match piece_type {
        King => KING_MOVEMENTS,
        Queen => QUEEN_MOVEMENTS,
        Rook => ROOK_MOVEMENTS,
        Bishop => BISHOP_MOVEMENTS,
        Pawn => &[MovementPattern::Pawn],
        Knight => KNIGHT_MOVEMENTS,
    }
}

pub const fn pawn_capture_moves(color: Side) -> &'static [(isize, isize); 2] {
    match color {
        Side::White => &[(1, 0), (-1, 1)],
        Side::Black => &[(-1, 0), (1, -1)],
    }
}
