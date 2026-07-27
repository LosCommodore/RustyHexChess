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

    let piece_pos = Position::try_from(('D', 6))?;
    let piece = Piece {
        piece_type: PieceType::Bishop,
        color: Color::Black,
    };
    board.pieces.insert(piece_pos, piece);

    let king_pos = Position::try_from(('H', 6))?;
    let king = Piece {
        piece_type: PieceType::King,
        color: Color::Black,
    };
    board.pieces.insert(king_pos, king);

    let options = board.get_movement_options(piece_pos)?;

    for p in options {
        board.markers.insert(p, Marker::MovementOption);
    }

    let terminal = ChessTerminal;
    terminal.display(&board)?;

    Ok(())
}
