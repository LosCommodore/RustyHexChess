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

fn main() -> Result<()> {
    ChessTerminal::clc()?;
    let mut game = Game::new();
    let terminal = ChessTerminal;

    game.make_move(('B', 5), ('B', 6))?;
    game.mark_move_options(('B', 6));

    terminal.display(game.board())?;

    let mut output_dir = find_output_directory();
    output_dir.push("my_board.html");
    save_board_to_html_file(game.board(), output_dir)?;
    Ok(())
}
