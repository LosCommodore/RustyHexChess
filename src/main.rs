#![allow(dead_code)]

use anyhow::Result;
use std::{char };

struct Board {
    pieces: Vec<Piece>
    
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


enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn
}

const ROOK_MOVEMENTS: &'static [Movement] =  &[Movement::Walk(UP), Movement::Walk(LEFT), Movement::Walk(RIGHT), Movement::Walk(DOWN)];
const KING_MOVEMENTS: &'static [Movement] =   &[Movement::Walk(UP), Movement::Walk(LEFT), Movement::Walk(RIGHT), Movement::Walk(DOWN)];




/*
impl MovementPattern {
    fn next_offset(&self) -> (usize,usize) {
        use MovementPattern::*;
        match self {
            Up => (1,0),
            // fill out
            _ => (0,0), 
        }

    }
}

 */
use std::io::{stdout, Write};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};

fn draw() -> std::io::Result<()> {
    let mut out = stdout();

    execute!(
        out,
        SetForegroundColor(Color::Green),
        Print(". "),
        ResetColor,

        SetForegroundColor(Color::Red),
        Print("K "),
        ResetColor,

        SetForegroundColor(Color::Blue),
        Print("B "),
        ResetColor,

        Print("\n")
    )?;

    out.flush()?;
    Ok(())
}

/*
impl PieceType {
    fn iterator_moves(&self) {
        match self {
            PieceType::Bishop => get_iterator_for_bishop
            ...
        }

    }

    fn iterator_bishop() -> Option<(usize,usize)> {
        // an iterator !
        todo!()
    }
}
 */
struct Piece {
    position: (usize, usize),
}

impl Piece {
    fn to_human_coordinates(&self) -> Result<(char, usize)> {
        let code = self.position.0 + 65;
        let c = char::from(u8::try_from(code)?);

        let y=( self.position.1)+1; 
        Ok((c,y))
    }
}

fn main() -> Result<()> {
    let piece = Piece { position: (0, 0) };

    let x = piece.to_human_coordinates()?;
    draw()?;
    println!("Hello, world! {x:?}");
    Ok(())
}
