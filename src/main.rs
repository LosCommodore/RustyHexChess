mod display;
mod game;
use crate::{
    display::print_board,
    game::piece::{get_startup_pieces_black, get_startup_pieces_white},
};
use anyhow::Result;

use crossterm::{
    cursor::{MoveTo, Show},
    execute,
    style::ResetColor,
    terminal::{Clear, ClearType, disable_raw_mode},
};
use std::io::stdout;

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = execute!(stdout(), ResetColor, Show);
        let _ = disable_raw_mode();
    }
}

fn main() -> Result<()> {
    let _cleanup = TerminalCleanup;

    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

    //let _piece = Piece::new(('A', 1), game::PieceType::King, game::PlayerColor::Black)?;
    let mut pieces = get_startup_pieces_black()?;
    pieces.extend(get_startup_pieces_white()?);

    print_board(&pieces)?;
    Ok(())
}
