mod display;
mod game;

use crate::{
    display::{BoardDisplay, ChessTerminal},
    game::{
        board::Board,
        piece::{get_startup_pieces_black, get_startup_pieces_white},
    },
};
use anyhow::Result;

fn main() -> Result<()> {
    ChessTerminal::clc()?;

    //let _piece = Piece::new(('A', 1), game::PieceType::King, game::PlayerColor::Black)?;
    let mut pieces = get_startup_pieces_black()?;
    pieces.extend(get_startup_pieces_white()?);
    let board = Board::new(pieces);

    let terminal = ChessTerminal;
    terminal.display(&board)?;

    Ok(())
}
