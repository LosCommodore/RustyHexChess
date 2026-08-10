pub mod board;
pub mod coordinates;
pub mod display;
mod movement;
pub mod piece;

use crate::{
    board::{Action, Board, MoveError, MoveOption},
    coordinates::Position,
    piece::{
        BLACK_PAWNS_PROMOTION_POSITIONS, Piece, PieceType, WHITE_PAWNS_PROMOTION_POSITIONS,
        get_startup_pieces_black, get_startup_pieces_white,
    },
};
use strum::EnumIter;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UserError {
    #[error(transparent)]
    MoveError(#[from] board::MoveError),

    #[error("piece belongs to the other player")]
    WrongPlayer,
}

pub struct GameError<T> {
    pub game: T,
    pub error: UserError,
}

impl<G> GameError<G> {
    fn new(game: G, error: UserError) -> Self {
        Self { game, error }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum Side {
    #[default]
    White,
    Black,
}

#[allow(unused)]
pub struct Move {
    origin: Position,
    destination: Position,
    action: Action,
}

#[allow(unused)]
#[derive(Default)]
pub struct Game<T> {
    board: Board,
    active_side: Side,
    moves: Vec<Move>,
    state: T,
}

pub struct NormalTurn;
pub struct GameOver;
pub struct PromotePawn {
    origin: Position,
    destination: Position,
}

pub enum NextTurn {
    Continued(Game<NormalTurn>),
    PromotionRequired(Game<PromotePawn>),
    GameOver(Game<GameOver>),
}

pub type GameResult<Game> = std::result::Result<NextTurn, GameError<Game>>;
pub type Result<T> = std::result::Result<T, UserError>;

pub fn new_game() -> Game<NormalTurn> {
    let mut board = Board::default();
    board.pieces.extend(get_startup_pieces_white());
    board.pieces.extend(get_startup_pieces_black());
    Game {
        board,
        active_side: Side::White,
        moves: Vec::new(),
        state: NormalTurn,
    }
}

impl<T> Game<T> {
    pub fn transition<N>(self, new_state: N) -> Game<N> {
        Game {
            board: self.board,
            active_side: self.active_side,
            moves: self.moves,
            state: new_state,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}

impl Game<NormalTurn> {
    /// Make a move on the board. Move must be valid, otherwise an error will be returned
    pub fn make_move<T, U>(mut self, origin_pos: T, destination_pos: U) -> GameResult<Self>
    where
        T: TryInto<Position>,
        T::Error: std::fmt::Debug, // Accepts () or Infallible or any Debug type
        U: TryInto<Position>,
        U::Error: std::fmt::Debug,
    {
        let Ok(origin) = origin_pos.try_into() else {
            return Err(GameError::new(self, MoveError::InvalidPosition.into()));
        };

        let Ok(destination) = destination_pos.try_into() else {
            return Err(GameError::new(self, MoveError::InvalidPosition.into()));
        };

        let Some(&Piece {
            side, piece_type, ..
        }) = self.board.pieces.get(&origin)
        else {
            let err = GameError::new(self, MoveError::NoPieceAtPosition.into());
            return Err(err);
        };

        if side != self.active_side {
            return Err(GameError::new(self, UserError::WrongPlayer));
        }

        let options = match self.board.get_movement_options(origin) {
            Ok(x) => x,
            Err(err) => return Err(GameError::new(self, err.into())),
        };

        let Some(valid_move) = options.iter().find(|option| option.pos == destination) else {
            return Err(GameError::new(self, MoveError::IllegalMove.into()));
        };

        let target_occupied = self.board.pieces.contains_key(&destination);

        match valid_move.action {
            Action::Move => {
                assert!(!target_occupied, "Move action targeted an occupied square!")
            }
            Action::Capture => {
                assert!(target_occupied, "Capture action targeted an empty square!")
            }
        }

        // -- Promotion logic
        if piece_type == PieceType::Pawn {
            let promotion_fields = match side {
                Side::Black => &BLACK_PAWNS_PROMOTION_POSITIONS,
                Side::White => &WHITE_PAWNS_PROMOTION_POSITIONS,
            };

            let is_promotion = promotion_fields.iter().any(|x| *x == destination);
            if is_promotion {
                return Ok(NextTurn::PromotionRequired(self.transition(PromotePawn {
                    origin,
                    destination,
                })));
            }
        }

        // -- Normal move logic
        self.move_piece(origin, destination, valid_move.action);
        self.next_turn();
        Ok(NextTurn::Continued(self))
    }

    // Moves a piece and removes anything at the destination from the board
    fn move_piece(&mut self, origin: Position, destination: Position, action: Action) {
        let piece = self
            .board
            .pieces
            .remove(&origin)
            .expect("No piece at origin");

        self.board.pieces.insert(destination, piece);

        self.moves.push(Move {
            origin,
            destination,
            action,
        });
    }

    pub fn get_movement_options(&self, pos: Position) -> Result<Vec<MoveOption>> {
        self.board.get_movement_options(pos).map_err(|e| e.into())
        // todo -> add en passant
    }

    fn next_turn(&mut self) {
        self.active_side = match self.active_side {
            Side::Black => Side::White,
            Side::White => Side::Black,
        };
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

impl Game<PromotePawn> {
    pub fn promote(mut self, piece_type: PieceType) -> GameResult<Self> {
        self.board
            .pieces
            .remove(&self.state.origin)
            .expect("No piece at origin");

        let new_piece = Piece {
            piece_type,
            side: self.active_side,
        };

        self.board.pieces.insert(self.state.destination, new_piece);

        Ok(NextTurn::Continued(self.transition(NormalTurn)))
    }
}
