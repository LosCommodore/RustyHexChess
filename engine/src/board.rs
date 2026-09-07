use serde::Serialize;
use serde_with::Same;
use serde_with::serde_as;
use thiserror::Error;

use super::piece::Piece;
use std::collections::BTreeMap;

use crate::movement::pawn_capture_moves;
use crate::movement::pawn_capture_moves_reversed;
use crate::piece::BLACK_PAWNS_PROMOTION_POSITIONS;
use crate::piece::PieceType;
use crate::piece::WHITE_PAWNS_PROMOTION_POSITIONS;
use crate::piece::pawn_starting_positions;
use crate::{
    Side,
    coordinates::{BOARD_DIM, Position},
    movement::{MovementPattern, get_movement_patterns},
};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum MoveError {
    #[error("Given position y: {y} / x:{x} invalid")]
    InvalidPosition { y: usize, x: usize },

    #[error("no piece at position: {0}")]
    NoPieceAtPosition(Position),

    #[error("destination is not reachable")]
    IllegalMove,
}

type Result<T> = std::result::Result<T, MoveError>;

/// Possible Actions for a Piece
/// Note:
///     - There is no castling
///     - The Pawn is the special case:
/// The pawn may move one vacant cell vertically forward.
///     1. If it stands on its starting cell or on the starting cell of any other pawn of its color,
///      then it is also allowed to move two vacant cells vertically forward.
///     2. It may capture one cell orthogonally forward at a 60° angle to the vertical, including capturing en passant.
///     3. It is promoted when it reaches the end of any file.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum Action {
    Move,
    Capture { enemy: Piece, pos: Position }, // pos for en passant capture
    Promote { to: Piece },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GameMove {
    pub piece: Piece,
    pub origin: Position,
    pub destination: Position,
    pub action: Action,
}

#[serde_as]
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Board {
    #[serde_as(as = "Vec<(Same, Same)>")]
    pub pieces: BTreeMap<Position, Piece>,
}

#[derive(Clone, Copy, Debug)]
pub enum Capability {
    Both,
    Capture,
    Move,
}

impl GameMove {
    pub fn does_promote(&self) -> bool {
        if self.piece.piece_type == PieceType::Pawn {
            let promotion_fields = match self.piece.side {
                Side::Black => &BLACK_PAWNS_PROMOTION_POSITIONS,
                Side::White => &WHITE_PAWNS_PROMOTION_POSITIONS,
            };

            promotion_fields.contains(&self.destination)
        } else {
            false
        }
    }

    pub fn is_en_passant(&self) -> bool {
        if let Action::Capture { pos, .. } = self.action {
            pos != self.destination
        } else {
            false
        }
    }
}

impl Board {
    pub fn get_movement_options(&self, pos: Position) -> Result<Vec<GameMove>> {
        let me = self
            .pieces
            .get(&pos)
            .ok_or(MoveError::NoPieceAtPosition(pos))?;

        let mut moves = Vec::new();
        for p in get_movement_patterns(me.piece_type()) {
            match p {
                MovementPattern::Walk { direction, limit } => {
                    moves.extend(self.get_walk_moves(me, pos, *direction, *limit, Capability::Both))
                }
                MovementPattern::Step(steps) => {
                    moves.extend(self.get_step_moves(me, pos, steps, Capability::Both))
                }
                MovementPattern::Pawn => moves.extend(self.get_pawn_moves(me, pos)),
            }
        }
        Ok(moves)
    }

    pub fn is_movement_option(
        &self,
        origin: Position,
        me: &Piece,
        dy: isize,
        dx: isize,
        capture_mode: Capability,
    ) -> Option<GameMove> {
        let (mut y, mut x) = origin.pos();

        y = y.checked_add_signed(dy)?;
        x = x.checked_add_signed(dx)?;

        let destination = Position::new(y, x)?;

        if let Some(enemy_piece) = self.pieces.get(&destination) {
            if enemy_piece.side() == me.side {
                return None;
            } else {
                match capture_mode {
                    Capability::Move => return None,
                    _ => {
                        return Some(GameMove {
                            piece: me.clone(),
                            origin,
                            destination,
                            action: Action::Capture {
                                enemy: enemy_piece.clone(),
                                pos: destination,
                            },
                        });
                    }
                }
            };
        }

        match capture_mode {
            Capability::Capture => None,
            _ => Some(GameMove {
                piece: me.clone(),
                origin,
                destination,
                action: Action::Move,
            }),
        }
    }

    // The pawn is a special case and therefore has its own function
    fn get_pawn_moves(&self, me: &Piece, pos: Position) -> Vec<GameMove> {
        let mut options = Vec::new();
        let color = me.side;
        let orientation = if color == Side::White { 1 } else { -1 };
        let direction = (0, orientation);

        // --- walk two steps from starting position
        let starting_positions = pawn_starting_positions(color);

        if starting_positions.contains(&pos) {
            options.extend(self.get_walk_moves(me, pos, direction, Some(2), Capability::Move));
        } else {
            // --- the normal step of the figure
            options.extend(self.get_step_moves(me, pos, &[direction], Capability::Move));
        }

        // --- capture diagonally
        let capture_moves = pawn_capture_moves(color);

        for (dy, dx) in capture_moves {
            let option = self.is_movement_option(pos, me, *dy, *dx, Capability::Capture);
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
        capability: Capability,
    ) -> Vec<GameMove> {
        let mut options = Vec::new();

        for (dy, dx) in steps {
            let option = self.is_movement_option(pos, me, *dy, *dx, capability);
            options.extend(option);
        }

        options
    }

    // Walking is stepping into the direction (dy,dx) for nr_steps
    fn get_walk_moves(
        &self,
        me: &Piece,
        origin: Position,
        (dy, dx): (isize, isize),
        nr_steps: Option<usize>,
        capability: Capability,
    ) -> Vec<GameMove> {
        let mut pos = origin;
        let mut options = Vec::new();
        for _ in 0..nr_steps.unwrap_or(BOARD_DIM) {
            let Some(mut new_move) = self.is_movement_option(pos, me, dy, dx, capability) else {
                return options;
            };
            new_move.origin = origin;
            options.push(new_move.clone());
            if let Action::Capture { .. } = new_move.action {
                break;
            }

            pos = new_move.destination;
        }
        options
    }

    pub fn execute(&mut self, game_move: &GameMove) {
        let me = self
            .pieces
            .remove(&game_move.origin)
            .expect("No piece at origin");

        match &game_move.action {
            Action::Move => {
                self.pieces.insert(game_move.destination, me);
            }
            Action::Capture { pos: enemy_pos, .. } => {
                self.pieces.remove(enemy_pos);
                self.pieces.insert(game_move.destination, me);
            }
            Action::Promote { to } => {
                self.pieces.insert(game_move.destination, to.clone());
            }
        }
    }

    pub fn undo(&mut self, game_move: &GameMove) {
        let me = self
            .pieces
            .remove(&game_move.destination)
            .expect("No piece at destination");

        match &game_move.action {
            Action::Move => {
                self.pieces.insert(game_move.origin, me);
            }
            Action::Capture {
                pos: enemy_pos,
                enemy: taken_piece,
            } => {
                self.pieces.insert(*enemy_pos, taken_piece.clone());
                self.pieces.insert(game_move.origin, me);
            }
            Action::Promote { .. } => {
                self.pieces
                    .insert(game_move.destination, game_move.piece.clone());
            }
        }
    }

    // Get en passant field (the destination where can be captured) for a given game move. Move has to be by a pawn doing two steps
    pub fn get_en_passant_field(&self, game_move: &GameMove) -> Option<Position> {
        if game_move.piece.piece_type != PieceType::Pawn {
            return None;
        }

        let dx = game_move.destination.coordinates().1 as isize
            - game_move.origin.coordinates().1 as isize;
        if dx.abs() < 2 {
            return None;
        }

        let x = (game_move.origin.pos().1 as isize + dx.signum()) as usize;
        Some(Position::new(game_move.origin.pos().0, x).expect("Invalid position ???"))
    }

    // Get en passant game moves for a given board position
    pub fn get_en_passant_moves(&self, active_player: Side, last_move: &GameMove) -> Vec<GameMove> {
        let mut game_moves = Vec::new();

        let Some(en_passant_pos) = self.get_en_passant_field(last_move) else {
            return game_moves;
        };

        for (dy, dx) in pawn_capture_moves_reversed(active_player) {
            let Some(possible_pawn_pos) = en_passant_pos.add(*dy, *dx) else {
                continue;
            };

            let Some(piece) = self.pieces.get(&possible_pawn_pos) else {
                continue;
            };

            if piece.piece_type != PieceType::Pawn || piece.side != active_player {
                continue;
            }

            let new_move = GameMove {
                piece: piece.clone(),
                origin: possible_pawn_pos,
                destination: en_passant_pos,
                action: Action::Capture {
                    enemy: last_move.piece.clone(),
                    pos: last_move.destination,
                },
            };

            game_moves.push(new_move);
        }
        game_moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::save_board_to_html_file;
    use crate::piece::{BLACK_PAWNS_STARTING_POSITIONS, PieceType, WHITE_PAWNS_STARTING_POSITIONS};
    use std::collections::HashSet;
    use std::path::PathBuf;
    use strum::IntoEnumIterator;

    fn snap_board(board: &Board, markers: &HashSet<Position>, snapshot_name: &str) {
        let path = get_html_repr_path(snapshot_name);
        save_board_to_html_file(board, markers, path).expect("html could not be generated");
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

        let markers: HashSet<Position> = options.iter().map(|x| x.destination).collect();

        snap_board(&board, &markers, snapshot_name);
        insta::assert_debug_snapshot!(snapshot_name, options);
    }

    fn move_piece(pos: Position, piece: Piece, snapshot_name: &str) -> Board {
        let mut board = Board::default();
        board.pieces.insert(pos, piece);
        mark_and_snap(&mut board, &[pos], snapshot_name);
        board
    }

    #[test]
    fn test_move_rook() {
        let piece = Piece {
            piece_type: PieceType::Rook,
            side: Side::Black,
        };
        move_piece(
            Position::from_human(('F', 5)).unwrap(),
            piece,
            "test_move_rook",
        );
    }

    #[test]
    fn test_move_queen() {
        let piece = Piece {
            piece_type: PieceType::Queen,
            side: Side::Black,
        };
        move_piece(
            Position::from_human(('F', 5)).unwrap(),
            piece,
            "test_move_queen",
        );
    }

    #[test]
    fn test_move_king() {
        let piece = Piece {
            piece_type: PieceType::King,
            side: Side::Black,
        };
        move_piece(
            Position::from_human(('F', 5)).unwrap(),
            piece,
            "test_move_king",
        );
    }

    #[test]
    fn test_move_bishop() {
        let piece = Piece {
            piece_type: PieceType::Bishop,
            side: Side::Black,
        };
        move_piece(
            Position::from_human(('F', 5)).unwrap(),
            piece,
            "test_move_bishop",
        );
    }

    #[test]
    fn test_move_knight() {
        let piece = Piece {
            piece_type: PieceType::Knight,
            side: Side::Black,
        };
        move_piece(
            Position::from_human(('F', 5)).unwrap(),
            piece,
            "test_move_knight",
        );
    }

    #[test]
    fn test_move_pawn_black() {
        let piece = Piece {
            piece_type: PieceType::Pawn,
            side: Side::Black,
        };
        move_piece(
            Position::from_human(('F', 5)).unwrap(),
            piece,
            "test_move_pawn_black",
        );
    }

    #[test]
    fn test_move_pawn_white() {
        let piece = Piece {
            piece_type: PieceType::Pawn,
            side: Side::White,
        };
        move_piece(
            Position::from_human(('F', 5)).unwrap(),
            piece,
            "test_move_pawn_white",
        );
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

    #[test]
    fn test_capture_with_pawn() {
        let mut board = Board::default();
        let pos = Position::from_human(('F', 5)).unwrap();
        board.pieces.insert(
            pos,
            Piece {
                piece_type: PieceType::Pawn,
                side: Side::White,
            },
        );
        board.pieces.insert(
            Position::from_human(('G', 5)).unwrap(),
            Piece {
                piece_type: PieceType::Bishop,
                side: Side::Black,
            },
        );
        board.pieces.insert(
            Position::from_human(('F', 6)).unwrap(),
            Piece {
                piece_type: PieceType::Rook,
                side: Side::Black,
            },
        );

        board.pieces.insert(
            Position::from_human(('E', 6)).unwrap(),
            Piece {
                piece_type: PieceType::Queen,
                side: Side::Black,
            },
        );
        mark_and_snap(&mut board, &[pos], &format!("test_capture_with_pawn"));
    }

    #[test]
    fn test_normal_capture() {
        let mut board = Board::default();
        let pos = Position::from_human(('F', 5)).unwrap();
        board.pieces.insert(
            pos,
            Piece {
                piece_type: PieceType::Queen,
                side: Side::White,
            },
        );
        board.pieces.insert(
            Position::from_human(('F', 8)).unwrap(),
            Piece {
                piece_type: PieceType::King,
                side: Side::Black,
            },
        );
        board.pieces.insert(
            Position::from_human(('I', 8)).unwrap(),
            Piece {
                piece_type: PieceType::King,
                side: Side::White,
            },
        );
        mark_and_snap(&mut board, &[pos], &format!("test_normal_capture"));
    }
}
