use std::todo;

use super::piece::Piece;
use crate::game::{
    coordinates::InternalCoordinates,
    movement::{MovementPattern, get_movement_patterns},
};
use anyhow::{Result, bail};

#[allow(unused)]
struct Board {
    pieces: Vec<Piece>,
}

impl Board {
    #[allow(unused)]
    fn get_movement_options(
        &self,
        source: InternalCoordinates,
    ) -> Result<Vec<InternalCoordinates>> {
        let Some(me) = self.pieces.iter().find(|p| p.position() == source) else {
            bail!("No piece on this coordinate");
        };

        let mut options: Vec<InternalCoordinates> = Vec::new();
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

fn do_walk(me: &Piece, offset: (isize, isize), pieces: &Vec<Piece>) -> Vec<InternalCoordinates> {
    let mut options = Vec::new();
    let mut pos = me.position();
    loop {
        let Some(x) = pos.0.checked_add_signed(offset.0) else {
            return options;
        };
        pos.0 = x;

        let Some(x) = pos.1.checked_add_signed(offset.1) else {
            return options;
        };
        pos.1 = x;

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

fn check_in_field(pos: InternalCoordinates) -> bool {
    if pos.0 > 10 {
        return false;
    }
    let distance_to_middle = 5usize.abs_diff(pos.0);
    let max_x = 10 - distance_to_middle;

    pos.1 < max_x
}
