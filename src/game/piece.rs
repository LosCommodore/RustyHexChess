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

pub fn get_startup_pieces_black() -> Result<HashMap<Position,Piece>> {
    let mut out = HashMap::new();
    let mut insert = |pos: HumanCoordinates, piece_type: PieceType| -> Result<()> {
         out.insert(Position::try_from(pos)?, Piece{piece_type,  color: Color::Black});
         Ok(())
        };

    insert(('E', 10),PieceType::King)?;
    insert(('G', 10), PieceType::Queen)?;
    insert(('F', 11), PieceType::Bishop)?;
    insert(('F', 10), PieceType::Bishop)?;
    insert(('F', 9), PieceType::Bishop,)?;
    insert(('D', 9), PieceType::Knight,)?;
    insert(('H', 9), PieceType::Knight,)?;
    insert(('C', 8), PieceType::Rook)?;
    insert(('I', 8), PieceType::Rook)?;

    for pawn in ['b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'k'] {
        insert((pawn, 7), PieceType::Pawn)?;
    }
    Ok(out)
}

pub fn get_startup_pieces_white() ->  Result<HashMap<Position,Piece>>  {
    let mut out = HashMap::new();
    let mut insert = |pos: HumanCoordinates, piece_type: PieceType| -> Result<()> {
         out.insert(Position::try_from(pos)?, Piece{piece_type,  color: Color::White});
         Ok(())
        };
    let mut pieces = vec![
        insert(('E', 1), PieceType::King)?,
        insert(('G', 1), PieceType::Queen)?,
        insert(('F', 1), PieceType::Bishop)?,
        insert(('F', 2), PieceType::Bishop)?,
        insert(('F', 3), PieceType::Bishop)?,
        insert(('D', 1), PieceType::Knight)?,
        insert(('H', 1), PieceType::Knight)?,
        insert(('C', 1), PieceType::Rook)?,
        insert(('I', 1), PieceType::Rook)?,
    ];

    let pawns = [
        ('b', 1),
        ('c', 2),
        ('d', 3),
        ('e', 4),
        ('f', 5),
        ('g', 4),
        ('h', 3),
        ('i', 2),
        ('k', 1),
    ];


    for pawn in pawns {
        insert(pawn, PieceType::Pawn)?;
    };

    Ok(out)
}
