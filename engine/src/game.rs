use std::collections::{HashMap, HashSet};

use crate::piece::PieceType::Pawn;
use crate::piece::pawn_starting_positions;
use crate::{Side, board, zobrist::PositionHash};
use crate::{
    board::{Action, Board, GameMove, MoveError},
    coordinates::{HumanNotation, Position},
    piece::{Piece, PieceType, get_startup_pieces_black, get_startup_pieces_white},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UserError {
    #[error(transparent)]
    MoveError(#[from] board::MoveError),

    #[error("This notation is invalid: {0:?}")]
    InvalidHumanNotation(HumanNotation),

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
    pub winner: Option<Side>,
    pub outcome: OutCome,
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Play {
    pub game_move: GameMove,
    pub hash: PositionHash, // hash of the position this play produced
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
    plays: Vec<Play>,
    position_hash: PositionHash,
    position_hash_initial: PositionHash,
    en_passant_field_initial: Option<Position>,
    state: GameState,
}

#[derive(PartialEq, Eq, Debug)]
pub enum KingState {
    Ok,
    Check,
    CheckMate,
    StaleMate,
}

pub type Result<T> = std::result::Result<T, UserError>;

impl Game {
    pub fn new() -> Self {
        let mut board = Board::default();
        board.pieces.extend(get_startup_pieces_white());
        board.pieces.extend(get_startup_pieces_black());

        Self::from_board(board, Side::White, None).expect("Invalid board ???")
    }

    pub fn from_board(
        board: Board,
        active_player: Side,
        en_passant_field: Option<Position>,
    ) -> Result<Self> {
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

        let position_hash = PositionHash::from_board(&board, active_player, en_passant_field);

        let mut game = Game {
            board,
            position_hash,
            position_hash_initial: position_hash,
            en_passant_field_initial: en_passant_field,
            active_side: active_player,
            plays: Vec::new(),
            state: GameState::Normal,
        };

        if let Some(field) = en_passant_field {
            game.validate_en_passant(field)?;
        };

        // A hand-set-up position can already be finished before anyone moved.
        game.update_state();

        Ok(game)
    }

    // Validate a given en passant field:
    // (1) en passant field must be empty
    // (2) a correct pawn must exist to have created the en passant
    // (3) en passant must be playable by active player
    fn validate_en_passant(&mut self, field: Position) -> Result<()> {
        let err_en_passant = |x: &str| {
            let msg = format!("Invalid en passant: {x}");
            UserError::InvalidBoard(msg)
        };

        // Check (1) - en passant field must be empty
        if self.board.pieces.contains_key(&field) {
            return Err(err_en_passant("figure on field"));
        };

        // Check (2) - a correct pawn must exist to have created the en passant
        let err_msg2 = "No pawn that could have created this en passant";
        let enemy = !self.active_side;
        let (_, dx) = enemy.move_direction();
        let expected_pawn_pos = field.add(0, dx).ok_or(err_en_passant(err_msg2))?;

        // 2a - check pawn at destination
        let pawn = self
            .board
            .pieces
            .get(&expected_pawn_pos)
            .ok_or(err_en_passant(err_msg2))?;

        if *pawn != Piece::new(PieceType::Pawn, enemy) {
            return Err(err_en_passant(err_msg2));
        }

        // 2b - check origin of pawn
        let pawn_origin = field.add(0, -dx).ok_or(err_en_passant(err_msg2))?;
        let starting_positions = pawn_starting_positions(enemy);
        if !starting_positions.contains(&pawn_origin) {
            return Err(err_en_passant(err_msg2));
        }

        if self.board.pieces.contains_key(&pawn_origin) {
            return Err(err_en_passant(err_msg2));
        };

        // (3) - Check en passant must be playable by active player
        if self.get_en_passant_moves(self.active_side, None).is_empty() {
            return Err(err_en_passant("En passant cannot be played"));
        };

        Ok(())
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
    pub fn plays(&self) -> &[Play] {
        &self.plays
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

    pub fn move_leaves_king_in_check(&mut self, mv: &GameMove) -> bool {
        self.board.execute(mv);
        let check = self.king_in_check(mv.piece.side);
        self.board.undo(mv);
        check
    }

    fn player_has_movement_options(&mut self) -> bool {
        let my_pieces = self.pieces_by_side(self.active_side);

        for (origin, _) in my_pieces {
            let mv_options = self
                .get_movement_options(origin)
                .expect("A piece must be here");

            if !mv_options.is_empty() {
                return true;
            }
        }
        false
    }

    // Get en_passant_moves and filter out pinned pieces
    fn get_en_passant_moves(
        &mut self,
        active_player: Side,
        last_move: Option<GameMove>,
    ) -> Vec<GameMove> {
        let en_passant_field = match last_move {
            Some(move_) => self.board.get_en_passant_field(&move_),
            None => self.en_passant_field_initial,
        };

        let Some(field) = en_passant_field else {
            return Vec::new();
        };

        self.board
            .get_en_passant_moves(active_player, field)
            .expect("En passant field not valid ???")
            .into_iter()
            .filter(|x| !self.move_leaves_king_in_check(x))
            .collect()
    }

    // Get valid movement options for a piece at a given position.
    pub fn get_movement_options(&mut self, pos: Position) -> Result<Vec<GameMove>> {
        let mut mv = self.board.get_movement_options(pos)?;
        mv.retain(|x| !self.move_leaves_king_in_check(x));

        if matches!(
            self.board.pieces.get(&pos),
            Some(&Piece {
                piece_type: Pawn,
                ..
            })
        ) {
            let en = self.get_en_passant_moves(
                self.active_side,
                self.plays().last().map(|x| x.game_move.clone()),
            );

            if let Some(move_) = en.iter().find(|x| x.origin == pos) {
                mv.push(move_.clone());
            }
        }

        Ok(mv)
    }

    pub fn check_king(&mut self) -> KingState {
        let has_options = self.player_has_movement_options();
        let is_check = self.king_in_check(self.active_side);

        match (is_check, has_options) {
            (false, false) => KingState::StaleMate,
            (true, false) => KingState::CheckMate,
            (false, true) => KingState::Ok,
            (true, true) => KingState::Check,
        }
    }

    // Derives the game state from the position: the side to move having no legal
    // move ends the game. Must be called whenever the position or the side to move
    // changes, and only while the game is not waiting for a promotion.
    fn update_state(&mut self) {
        self.state = match self.check_king() {
            KingState::CheckMate => GameState::GameOver(GameResult {
                winner: Some(!self.active_side),
                outcome: OutCome::CheckMate,
            }),
            KingState::StaleMate => GameState::GameOver(GameResult {
                winner: None,
                outcome: OutCome::StaleMate,
            }),
            KingState::Ok | KingState::Check => GameState::Normal,
        };
    }

    // This function is executed after a new move is played.
    fn next_turn(&mut self) {
        self.active_side = !self.active_side;
        self.update_state();
    }

    // Make a move using human coordinates
    pub fn make_human_move(
        &mut self,
        origin: HumanNotation,
        destination: HumanNotation,
    ) -> Result<()> {
        let origin = Position::from_human(origin).ok_or(UserError::InvalidHumanNotation(origin))?;
        let destination = Position::from_human(destination)
            .ok_or(UserError::InvalidHumanNotation(destination))?;

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

    // Returns the validated en passant field (not pinned)
    fn get_valid_en_passant_field(
        &mut self,
        active_player: Side,
        last_move: Option<GameMove>,
    ) -> Option<Position> {
        Some(
            self.get_en_passant_moves(active_player, last_move)
                .first()?
                .destination,
        )
    }

    /// Make a move on the board. Move must be valid, otherwise an error will be returned
    pub fn make_move(&mut self, origin: Position, destination: Position) -> Result<()> {
        if !matches!(self.state, GameState::Normal) {
            return Err(UserError::WrongGameState(self.state));
        }
        let game_move = self.validate_move(origin, destination)?;
        let does_promote = game_move.does_promote();

        let last_move = self.plays().last().map(|x| x.game_move.clone());
        let old_en_passant = self.get_valid_en_passant_field(self.active_side, last_move);

        self.board.execute(&game_move);

        let new_en_passant =
            self.get_valid_en_passant_field(!self.active_side, Some(game_move.clone()));

        self.position_hash.update_move(&game_move);
        if !does_promote {
            self.position_hash.update_active_player();
        }

        self.position_hash
            .update_en_passant(old_en_passant, new_en_passant);

        self.plays.push(Play {
            game_move: game_move.clone(),
            hash: self.position_hash,
        });

        if does_promote {
            self.state = GameState::Promotion;
        } else {
            self.next_turn();
        }

        Ok(())
    }

    // Undo the last game move
    pub fn undo(&mut self) -> Result<()> {
        let Some(play) = self.plays.pop() else {
            return Err(UserError::CannotUndo);
        };

        self.board.undo(&play.game_move);

        // The GameMove stores the hash after the move, so we have to go back once more
        self.position_hash = self
            .plays
            .last()
            .map(|x| x.hash)
            .unwrap_or(self.position_hash_initial);

        // Whoever played the undone move is on turn again. For a promotion that
        // is the pawn's side, since `promote` records the pawn as the moved piece.
        self.active_side = play.game_move.piece.side;
        self.state = match play.game_move.action {
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
            .plays
            .last()
            .expect("promotion without history ???")
            .game_move
            .destination;

        let new_piece = Piece {
            piece_type,
            side: self.active_side,
        };

        let old_piece = self.board.pieces.insert(destination, new_piece.clone());
        let game_move = GameMove {
            piece: old_piece.expect("no piece to promote ??"),
            origin: destination,
            destination,
            action: Action::Promote { to: new_piece },
        };

        self.position_hash.update_move(&game_move);
        self.position_hash.update_active_player();

        let hash = self.position_hash;
        self.plays.push(Play { game_move, hash });

        self.state = GameState::Normal;
        self.next_turn();
        Ok(())
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
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

    fn pos(square: HumanNotation) -> Position {
        Position::from_human(square).expect("invalid square in test")
    }

    fn board_with(pieces: &[(HumanNotation, PieceType, Side)]) -> Board {
        let mut board = Board::default();
        for &(square, piece_type, side) in pieces {
            board
                .pieces
                .insert(pos(square), Piece::new(piece_type, side));
        }
        board
    }

    /// The incrementally updated hash must equal one built from the position
    /// alone. `en_passant` is the field a capture is available on right now.
    fn assert_hash(game: &Game, en_passant: Option<Position>, when: &str) {
        assert_eq!(
            game.position_hash,
            PositionHash::from_board(&game.board, game.active_side, en_passant),
            "hash drifted from the position {when}"
        );
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
        let mut game = Game::from_board(board, Side::White, None).expect("invalid board ??");

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

        let mut game = Game::from_board(board, Side::White, None)?;

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
        let mut game = Game::from_board(board, Side::White, None)?;
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

        println!("undoing move {:?}", game.plays.last());
        game.undo()?;
        let new_state = serde_json::to_string(&game)?;
        assert_eq!(new_state, last_state, "game state not identical");

        for game_state in game_states.into_iter().rev() {
            println!("undoing move {:?}", game.plays.last());
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

        let mut game = Game::from_board(board, Side::White, None).expect("Invalid board ???");

        assert_eq!(game.check_king(), KingState::Ok);

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

    // A position that is already stalemate when the game is set up must be
    // reported as over, not as a normal position whose every move is illegal.
    #[test]
    fn test_stale_mate_on_setup() {
        use PieceType::*;
        use Side::*;
        let human = Position::from_human;

        let mut board = Board::default();

        board
            .pieces
            .insert(human(('A', 6)).unwrap(), Piece::new(King, Black));

        board
            .pieces
            .insert(human(('C', 7)).unwrap(), Piece::new(King, White));

        board
            .pieces
            .insert(human(('D', 4)).unwrap(), Piece::new(Queen, White));

        // The same position is a live game for White and stalemate for Black, so
        // who is on turn is what decides whether it is over.
        let mut white_to_move =
            Game::from_board(board.clone(), White, None).expect("Invalid board ???");

        assert_eq!(white_to_move.check_king(), KingState::Ok);
        assert!(matches!(white_to_move.state, GameState::Normal));

        // Black's king is unattacked but every one of its moves runs into the queen
        let mut black_to_move = Game::from_board(board, Black, None).expect("Invalid board ???");

        assert_eq!(black_to_move.check_king(), KingState::StaleMate);
        assert!(
            matches!(black_to_move.state, GameState::GameOver { .. }),
            "should be game over"
        );

        let result = black_to_move.game_result().expect("no game result !");
        assert_eq!(result.winner, None);
        assert!(matches!(result.outcome, OutCome::StaleMate));

        assert!(matches!(
            black_to_move.make_human_move(('A', 6), ('A', 7)),
            Err(UserError::WrongGameState(_))
        ));
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

        let mut game = Game::from_board(board, Side::White, None).expect("Invalid Board ??");
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

        let mut game = Game::from_board(board, Side::White, None).expect("Invalid board ???");

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

    // The point of the incremental hash: it must never drift from the position
    // it claims to describe, through any kind of move or undo.
    #[test]
    fn hash_invariant_across_a_scripted_game() {
        use PieceType::*;
        use Side::*;

        let board = board_with(&[
            (('A', 11), King, White),
            (('A', 6), King, Black),
            (('J', 1), Pawn, White),  // double-pushes
            (('I', 3), Pawn, Black),  // answers en passant
            (('J', 5), Rook, White),  // recaptures
            (('C', 10), Pawn, Black), // a quiet move
            (('H', 8), Pawn, White),  // promotes
        ]);

        let mut game = Game::from_board(board, White, None).expect("valid board");
        assert_hash(&game, None, "at setup");

        // each move with the en-passant field it leaves behind
        let script = [
            ((('J', 1), ('J', 3)), Some(('J', 2))),
            ((('I', 3), ('J', 2)), None),
            ((('J', 5), ('J', 2)), None),
            ((('C', 10), ('C', 9)), None),
            ((('H', 8), ('H', 9)), None), // triggers promotion
        ];

        for ((origin, destination), en_passant) in script {
            game.make_move(pos(origin), pos(destination))
                .unwrap_or_else(|err| panic!("{origin:?} -> {destination:?} rejected: {err}"));
            assert_hash(
                &game,
                en_passant.map(pos),
                &format!("after {origin:?} -> {destination:?}"),
            );
        }

        game.promote(Queen).expect("promotion");
        assert_hash(&game, None, "after promoting");

        // and back out again, one ply at a time. Undoing the en-passant capture
        // puts its field back on the board.
        let rewind = [None, None, None, None, Some(('J', 2)), None];
        for (i, en_passant) in rewind.into_iter().enumerate() {
            game.undo().expect("undo");
            assert_hash(&game, en_passant.map(pos), &format!("after undo {}", i + 1));
        }

        assert!(game.plays().is_empty(), "back at the starting position");
    }

    // Nothing checked a freshly built game before, which is how an inverted
    // starting hash once survived a green suite.
    #[test]
    fn hash_invariant_for_fresh_games() {
        use PieceType::*;
        use Side::*;

        assert_hash(&Game::new(), None, "for Game::new()");

        let kings = [(('A', 11), King, White), (('K', 6), King, Black)];
        let game = Game::from_board(board_with(&kings), Black, None).expect("valid board");
        assert_hash(&game, None, "for a board with Black to move");

        // A hand-set-up en passant belongs in the starting hash, and has to come
        // back out on the move that lets it lapse.
        let board = board_with(&[
            (('A', 11), King, White),
            (('K', 6), King, Black),
            (('B', 9), Pawn, Black),
            (('C', 9), Pawn, White),
        ]);
        let mut game = Game::from_board(board, White, Some(pos(('B', 10)))).expect("valid board");
        assert_hash(
            &game,
            Some(pos(('B', 10))),
            "for a board set up with an en passant",
        );

        game.make_move(pos(('A', 11)), pos(('A', 10)))
            .expect("king move");
        assert_hash(&game, None, "after the en passant lapsed");
    }

    // Playing an en passant used to panic: `make_move` re-read the initial field
    // after the capture had already removed the pawn behind it.
    #[test]
    fn en_passant_can_be_played_from_a_set_up_position() {
        use PieceType::*;
        use Side::*;

        let board = board_with(&[
            (('A', 11), King, White),
            (('K', 6), King, Black),
            (('B', 9), Pawn, Black), // just double-pushed from B11
            (('C', 9), Pawn, White),
        ]);
        let mut game = Game::from_board(board, White, Some(pos(('B', 10)))).expect("valid board");

        game.make_move(pos(('C', 9)), pos(('B', 10)))
            .expect("the en passant the position promises must be playable");

        assert!(
            !game.board.pieces.contains_key(&pos(('B', 9))),
            "the captured pawn is taken off its own square, not the destination"
        );
        assert_eq!(
            game.board.pieces.get(&pos(('B', 10))),
            Some(&Piece::new(Pawn, White))
        );
        assert_hash(&game, None, "after an en passant capture");
    }

    // Both sides, and both squares a capture can come from.
    #[test]
    fn a_playable_en_passant_field_is_accepted() {
        use PieceType::*;
        use Side::*;

        // Black B11->B9 leaves B10; a white pawn takes it from A10 or C9.
        for capturer in [('A', 10), ('C', 9)] {
            let board = board_with(&[
                (('A', 11), King, White),
                (('K', 6), King, Black),
                (('B', 9), Pawn, Black),
                (capturer, Pawn, White),
            ]);
            Game::from_board(board, White, Some(pos(('B', 10))))
                .unwrap_or_else(|err| panic!("white pawn on {capturer:?} may take: {err}"));
        }

        // White J1->J3 leaves J2; a black pawn takes it from I3 or K2.
        for capturer in [('I', 3), ('K', 2)] {
            let board = board_with(&[
                (('A', 11), King, White),
                (('K', 6), King, Black),
                (('J', 3), Pawn, White),
                (capturer, Pawn, Black),
            ]);
            Game::from_board(board, Black, Some(pos(('J', 2))))
                .unwrap_or_else(|err| panic!("black pawn on {capturer:?} may take: {err}"));
        }
    }

    // A field that no legal double push could have produced is a caller error,
    // not an invariant violation: it must come back as an error, never a panic.
    #[test]
    fn an_impossible_en_passant_field_is_rejected() {
        use PieceType::*;
        use Side::*;

        fn assert_rejected(
            pieces: &[(HumanNotation, PieceType, Side)],
            en_passant: HumanNotation,
            why: &str,
        ) {
            match Game::from_board(board_with(pieces), Side::White, Some(pos(en_passant))) {
                Err(UserError::InvalidBoard(_)) => {}
                other => panic!(
                    "{why}: expected InvalidBoard, got {:?}",
                    other.map(|_| "Ok")
                ),
            }
        }

        let kings = [(('A', 11), King, White), (('K', 6), King, Black)];
        // Baseline: Black B11->B9, ep field B10, the white pawn on C9 may take.
        let base = |extra: &[(HumanNotation, PieceType, Side)]| {
            let mut pieces = kings.to_vec();
            pieces.extend_from_slice(&[(('B', 9), Pawn, Black), (('C', 9), Pawn, White)]);
            pieces.extend_from_slice(extra);
            pieces
        };

        assert_rejected(
            &base(&[(('B', 10), Rook, Black)]),
            ('B', 10),
            "a piece is standing on the skipped field",
        );

        let mut no_pawn = kings.to_vec();
        no_pawn.push((('C', 9), Pawn, White));
        assert_rejected(&no_pawn, ('B', 10), "no pawn ever arrived behind the field");

        assert_rejected(
            &base(&[(('B', 11), Rook, Black)]),
            ('B', 10),
            "the square the pawn would have come from is occupied",
        );

        // B7 is a real black pawn, but B9 is not a black starting square, so no
        // double push could have skipped B8.
        let mut not_a_start = kings.to_vec();
        not_a_start.extend_from_slice(&[(('B', 7), Pawn, Black), (('C', 7), Pawn, White)]);
        assert_rejected(&not_a_start, ('B', 8), "no double push starts from B9");

        let mut nobody_can_take = kings.to_vec();
        nobody_can_take.push((('B', 9), Pawn, Black));
        assert_rejected(
            &nobody_can_take,
            ('B', 10),
            "no pawn of White's can capture there",
        );
    }

    // An en-passant capture belongs to the pawn that can make it, not to every
    // piece of the side to move.
    #[test]
    fn en_passant_is_only_offered_to_the_pawn_that_can_take() {
        use PieceType::*;
        use Side::*;

        let board = board_with(&[
            (('A', 11), King, White),
            (('K', 6), King, Black),
            (('B', 9), Pawn, Black),
            (('C', 9), Pawn, White), // may take on B10
            (('G', 5), Pawn, White), // a pawn that may not — the case a type check alone misses
            (('G', 1), Rook, White), // may not
        ]);
        let mut game = Game::from_board(board, White, Some(pos(('B', 10)))).expect("valid board");
        let en_passant = pos(('B', 10));

        let mine: Vec<Position> = game.pieces_by_side(White).into_keys().collect();
        for square in mine {
            let options = game.get_movement_options(square).expect("a piece is there");
            assert!(
                options.iter().all(|option| option.origin == square),
                "{square} was offered a move belonging to another piece"
            );
        }

        let pawn = game.get_movement_options(pos(('C', 9))).expect("the pawn");
        assert!(
            pawn.iter().any(|option| option.destination == en_passant),
            "the pawn that can take must still be offered the capture"
        );

        assert!(
            matches!(
                game.make_move(pos(('G', 1)), en_passant),
                Err(UserError::MoveError(MoveError::IllegalMove))
            ),
            "the rook must not be able to play the pawn's capture"
        );
    }

    // Equal positions must hash equal however they were reached, or a threefold
    // repetition would go unnoticed.
    #[test]
    fn transpositions_and_undo_agree() {
        use PieceType::*;
        use Side::*;

        let pieces = [
            (('A', 11), King, White),
            (('K', 6), King, Black),
            (('G', 1), Rook, White),
            (('G', 10), Rook, Black),
        ];

        let play = |script: [(HumanNotation, HumanNotation); 4]| {
            let mut game = Game::from_board(board_with(&pieces), White, None).expect("valid board");
            for (origin, destination) in script {
                game.make_move(pos(origin), pos(destination))
                    .unwrap_or_else(|err| panic!("{origin:?} -> {destination:?} rejected: {err}"));
            }
            game
        };

        let kings_first = play([
            (('A', 11), ('B', 11)),
            (('K', 6), ('K', 5)),
            (('G', 1), ('G', 2)),
            (('G', 10), ('G', 9)),
        ]);
        let rooks_first = play([
            (('G', 1), ('G', 2)),
            (('G', 10), ('G', 9)),
            (('A', 11), ('B', 11)),
            (('K', 6), ('K', 5)),
        ]);

        assert_eq!(
            kings_first.position_hash, rooks_first.position_hash,
            "the same position reached two ways must hash the same"
        );

        let mut game = kings_first;
        let before = game.position_hash;
        game.make_move(pos(('G', 2)), pos(('G', 3)))
            .expect("rook move");
        assert_ne!(game.position_hash, before, "a move must change the hash");

        game.undo().expect("undo");
        assert_eq!(
            game.position_hash, before,
            "undo must restore the exact hash"
        );
    }
}
