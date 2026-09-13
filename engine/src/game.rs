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
        // -- Check three fold repetition
        if self.check_three_fold_repetition() {
            self.state = GameState::GameOver(GameResult {
                winner: None,
                outcome: OutCome::ThreefoldRepetition,
            });
            return;
        }

        if self.check_is_insufficient_material() {
            self.state = GameState::GameOver(GameResult {
                winner: None,
                outcome: OutCome::InsufficientMaterial,
            });
            return;
        }

        // -- Check for Checkmate, Stalemate and Normal
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

    // A position seen three times is a draw. Hashes stand in for positions, so
    // this assumes no Zobrist collision (as every hash-based repetition check does).
    fn check_three_fold_repetition(&self) -> bool {
        // The current position (`self.position_hash`, == `plays.last()`) is the
        // first occurrence; skip(1) walks the history before it.
        let mut count = 1;
        for Play {
            game_move: GameMove { piece, action, .. },
            hash,
        } in self.plays.iter().rev().skip(1)
        {
            if *hash == self.position_hash {
                count += 1;
                if count == 3 {
                    return true;
                }
            }

            // An irreversible move (pawn move, capture, promotion): the position
            // it produced was just counted, but nothing before it can match the
            // current one, so the scan stops here.
            if piece.piece_type == PieceType::Pawn
                || matches!(action, Action::Capture { .. } | Action::Promote { .. })
            {
                return false;
            }
        }

        // No irreversible move separated us from the start, so the initial
        // position is the remaining candidate occurrence.
        count += (self.position_hash == self.position_hash_initial) as u32;
        count >= 3
    }

    // A game is drawn due to insufficient material if it is mathematically impossible to construct a legal checkmate position
    // King + 1 Bishop vs. King: A single bishop can only traverse hexes of its own color. Because a hex board uses 3 colors (instead of 2), a lone bishop is completely powerless to trap a king.
    // King + 1 Knight vs. King: Just like standard chess, a single knight cannot trap and mate a lone king by itself.
    fn check_is_insufficient_material(&self) -> bool {
        use PieceType::*;
        let mut knight_or_bishop_count = [0u8; 2];

        for Piece { piece_type, side } in self.board.pieces.values() {
            let i = *side as usize;
            match piece_type {
                Queen | Pawn | Rook => return false,
                King => (),
                Bishop | Knight => {
                    knight_or_bishop_count[i] += 1;
                    if knight_or_bishop_count[i] > 1 {
                        return false;
                    }
                }
            }
        }

        true
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
