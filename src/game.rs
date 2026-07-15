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

pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

pub enum PieceColor {
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
    color: PieceColor,
}

impl Piece {
    pub fn new(pos: (char, usize), piece_type: PieceType, color: PieceColor) -> Result<Self> {
        let position = to_internal_coordinates(pos)?;
        Ok(Piece {
            position,
            piece_type,
            color,
        })
    }

    fn to_human_coordinates(&self) -> Result<(char, usize)> {
        let code = self.position.0 + 65;
        let c = char::from(u8::try_from(code)?);

        let y = (self.position.1) + 1;
        Ok((c, y))
    }
}

fn to_internal_coordinates((y, x): (char, usize)) -> Result<(usize, usize)> {
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

    // Todo: check if coordinates are on board
    Ok((y, x))
}
