#![allow(unused)]

use crate::game::piece::PieceType;
type Offset = (isize, isize);

/*
const UP_LEFT: Offset = (-1, -1);
const UP_RIGHT: Offset = (-1, 1);
const DOWN_LEFT: Offset = (1, -1);
const DOWN_RIGHT: Offset = (1, 1);
 */


 
const UP: Offset = (-1, 0);
const DOWN: Offset = (1, 0);
const LEFT: Offset = (0, -1);
const RIGHT: Offset = (0, 1);

#[derive(Clone)]
pub enum MovementPattern {
    Walk(Offset),
    Step(Vec<Offset>),
    Jump(Vec<Offset>),
}


#[allow(unused)]
const ROOK_MOVEMENTS: &'static [MovementPattern] = &[
    MovementPattern::Walk(UP),
    MovementPattern::Walk(LEFT),
    MovementPattern::Walk(RIGHT),
    MovementPattern::Walk(DOWN),
];

const KING_MOVEMENTS: &'static [MovementPattern] = &[
    MovementPattern::Walk(UP),
    MovementPattern::Walk(LEFT),
    MovementPattern::Walk(RIGHT),
    MovementPattern::Walk(DOWN),
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