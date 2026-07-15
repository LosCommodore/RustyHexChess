use anyhow::Result;
use crossterm::{cursor::MoveToNextLine, execute, style::Print};
use std::io::{Write, stdout};

/*

use crossterm::style::{Color, Print, ResetColor, SetForegroundColor},

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
*/

pub fn print_board() -> Result<()> {
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
        for _ in 0..columns {
            execute!(out, Print("[ x ] "),)?;
        }
        execute!(out, MoveToNextLine(1))?;

        space_count += inc;
        columns += inc;
    }

    out.flush()?;
    Ok(())
}
