use std::collections::HashMap;

use super::coordinates::*;
use anyhow::{Ok, Result};

pub const WHITE_PAWNS_STARTING_POSITIONS: [(char, usize); 9] = [
    ('B', 5),
    ('C', 5),
    ('D', 5),
    ('E', 5),
    ('F', 5),
    ('G', 4),
    ('H', 3),
    ('I', 2),
    ('J', 1),
];

pub const BLACK_PAWNS_STARTING_POSITIONS: [(char, usize); 9] = [
    ('B', 11),
    ('C', 10),
    ('D', 9),
    ('E', 8),
    ('F', 7),
    ('G', 7),
    ('H', 7),
    ('I', 7),
    ('J', 7),
];

pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
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

    pub fn color(&self) -> Color {
        self.color
    }
}

pub fn get_startup_pieces_black() -> Result<HashMap<Position, Piece>> {
    let mut out = HashMap::new();
    let mut insert = |pos: HumanCoordinates, piece_type: PieceType| -> Result<()> {
        out.insert(
            Position::try_from(pos)?,
            Piece {
                piece_type,
                color: Color::Black,
            },
        );
        Ok(())
    };

    insert(('E', 11), PieceType::King)?;
    insert(('G', 10), PieceType::Queen)?;
    insert(('F', 11), PieceType::Bishop)?;
    insert(('F', 10), PieceType::Bishop)?;
    insert(('F', 9), PieceType::Bishop)?;
    insert(('D', 11), PieceType::Knight)?;
    insert(('H', 9), PieceType::Knight)?;
    insert(('C', 11), PieceType::Rook)?;
    insert(('I', 8), PieceType::Rook)?;

    for pawn in BLACK_PAWNS_STARTING_POSITIONS {
        insert(pawn, PieceType::Pawn)?;
    }
    Ok(out)
}

pub fn get_startup_pieces_white() -> Result<HashMap<Position, Piece>> {
    let mut out = HashMap::new();
    let mut insert = |pos: HumanCoordinates, piece_type: PieceType| -> Result<()> {
        out.insert(
            Position::try_from(pos)?,
            Piece {
                piece_type,
                color: Color::White,
            },
        );
        Ok(())
    };
    let mut pieces = vec![
        insert(('E', 2), PieceType::King)?,
        insert(('G', 1), PieceType::Queen)?,
        insert(('F', 1), PieceType::Bishop)?,
        insert(('F', 2), PieceType::Bishop)?,
        insert(('F', 3), PieceType::Bishop)?,
        insert(('D', 3), PieceType::Knight)?,
        insert(('H', 1), PieceType::Knight)?,
        insert(('C', 4), PieceType::Rook)?,
        insert(('I', 1), PieceType::Rook)?,
    ];

    for pawn in WHITE_PAWNS_STARTING_POSITIONS {
        insert(pawn, PieceType::Pawn)?;
    }

    Ok(out)
}
