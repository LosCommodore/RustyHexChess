use std::collections::HashMap;

use super::coordinates::*;
use anyhow::{Ok, Result};

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
    
    let pawns = [
        ('b', 11),
        ('c', 10),
        ('d', 9),
        ('e', 8),
        ('f', 7),
        ('g', 7),
        ('h', 7),
        ('i', 7),
        ('j', 7),
    ];

    for pawn in pawns {
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

    let pawns = [
        ('b', 5),
        ('c', 5),
        ('d', 5),
        ('e', 5),
        ('f', 5),
        ('g', 4),
        ('h', 3),
        ('i', 2),
        ('j', 1),
    ];

    for pawn in pawns {
        insert(pawn, PieceType::Pawn)?;
    }

    Ok(out)
}
