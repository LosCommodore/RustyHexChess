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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerColor {
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
    position: (usize, usize),
    piece_type: PieceType,
    color: PlayerColor,
}

type HumanCoordinates = (char, usize); // e.g. ('a',1) -> see doc folder
type InternalCooridates = (usize, usize); // 0-indexed

impl Piece {
    pub fn new(pos: HumanCoordinates, piece_type: PieceType, color: PlayerColor) -> Result<Self> {
        let position = to_internal_coordinates(pos)?;
        Ok(Piece {
            position,
            piece_type,
            color,
        })
    }
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

fn get_startup_pieces_black() -> Result<Vec<Piece>> {
    let color = PlayerColor::Black;

    Ok(vec![
        Piece::new(('A', 1), PieceType::King, color)?,
    ])
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

    let y = y as usize; // a = 65
    if !(97..=108).contains(&y) {
        bail!("First item of position must be between 'a' and 'l'")
    }
    Ok((y, x))
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
}
