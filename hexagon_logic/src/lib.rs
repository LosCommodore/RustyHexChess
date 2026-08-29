pub mod board;
pub mod coordinates;
pub mod display;
mod movement;
pub mod piece;

use std::{collections::HashMap, ops::Not};

use crate::{
    board::{Action, Board, GameMove, MoveError},
    coordinates::{HumanNotation, Position},
    piece::{
        BLACK_PAWNS_PROMOTION_POSITIONS, Piece, PieceType, WHITE_PAWNS_PROMOTION_POSITIONS,
        get_startup_pieces_black, get_startup_pieces_white,
    },
};
use serde::Serialize;
use strum::EnumIter;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UserError {
    #[error(transparent)]
    MoveError(#[from] board::MoveError),

    #[error(transparent)]
    CoordinateError(#[from] coordinates::CoordinateError),

    #[error("piece belongs to the other player")]
    WrongPlayer,

    #[error("There is no move to undo")]
    CannotUndo,
}

#[derive(Debug)]
pub struct GameError<G> {
    pub game: G,
    pub error: UserError,
}

impl<G> GameError<G> {
    fn new(game: G, error: impl Into<UserError>) -> Self {
        Self {
            game,
            error: error.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Serialize)]
pub enum Side {
    #[default]
    White,
    Black,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Game<T> {
    board: Board,
    active_side: Side,
    moves: Vec<GameMove>,
    _state: T,
}

#[derive(Debug, Serialize)]
pub struct NormalTurn;

#[derive(Debug, Serialize)]
pub struct GameOver;

#[derive(Debug, Serialize)]
pub struct PromotePawn;

#[derive(Debug)]
pub enum NextTurn {
    Continued(Game<NormalTurn>),
    PromotionRequired(Game<PromotePawn>),
    GameOver(Game<GameOver>),
}

#[derive(PartialEq, Eq, Debug)]
pub enum KingState {
    Nothing,
    Check,
    Mate,
}

pub type GameResult<Game> = std::result::Result<NextTurn, GameError<Game>>;
pub type Result<T> = std::result::Result<T, UserError>;

pub fn new_game(board: Option<Board>) -> Game<NormalTurn> {
    let board = board.unwrap_or_else(|| {
        let mut board = Board::default();
        board.pieces.extend(get_startup_pieces_white());
        board.pieces.extend(get_startup_pieces_black());
        board
    });

    Game {
        board,
        active_side: Side::White,
        moves: Vec::new(),
        _state: NormalTurn,
    }
}

impl<T> Game<T> {
    pub fn transition<N>(self, new_state: N) -> Game<N> {
        Game {
            board: self.board,
            active_side: self.active_side,
            moves: self.moves,
            _state: new_state,
        }
    }

    fn undo_next_turn(self, mv: GameMove) -> NextTurn {
        match mv {
            GameMove {
                action: Action::Capture { .. } | Action::Move,
                ..
            } => NextTurn::Continued(self.transition(NormalTurn)),
            GameMove {
                action: Action::Promote { .. },
                ..
            } => NextTurn::PromotionRequired(self.transition(PromotePawn)),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    fn next_turn(&mut self) {
        self.active_side = !self.active_side;
    }

    pub fn pieces_by_side(&self, side: Side) -> HashMap<Position, Piece> {
        self.board
            .pieces
            .iter()
            .filter(|(_, piece)| piece.side == side)
            .map(|(&pos, piece)| (pos, piece.clone()))
            .collect()
    }
}

impl Game<NormalTurn> {
    // Make a move using human coordinates
    pub fn make_human_move(
        self,
        origin: HumanNotation,
        destination: HumanNotation,
    ) -> GameResult<Self> {
        let origin = match Position::from_human(origin) {
            Ok(x) => x,
            Err(e) => {
                return Err(GameError::new(self, e));
            }
        };

        let destination = match Position::from_human(destination) {
            Ok(x) => x,
            Err(e) => {
                return Err(GameError {
                    game: self,
                    error: e.into(),
                });
            }
        };

        self.make_move(origin, destination)
    }

    fn validate_move(&self, origin: Position, destination: Position) -> Result<GameMove> {
        let piece = self
            .board
            .pieces
            .get(&origin)
            .ok_or(MoveError::NoPieceAtPosition(origin))?;

        if piece.side != self.active_side {
            return Err(UserError::WrongPlayer);
        }

        let options = self.board.get_movement_options(origin)?;

        let option = options
            .iter()
            .find(|option| option.destination == destination)
            .cloned()
            .ok_or(MoveError::IllegalMove)?;

        Ok(option)
    }

    /// Make a move on the board. Move must be valid, otherwise an error will be returned
    pub fn make_move(mut self, origin: Position, destination: Position) -> GameResult<Self> {
        let game_move = match self.validate_move(origin, destination) {
            Ok(game_move) => game_move,
            Err(e) => return Err(GameError::new(self, e)),
        };

        // -- Normal move logic
        self.board.execute(&game_move);
        self.moves.push(game_move.clone());

        // -- Promotion logic
        if game_move.piece.piece_type == PieceType::Pawn {
            let promotion_fields = match game_move.piece.side {
                Side::Black => &BLACK_PAWNS_PROMOTION_POSITIONS,
                Side::White => &WHITE_PAWNS_PROMOTION_POSITIONS,
            };

            let is_promotion = promotion_fields.contains(&destination);
            if is_promotion {
                return Ok(NextTurn::PromotionRequired(self.transition(PromotePawn)));
            }
        }

        // -- Next turn
        self.next_turn();
        Ok(NextTurn::Continued(self))
    }

    pub fn get_movement_options(&self, pos: Position) -> Result<Vec<GameMove>> {
        self.board.get_movement_options(pos).map_err(|e| e.into())
        // todo -> add en passant
    }

    pub fn king_in_check(&self, kings_side: Side) -> bool {
        let enemy_pieces = self.pieces_by_side(!kings_side);
        let Some((&pos_king, _)) = self.board.pieces.iter().find(|(_, piece)| {
            piece.side == self.active_side && piece.piece_type == PieceType::King
        }) else {
            panic!("King is missing on board")
        };

        let is_check: bool = enemy_pieces.iter().any(|(&pos, _)| {
            self.get_movement_options(pos)
                .map(|options| options.iter().any(|x| x.destination == pos_king))
                .unwrap()
        });

        is_check
    }

    pub fn check_king(&mut self) -> KingState {
        if !self.king_in_check(self.active_side) {
            return KingState::Nothing;
        }

        let my_pieces = self.pieces_by_side(self.active_side);

        for (origin, _) in my_pieces {
            let mv_options = self
                .get_movement_options(origin)
                .expect("A piece must be here");

            for mv in mv_options {
                self.board.execute(&mv);

                if !self.king_in_check(self.active_side) {
                    self.board.undo(&mv);
                    return KingState::Check;
                }
                self.board.undo(&mv);
            }
        }
        KingState::Mate
    }

    // Undo the last game move
    pub fn undo(mut self) -> GameResult<Self> {
        let Some(mv) = self.moves.pop() else {
            return Err(GameError {
                game: self,
                error: UserError::CannotUndo,
            });
        };

        self.board.undo(&mv);
        self.active_side = !self.active_side;
        Ok(self.undo_next_turn(mv))
    }
}

impl Game<PromotePawn> {
    pub fn promote(mut self, piece_type: PieceType) -> GameResult<Self> {
        let destination = self
            .moves
            .last()
            .expect("promotion without history?")
            .destination;

        let new_piece = Piece {
            piece_type,
            side: self.active_side,
        };

        let old_piece = self.board.pieces.insert(destination, new_piece.clone());
        self.moves.push(GameMove {
            piece: old_piece.expect("no piece to promote ??"),
            origin: destination,
            destination,
            action: Action::Promote { to: new_piece },
        });
        self.next_turn();

        Ok(NextTurn::Continued(self.transition(NormalTurn)))
    }

    pub fn undo(mut self) -> GameResult<Self> {
        let Some(mv) = self.moves.pop() else {
            return Err(GameError {
                game: self,
                error: UserError::CannotUndo,
            });
        };

        self.board.undo(&mv);
        Ok(self.undo_next_turn(mv))
    }
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, bail};
    use core::panic;

    #[test]
    fn test_promote_pawn() {
        // -- Create game with pawn
        let mut board = Board::default();
        let origin: Position = Position::from_human(('K', 5)).unwrap();
        let destination: Position = Position::from_human(('K', 6)).unwrap();

        board.pieces.insert(
            origin,
            Piece {
                piece_type: PieceType::Pawn,
                side: Side::White,
            },
        );
        let game = new_game(Some(board));

        // -- Move pawn
        let game_result = game.make_move(origin, destination);

        // -- Promote
        let game = match game_result.expect("Error while playing move") {
            NextTurn::PromotionRequired(game) => game,
            _ => panic!("Wrong game state"),
        };

        // -- Check normal game state again
        let NextTurn::Continued(game) = game
            .promote(PieceType::Queen)
            .expect("Error while promoting")
        else {
            panic!("Should be again normal game")
        };

        // -- Check Queen exists
        assert!(
            game.board()
                .pieces
                .get(&destination)
                .expect("no piece ??")
                .piece_type
                == PieceType::Queen
        );
        println!("{game:#?}");
    }

    #[test]
    fn test_serde_json() -> Result<()> {
        let game = new_game(None);
        serde_json::to_string(&game)?;
        Ok(())
    }

    #[test]
    fn test_check() -> Result<()> {
        let mut board = Board::default();
        board.pieces.insert(
            Position::from_human(('F', 5)).unwrap(),
            Piece {
                piece_type: PieceType::King,
                side: Side::White,
            },
        );

        let mut game = new_game(Some(board));

        let is_check = game.king_in_check(Side::White);
        assert!(!is_check, "expected no check here 1");

        game.board.pieces.insert(
            Position::from_human(('K', 5)).unwrap(),
            Piece {
                piece_type: PieceType::Bishop,
                side: Side::White,
            },
        );

        let is_check = game.king_in_check(Side::White);
        assert!(!is_check, "expected no check here 2");

        game.board.pieces.insert(
            Position::from_human(('F', 1)).unwrap(),
            Piece {
                piece_type: PieceType::Rook,
                side: Side::Black,
            },
        );

        let is_check = game.king_in_check(Side::White);
        assert!(is_check, "expected check due to rook here");

        Ok(())
    }

    #[test]
    fn test_undo() -> Result<()> {
        let mut board = Board::default();
        board.pieces.insert(
            Position::from_human(('F', 5)).unwrap(),
            Piece {
                piece_type: PieceType::King,
                side: Side::White,
            },
        );

        board.pieces.insert(
            Position::from_human(('I', 2)).unwrap(),
            Piece {
                piece_type: PieceType::King,
                side: Side::Black,
            },
        );

        let origin = Position::from_human(('F', 9)).unwrap();
        board.pieces.insert(
            origin,
            Piece {
                piece_type: PieceType::Pawn,
                side: Side::White,
            },
        );

        board.pieces.insert(
            Position::from_human(('G', 9)).unwrap(),
            Piece {
                piece_type: PieceType::Bishop,
                side: Side::Black,
            },
        );

        // White pawn takes Bishop
        let game = new_game(Some(board));
        let mut game_states = Vec::new();
        game_states.push(serde_json::to_string(&game)?);

        let NextTurn::Continued(game) = game
            .make_move(origin, Position::from_human(('G', 9)).unwrap())
            .map_err(|e| e.error)?
        else {
            bail!("wrong game state 1")
        };
        game_states.push(serde_json::to_string(&game)?);

        // Black King moves
        let NextTurn::Continued(game) = game
            .make_human_move(('I', 2), ('I', 3))
            .map_err(|e| e.error)?
        else {
            bail!("Wrong game state 2")
        };
        let last_state = serde_json::to_string(&game)?;

        // White Pawn moves
        let NextTurn::PromotionRequired(game) = game
            .make_human_move(('G', 9), ('G', 10))
            .map_err(|e| e.error)?
        else {
            bail!("Wrong game state 3")
        };

        println!("undoing move {:?}", game.moves.last());
        let NextTurn::Continued(mut game) = game.undo().map_err(|e| e.error)? else {
            bail!("wrong state while undoing")
        };
        let new_state = serde_json::to_string(&game)?;
        assert_eq!(new_state, last_state, "game state not identical");

        for game_state in game_states.into_iter().rev() {
            println!("undoing move {:?}", game.moves.last());
            let NextTurn::Continued(next_game) = game.undo().map_err(|e| e.error)? else {
                bail!("wrong state while undoing")
            };
            game = next_game;
            let new_state = serde_json::to_string(&game)?;
            assert_eq!(new_state, game_state, "game state not identical");
        }
        Ok(())
    }

    #[test]
    fn test_check_mate() {
        use PieceType::*;
        use Side::*;
        let human = Position::from_human;

        let board = Board::default();
        let mut game = new_game(Some(board));

        game.board
            .pieces
            .insert(human(('F', 5)).unwrap(), Piece::new(King, White));

        assert_eq!(game.check_king(), KingState::Nothing);

        game.board
            .pieces
            .insert(human(('F', 1)).unwrap(), Piece::new(Rook, Black));

        assert_eq!(game.check_king(), KingState::Check);

        game.board
            .pieces
            .insert(human(('E', 2)).unwrap(), Piece::new(Rook, Black));

        assert_eq!(game.check_king(), KingState::Check);

        let rook3_pos = human(('G', 1)).unwrap();
        game.board.pieces.insert(rook3_pos, Piece::new(Rook, Black));

        game.board
            .pieces
            .insert(human(('D', 3)).unwrap(), Piece::new(Rook, Black));

        assert_eq!(game.check_king(), KingState::Check);

        game.board
            .pieces
            .insert(human(('H', 1)).unwrap(), Piece::new(Rook, Black));

        assert_eq!(game.check_king(), KingState::Mate);

        game.board
            .pieces
            .insert(human(('K', 3)).unwrap(), Piece::new(Queen, White));

        assert_eq!(game.check_king(), KingState::Check);
    }
}
