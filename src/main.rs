#![allow(dead_code)]

use anyhow::Result;
use std::{char };
use std::io::{stdout, Write};

use crossterm::{
    execute,
    cursor::MoveToNextLine,
    style::{Color, Print, ResetColor, SetForegroundColor},
};


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


/// 11x11 board with spaces
/// - no "j"
/// - 1..11 + a-l
/// - not all cominaitons allowed
/// 
/// l
fn print_board() -> Result<()> {
    let board_dim = 11;
    let edge_len= 6;

    let mut out = stdout();

    let mut columns = edge_len;
    
    let initial_x_offset = 20;
    let mut space_count=0isize; 
    for y in 0..board_dim {
        let inc: isize =if  y< board_dim /2 { 1 } else {-1};

        let nr_spaces = (initial_x_offset-space_count*3) as usize;
        let s = " ".repeat(nr_spaces);
        
        execute!(out, Print(s))?;
        for _ in 0..columns {
            execute!(
                out,
                Print("[ x ] "),

            )?;
        }
        execute!(out,MoveToNextLine(1))?;
        
        space_count+=inc;
        columns+=inc;

    }

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
    print_board()?;
    Ok(())
}
