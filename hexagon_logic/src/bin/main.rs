#![allow(unused)]

use std::{error::Error, marker};

use hexagon_logic::{
    Game, NextTurn, Side,
    board::Board,
    coordinates::Position,
    display::{self, save_board_to_html_file, write_html},
    new_game,
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
    let mut game = new_game(None);
    let terminal = ChessTerminal;

    let NextTurn::Continued(mut game) = game.make_move(('B', 5), ('B', 6)).map_err(|e| e.error)?
    else {
        panic!("??")
    };

    let mv_options = game.get_movement_options(('B', 6).try_into().unwrap())?;
    let markers = mv_options.iter().map(|x| x.pos).collect();
    terminal.display(game.board(), &markers)?;

    let mut output_dir = find_output_directory();
    output_dir.push("my_board.html");
    save_board_to_html_file(game.board(), &markers, output_dir)?;
    Ok(())
}
