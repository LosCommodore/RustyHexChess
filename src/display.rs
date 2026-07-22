use crate::game::board::Board;
use crate::game::board::Marker;
use crate::game::coordinates::Position;
use crate::game::coordinates::num_to_char_notation;
use anyhow::Result;
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::disable_raw_mode;
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
};
use crossterm::{cursor::MoveToNextLine, execute, style::Print};
use crossterm::{cursor::Show, style::ResetColor};
use std::io::{Write, stdout};
use crate::game::piece::Color as PieceColor;


const BOARD_DIM: usize = 11;
const EDGE_LEN: usize = 6;
const CELL_WIDTH: usize = 5;
const INITIAL_X_OFFSET: isize = 20;

pub trait BoardDisplay {
    fn display(&self, board: &Board) -> Result<()>;
}

pub struct ChessTerminal;

impl ChessTerminal {
    pub fn clc() -> Result<()> {
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        Ok(())
    }
}
impl Drop for ChessTerminal {
    fn drop(&mut self) {
        let _ = execute!(stdout(), ResetColor, Show);
        let _ = disable_raw_mode();
    }
}

impl BoardDisplay for ChessTerminal {
    fn display(&self, board: &Board) -> Result<()> {
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
                let pos = Position{y,x};
                print_cell(board, pos, &mut out)?;

            }
            print_diagonal_column_label(&mut out, y)?;
            execute!(out, MoveToNextLine(1))?;

            space_count += inc;
            columns = (columns as isize + inc) as usize;
        }

        out.flush()?;
        Ok(())
    }
}

fn print_cell(board: &Board, pos: Position, out: &mut impl Write) -> Result<()> {
    let piece = board.pieces().iter().find(|p| p.position() == pos);

    
    let foreground_color = match piece {
        Some(p) => match p.color() {
            PieceColor::Black => Color::Blue,
            PieceColor::White => Color::Red,
        },
        None => Color::Reset,
    };    

    let symbol = match piece {
        Some(x) => x.piece_type().symbol(),
        None => '.',
    };

    let symbol: String = format!("[ {symbol} ]");


    let cell_color = board
    .markers
    .get(&pos)
    .map_or(Color::Reset, |marker| match marker {
        Marker::MovementOption => Color::DarkYellow,
    });


    execute!(
        out,
        SetBackgroundColor(cell_color),
        SetForegroundColor(foreground_color),
        Print(&symbol),
        SetBackgroundColor(Color::Reset),
        SetForegroundColor(Color::Reset),
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
