use std::{collections::HashMap, fmt::Pointer, todo};

use super::piece::Piece;

use crate::game::{
    coordinates::{BOARD_DIM, HumanCoordinates, Position, X_RANGE}, movement::{MovementPattern, get_movement_patterns}, piece::{BLACK_PAWNS_STARTING_POSITIONS, Color, WHITE_PAWNS_STARTING_POSITIONS},
};
use anyhow::{Result, bail};

#[allow(unused)]
pub enum Marker {
    MovementOption,
}

/// Possible Actions for a Piece
/// Note:
/// - There is no castling
/// The Pawn is the special case:
/// - The pawn may move one vacant cell vertically forward.
///   1. If it stands on its starting cell or on the starting cell of any other pawn of its colour,
///      then it is also allowed to move two vacant cells vertically forward.
///   2. It may capture one cell orthogonally forward at a 60° angle to the vertical, including capturing en passant.
///   3. It is promoted when it reaches the end of any file.
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Move,
    Capture,
    CaputreEnPassant,
    Promote,
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub pos: Position,
    pub action: Action,
}

#[derive(Default)]
pub struct Board {
    pub pieces: HashMap<Position, Piece>,
    pub markers: HashMap<Position, Marker>,
}

impl Board {
    pub fn get_movement_options(&self, pos: Position) -> Result<Vec<Move>> {
        let Some(me) = self.pieces.get(&pos) else {
            bail!("No piece on this coordinate");
        };

        let mut moves = Vec::new();
        for p in get_movement_patterns(me.piece_type()) {
            match p {
                MovementPattern::Walk { direction, limit } => {
                    moves.extend(self.do_walk(me, pos, *direction, *limit))
                }
                MovementPattern::Step(steps) => moves.extend(self.do_step(me, pos, *steps)),
                MovementPattern::Pawn => moves.extend(self.move_pawn(me, pos)?),
            }
        }
        Ok(moves)
    }

    pub fn is_movement_option(
        &self,
        pos: &Position,
        me: &Piece,
        dy: isize,
        dx: isize,
        capture_only: bool, // move only allowed if a piece is captured (used for pawn)
    ) -> Option<Move> {
        let Some(y) = pos.y.checked_add_signed(dy) else {
            return None;
        };
        let Some(x) = pos.x.checked_add_signed(dx) else {
            return None;
        };

        let pos = Position { y, x };
        if !pos.is_in_field() {
            return None;
        }

        if let Some(piece) = self.pieces.get(&pos) {
            if piece.color() == me.color() {
                return None;
            } else {
                return Some(Move {
                    pos,
                    action: Action::Capture,
                });
            };
        }

        if capture_only {
            return None;
        }
        Some(Move {
            pos: pos,
            action: Action::Move,
        })
    }

    fn move_pawn(&self, me: &Piece, pos: Position) -> Result<Vec<Move>> {
        let human_pos: HumanCoordinates = pos.try_into()?;
        let mut options = Vec::new();
        let color = me.color;
        let orientation = if color == Color::White { 1 } else { -1 };
        let direction = (0, orientation);

        // --- the normal step of the figure
        options.extend(self.do_step(me, pos, &[direction]));

        // --- walk two steps from starting position
        let starting_positions = match color {
            Color::White => WHITE_PAWNS_STARTING_POSITIONS,
            Color::Black => BLACK_PAWNS_STARTING_POSITIONS,
        };

        if starting_positions.contains(&human_pos) {
            options.extend(self.do_walk(me, pos, direction, Some(2)));
        }

        // --- capture diagonally
        let capture_moves = match color {
            Color::White => &[(1, 0), (-1, 1)],
            Color::Black => &[(-1, 0), (1, -1)],
        };

        for (dy, dx) in capture_moves {
            let option = self.is_movement_option(&pos, me, *dy, *dx, true);
            options.extend(option);
        }
        Ok(options)
    }

    /// A step is a direkt move to another position, no blocking of movements.
    fn do_step(&self, me: &Piece, pos: Position, steps: &[(isize, isize)]) -> Vec<Move> {
        let mut options = Vec::new();

        for (dy, dx) in steps {
            let option = self.is_movement_option(&pos, me, *dy, *dx, false);
            options.extend(option);
        }

        options
    }

    fn do_walk(
        &self,
        me: &Piece,
        mut pos: Position,
        (dy, dx): (isize, isize),
        limit: Option<usize>,
    ) -> Vec<Move> {
        let mut options = Vec::new();
        for _ in 0..limit.unwrap_or(BOARD_DIM) {
            let Some(new_move) = self.is_movement_option(&pos, me, dy, dx, false) else {
                return options;
            };
            options.push(new_move);
            pos = new_move.pos;
        }
        options
    }
}

#[cfg(test)]
mod tests {
    use std::println;

    use super::*;

    use crate::game::piece::{self, Color, Piece, PieceType};

    #[test]
    fn test_move_rook() {
        let mut board = Board::default();
        let pos = Position::try_from(('F', 5)).expect("invalid position");
        let piece = Piece {
            piece_type: PieceType::Rook,
            color: Color::Black,
        };
        board.pieces.insert(pos, piece);

        let internal_pos = pos;

        let options = board
            .get_movement_options(internal_pos)
            .expect("error on movement options");

        println!("{options:#?}");
    }
}
