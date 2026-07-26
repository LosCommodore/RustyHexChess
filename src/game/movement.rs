#![allow(unused)]

use crate::game::piece::PieceType;
type Offset = (isize, isize);

// Name of Directions like in the pictures from:  https://en.wikipedia.org/wiki/Hexagonal_chess

// Main axis:
const UP: Offset = (0, -1);
const DOWN: Offset = (0, 1);

// 2nd coordinate:
const DOWN_RIGHT: Offset = (1,-1);
const UP_LEFT: Offset = (-1, 0);

// both
const UP_RIGHT: Offset = (1, 0);
const DOWN_LEFT: Offset = (-1, 0);


#[derive(Clone)]
pub enum MovementPattern {
    Walk(Offset),
    Step(Vec<Offset>),
    Jump(Vec<Offset>),
}

#[allow(unused)]
const ROOK_MOVEMENTS: &'static [MovementPattern] = &[
    MovementPattern::Walk(UP),
    MovementPattern::Walk(DOWN),
    MovementPattern::Walk(UP_LEFT),
    MovementPattern::Walk(UP_RIGHT),
    MovementPattern::Walk(DOWN_LEFT),
    MovementPattern::Walk(DOWN_RIGHT),
];

const KING_MOVEMENTS: &'static [MovementPattern] = &[
    MovementPattern::Walk(UP),
    //MovementPattern::Walk(LEFT),
    //MovementPattern::Walk(RIGHT),
    //MovementPattern::Walk(DOWN),
];

pub fn get_movement_patterns(piece_type: PieceType) -> &'static [MovementPattern] {
    use PieceType::*;

    match piece_type {
        King => KING_MOVEMENTS,
        Queen => todo!(),
        Rook => ROOK_MOVEMENTS,
        Bishop => todo!(),
        Pawn => todo!(),
        Knight => todo!(),
    }
}
