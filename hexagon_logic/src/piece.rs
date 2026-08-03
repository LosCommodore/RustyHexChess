use crate::Side;
use std::collections::HashMap;

use super::coordinates::*;
use anyhow::{Ok, Result};

pub const WHITE_PAWNS_STARTING_POSITIONS: [Position; 9] = [
    Position { y: 1, x: 4 }, // ('B', 5))
    Position { y: 2, x: 4 }, // ('C', 5))
    Position { y: 3, x: 4 }, // ('D', 5))
    Position { y: 4, x: 4 }, // ('E', 5))
    Position { y: 5, x: 4 }, // ('F', 5))
    Position { y: 6, x: 3 }, // ('G', 4))
    Position { y: 7, x: 2 }, // ('H', 3))
    Position { y: 8, x: 1 }, // ('I', 2))
    Position { y: 9, x: 0 }, // ('J', 1))
];

pub const BLACK_PAWNS_STARTING_POSITIONS: [Position; 9] = [
    Position { y: 1, x: 10 }, // ('B', 11)
    Position { y: 2, x: 9 },  // ('C', 10)
    Position { y: 3, x: 8 },  // ('D', 9)
    Position { y: 4, x: 7 },  // ('E', 8)
    Position { y: 5, x: 6 },  // ('F', 7)
    Position { y: 6, x: 6 },  // ('G', 7)
    Position { y: 7, x: 6 },  // ('H', 7)
    Position { y: 8, x: 6 },  // ('I', 7)
    Position { y: 9, x: 6 },  // ('J', 7)
];


pub const WHITE_PAWNS_PROMOTION_POSITIONS: [Position; 11] = [
    Position { y: 0, x: 10 },
    Position { y: 1, x: 10 },
    Position { y: 2, x: 10 },
    Position { y: 3, x: 10 },
    Position { y: 4, x: 10 },
    Position { y: 5, x: 10 },
    Position { y: 6, x: 9 },
    Position { y: 7, x: 8 },
    Position { y: 8, x: 7 },
    Position { y: 9, x: 6 },
    Position { y: 10, x: 5 },
];


pub const BLACK_PAWNS_PROMOTION_POSITIONS: [Position; 11] = [
    Position { y: 0, x: 5 },
    Position { y: 1, x: 4 },
    Position { y: 2, x: 3 },
    Position { y: 3, x: 2 },
    Position { y: 4, x: 1 },
    Position { y: 5, x: 0 },
    Position { y: 6, x: 0 },
    Position { y: 7, x: 0 },
    Position { y: 8, x: 0 },
    Position { y: 9, x: 0 },
    Position { y: 10, x: 0 },
];


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
    pub fn symbol(self) -> char {
        match self {
            PieceType::King => 'K',
            PieceType::Queen => 'Q',
            PieceType::Bishop => 'B',
            PieceType::Knight => 'N',
            PieceType::Rook => 'R',
            PieceType::Pawn => 'P',
        }
    }
}

impl Piece {
    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn side(&self) -> Side {
        self.side
    }
}

pub fn get_startup_pieces_black() -> HashMap<Position, Piece> {
    let color = Side::Black;
    let mut out = HashMap::new();

    let mut insert = |pos: HumanCoordinates, piece_type: PieceType| -> Result<()> {
        out.insert(
            Position::try_from(pos)?,
            Piece {
                piece_type,
                side: color,
            },
        );
        Ok(())
    };

    insert(('E', 11), PieceType::King).unwrap();
    insert(('G', 10), PieceType::Queen).unwrap();
    insert(('F', 11), PieceType::Bishop).unwrap();
    insert(('F', 10), PieceType::Bishop).unwrap();
    insert(('F', 9), PieceType::Bishop).unwrap();
    insert(('D', 11), PieceType::Knight).unwrap();
    insert(('H', 9), PieceType::Knight).unwrap();
    insert(('C', 11), PieceType::Rook).unwrap();
    insert(('I', 8), PieceType::Rook).unwrap();

    for pos in BLACK_PAWNS_STARTING_POSITIONS {
        out.insert(
            pos,
            Piece {
                piece_type: PieceType::Pawn,
                side: color,
            },
        );
    }
    out
}

pub fn get_startup_pieces_white() -> HashMap<Position, Piece> {
    let color = Side::White;

    let mut out = HashMap::new();
    let mut insert = |pos: HumanCoordinates, piece_type: PieceType| -> Result<()> {
        out.insert(
            Position::try_from(pos)?,
            Piece {
                piece_type,
                side: color,
            },
        );
        Ok(())
    };

    insert(('E', 2), PieceType::King).unwrap();
    insert(('G', 1), PieceType::Queen).unwrap();
    insert(('F', 1), PieceType::Bishop).unwrap();
    insert(('F', 2), PieceType::Bishop).unwrap();
    insert(('F', 3), PieceType::Bishop).unwrap();
    insert(('D', 3), PieceType::Knight).unwrap();
    insert(('H', 1), PieceType::Knight).unwrap();
    insert(('C', 4), PieceType::Rook).unwrap();
    insert(('I', 1), PieceType::Rook).unwrap();

    for pos in WHITE_PAWNS_STARTING_POSITIONS {
        out.insert(
            pos,
            Piece {
                piece_type: PieceType::Pawn,
                side: color,
            },
        );
    }

    out
}
