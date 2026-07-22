mod display;
mod game;

use crate::{
    display::{BoardDisplay, ChessTerminal}, game::{
        board::{Board, Marker}, coordinates::Position, piece::{get_startup_pieces_black, get_startup_pieces_white},
    },
};
use anyhow::Result;

fn main() -> Result<()> {
    ChessTerminal::clc()?;

    //let _piece = Piece::new(('A', 1), game::PieceType::King, game::PlayerColor::Black)?;
    let mut pieces = get_startup_pieces_black()?;
    pieces.extend(get_startup_pieces_white()?);
    let mut board = Board::new(pieces);

    board.markers.insert(Position{y:5,x:5}, Marker::MovementOption);
    board.markers.insert(Position{y:5,x:4}, Marker::MovementOption);
    board.markers.insert(Position{y:5,x:6}, Marker::MovementOption);


    let terminal = ChessTerminal;
    terminal.display(&board)?;

    Ok(())
}
