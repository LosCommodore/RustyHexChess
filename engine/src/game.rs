use std::{
    collections::{HashMap, HashSet},
    ops::Not,
};

use crate::{Side, board};
use crate::{
    board::{Action, Board, GameMove, MoveError},
    coordinates::{self, HumanNotation, Position},
    movement::pawn_capture_moves,
    piece::{
        BLACK_PAWNS_PROMOTION_POSITIONS, Piece, PieceType, WHITE_PAWNS_PROMOTION_POSITIONS,
        get_startup_pieces_black, get_startup_pieces_white,
    },
};
use serde::Serialize;

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

    #[error(
        "The function cannot be executed in this game state. Game is currently in state: {0:?}"
    )]
    WrongGameState(GameState),

    #[error("Promotion to type: {0:?} not allowed")]
    WrongPromotionType(PieceType),

    #[error("Invalid Board: {0:?}")]
    InvalidBoard(String),
}

#[derive(Copy, Debug, Clone, Serialize)]

pub enum OutCome {
    CheckMate,

    // no legal moves available anywhere on the board, but king is not in check
    StaleMate,

    ThreefoldRepetition,

    // A player can claim a draw if 50 consecutive moves have been played by each side (amounting to 100 total ply/half-moves) without:♟️ Any pawn being moved.⚔️ Any piece being captured.If either of those two actions happens, the counter instantly resets to zero, and the 50-move countdown starts all over again.
    FiftyMoves,

    // A game is drawn due to insufficient material if it is mathematically impossible to construct a legal checkmate position
    // King + 1 Bishop vs. King: A single bishop can only traverse hexes of its own color. Because a hex board uses 3 colors (instead of 2), a lone bishop is completely powerless to trap a king.
    // King + 1 Knight vs. King: Just like standard chess, a single knight cannot trap and mate a lone king by itself.
    InsufficientMaterial,

    Agreement,
    Resignation,
}

#[derive(Copy, Debug, Clone, Serialize)]

pub struct GameResult {
    winner: Option<Side>,
    outcome: OutCome,
}

#[derive(Copy, Default, Debug, Clone, Serialize)]
pub enum GameState {
    #[default]
    Normal,
    Promotion,
    GameOver(GameResult),
}

#[derive(Debug, Clone, Serialize)]
pub struct Game {
    board: Board,
    active_side: Side,
    moves: Vec<GameMove>,
    state: GameState,
}

#[derive(PartialEq, Eq, Debug)]
pub enum KingState {
    Nothing,
    Check { allowed_moves: Vec<GameMove> },
    Mate,
}

pub type Result<T> = std::result::Result<T, UserError>;

impl Game {
    pub fn new() -> Self {
        let mut board = Board::default();
        board.pieces.extend(get_startup_pieces_white());
        board.pieces.extend(get_startup_pieces_black());

        return Self::from_board(board).expect("Invalid board ???");
    }

    pub fn from_board(board: Board) -> Result<Self> {
        let colors: HashSet<_> = board
            .pieces
            .values()
            .filter(|piece| piece.piece_type == PieceType::King)
            .map(|piece| piece.side)
            .collect();

        if colors != HashSet::from([Side::Black, Side::White]) {
            return Err(UserError::InvalidBoard(
                "Both Kings must exist on board".into(),
            ));
        }

        Ok(Game {
            board,
            active_side: Side::White,
            moves: Vec::new(),
            state: GameState::Normal,
        })
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn active_side(&self) -> Side {
        self.active_side
    }

    /// Which command the game will accept next.
    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn game_result(&self) -> Option<GameResult> {
        match self.state {
            GameState::GameOver(x) => Some(x),
            _ => None,
        }
    }

    /// The moves played so far, oldest first. A promotion appears as its own
    /// entry after the pawn move that triggered it.
    pub fn moves(&self) -> &[GameMove] {
        &self.moves
    }

    pub fn pieces_by_side(&self, side: Side) -> HashMap<Position, Piece> {
        self.board
            .pieces
            .iter()
            .filter(|(_, piece)| piece.side == side)
            .map(|(&pos, piece)| (pos, piece.clone()))
            .collect()
    }

    pub fn king_in_check(&mut self, kings_side: Side) -> bool {
        let enemy_pieces = self.pieces_by_side(!kings_side);
        let Some((&pos_king, _)) = self
            .board
            .pieces
            .iter()
            .find(|(_, piece)| piece.side == kings_side && piece.piece_type == PieceType::King)
        else {
            unreachable!("Game invariant: both kings are present on the board")
        };

        let is_check: bool = enemy_pieces.iter().any(|(&pos, _)| {
            // movement options from board here, just the direct threads to the king, ignoring that the figure is pinned, en passant etc.
            self.board
                .get_movement_options(pos)
                .expect("Invalid position ???")
                .iter()
                .any(|x| x.destination == pos_king)
        });

        is_check
    }

    // Check if a move cannot be executed because the figure is pinned and thus the move
    // would create a check on the own king
    pub fn is_pinned_move(&mut self, mv: &GameMove) -> bool {
        self.board.execute(&mv);
        let check = self.king_in_check(mv.piece.side);
        self.board.undo(&mv);
        check
    }

    pub fn check_king(&mut self) -> KingState {
        if !self.king_in_check(self.active_side) {
            return KingState::Nothing;
        }

        let my_pieces = self.pieces_by_side(self.active_side);

        let mut allowed_moves = Vec::new();
        for (origin, _) in my_pieces {
            let mv_options = self
                .get_movement_options(origin)
                .expect("A piece must be here");

            for mv in mv_options {
                if !self.is_pinned_move(&mv) {
                    allowed_moves.push(mv.clone());
                }
            }
        }
        if allowed_moves.len() == 0 {
            return KingState::Mate;
        } else {
            return KingState::Check { allowed_moves };
        }
    }

    pub fn get_movement_options(&mut self, pos: Position) -> Result<Vec<GameMove>> {
        let mut mv = self.board.get_movement_options(pos)?;
        let p = self.board.pieces.get(&pos).expect("no piece ???");
        if p.piece_type == PieceType::Pawn {
            mv.extend(self.get_en_passant_moves(&pos, &p));
        }

        mv.retain(|x| !self.is_pinned_move(x));
        Ok(mv)
    }

    // Get en_passant_moves for an existing pawn at the given position
    fn get_en_passant_moves(&self, pos: &Position, pawn: &Piece) -> Vec<GameMove> {
        // -- check if last move enables a potential en passant
        let Some(last) = self.moves.last() else {
            return Vec::new();
        };

        if last.piece.piece_type != PieceType::Pawn {
            return Vec::new();
        }

        let dx = last.destination.coordinates().1 as isize - last.origin.coordinates().1 as isize;
        if dx.abs() < 2 {
            return Vec::new();
        }

        // -- calculate the destination of en-passant
        let x = (last.origin.pos().1 as isize + dx.signum()) as usize;
        let en_passant_pos = Position::new(last.origin.pos().0, x).expect("Invalid position ???");

        // -- Check if given pawn can capture en passant
        let capture_moves = pawn_capture_moves(pawn.side);
        let mut moves = Vec::new();

        for (dy, dx) in capture_moves {
            let (y, x) = pos.coordinates();
            let y = y.checked_add_signed(*dy);
            let x = x.checked_add_signed(*dx);

            let (Some(y), Some(x)) = (y, x) else {
                continue;
            };
            let Ok(destination) = Position::new(y, x) else {
                continue;
            };

            if destination == en_passant_pos {
                let new_move = GameMove {
                    piece: pawn.clone(),
                    origin: *pos,
                    destination,
                    action: Action::Capture {
                        enemy: last.piece.clone(),
                        pos: last.destination,
                    },
                };

                moves.push(new_move);
            }
        }

        moves
    }

    fn next_turn(&mut self) {
        self.active_side = !self.active_side;

        if matches!(self.check_king(), KingState::Mate) {
            self.state = GameState::GameOver(GameResult {
                winner: Some(!self.active_side),
                outcome: OutCome::CheckMate,
            })
        }
    }

    /// Sets who moves first. Only meaningful before any move has been played,
    /// i.e. when a game starts from a position that was set up by hand.
    pub fn with_active_side(&mut self, side: Side) {
        self.active_side = side;
    }

    // Make a move using human coordinates
    pub fn make_human_move(
        &mut self,
        origin: HumanNotation,
        destination: HumanNotation,
    ) -> Result<()> {
        let origin = Position::from_human(origin)?;
        let destination = Position::from_human(destination)?;
        self.make_move(origin, destination)
    }

    fn validate_move(&mut self, origin: Position, destination: Position) -> Result<GameMove> {
        let piece = self
            .board
            .pieces
            .get(&origin)
            .ok_or(MoveError::NoPieceAtPosition(origin))?;

        if piece.side != self.active_side {
            return Err(UserError::WrongPlayer);
        }

        let options = self.get_movement_options(origin)?;

        let option = options
            .iter()
            .find(|option| option.destination == destination)
            .cloned()
            .ok_or(MoveError::IllegalMove)?;

        Ok(option)
    }

    /// Make a move on the board. Move must be valid, otherwise an error will be returned
    pub fn make_move(&mut self, origin: Position, destination: Position) -> Result<()> {
        if !matches!(self.state, GameState::Normal) {
            return Err(UserError::WrongGameState(self.state));
        }
        let game_move = self.validate_move(origin, destination)?;

        // -- Normal move logic
        self.board.execute(&game_move);
        self.moves.push(game_move.clone());

        // -- Promotion logic
        if game_move.piece.piece_type == PieceType::Pawn {
            let promotion_fields = match game_move.piece.side {
                Side::Black => &BLACK_PAWNS_PROMOTION_POSITIONS,
                Side::White => &WHITE_PAWNS_PROMOTION_POSITIONS,
            };

            if promotion_fields.contains(&destination) {
                // The mover stays on turn until they pick a piece; `promote`
                // ends the turn for them.
                self.state = GameState::Promotion;
                return Ok(());
            }
        }

        self.next_turn();
        Ok(())
    }

    // Undo the last game move
    pub fn undo(&mut self) -> Result<()> {
        let Some(mv) = self.moves.pop() else {
            return Err(UserError::CannotUndo);
        };

        self.board.undo(&mv);

        // Whoever played the undone move is on turn again. For a promotion that
        // is the pawn's side, since `promote` records the pawn as the moved piece.
        self.active_side = mv.piece.side;
        self.state = match mv.action {
            Action::Promote { .. } => GameState::Promotion,
            Action::Move | Action::Capture { .. } => GameState::Normal,
        };

        Ok(())
    }

    pub fn promote(&mut self, piece_type: PieceType) -> Result<()> {
        if matches!(piece_type, PieceType::Pawn | PieceType::King) {
            return Err(UserError::WrongPromotionType(piece_type));
        }

        if !matches!(self.state, GameState::Promotion) {
            return Err(UserError::WrongGameState(self.state));
        }

        let destination = self
            .moves
            .last()
            .expect("promotion without history ???")
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

        self.state = GameState::Normal;
        self.next_turn();
        Ok(())
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
    use crate::display::save_board_to_html_file;
    use anyhow::Result;
    use std::{collections::HashSet, path::PathBuf};

    fn get_html_repr_path(snapshot_name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src");
        path.push("snapshots");
        path.push(format!("{snapshot_name}.html"));
        path
    }

    fn snap_board(board: &Board, markers: &HashSet<Position>, snapshot_name: &str) {
        let path = get_html_repr_path(snapshot_name);
        save_board_to_html_file(board, markers, path).expect("html could not be generated");
    }

    fn mark_and_snap(game: &mut Game, positions: &[Position], snapshot_name: &str) {
        let mut options = Vec::new();

        for p in positions {
            options.extend(
                game.get_movement_options(p.clone())
                    .expect("error on movement options"),
            );
        }

        let markers: HashSet<Position> = options.iter().map(|x| x.destination).collect();

        snap_board(&game.board, &markers, snapshot_name);
        insta::assert_debug_snapshot!(snapshot_name, options);
    }

    #[test]
    fn test_promote_pawn() {
        // -- Create game with pawn
        let mut board = Board::default();

        let origin: Position = Position::from_human(('K', 5)).unwrap();
        let destination: Position = Position::from_human(('K', 6)).unwrap();

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

        board.pieces.insert(
            origin,
            Piece {
                piece_type: PieceType::Pawn,
                side: Side::White,
            },
        );
        let mut game = Game::from_board(board).expect("invalid board ??");

        // -- Move pawn
        game.make_move(origin, destination)
            .expect("Error while playing move");

        // -- The mover stays on turn until the piece is picked
        assert!(matches!(game.state, GameState::Promotion));
        assert_eq!(game.active_side(), Side::White);

        // -- Promote
        game.promote(PieceType::Queen)
            .expect("Error while promoting");

        // -- Check normal game state again
        assert!(matches!(game.state, GameState::Normal));
        assert_eq!(game.active_side(), Side::Black);

        // -- Check Queen exists, and belongs to the player that promoted
        let promoted = game.board().pieces.get(&destination).expect("no piece ??");
        assert_eq!(promoted.piece_type, PieceType::Queen);
        assert_eq!(promoted.side, Side::White);
        println!("{game:#?}");
    }

    #[test]
    fn test_serde_json() -> Result<()> {
        let game = Game::new();
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

        board.pieces.insert(
            Position::from_human(('I', 2)).unwrap(),
            Piece {
                piece_type: PieceType::King,
                side: Side::Black,
            },
        );

        let mut game = Game::from_board(board)?;

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
        let mut game = Game::from_board(board)?;
        let mut game_states = Vec::new();
        game_states.push(serde_json::to_string(&game)?);

        game.make_move(origin, Position::from_human(('G', 9)).unwrap())?;
        assert!(
            matches!(game.state, GameState::Normal),
            "wrong game state 1"
        );
        game_states.push(serde_json::to_string(&game)?);

        // Black King moves
        game.make_human_move(('I', 2), ('I', 3))?;
        assert!(
            matches!(game.state, GameState::Normal),
            "wrong game state 2"
        );
        let last_state = serde_json::to_string(&game)?;

        // White Pawn moves onto the promotion rank
        game.make_human_move(('G', 9), ('G', 10))?;
        assert!(
            matches!(game.state, GameState::Promotion),
            "wrong game state 3"
        );

        println!("undoing move {:?}", game.moves.last());
        game.undo()?;
        let new_state = serde_json::to_string(&game)?;
        assert_eq!(new_state, last_state, "game state not identical");

        for game_state in game_states.into_iter().rev() {
            println!("undoing move {:?}", game.moves.last());
            game.undo()?;
            let new_state = serde_json::to_string(&game)?;
            assert_eq!(new_state, game_state, "game state not identical");
        }

        assert!(game.undo().is_err(), "nothing left to undo");
        Ok(())
    }

    #[test]
    fn test_check_mate() {
        use PieceType::*;
        use Side::*;
        let human = Position::from_human;

        let mut board = Board::default();

        board
            .pieces
            .insert(human(('F', 5)).unwrap(), Piece::new(King, White));

        board
            .pieces
            .insert(human(('A', 11)).unwrap(), Piece::new(King, Black));

        let mut game = Game::from_board(board).expect("Invalid board ???");

        assert_eq!(game.check_king(), KingState::Nothing);

        game.board
            .pieces
            .insert(human(('F', 1)).unwrap(), Piece::new(Rook, Black));

        assert!(matches!(game.check_king(), KingState::Check { .. }));

        game.board
            .pieces
            .insert(human(('E', 2)).unwrap(), Piece::new(Rook, Black));

        assert!(matches!(game.check_king(), KingState::Check { .. }));

        let rook3_pos = human(('G', 1)).unwrap();
        game.board.pieces.insert(rook3_pos, Piece::new(Rook, Black));

        game.board
            .pieces
            .insert(human(('D', 3)).unwrap(), Piece::new(Rook, Black));

        assert!(matches!(game.check_king(), KingState::Check { .. }));

        game.board
            .pieces
            .insert(human(('I', 1)).unwrap(), Piece::new(Rook, Black));

        game.active_side = Side::Black;
        game.make_human_move(('I', 1), ('H', 1))
            .expect("move error ?");

        assert!(
            matches!(game.state, GameState::GameOver { .. }),
            "should be game over"
        );
        assert_eq!(
            game.game_result().expect("no game result !").winner,
            Some(Side::Black)
        );

        // A finished game rejects further moves
        assert!(matches!(
            game.make_human_move(('H', 1), ('I', 1)),
            Err(UserError::WrongGameState(_))
        ));

        game.board
            .pieces
            .insert(human(('K', 3)).unwrap(), Piece::new(Queen, White));

        assert!(matches!(game.check_king(), KingState::Check { .. }));
    }

    #[test]
    fn test_en_passant() -> Result<(), UserError> {
        use PieceType::*;
        use Side::*;
        let human = Position::from_human;

        let mut board = Board::default();

        board
            .pieces
            .insert(human(('A', 11)).unwrap(), Piece::new(King, White));

        board
            .pieces
            .insert(human(('K', 6)).unwrap(), Piece::new(King, Black));

        let mut game = Game::from_board(board).expect("Invalid Board ??");
        let white_pawn_origin = human(('J', 1)).unwrap();
        let white_pawn_destination = human(('J', 3)).unwrap();
        let black_pawn_origin = human(('I', 3)).unwrap();

        game.board
            .pieces
            .insert(white_pawn_origin, Piece::new(Pawn, White));

        game.board
            .pieces
            .insert(black_pawn_origin, Piece::new(Pawn, Black));

        game.make_move(white_pawn_origin, white_pawn_destination)?;
        mark_and_snap(&mut game, &[black_pawn_origin], "test_en_passant");

        game.make_move(black_pawn_origin, human(('j', 2)).unwrap())?;
        mark_and_snap(&mut game, &[], "test_en_passant_2");

        game.undo()?;
        mark_and_snap(&mut game, &[], "test_en_passant_3");
        Ok(())
    }

    #[test]
    fn test_disallow_pinned_moves() -> Result<(), UserError> {
        use PieceType::*;
        use Side::*;
        let human = Position::from_human;

        let mut board = Board::default();

        board
            .pieces
            .insert(human(('I', 6)).unwrap(), Piece::new(King, White));

        board
            .pieces
            .insert(human(('C', 5)).unwrap(), Piece::new(King, Black));

        let mut game = Game::from_board(board).expect("Invalid board ???");

        let white_rook_pos = human(('I', 4)).unwrap();

        game.board
            .pieces
            .insert(white_rook_pos, Piece::new(Rook, White));

        game.board
            .pieces
            .insert(human(('I', 1)).unwrap(), Piece::new(Rook, Black));

        mark_and_snap(&mut game, &[white_rook_pos], "test_disallow_pinned_moves");

        Ok(())
    }
}
