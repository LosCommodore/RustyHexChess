#![allow(unused)]

use anyhow::{Result, bail};
const EDGE_LEN: usize = 6;

struct Board {
    pieces: Vec<Piece>,
}

type Offset = (isize, isize);

const UP: Offset = (-1, 0);
const DOWN: Offset = (1, 0);
const LEFT: Offset = (0, -1);
const RIGHT: Offset = (0, 1);

const UP_LEFT: Offset = (-1, -1);
const UP_RIGHT: Offset = (-1, 1);
const DOWN_LEFT: Offset = (1, -1);
const DOWN_RIGHT: Offset = (1, 1);

enum Movement {
    Walk(Offset),
    Step(Vec<Offset>),
    Jump(Vec<Offset>),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

const ROOK_MOVEMENTS: &'static [Movement] = &[
    Movement::Walk(UP),
    Movement::Walk(LEFT),
    Movement::Walk(RIGHT),
    Movement::Walk(DOWN),
];
const KING_MOVEMENTS: &'static [Movement] = &[
    Movement::Walk(UP),
    Movement::Walk(LEFT),
    Movement::Walk(RIGHT),
    Movement::Walk(DOWN),
];

pub struct Piece {
    position: InternalCooridates,
    piece_type: PieceType,
    color: Color,
}

type HumanCoordinates = (char, usize); // e.g. ('a',1) -> see doc folder
type InternalCooridates = (usize, usize); // 0-indexed

impl Piece {
    pub fn new(pos: HumanCoordinates, piece_type: PieceType, color: Color) -> Result<Self> {
        let position = to_internal_coordinates(pos)?;
        Ok(Piece {
            position,
            piece_type,
            color,
        })
    }

    pub fn position(&self) -> InternalCooridates {
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

fn to_human_coordinates((y, x): InternalCooridates) -> Result<HumanCoordinates> {
    let mut code = y + 65;

    if code >= 74 {
        // skip J (74)
        code += 1;
    }
    let c = char::from(u8::try_from(code)?);

    Ok((c, x + 1))
}

fn to_internal_coordinates((y, x): HumanCoordinates) -> Result<InternalCooridates> {
    // TODO: Check if coordinates are on board.

    if !y.is_ascii() {
        bail!("First item of position must be ascii")
    }
    let y = y.to_ascii_lowercase();

    if y == 'j' {
        bail!("Letter 'j' is not used!")
    }

    let mut y = y as usize; // a = 97
    if !(97..=108).contains(&y) {
        bail!("First item of position must be between 'a' and 'l'")
    }

    if y > 106 {
        y -= 1; // skip J
    }

    y -= 97;

    Ok((y, x - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_human() {
        let human = to_human_coordinates((1, 1)).unwrap();
        assert_eq!(human, ('B', 2));

        let human = to_human_coordinates((10, 5)).unwrap();
        assert_eq!(human, ('L', 6));
    }

    #[test]
    fn to_internal_skips_j() {
        let internal = to_internal_coordinates(('K', 1)).unwrap();
        assert_eq!(internal, (9, 0));

        let internal = to_internal_coordinates(('L', 1)).unwrap();
        assert_eq!(internal, (10, 0));
    }
}
