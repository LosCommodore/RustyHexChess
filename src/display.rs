use crate::game;
use crate::game::Piece;
use anyhow::Result;
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor::MoveToNextLine, execute, style::Print};
use std::io::{Write, stdout};


pub fn print_board(pieces: &Vec<Piece>) -> Result<()> {
    let board_dim = 11;
    let edge_len = 6;

    let mut out = stdout();

    let mut columns = edge_len;

    let initial_x_offset = 20;
    let mut space_count = 0isize;
    for y in 0..board_dim {
        let inc: isize = if y < board_dim / 2 { 1 } else { -1 };

        let nr_spaces = (initial_x_offset - space_count * 3) as usize;
        let s = " ".repeat(nr_spaces);

        execute!(out, Print(s))?;
        for x in 0..columns {
            let piece = pieces.iter().find(|p| p.position() == (y, x));

            let cell_color = match piece {
                Some(p) => match p.color() {
                    game::Color::Black => Color::Blue,
                    game::Color::White => Color::Red,
                },
                None => Color::Reset,
            };

            let symbol = match piece {
                Some(x) => x.piece_type().symbol(),
                None => '.',
            };

            let symbol = format!("[ {symbol} ]");

            execute!(
                out,
                SetBackgroundColor(cell_color),
                Print(&symbol),
                SetForegroundColor(Color::White),
                SetBackgroundColor(Color::Reset),
            )?;
        }
        execute!(out, MoveToNextLine(1))?;

        space_count += inc;
        columns = (columns as isize + inc) as usize;
    }

    out.flush()?;
    Ok(())
}
