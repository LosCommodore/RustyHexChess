use std::todo;

use crate::{
    board::{Action, Board, MoveError},
    coordinates::Position,
    piece::{Piece, get_startup_pieces_black, get_startup_pieces_white},
};

pub mod board;
pub mod coordinates;
mod movement;
pub mod piece;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Side {
    #[default]
    White,
    Black,
}

#[allow(unused)]
pub struct Move {
    piece: Piece,
    origin: Position,
    destination: Position,
    action: Action,
}

#[allow(unused)]
#[derive(Default)]
pub struct Game {
    board: Board,
    active_side: Side,
    moves: Vec<Move>,
}

impl Game {
    pub fn new() -> Self {
        let mut board = Board::default();
        board.pieces.extend(get_startup_pieces_white());
        board.pieces.extend(get_startup_pieces_black());
        Game {
            board,
            ..Default::default()
        }
    }

    pub fn make_move(
        &mut self,
        origin: Position,
        destination: Position,
    ) -> Result<Move, MoveError> {
        let options = self.board.get_movement_options(origin)?;

        let valid_move = options
            .iter()
            .find(|option| option.pos == destination)
            .ok_or(MoveError::IllegalMove)?;

        match valid_move.action {
            Action::Capture => todo!(),
            Action::Move => todo!(),
            Action::Promote => todo!(),
            Action::CaptureEnPassant => todo!(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
    //pub fn possible_moves(...) -> Vec<Move>;
}
