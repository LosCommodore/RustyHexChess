use std::todo;

use crate::{
    board::{Action, Board},
    coordinates::Position,
    piece::{Piece, get_startup_pieces_black, get_startup_pieces_white},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("Given position {0} is outside of the board")]
    OutsideBoard(Position),

    #[error("no piece at source position")]
    NoPieceAtPosition,

    #[error("destination is not reachable")]
    IllegalMove,

    #[error("piece belongs to the other player")]
    WrongPlayer,
}
type Result<T> = std::result::Result<T, MoveError>;

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

    /// Make a move on the board. Move must be valid, otherwise an error will be returned
    pub fn make_move(&mut self, origin: Position, destination: Position) -> Result<Move> {
        use MoveError::*;

        let piece = self.board.pieces.get(&origin).ok_or(NoPieceAtPosition)?;

        if piece.side != self.active_side {
            return Err(WrongPlayer);
        }

        let options = self.board.get_movement_options(origin)?;

        let valid_move = options
            .iter()
            .find(|option| option.pos == destination)
            .ok_or(IllegalMove)?;

        let move_ = match valid_move.action {
            Action::Move => {
                let piece = self.board.pieces.remove(&origin).expect("No piece ???");
                let piece_clone = piece.clone();
                assert!(self.board.pieces.insert(destination, piece).is_none());
                Move {
                    piece: piece_clone,
                    origin,
                    destination,
                    action: valid_move.action,
                }
            }
            Action::Capture => todo!(),

            Action::Promote => todo!(),
            Action::CaptureEnPassant => todo!(),
        };
        Ok(move_)
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
    //pub fn possible_moves(...) -> Vec<Move>;
}
