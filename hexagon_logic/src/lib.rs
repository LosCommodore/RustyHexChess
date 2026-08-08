use crate::{
    board::{Action, Board, MoveOption},
    coordinates::Position,
    piece::{
        BLACK_PAWNS_PROMOTION_POSITIONS, Piece, PieceType, WHITE_PAWNS_PROMOTION_POSITIONS,
        get_startup_pieces_black, get_startup_pieces_white,
    },
};
use strum::EnumIter;
use thiserror::Error; // Cleaned up unified import

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

    #[error("invalid position")]
    InvalidPosition,
}
type Result<T> = std::result::Result<T, MoveError>;

pub mod board;
pub mod coordinates;
pub mod display;
mod movement;
pub mod piece;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
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
    pub fn make_move<T, U>(&mut self, origin: T, destination: U) -> Result<Move>
    where
        T: TryInto<Position>,
        T::Error: std::fmt::Debug, // Accepts () or Infallible or any Debug type
        U: TryInto<Position>,
        U::Error: std::fmt::Debug,
    {
        use MoveError::*;
        let origin = origin.try_into().map_err(|_| InvalidPosition)?;
        let destination = destination.try_into().map_err(|_| InvalidPosition)?;

        let piece_clone = self
            .board
            .pieces
            .get(&origin)
            .ok_or(NoPieceAtPosition)?
            .clone();

        if piece_clone.side != self.active_side {
            return Err(WrongPlayer);
        }

        let options = self.board.get_movement_options(origin)?;

        // todo: add en passant here -> extend movement options for pawn.

        let valid_move = options
            .iter()
            .find(|option| option.pos == destination)
            .ok_or(IllegalMove)?;

        match valid_move.action {
            Action::Move => {
                let piece = self.board.pieces.remove(&origin).expect("No piece ???");
                assert!(self.board.pieces.insert(destination, piece).is_none());
            }
            Action::Capture => {
                let piece = self.board.pieces.remove(&origin).expect("No piece ???");
                assert!(self.board.pieces.insert(destination, piece).is_some());
            }
        };

        let move_ = Move {
            piece: piece_clone.clone(),
            origin,
            destination,
            action: valid_move.action,
        };

        if piece_clone.piece_type == PieceType::Pawn {
            let promotion_fields = match piece_clone.side {
                Side::Black => BLACK_PAWNS_PROMOTION_POSITIONS,
                Side::White => WHITE_PAWNS_PROMOTION_POSITIONS,
            };

            let is_promotion = promotion_fields.iter().any(|x| *x == destination);
            if is_promotion {
                println!("You are promoted !")
            }
        }

        self.next_turn();
        Ok(move_)
    }

    pub fn get_movement_options(&self, pos: Position) -> Result<Vec<MoveOption>> {
        self.board.get_movement_options(pos)
        // todo -> add en passant
    }

    pub fn next_turn(&mut self) {
        self.active_side = match self.active_side {
            Side::Black => Side::White,
            Side::White => Side::Black,
        };
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn mark_move_options<T>(&mut self, pos: T) -> Result<()>
    where
        T: TryInto<Position>,
        T::Error: std::fmt::Debug, // Accepts () or Infallible or any Debug type
    {
        let pos = pos.try_into().map_err(|_| MoveError::InvalidPosition)?;
        let move_options = self.get_movement_options(pos)?;
        self.board.mark_move_options(&move_options);
        Ok(())
    }
}
