use crate::game::{
    board::Board,
    piece::{get_startup_pieces_black, get_startup_pieces_white},
};

pub mod board;
pub mod coordinates;
pub mod piece;
mod movement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
}

pub struct Game {
    board: Board,
    active_side: Side,
}

impl Game {
    pub fn new() -> Self {
        let mut board = Board::default();
        board.pieces.extend(get_startup_pieces_white());
        board.pieces.extend(get_startup_pieces_black());
        Game {
            board,
            active_side: Side::White,
        }
    }
    
    //pub fn make_move(...) -> Result<(), MoveError>;
    //pub fn possible_moves(...) -> Vec<Move>;
    //pub fn piece_at(...) -> Option<&Piece>;

}
