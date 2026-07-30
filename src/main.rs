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

fn show_options<T>(board: &mut Board, piece_pos: T) -> Result<()>
where
    T: TryInto<Position>,
    T::Error: Into<anyhow::Error>,
{
    let pos = piece_pos.try_into().map_err(Into::into)?;
    let options = board.get_movement_options(pos)?;
    for p in options {
        board.markers.insert(p.pos, Marker::MovementOption);
    }
    Ok(())
}

fn main() -> Result<()> {
    ChessTerminal::clc()?;

    let mut board = Board::default();

    let white_pieces = get_startup_pieces_white()?;
    let black_pieces = get_startup_pieces_black()?;

    let piece_pos = Position::try_from(('D', 6))?;
    let piece = Piece {
        piece_type: PieceType::Knight,
        color: Color::Black,
    };

    let king_pos = Position::try_from(('H', 6))?;
    let king = Piece {
        piece_type: PieceType::King,
        color: Color::Black,
    };

    board.pieces.extend(white_pieces);
    board.pieces.extend(black_pieces);
    //board.pieces.insert(king_pos, king);
    //board.pieces.insert(piece_pos, piece);
    let options = show_options(&mut board, ('F', 5));

    let terminal = ChessTerminal;
    terminal.display(&board)?;

    Ok(())
}
