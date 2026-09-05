use crate::Side;
use crate::board::Board;
use crate::coordinates::{Position, X_RANGE, num_to_char_notation};
use ansi_to_html::convert;
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::disable_raw_mode;
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
};
use crossterm::{cursor::Show, style::ResetColor};
use crossterm::{execute, style::Print};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufWriter;
use std::io::{Write, stdout};
use std::path::Path;
use thiserror::Error;

const BOARD_DIM: usize = 11;
const EDGE_LEN: usize = 6;
const CELL_WIDTH: usize = 5;
const INITIAL_X_OFFSET: isize = 20;

#[derive(Error, Debug)]
pub enum DisplayError {
    /// Forwards all standard I/O errors (file creation, writing, flushing)
    #[error("I/O error occurred while rendering layout: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DisplayError>;

pub trait BoardDisplay {
    fn display(&self, board: &Board, markers: &HashSet<Position>) -> Result<()>;
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
    fn display(&self, board: &Board, markers: &HashSet<Position>) -> Result<()> {
        let mut out = stdout();
        write_board(board, markers, &mut out)?;
        out.flush()?;
        Ok(())
    }
}

pub fn write_board(board: &Board, markers: &HashSet<Position>, out: &mut impl Write) -> Result<()> {
    let mut columns = EDGE_LEN;
    let mut space_count = 0isize;
    for (y, x_range) in X_RANGE.iter().enumerate() {
        let inc: isize = if y < BOARD_DIM / 2 { 1 } else { -1 };

        let nr_spaces = (INITIAL_X_OFFSET - space_count * 3) as usize;
        let whitespace = " ".repeat(nr_spaces);
        let Some(char_notation) = num_to_char_notation(y) else {
            panic!("Invalid board")
        };
        execute!(out, Print(whitespace), Print(format!("{char_notation}  ")))?;

        for x in x_range.0..x_range.1 {
            let pos = Position::new(y, x).expect("invalid position ???");
            print_cell(board, pos, markers, out)?;
        }
        print_diagonal_column_label(out, y)?;
        write!(out, "\r\n")?;

        space_count += inc;
        columns = (columns as isize + inc) as usize;
    }

    print_bottom_column_labels(out)?;

    out.flush()?;
    Ok(())
}

pub fn write_html(board: &Board, markers: &HashSet<Position>, out: &mut impl Write) -> Result<()> {
    let mut buffer = Vec::new();
    write_board(board, markers, &mut buffer)?;
    let ansi_string = String::from_utf8_lossy(&buffer);

    let html_content = convert(&ansi_string)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    write!(
        out,
        "<pre style=\"font-family: monospace; white-space: pre; line-height: 1.2; background-color: #000; color: #fff; padding: 10px;\">{}</pre>",
        html_content
    )?;

    Ok(())
}

pub fn save_board_to_html_file(
    board: &Board,
    markers: &HashSet<Position>,
    file_path: impl AsRef<Path>,
) -> Result<()> {
    let file_path = &file_path.as_ref();

    let file = File::create(Path::new(file_path))?;
    let mut writer = BufWriter::new(file);
    write_html(board, markers, &mut writer)?;
    writer.flush()?;
    Ok(())
}

fn print_cell(
    board: &Board,
    pos: Position,
    markers: &HashSet<Position>,
    out: &mut impl Write,
) -> Result<()> {
    let piece = board.pieces.get(&pos);

    let foreground_color = match piece {
        Some(p) => match p.side() {
            Side::Black => Color::Blue,
            Side::White => Color::Red,
        },
        None => Color::Reset,
    };

    let symbol = match piece {
        Some(x) => x.piece_type().symbol(),
        None => '.',
    };

    let symbol: String = format!("[ {symbol} ]");

    let cell_color = if markers.contains(&pos) {
        Color::DarkYellow
    } else {
        Color::Reset
    };

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

fn print_bottom_column_labels(out: &mut impl Write) -> Result<()> {
    let board_start = INITIAL_X_OFFSET as usize + 3;
    let padding = " ".repeat(board_start);

    execute!(out, Print(&padding))?;
    for _ in 0..EDGE_LEN {
        execute!(out, Print(format!("{:^width$}", "\\", width = CELL_WIDTH)))?;
    }
    execute!(out, Print(format!(" {}", EDGE_LEN + 1)))?;
    write!(out, "\r\n")?;

    execute!(out, Print(&padding))?;
    for x in 1..=EDGE_LEN {
        execute!(out, Print(format!("{:^width$}", x, width = CELL_WIDTH)))?;
    }
    write!(out, "\r\n")?;

    Ok(())
}

fn print_diagonal_column_label(out: &mut impl Write, y: usize) -> Result<()> {
    if y < EDGE_LEN {
        return Ok(());
    }

    execute!(out, Print(" \\"))?;
    if y > EDGE_LEN {
        execute!(out, Print(format!("  {}", BOARD_DIM - (y - EDGE_LEN) + 1)))?;
    }

    Ok(())
}
