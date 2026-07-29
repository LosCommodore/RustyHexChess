use std::{collections::HashMap, fmt::Pointer, todo};

use super::piece::Piece;

use crate::game::{
    coordinates::{Position, X_RANGE},
    movement::{MovementPattern, get_movement_patterns},
    piece::{Color}
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
                MovementPattern::Walk(offset) => options.extend(self.do_walk(me, pos, *offset)),
                MovementPattern::Step(steps) => options.extend(self.do_step(me, pos, *steps)),
                MovementPattern::Pawn => options.extend(self.move_pawn(me, pos)),
            }
        }
        Ok(options)
    }

    pub fn is_movement_option(
        &self,
        pos: &Position,
        me: &Piece,
        dy: isize,
        dx: isize,
        capture_only: bool  // move only allowed if a piece is captured (used for pawn)
    ) -> Option<Position> {
        let Some(y) = pos.y.checked_add_signed(dy) else {
            return None;
        };
        let Some(x) = pos.x.checked_add_signed(dx) else {
            return None;
        };

        let pos = Position { y, x };
        if !pos.is_in_field() {
            return None;
        }

        if let Some(piece) = self.pieces.get(&pos) {
            if piece.color() == me.color() {
                return None;
            }
        } else {
            if capture_only {return None;}
        }

        Some(pos)
    }

    
    fn move_pawn(&self, me: &Piece, pos: Position) -> Vec<Position> {
        let mut options = Vec::new();
        let color = me.color;

        let step = if color == Color::White {1} else {-1};
        let offset = (0,step);
        options.extend(self.do_step(me, pos, &[offset]));

        let capture_moves = match color {
            Color::White => &[(1,0),(-1,1)],
            Color::Black => &[(-1,0),(1,-1)]
        };

        for (dy, dx) in capture_moves {
                  let option =  self.is_movement_option(&pos, me, *dy, *dx, true); 
           options.extend(option);
        }
        options
    }
   

    /// A step is a direkt move to another position, no blocking of movements.
    fn do_step(&self, me: &Piece, pos: Position, steps: &[(isize, isize)]) -> Vec<Position> {
        let mut options: Vec<Position> = Vec::new();

        for (dy, dx) in steps {
           let option =  self.is_movement_option(&pos, me, *dy, *dx, false); 
           options.extend(option);
        }

        options
    }

    fn do_walk(&self, me: &Piece, mut pos: Position, (dy, dx): (isize, isize)) -> Vec<Position> {
        let mut options = Vec::new();
        loop {
            let Some(new_pos) = self.is_movement_option(&pos, me, dy, dx, false) else {
                return options;
            };
            options.push(new_pos);
            pos = new_pos;
        }
    }
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
        let piece = Piece {
            piece_type: PieceType::Rook,
            color: Color::Black,
        };
        board.pieces.insert(pos, piece);

        let internal_pos = pos;

        let options = board
            .get_movement_options(internal_pos)
            .expect("error on movement options");

        println!("{options:#?}");
    }
}
