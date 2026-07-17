use crate::game::{coordinates::InternalCooridates, movement::{MovementPattern, get_movement_patterns}};
use super::piece::Piece;
use anyhow::{Result, bail};

#[allow(unused)]
struct Board {
    pieces: Vec<Piece>,
}

impl Board {
    fn get_movement_options(&self, source: InternalCooridates ) -> Result<Vec<InternalCooridates>> {
        let Some(piece) = self.pieces.iter().find(|p| p.position() == source) else {
            bail!("No piece on this cooridate");
        };

        let patterns = get_movement_patterns(piece.piece_type());
        for p in patterns {
            todo!()
            /*
            match p {
                MovementPattern::Walk(offset) => {
                    todo!()
                }
            }
            */
        }
        todo!()
    }
    
}