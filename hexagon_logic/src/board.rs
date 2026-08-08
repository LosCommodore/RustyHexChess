use super::piece::Piece;
use std::collections::HashMap;

use crate::{
    MoveError, Result, Side,
    coordinates::{BOARD_DIM, Position},
    movement::{MovementPattern, get_movement_patterns},
    piece::{BLACK_PAWNS_STARTING_POSITIONS, WHITE_PAWNS_STARTING_POSITIONS},
};
pub enum Marker {
    MovementOption,
}

/// Possible Actions for a Piece
/// Note:
/// - There is no castling
/// The Pawn is the special case:
/// - The pawn may move one vacant cell vertically forward.
///   1. If it stands on its starting cell or on the starting cell of any other pawn of its color,
///      then it is also allowed to move two vacant cells vertically forward.
///   2. It may capture one cell orthogonally forward at a 60° angle to the vertical, including capturing en passant.
///   3. It is promoted when it reaches the end of any file.
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Move,
    Capture,
}

#[derive(Clone, Copy, Debug)]
pub struct MoveOption {
    pub pos: Position,
    pub action: Action,
}

#[derive(Default)]
pub struct Board {
    pub pieces: HashMap<Position, Piece>,
    pub markers: HashMap<Position, Marker>,
}

pub enum Capability {
    Both, // move or capture
    Capture,
    Move,
}

impl Board {
    pub fn get_movement_options(&self, pos: Position) -> Result<Vec<MoveOption>> {
        if !pos.is_on_board() {
            return Err(MoveError::OutsideBoard(pos));
        }
        let me = self.pieces.get(&pos).ok_or(MoveError::NoPieceAtPosition)?;

        let mut moves = Vec::new();
        for p in get_movement_patterns(me.piece_type()) {
            match p {
                MovementPattern::Walk { direction, limit } => {
                    moves.extend(self.get_walk_moves(me, pos, *direction, *limit))
                }
                MovementPattern::Step(steps) => moves.extend(self.get_step_moves(me, pos, *steps)),
                MovementPattern::Pawn => moves.extend(self.get_pawn_moves(me, pos)),
            }
        }
        Ok(moves)
    }

    pub fn is_movement_option(
        &self,
        pos: &Position,
        my_side: Side,
        dy: isize,
        dx: isize,
        capture_mode: Capability,
    ) -> Option<MoveOption> {
        let Some(y) = pos.y.checked_add_signed(dy) else {
            return None;
        };
        let Some(x) = pos.x.checked_add_signed(dx) else {
            return None;
        };

        let pos = Position { y, x };
        if !pos.is_on_board() {
            return None;
        }

        if let Some(piece) = self.pieces.get(&pos) {
            if piece.side() == my_side {
                return None;
            } else {
                match capture_mode {
                    Capability::Move => return None,
                    _ => {
                        return Some(MoveOption {
                            pos,
                            action: Action::Capture,
                        });
                    }
                }
            };
        }

        match capture_mode {
            Capability::Capture => None,
            _ => Some(MoveOption {
                pos: pos,
                action: Action::Move,
            }),
        }
    }

    // The pawn is a special case and therefore has its own function
    fn get_pawn_moves(&self, me: &Piece, pos: Position) -> Vec<MoveOption> {
        let mut options = Vec::new();
        let color = me.side;
        let orientation = if color == Side::White { 1 } else { -1 };
        let direction = (0, orientation);

        // --- the normal step of the figure
        options.extend(self.get_step_moves(me, pos, &[direction]));

        // --- walk two steps from starting position
        let starting_positions = match color {
            Side::White => WHITE_PAWNS_STARTING_POSITIONS,
            Side::Black => BLACK_PAWNS_STARTING_POSITIONS,
        };

        if starting_positions.contains(&pos) {
            options.extend(self.get_walk_moves(me, pos, direction, Some(2)));
        }

        // --- capture diagonally
        let capture_moves = match color {
            Side::White => &[(1, 0), (-1, 1)],
            Side::Black => &[(-1, 0), (1, -1)],
        };

        for (dy, dx) in capture_moves {
            let option = self.is_movement_option(&pos, me.side, *dy, *dx, Capability::Capture);
            options.extend(option);
        }
        options
    }

    // A step is a direct move to another position, no blocking of movements.
    fn get_step_moves(
        &self,
        me: &Piece,
        pos: Position,
        steps: &[(isize, isize)],
    ) -> Vec<MoveOption> {
        let mut options = Vec::new();

        for (dy, dx) in steps {
            let option = self.is_movement_option(&pos, me.side, *dy, *dx, Capability::Both);
            options.extend(option);
        }

        options
    }

    // Walking is stepping into the direction (dy,dx) for nr_steps
    fn get_walk_moves(
        &self,
        me: &Piece,
        mut pos: Position,
        (dy, dx): (isize, isize),
        nr_steps: Option<usize>,
    ) -> Vec<MoveOption> {
        let mut options = Vec::new();
        for _ in 0..nr_steps.unwrap_or(BOARD_DIM) {
            let Some(new_move) = self.is_movement_option(&pos, me.side, dy, dx, Capability::Both)
            else {
                return options;
            };
            options.push(new_move);
            pos = new_move.pos;
        }
        options
    }

    pub fn mark_move_options(&mut self, options: &[MoveOption]) {
        self.markers.clear();

        for MoveOption { pos, action: _ } in options {
            self.markers.insert(*pos, Marker::MovementOption);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinates::HumanCoordinate;
    use crate::display::save_board_to_html_file;
    use crate::piece::PieceType;
    use std::path::PathBuf;
    use strum::IntoEnumIterator;

    fn snap_board(board: &Board, snapshot_name: &str) {
        let path = get_html_repr_path(snapshot_name);
        save_board_to_html_file(board, path).expect("html could not be generated");
    }

    fn get_html_repr_path(snapshot_name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src");
        path.push("snapshots");
        path.push(format!("{snapshot_name}.html"));
        path
    }

    fn mark_and_snap(board: &mut Board, positions: &[Position], snapshot_name: &str) {
        let mut options = Vec::new();

        for p in positions {
            options.extend(
                board
                    .get_movement_options(p.clone())
                    .expect("error on movement options"),
            );
        }
        board.mark_move_options(&options);
        snap_board(&board, snapshot_name);
        insta::assert_debug_snapshot!(snapshot_name, options);
    }

    fn move_piece(pos: HumanCoordinate, piece: &Piece, snapshot_name: &str) -> Board {
        let mut board = Board::default();
        let pos = Position::try_from(pos).expect("invalid position");
        board.pieces.insert(pos, piece.clone());
        mark_and_snap(&mut board, &[pos], snapshot_name);
        board
    }

    #[test]
    fn test_move_rook() {
        let piece = Piece {
            piece_type: PieceType::Rook,
            side: Side::Black,
        };
        move_piece(('F', 5), &piece, "test_move_rook");
    }

    #[test]
    fn test_move_queen() {
        let piece = Piece {
            piece_type: PieceType::Queen,
            side: Side::Black,
        };
        move_piece(('F', 5), &piece, "test_move_queen");
    }

    #[test]
    fn test_move_king() {
        let piece = Piece {
            piece_type: PieceType::King,
            side: Side::Black,
        };
        move_piece(('F', 5), &piece, "test_move_king");
    }

    #[test]
    fn test_move_bishop() {
        let piece = Piece {
            piece_type: PieceType::Bishop,
            side: Side::Black,
        };
        move_piece(('F', 5), &piece, "test_move_bishop");
    }

    #[test]
    fn test_move_knight() {
        let piece = Piece {
            piece_type: PieceType::Knight,
            side: Side::Black,
        };
        move_piece(('F', 5), &piece, "test_move_knight");
    }

    #[test]
    fn test_move_pawn_black() {
        let piece = Piece {
            piece_type: PieceType::Pawn,
            side: Side::Black,
        };
        move_piece(('F', 5), &piece, "test_move_pawn_black");
    }

    #[test]
    fn test_move_pawn_white() {
        let piece = Piece {
            piece_type: PieceType::Pawn,
            side: Side::White,
        };
        move_piece(('F', 5), &piece, "test_move_pawn_white");
    }

    #[test]
    fn test_move_all_pawns() {
        let mut board = Board::default();

        for side in Side::iter() {
            let start = match side {
                Side::White => WHITE_PAWNS_STARTING_POSITIONS,
                Side::Black => BLACK_PAWNS_STARTING_POSITIONS,
            };

            for pos in start {
                board.pieces.insert(
                    pos,
                    Piece {
                        piece_type: PieceType::Pawn,
                        side,
                    },
                );
            }

            let name = match side {
                Side::White => "white",
                Side::Black => "black",
            };
            mark_and_snap(&mut board, &start, &format!("test_move_all_{name}_pawns"));
        }
    }
}
