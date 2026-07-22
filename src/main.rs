#![allow(unused)]

mod display;
mod game;

use crate::{
    display::{BoardDisplay, ChessTerminal}, game::{
        board::{Board, Marker}, coordinates::Position, piece::{Color, Piece, PieceType, get_startup_pieces_black, get_startup_pieces_white},
    },
};
use anyhow::Result;

fn main() -> Result<()> {
    ChessTerminal::clc()?;

    //let _piece = Piece::new(('A', 1), game::PieceType::King, game::PlayerColor::Black)?;
    let mut pieces = Vec::new();
    let rook_pos = Position::from_human(('F',6))?;
    let rook = Piece::new(rook_pos, PieceType::Rook, Color::Black)?;
    pieces.push(rook);

    //pieces.extend(get_startup_pieces_black()?);
    //pieces.extend(get_startup_pieces_white()?);
    let mut board = Board::new(pieces);

    /*
    board.markers.insert(Position{y:5,x:5}, Marker::MovementOption);
    board.markers.insert(Position{y:5,x:4}, Marker::MovementOption);
    board.markers.insert(Position{y:5,x:6}, Marker::MovementOption);
    */
    
    let options = board.get_movement_options(rook_pos)?;
    for p in options {
       board.markers.insert(p, Marker::MovementOption);
    }
    
    let terminal = ChessTerminal;
    terminal.display(&board)?;

    Ok(())
}
