#![allow(unused)]

use std::error::Error;

use hexagon_logic::{
    Game, Side,
    board::{Board, Marker},
    coordinates::Position,
    display::{self, save_board_to_html_file, write_html},
    piece::{Piece, PieceType, get_startup_pieces_black, get_startup_pieces_white},
};

use crate::display::{BoardDisplay, ChessTerminal};
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;

use std::path::{Path, PathBuf};

fn find_output_directory() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(manifest_dir);

    path.pop();
    path.pop();
    path.push("temp_files");
    path
}

fn show_options<T>(board: &mut Board, piece_pos: T) -> Result<()>
where
    T: TryInto<Position, Error = ()>,
{
    let pos = piece_pos
        .try_into()
        .map_err(|_| anyhow!("Invalid position"))?;
    let options = board.get_movement_options(pos)?;
    for p in options {
        board.markers.insert(p.pos, Marker::MovementOption);
    }
    Ok(())
}

fn config_a_board() -> Result<Board> {
    let mut board = Board::default();

    let white_pieces = get_startup_pieces_white();
    let black_pieces = get_startup_pieces_black();

    let piece_pos = Position::try_from(('D', 6)).ok().context("Invalid pos")?;

    let piece = Piece {
        piece_type: PieceType::Knight,
        side: Side::Black,
    };

    let king_pos = Position::try_from(('H', 6)).ok().context("Invalid pos");
    let king = Piece {
        piece_type: PieceType::King,
        side: Side::Black,
    };

    board.pieces.extend(white_pieces);
    board.pieces.extend(black_pieces);
    //board.pieces.insert(king_pos, king);
    //board.pieces.insert(piece_pos, piece);
    let options = show_options(&mut board, ('F', 5));
    Ok(board)
}

fn main() -> Result<()> {
    ChessTerminal::clc()?;
    let mut game = Game::new();
    let terminal = ChessTerminal;

    game.make_move(('B', 5), Position { y: 1, x: 5 })?;
    //game.make_move(Position { y: 1, x: 5 }, Position { y: 1, x: 6 })?;
    terminal.display(game.board())?;

    let mut output_dir = find_output_directory();
    output_dir.push("my_board.html");
    save_board_to_html_file(game.board(), output_dir)?;
    Ok(())
}
