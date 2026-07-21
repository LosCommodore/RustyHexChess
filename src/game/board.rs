use std::{collections::HashMap, todo};

use super::piece::Piece;

use crate::game::{
    coordinates::Coordinates,
    movement::{MovementPattern, get_movement_patterns},
};
use anyhow::{Result, bail};

/*
enum Marker {
    MovementOption,
}
*/

#[derive(Default)]
pub struct Board {
    pieces: Vec<Piece>,
    //markers: HashMap<Coordinates, Marker>,
}

impl Board {
    pub fn new(pieces: Vec<Piece>) -> Self {
        Self {
            pieces,
            ..Default::default()
        }
    }

    pub fn pieces(&self) -> &Vec<Piece> {
        &self.pieces
    }

    #[allow(unused)]
    fn get_movement_options(&self, source: Coordinates) -> Result<Vec<Coordinates>> {
        let Some(me) = self.pieces.iter().find(|p| p.position() == source) else {
            bail!("No piece on this coordinate");
        };

        let mut options: Vec<Coordinates> = Vec::new();
        for p in get_movement_patterns(me.piece_type()) {
            match p {
                MovementPattern::Walk(offset) => {
                    options.extend(do_walk(me, *offset, &self.pieces));
                }
                _ => todo!(),
            }
        }
        Ok(options)
    }
}

fn do_walk(me: &Piece, offset: (isize, isize), pieces: &Vec<Piece>) -> Vec<Coordinates> {
    let mut options = Vec::new();
    let mut pos = me.position();
    loop {
        let Some(x) = pos.y.checked_add_signed(offset.0) else {
            return options;
        };
        pos.x = x;

        let Some(x) = pos.x.checked_add_signed(offset.1) else {
            return options;
        };
        pos.x = x;

        if !check_in_field(pos) {
            return options;
        }

        if let Some(piece) = pieces.iter().find(|p| p.position() == pos) {
            if piece.color() == me.color() {
                options.push(pos);
            }
            return options;
        }

        options.push(pos);
    }
}

fn check_in_field(pos: Coordinates) -> bool {
    if pos.y > 10 {
        return false;
    }
    let distance_to_middle = 5usize.abs_diff(pos.y);
    let max_x = 10 - distance_to_middle;

    pos.x < max_x
}

#[cfg(test)]
mod tests {
    use std::println;

    use super::*;

    use crate::game::piece::{Color, Piece, PieceType};

    #[test]
    fn test_move_rook() {
        let mut board = Board::default();
        let pos = ('F', 5);
        board
            .pieces
            .push(Piece::new(pos, PieceType::Rook, Color::Black).expect("wrong piece definition"));

        let internal_pos = board.pieces[0].position();

        let options = board
            .get_movement_options(internal_pos)
            .expect("error on movement options");

        println!("{options:#?}");
    }
}
