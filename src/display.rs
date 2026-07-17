use crate::game::{self, num_to_char_notation};
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
        let whitespace = " ".repeat(nr_spaces);
        let char_notation = num_to_char_notation(y)?;
        execute!(out, Print(whitespace), Print(format!("{char_notation}  ")))?;

        for x in 0..columns {
            print_cell(&pieces,y,x,&mut out)?;
        }
        print_diagonal_column_label(&mut out, y)?;
        execute!(out, MoveToNextLine(1))?;

        space_count += inc;
        columns = (columns as isize + inc) as usize;
    }

    out.flush()?;
    Ok(())
}

fn print_cell(pieces: &[Piece], y: usize, x: usize, out: &mut impl Write) -> Result<()> {
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

    let symbol: String = format!("[ {symbol} ]");

    execute!(
        out,
        SetBackgroundColor(cell_color),
        Print(&symbol),
        SetForegroundColor(Color::White),
        SetBackgroundColor(Color::Reset),
    )?;
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

