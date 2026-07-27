#![allow(unused)]

mod display;
mod game;

use crate::{
    display::{BoardDisplay, ChessTerminal},
    game::{
        board::{Board, Marker},
        coordinates::Position,
        piece::{Color, Piece, PieceType, get_startup_pieces_black, get_startup_pieces_white},
    },
};
use anyhow::Result;

fn main() -> Result<()> {
    ChessTerminal::clc()?;

    let mut board = Board::default();

    let rook_pos = Position::try_from(('F', 6))?;
    let rook = Piece {
        piece_type: PieceType::Rook,
        color: Color::Black,
    };
    board.pieces.insert(rook_pos, rook);

    let king_pos = Position::try_from(('H', 6))?;
    let king = Piece {
        piece_type: PieceType::King,
        color: Color::Black,
    };
    board.pieces.insert(king_pos, king);

    let options = board.get_movement_options(rook_pos)?;

    for p in options {
        board.markers.insert(p, Marker::MovementOption);
    }

    let terminal = ChessTerminal;
    terminal.display(&board)?;

    Ok(())
}
