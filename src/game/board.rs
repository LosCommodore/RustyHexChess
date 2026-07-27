use std::{collections::HashMap, fmt::Pointer, todo};

use super::piece::Piece;

use crate::game::{
    coordinates::{Position, X_RANGE},
    movement::{MovementPattern, get_movement_patterns},
};
use anyhow::{Result, bail};

#[allow(unused)]
pub enum Marker {
    MovementOption,
}

#[derive(Default)]
pub struct Board {
    pub pieces: HashMap<Position, Piece>,
    pub markers: HashMap<Position, Marker>,
}

impl Board {
    pub fn get_movement_options(&self, pos: Position) -> Result<Vec<Position>> {
        let Some(me) = self.pieces.get(&pos) else {
            bail!("No piece on this coordinate");
        };

        let mut options: Vec<Position> = Vec::new();
        for p in get_movement_patterns(me.piece_type()) {
            match p {
                MovementPattern::Walk(offset) => {
                    options.extend(do_walk(me, pos,*offset, &self.pieces));
                }
                _ => todo!(),
            }
        }
        Ok(options)
    }
}

fn do_walk(me: &Piece, mut pos: Position, offset: (isize, isize), pieces: &HashMap<Position, Piece>) -> Vec<Position> {
    let mut options = Vec::new();
    loop {
        let Some(y) = pos.y.checked_add_signed(offset.0) else {
            return options;
        };
        pos.y = y;

        let Some(x) = pos.x.checked_add_signed(offset.1) else {
            return options;
        };
        pos.x = x;

        if !check_in_field(pos) {
            return options;
        }

        if let Some(piece) = pieces.get(&pos) {
            if piece.color() == me.color() {
                options.push(pos);
            }
            return options;
        }

        options.push(pos);
    }
}

fn check_in_field(Position { y, x }: Position) -> bool {
    if y > 10 {
        return false;
    }
    let x_range = X_RANGE[y];

    x >= x_range.0 && x <= x_range.1
}

#[cfg(test)]
mod tests {
    use std::println;

    use super::*;

    use crate::game::piece::{self, Color, Piece, PieceType};

    #[test]
    fn test_move_rook() {
        let mut board = Board::default();
        let pos = Position::try_from(('F', 5)).expect("invalid position");
        let piece = Piece{piece_type: PieceType::Rook, color: Color::Black};
        board
            .pieces
            .insert(pos, piece);

        let internal_pos =pos;

        let options = board
            .get_movement_options(internal_pos)
            .expect("error on movement options");

        println!("{options:#?}");
    }
}
