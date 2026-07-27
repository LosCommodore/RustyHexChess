#![allow(unused)]

use crate::game::piece::PieceType;
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
    Walk(Offset),
    Step(&'static [Offset]),
    Jump(&'static [Offset]),
}

const ROOK_MOVEMENTS: &'static [MovementPattern] = &[
    MovementPattern::Walk(UP),
    MovementPattern::Walk(DOWN),
    MovementPattern::Walk(UP_LEFT),
    MovementPattern::Walk(DOWN_RIGHT),
    MovementPattern::Walk(UP_RIGHT),
    MovementPattern::Walk(DOWN_LEFT),
];

const KING_MOVEMENTS: &'static [MovementPattern] = &[MovementPattern::Step(&[
    UP, DOWN, UP_LEFT, DOWN_RIGHT, UP_RIGHT, DOWN_LEFT, EDGE_1, EDGE_2, EDGE_3, EDGE_4, EDGE_5,
    EDGE_6,
])];

const BISHOP_MOVEMENTS: &'static [MovementPattern] = &[
    MovementPattern::Walk(EDGE_1),
    MovementPattern::Walk(EDGE_2),
    MovementPattern::Walk(EDGE_3),
    MovementPattern::Walk(EDGE_4),
    MovementPattern::Walk(EDGE_5),
    MovementPattern::Walk(EDGE_6),
];

pub fn get_movement_patterns(piece_type: PieceType) -> &'static [MovementPattern] {
    use PieceType::*;

    match piece_type {
        King => KING_MOVEMENTS,
        Queen => todo!(),
        Rook => ROOK_MOVEMENTS,
        Bishop => BISHOP_MOVEMENTS,
        Pawn => todo!(),
        Knight => todo!(),
    }
}
