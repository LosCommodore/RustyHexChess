#![allow(unused)]

use std::{error::Error, marker};

use engine::{
    Side,
    board::Board,
    coordinates::Position,
    display::{self, save_board_to_html_file, write_html},
    game::Game,
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

    game.make_human_move(('B', 5), ('B', 6))?;

    let mv_options = game.get_movement_options(Position::from_human(('B', 6)).unwrap())?;
    let markers = mv_options.iter().map(|x| x.destination).collect();
    terminal.display(game.board(), &markers)?;

    let mut output_dir = find_output_directory();
    output_dir.push("my_board.html");
    save_board_to_html_file(game.board(), &markers, output_dir)?;
    Ok(())
}
