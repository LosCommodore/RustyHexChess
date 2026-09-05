use crate::Side;
use std::collections::HashMap;

use super::coordinates::*;
use anyhow::{Ok, Result};
use serde::Serialize;
use strum::EnumCount;

pub const WHITE_PAWNS_STARTING_POSITIONS: [Position; 9] = [
    Position::new_const(1, 4), // ('B', 5))
    Position::new_const(2, 4), // ('C', 5))
    Position::new_const(3, 4), // ('D', 5))
    Position::new_const(4, 4), // ('E', 5))
    Position::new_const(5, 4), // ('F', 5))
    Position::new_const(6, 3), // ('G', 4))
    Position::new_const(7, 2), // ('H', 3))
    Position::new_const(8, 1), // ('I', 2))
    Position::new_const(9, 0), // ('J', 1))
];

pub const BLACK_PAWNS_STARTING_POSITIONS: [Position; 9] = [
    Position::new_const(1, 10), // ('B', 11)
    Position::new_const(2, 9),  // ('C', 10)
    Position::new_const(3, 8),  // ('D', 9)
    Position::new_const(4, 7),  // ('E', 8)
    Position::new_const(5, 6),  // ('F', 7)
    Position::new_const(6, 6),  // ('G', 7)
    Position::new_const(7, 6),  // ('H', 7)
    Position::new_const(8, 6),  // ('I', 7)
    Position::new_const(9, 6),  // ('J', 7)
];

pub const WHITE_PAWNS_PROMOTION_POSITIONS: [Position; 11] = [
    Position::new_const(0, 10),
    Position::new_const(1, 10),
    Position::new_const(2, 10),
    Position::new_const(3, 10),
    Position::new_const(4, 10),
    Position::new_const(5, 10),
    Position::new_const(6, 9),
    Position::new_const(7, 8),
    Position::new_const(8, 7),
    Position::new_const(9, 6),
    Position::new_const(10, 5),
];

pub const BLACK_PAWNS_PROMOTION_POSITIONS: [Position; 11] = [
    Position::new_const(0, 5),
    Position::new_const(1, 4),
    Position::new_const(2, 3),
    Position::new_const(3, 2),
    Position::new_const(4, 1),
    Position::new_const(5, 0),
    Position::new_const(6, 0),
    Position::new_const(7, 0),
    Position::new_const(8, 0),
    Position::new_const(9, 0),
    Position::new_const(10, 0),
];

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, EnumCount)]
#[repr(u8)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceType {
    pub const COUNT: usize = 6;
    pub const ALL: [PieceType; Self::COUNT] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];
}

pub const fn pawn_starting_positions(side: Side) -> [Position; 9] {
    match side {
        Side::White => WHITE_PAWNS_STARTING_POSITIONS,
        Side::Black => BLACK_PAWNS_STARTING_POSITIONS,
    }
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
    pub fn new(piece_type: PieceType, side: Side) -> Self {
        Self { piece_type, side }
    }
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

    let mut insert = |pos: HumanNotation, piece_type: PieceType| -> Result<()> {
        out.insert(
            Position::from_human(pos).expect("invalid starting positions ??"),
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
    let mut insert = |pos: HumanNotation, piece_type: PieceType| -> Result<()> {
        out.insert(
            Position::from_human(pos).expect("invalid starting positions ??"),
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
