use crate::game;
use crate::game::Piece;
use anyhow::Result;
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor::MoveToNextLine, execute, style::Print};
use std::io::{Write, stdout};

const BOARD_DIM: usize = 11;
const EDGE_LEN: usize = 6;
const CELL_WIDTH: usize = 5;
const INITIAL_X_OFFSET: isize = 20;

pub fn print_board(pieces: &[Piece]) -> Result<()> {
    let mut out = stdout();

    print_column_labels(&mut out)?;

    let mut columns = EDGE_LEN;
    let mut space_count = 0isize;
    for y in 0..BOARD_DIM {
        let inc: isize = if y < BOARD_DIM / 2 { 1 } else { -1 };

        let nr_spaces = (INITIAL_X_OFFSET - space_count * 3) as usize;
        let s = " ".repeat(nr_spaces);

        execute!(out, Print(s), Print(format!("{}  ", row_label(y))))?;
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
        print_diagonal_column_label(&mut out, y)?;
        execute!(out, MoveToNextLine(1))?;

        space_count += inc;
        columns = (columns as isize + inc) as usize;
    }

    out.flush()?;
    Ok(())
}

fn print_column_labels(out: &mut impl Write) -> Result<()> {
    let board_start = INITIAL_X_OFFSET as usize + 3;
    let padding = " ".repeat(board_start);

    execute!(out, Print(&padding))?;
    for x in 1..=EDGE_LEN {
        execute!(out, Print(format!("{:^width$}", x, width = CELL_WIDTH)))?;
    }
    execute!(out, MoveToNextLine(1))?;

    execute!(out, Print(&padding))?;
    for _ in 0..EDGE_LEN {
        execute!(out, Print(format!("{:^width$}", "/", width = CELL_WIDTH)))?;
    }
    execute!(out, Print(format!(" {}", EDGE_LEN + 1)))?;
    execute!(out, MoveToNextLine(1))?;

    Ok(())
}

fn print_diagonal_column_label(out: &mut impl Write, y: usize) -> Result<()> {
    if y >= EDGE_LEN - 1 {
        return Ok(());
    }

    execute!(out, Print(" /"))?;
    if y < EDGE_LEN - 2 {
        execute!(out, Print(format!(" {}", EDGE_LEN + y + 2)))?;
    }

    Ok(())
}

fn row_label(y: usize) -> char {
    let mut code = b'A' + y as u8;
    if code >= b'J' {
        code += 1;
    }
    code as char
}
