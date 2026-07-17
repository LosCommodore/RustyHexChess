use super::coordinates::*;
use anyhow::{Ok, Result};

pub struct Piece {
    position: InternalCoordinates,
    piece_type: PieceType,
    color: Color,
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
    pub fn new(pos: HumanCoordinates, piece_type: PieceType, color: Color) -> Result<Self> {
        let position = to_internal_coordinates(pos)?;
        Ok(Piece {
            position,
            piece_type,
            color,
        })
    }

    pub fn position(&self) -> InternalCoordinates {
        self.position
    }

    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

pub fn get_startup_pieces_black() -> Result<Vec<Piece>> {
    let color = Color::Black;

    let mut pieces = vec![
        Piece::new(('E', 10), PieceType::King, color)?,
        Piece::new(('G', 10), PieceType::Queen, color)?,
        Piece::new(('F', 11), PieceType::Bishop, color)?,
        Piece::new(('F', 10), PieceType::Bishop, color)?,
        Piece::new(('F', 9), PieceType::Bishop, color)?,
        Piece::new(('D', 9), PieceType::Knight, color)?,
        Piece::new(('H', 9), PieceType::Knight, color)?,
        Piece::new(('C', 8), PieceType::Rook, color)?,
        Piece::new(('I', 8), PieceType::Rook, color)?,
    ];

    let pawns: Vec<Piece> = ['b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'k']
        .into_iter()
        .map(|i| Piece::new((i, 7), PieceType::Pawn, color))
        .collect::<Result<_, _>>()?;

    pieces.extend(pawns);

    Ok(pieces)
}

pub fn get_startup_pieces_white() -> Result<Vec<Piece>> {
    let color = Color::White;

    let mut pieces = vec![
        Piece::new(('E', 1), PieceType::King, color)?,
        Piece::new(('G', 1), PieceType::Queen, color)?,
        Piece::new(('F', 1), PieceType::Bishop, color)?,
        Piece::new(('F', 2), PieceType::Bishop, color)?,
        Piece::new(('F', 3), PieceType::Bishop, color)?,
        Piece::new(('D', 1), PieceType::Knight, color)?,
        Piece::new(('H', 1), PieceType::Knight, color)?,
        Piece::new(('C', 1), PieceType::Rook, color)?,
        Piece::new(('I', 1), PieceType::Rook, color)?,
    ];

    let pawns: Vec<Piece> = [
        ('b', 1),
        ('c', 2),
        ('d', 3),
        ('e', 4),
        ('f', 5),
        ('g', 4),
        ('h', 3),
        ('i', 2),
        ('k', 1),
    ]
    .into_iter()
    .map(|pos| Piece::new(pos, PieceType::Pawn, color))
    .collect::<Result<_, _>>()?;

    pieces.extend(pawns);

    Ok(pieces)
}
