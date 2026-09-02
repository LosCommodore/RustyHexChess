//! The engine's outward-facing API: one mutable handle, plain data in and out.
//!
//! This layer translates. [`GameApi`] owns a [`Game`], applies commands to it,
//! and hands back snapshots — the caller never deals with [`Position`],
//! [`Side`] or [`UserError`].
//!
//! The wire types here are deliberately *not* the engine's own types. They use
//! their own spelling (lowercase names, `"f5"` squares) so the engine stays free
//! to change its internals without breaking anything that already speaks this
//! protocol. Nothing in this module depends on WASM; see `wasm.rs` for the shim
//! that exposes it to JavaScript.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    Game, GameState as EngineState, Side, UserError,
    board::{Action, Board, GameMove, MoveError},
    coordinates::{CoordinateError, Position},
    piece::{Piece, PieceType},
};

// --- Wire types ---------------------------------------------------------

#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Color {
    White,
    Black,
}

#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

/// Which command the game will accept next.
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// `play` — a normal turn.
    Normal,
    /// `promote` — a pawn reached the far end and the game is blocked until it is replaced.
    Promotion,
    /// Checkmate. Only `undo` and `reset` do anything.
    Finished,
}

impl Phase {
    /// How this phase reads in the [`ApiError::WrongPhase`] message.
    fn label(self) -> &'static str {
        match self {
            Phase::Normal => "a move",
            Phase::Promotion => "a promotion",
            Phase::Finished => "nothing, the game is over",
        }
    }
}

impl From<EngineState> for Phase {
    fn from(state: EngineState) -> Self {
        match state {
            EngineState::Normal => Phase::Normal,
            EngineState::Promotion => Phase::Promotion,
            EngineState::GameOver { .. } => Phase::Finished,
        }
    }
}

/// What a move does, beyond vacating its origin.
///
/// `Capture.square` is the square the taken piece stood on, which for en passant
/// is *not* the destination — the UI needs both to draw the capture correctly.
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MoveAction {
    Move,
    Capture {
        piece: Kind,
        color: Color,
        square: String,
    },
    Promote {
        to: Kind,
    },
}

#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPiece {
    pub square: String,
    pub kind: Kind,
    pub color: Color,
}

/// One move that was played, as the move list wants to render it.
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayedMove {
    pub from: String,
    pub to: String,
    pub kind: Kind,
    pub color: Color,
    pub action: MoveAction,
    /// Short algebraic-ish label, e.g. `"Rf5xg9"` or `"j1-j3"`.
    pub notation: String,
}

/// A destination the selected piece may legally reach.
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegalMove {
    pub to: String,
    pub action: MoveAction,
}

#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Captured {
    /// Pieces taken *by* white.
    pub white: Vec<Kind>,
    /// Pieces taken *by* black.
    pub black: Vec<Kind>,
}

/// A complete snapshot of the game. Every command returns one of these, so the
/// caller can render from it without tracking state of its own.
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub phase: Phase,
    /// Whose turn it is. During `Promotion` this is still the promoting player.
    pub active: Color,
    /// True when the active player's king is attacked.
    pub check: bool,
    /// Set only when `phase` is `Finished`.
    pub winner: Option<Color>,
    /// Full move number as shown in notation, starting at 1.
    pub move_number: u32,
    pub pieces: Vec<PlacedPiece>,
    pub captured: Captured,
    pub history: Vec<PlayedMove>,
    pub can_undo: bool,
}

// --- Errors -------------------------------------------------------------

/// The stable half of an [`ApiError`] — what a caller branches on, as opposed
/// to the message, which is for humans and may be reworded.
///
/// The TypeScript union is generated from this, so adding a variant cannot
/// leave `engine.d.ts` behind.
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ErrorCode {
    InvalidSquare,
    NoPieceAtSquare,
    InvalidPieceType,
    InvalidColor,
    WrongPlayer,
    IllegalMove,
    CannotUndo,
    WrongPhase,
    MissingKing,
    DuplicateSquare,
    Engine,
}

/// Everything that can go wrong, as a value the caller can branch on. `code` is
/// the stable part; `message` is for humans and may be reworded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApiError {
    #[error("'{square}' is not a square on this board")]
    InvalidSquare { square: String },

    #[error("no piece on {square}")]
    NoPieceAtSquare { square: String },

    #[error("'{name}' is not a piece type")]
    InvalidPieceType { name: String },

    #[error("'{name}' is not a colour")]
    InvalidColor { name: String },

    #[error("that piece belongs to the other player")]
    WrongPlayer,

    #[error("that piece cannot reach that square")]
    IllegalMove,

    #[error("there is no move to undo")]
    CannotUndo,

    /// `expected` is what the game is waiting for, `actual` what was sent.
    #[error("the game is waiting for {expected}, not {actual}")]
    WrongPhase {
        expected: &'static str,
        actual: &'static str,
    },

    #[error("the {color} king is missing from the board")]
    MissingKing { color: Color },

    #[error("two pieces were placed on {square}")]
    DuplicateSquare { square: String },

    #[error("{message}")]
    Engine { message: String },
}

impl ApiError {
    /// Stable identifier for callers that need to react to a specific failure.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidSquare { .. } => ErrorCode::InvalidSquare,
            Self::NoPieceAtSquare { .. } => ErrorCode::NoPieceAtSquare,
            Self::InvalidPieceType { .. } => ErrorCode::InvalidPieceType,
            Self::InvalidColor { .. } => ErrorCode::InvalidColor,
            Self::WrongPlayer => ErrorCode::WrongPlayer,
            Self::IllegalMove => ErrorCode::IllegalMove,
            Self::CannotUndo => ErrorCode::CannotUndo,
            Self::WrongPhase { .. } => ErrorCode::WrongPhase,
            Self::MissingKing { .. } => ErrorCode::MissingKing,
            Self::DuplicateSquare { .. } => ErrorCode::DuplicateSquare,
            Self::Engine { .. } => ErrorCode::Engine,
        }
    }
}

/// The serialized form of an [`ApiError`]: `{"code": "...", "message": "..."}`.
///
/// Not a wire type — `wasm.rs` unpacks it into a thrown JS `Error`, which the
/// hand-written `HexChessError` describes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDto {
    pub code: ErrorCode,
    pub message: String,
}

impl From<&ApiError> for ApiErrorDto {
    fn from(error: &ApiError) -> Self {
        Self {
            code: error.code(),
            message: error.to_string(),
        }
    }
}

impl From<UserError> for ApiError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::WrongPlayer => Self::WrongPlayer,
            UserError::CannotUndo => Self::CannotUndo,
            UserError::MoveError(MoveError::IllegalMove) => Self::IllegalMove,
            UserError::MoveError(MoveError::NoPieceAtPosition(pos)) => Self::NoPieceAtSquare {
                square: square_name(pos),
            },
            UserError::CoordinateError(e) => Self::from(e),
            // Unreachable through [`GameApi`], which checks the phase before it
            // calls the engine — but it must not degrade to `code: "engine"`.
            UserError::WrongGameState(state) => Self::WrongPhase {
                expected: Phase::from(state).label(),
                actual: "that command",
            },
            other => Self::Engine {
                message: other.to_string(),
            },
        }
    }
}

impl From<CoordinateError> for ApiError {
    fn from(error: CoordinateError) -> Self {
        match error {
            CoordinateError::OutsideBoard { y, x } => Self::InvalidSquare {
                square: format!("{y},{x}"),
            },
            CoordinateError::InvalidHumanNotation { y, x } => Self::InvalidSquare {
                square: format!("{y}{x}"),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;

// --- Squares ------------------------------------------------------------

/// Parses `"f5"` / `"F5"` into a board position. Files are `a`-`k`, ranks 1-11.
///
/// The board is a hexagon, so files have different lengths and a name can be
/// well-formed yet off the board — `a1` is not a square. `Position::from_human`
/// does not catch that, so the result is re-checked through `Position::new`.
pub fn parse_square(square: &str) -> Result<Position> {
    let invalid = || ApiError::InvalidSquare {
        square: square.to_string(),
    };

    let mut chars = square.chars();
    let file = chars.next().ok_or_else(invalid)?;
    let rank: usize = chars.as_str().parse().map_err(|_| invalid())?;

    let pos = Position::from_human((file, rank)).map_err(|_| invalid())?;
    let (y, x) = pos.pos();
    Position::new(y, x).map_err(|_| invalid())
}

/// The inverse of [`parse_square`], always lowercase: `"f5"`.
pub fn square_name(pos: Position) -> String {
    let (file, rank) = pos.to_human();
    format!("{}{rank}", file.to_ascii_lowercase())
}

// --- Conversions --------------------------------------------------------

impl From<Side> for Color {
    fn from(side: Side) -> Self {
        match side {
            Side::White => Color::White,
            Side::Black => Color::Black,
        }
    }
}

impl From<Color> for Side {
    fn from(color: Color) -> Self {
        match color {
            Color::White => Side::White,
            Color::Black => Side::Black,
        }
    }
}

impl From<PieceType> for Kind {
    fn from(piece_type: PieceType) -> Self {
        match piece_type {
            PieceType::King => Kind::King,
            PieceType::Queen => Kind::Queen,
            PieceType::Rook => Kind::Rook,
            PieceType::Bishop => Kind::Bishop,
            PieceType::Knight => Kind::Knight,
            PieceType::Pawn => Kind::Pawn,
        }
    }
}

impl From<Kind> for PieceType {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::King => PieceType::King,
            Kind::Queen => PieceType::Queen,
            Kind::Rook => PieceType::Rook,
            Kind::Bishop => PieceType::Bishop,
            Kind::Knight => PieceType::Knight,
            Kind::Pawn => PieceType::Pawn,
        }
    }
}

impl std::str::FromStr for Kind {
    type Err = ApiError;

    fn from_str(name: &str) -> Result<Self> {
        match name.to_ascii_lowercase().as_str() {
            "king" => Ok(Kind::King),
            "queen" => Ok(Kind::Queen),
            "rook" => Ok(Kind::Rook),
            "bishop" => Ok(Kind::Bishop),
            "knight" => Ok(Kind::Knight),
            "pawn" => Ok(Kind::Pawn),
            _ => Err(ApiError::InvalidPieceType {
                name: name.to_string(),
            }),
        }
    }
}

impl std::str::FromStr for Color {
    type Err = ApiError;

    fn from_str(name: &str) -> Result<Self> {
        match name.to_ascii_lowercase().as_str() {
            "white" => Ok(Color::White),
            "black" => Ok(Color::Black),
            _ => Err(ApiError::InvalidColor {
                name: name.to_string(),
            }),
        }
    }
}

impl From<&Action> for MoveAction {
    fn from(action: &Action) -> Self {
        match action {
            Action::Move => MoveAction::Move,
            Action::Capture { enemy, pos } => MoveAction::Capture {
                piece: enemy.piece_type.into(),
                color: enemy.side.into(),
                square: square_name(*pos),
            },
            Action::Promote { to } => MoveAction::Promote {
                to: to.piece_type.into(),
            },
        }
    }
}

// --- The handle ---------------------------------------------------------

/// A playable game. Commands mutate it in place and return the resulting state;
/// a rejected command leaves the game exactly as it was, because a command that
/// fails its phase check or its validation never reaches the board.
#[derive(Debug)]
pub struct GameApi {
    game: Game,
}

impl Default for GameApi {
    fn default() -> Self {
        Self::new()
    }
}

impl GameApi {
    /// A new game from the standard starting position, white to move.
    pub fn new() -> Self {
        Self {
            game: Game::new(None),
        }
    }

    /// A new game from a position that was set up by hand.
    ///
    /// Both kings must be present: the engine's check detection assumes they
    /// are, so a board without them would panic on the first move.
    pub fn from_pieces(pieces: &[PlacedPiece], active: Color) -> Result<Self> {
        let mut board = BTreeMap::new();
        for placed in pieces {
            let pos = parse_square(&placed.square)?;
            let piece = Piece::new(placed.kind.into(), placed.color.into());
            if board.insert(pos, piece).is_some() {
                return Err(ApiError::DuplicateSquare {
                    square: placed.square.clone(),
                });
            }
        }

        let board = Board { pieces: board };
        require_kings(&board)?;

        let mut game = Game::new(Some(board));
        game.with_active_side(active.into());
        Ok(Self { game })
    }

    /// Discards the game and starts over from the standard position.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn phase(&self) -> Phase {
        self.game.state().into()
    }

    /// Rejects a command the current phase does not accept. `attempted` names
    /// what was sent, for the message.
    fn require_phase(&self, needed: Phase, attempted: &'static str) -> Result<()> {
        let actual = self.phase();
        if actual == needed {
            return Ok(());
        }

        Err(ApiError::WrongPhase {
            expected: actual.label(),
            actual: attempted,
        })
    }

    /// The current position, in full.
    ///
    /// Infallible today; it stays a `Result` because it is what every command
    /// returns, and those can fail.
    pub fn state(&mut self) -> Result<GameState> {
        Ok(snapshot(&mut self.game))
    }

    /// Where the piece on `square` may go.
    ///
    /// Empty rather than an error for anything the player simply cannot move
    /// right now — an empty square, an enemy piece, or a game that is waiting
    /// for a promotion or already over — since this answers a click on the
    /// board, and a click on a dead square is not a mistake.
    pub fn legal_moves(&mut self, square: &str) -> Result<Vec<LegalMove>> {
        let pos = parse_square(square)?;
        if self.phase() != Phase::Normal {
            return Ok(Vec::new());
        }

        let Some(piece) = self.game.board().pieces.get(&pos) else {
            return Ok(Vec::new());
        };
        if piece.side != self.game.active_side() {
            return Ok(Vec::new());
        }
        require_kings(self.game.board())?;

        // The generator can offer the same destination twice — a pawn's double
        // step re-emits its single step — and a square the board can only be
        // entered once, so one entry per destination is what a caller wants.
        let mut destinations: Vec<LegalMove> = Vec::new();
        for mv in self.game.get_movement_options(pos)? {
            let to = square_name(mv.destination);
            let action = MoveAction::from(&mv.action);

            match destinations.iter_mut().find(|seen| seen.to == to) {
                // Of two ways onto one square, a capture is the one to show.
                Some(seen) => {
                    if matches!(action, MoveAction::Capture { .. }) {
                        seen.action = action;
                    }
                }
                None => destinations.push(LegalMove { to, action }),
            }
        }
        Ok(destinations)
    }

    /// Plays a move. Rejected moves leave the game untouched and playable.
    pub fn play(&mut self, from: &str, to: &str) -> Result<GameState> {
        let origin = parse_square(from)?;
        let destination = parse_square(to)?;

        self.require_phase(Phase::Normal, "a move")?;
        require_kings(self.game.board())?;

        self.game.make_move(origin, destination)?;
        self.state()
    }

    /// Replaces the pawn that just reached the far end.
    pub fn promote(&mut self, kind: Kind) -> Result<GameState> {
        self.require_phase(Phase::Promotion, "a promotion")?;

        self.game.promote(kind.into())?;
        self.state()
    }

    /// Takes back the last move, including the half-move that ended the game.
    ///
    /// Accepted in every phase: the engine restores whichever one the undone
    /// move came from, so a finished game becomes playable again.
    pub fn undo(&mut self) -> Result<GameState> {
        self.game.undo()?;
        self.state()
    }
}

// --- Snapshots ----------------------------------------------------------

fn require_kings(board: &Board) -> Result<()> {
    for (side, color) in [(Side::White, Color::White), (Side::Black, Color::Black)] {
        let present = board
            .pieces
            .values()
            .any(|piece| piece.side == side && piece.piece_type == PieceType::King);
        if !present {
            return Err(ApiError::MissingKing { color });
        }
    }
    Ok(())
}

fn snapshot(game: &mut Game) -> GameState {
    let pieces = game
        .board()
        .pieces
        .iter()
        .map(|(pos, piece)| PlacedPiece {
            square: square_name(*pos),
            kind: piece.piece_type.into(),
            color: piece.side.into(),
        })
        .collect();

    let history: Vec<PlayedMove> = game.moves().iter().map(played_move).collect();

    let mut captured = Captured::default();
    for mv in game.moves() {
        if let Action::Capture { enemy, .. } = &mv.action {
            match mv.piece.side {
                Side::White => captured.white.push(enemy.piece_type.into()),
                Side::Black => captured.black.push(enemy.piece_type.into()),
            }
        }
    }

    // A promotion is recorded as an extra move on top of the pawn move that
    // caused it, so it must not count towards the move number.
    let half_moves = game
        .moves()
        .iter()
        .filter(|mv| !matches!(mv.action, Action::Promote { .. }))
        .count();

    let active = game.active_side();
    // The engine's check detection only looks at the side to move, and panics
    // if that king is gone, so a set-up position without kings reports no check.
    let check = require_kings(game.board()).is_ok() && game.king_in_check(active);

    // `winner` errors while the game is still running, which is exactly the
    // `None` this field wants.
    let winner = game.winner().unwrap_or(None).map(Color::from);

    GameState {
        phase: game.state().into(),
        active: active.into(),
        check,
        winner,
        move_number: (half_moves / 2 + 1) as u32,
        pieces,
        captured,
        history,
        can_undo: !game.moves().is_empty(),
    }
}

fn played_move(mv: &GameMove) -> PlayedMove {
    let from = square_name(mv.origin);
    let to = square_name(mv.destination);
    let action = MoveAction::from(&mv.action);

    let prefix = match mv.piece.piece_type {
        PieceType::Pawn => String::new(),
        other => other.symbol().to_string(),
    };
    let notation = match &mv.action {
        Action::Move => format!("{prefix}{from}-{to}"),
        Action::Capture { .. } => format!("{prefix}{from}x{to}"),
        Action::Promote { to: piece } => format!("{to}={}", piece.piece_type.symbol()),
    };

    PlayedMove {
        from,
        to,
        kind: mv.piece.piece_type.into(),
        color: mv.piece.side.into(),
        action,
        notation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placed(square: &str, kind: Kind, color: Color) -> PlacedPiece {
        PlacedPiece {
            square: square.to_string(),
            kind,
            color,
        }
    }

    /// Both kings and nothing else, so tests can add only what they care about.
    fn kings() -> Vec<PlacedPiece> {
        vec![
            placed("f5", Kind::King, Color::White),
            placed("i2", Kind::King, Color::Black),
        ]
    }

    #[test]
    fn squares_round_trip() {
        for square in ["a6", "f1", "f5", "k6", "b11", "f11"] {
            let pos = parse_square(square).expect("valid square");
            assert_eq!(square_name(pos), square);
        }

        // Uppercase in, canonical lowercase out.
        assert_eq!(square_name(parse_square("F5").unwrap()), "f5");

        // The board is a hexagon: `a1` and `k7` are well-formed but off it.
        for square in ["", "z1", "f0", "f12", "l1", "f", "5f", "a1", "k7"] {
            assert!(
                matches!(parse_square(square), Err(ApiError::InvalidSquare { .. })),
                "{square} should be rejected"
            );
        }
    }

    #[test]
    fn new_game_starts_with_white_to_move() {
        let mut api = GameApi::new();
        let state = api.state().unwrap();

        assert_eq!(state.phase, Phase::Normal);
        assert_eq!(state.active, Color::White);
        assert_eq!(state.pieces.len(), 36);
        assert_eq!(state.move_number, 1);
        assert!(!state.check);
        assert!(!state.can_undo);
        assert!(state.history.is_empty());
    }

    #[test]
    fn legal_moves_only_for_the_player_to_move() {
        let mut api = GameApi::new();

        // A white pawn on its starting square: one or two steps forward, each
        // destination named once even though the generator repeats the first.
        let moves = api.legal_moves("b5").unwrap();
        let destinations: Vec<&str> = moves.iter().map(|m| m.to.as_str()).collect();
        assert_eq!(destinations, ["b6", "b7"]);

        // On the f file the black pawn on f7 blocks the second step.
        let moves = api.legal_moves("f5").unwrap();
        let destinations: Vec<&str> = moves.iter().map(|m| m.to.as_str()).collect();
        assert_eq!(destinations, ["f6"]);

        // Black's pieces and empty squares yield nothing while white is to move.
        assert!(api.legal_moves("f7").unwrap().is_empty());
        assert!(api.legal_moves("f6").unwrap().is_empty());
    }

    #[test]
    fn legal_moves_never_repeat_a_destination() {
        let mut api = GameApi::new();

        for square in ["b5", "c5", "d5", "e5", "f5", "g4", "h3", "i2", "j1"] {
            let moves = api.legal_moves(square).unwrap();
            let mut seen: Vec<&str> = moves.iter().map(|m| m.to.as_str()).collect();
            let count = seen.len();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), count, "{square} offered a destination twice");
        }
    }

    #[test]
    fn playing_a_move_switches_sides_and_records_history() {
        let mut api = GameApi::new();
        let state = api.play("f5", "f6").unwrap();

        assert_eq!(state.active, Color::Black);
        assert_eq!(state.move_number, 1);
        assert!(state.can_undo);
        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0].notation, "f5-f6");
        assert_eq!(state.history[0].action, MoveAction::Move);
        assert!(state.pieces.iter().any(|p| p.square == "f6"));
        assert!(!state.pieces.iter().any(|p| p.square == "f5"));

        // Black's reply completes the full move.
        assert_eq!(api.play("b11", "b8").unwrap_err(), ApiError::IllegalMove);
        let state = api.play("b11", "b10").unwrap();
        assert_eq!(state.move_number, 2);
    }

    #[test]
    fn a_rejected_move_leaves_the_game_playable() {
        let mut api = GameApi::new();

        assert_eq!(api.play("f5", "f9").unwrap_err(), ApiError::IllegalMove);
        assert_eq!(api.play("b11", "b10").unwrap_err(), ApiError::WrongPlayer);
        assert_eq!(
            api.play("f6", "f7").unwrap_err(),
            ApiError::NoPieceAtSquare {
                square: "f6".to_string()
            }
        );
        assert!(matches!(
            api.play("f5", "z9").unwrap_err(),
            ApiError::InvalidSquare { .. }
        ));

        // None of that disturbed the position.
        let state = api.state().unwrap();
        assert_eq!(state.active, Color::White);
        assert!(!state.can_undo);
        assert!(api.play("f5", "f6").is_ok());
    }

    #[test]
    fn capture_reports_the_taken_piece() {
        let mut pieces = kings();
        pieces.push(placed("f9", Kind::Pawn, Color::White));
        pieces.push(placed("g9", Kind::Bishop, Color::Black));

        let mut api = GameApi::from_pieces(&pieces, Color::White).unwrap();
        let state = api.play("f9", "g9").unwrap();

        assert_eq!(state.captured.white, [Kind::Bishop]);
        assert!(state.captured.black.is_empty());
        assert_eq!(state.history[0].notation, "f9xg9");
        assert_eq!(
            state.history[0].action,
            MoveAction::Capture {
                piece: Kind::Bishop,
                color: Color::Black,
                square: "g9".to_string(),
            }
        );
    }

    #[test]
    fn promotion_blocks_the_game_until_it_is_answered() {
        let mut pieces = kings();
        pieces.push(placed("k5", Kind::Pawn, Color::White));

        let mut api = GameApi::from_pieces(&pieces, Color::White).unwrap();
        let state = api.play("k5", "k6").unwrap();
        assert_eq!(state.phase, Phase::Promotion);
        assert_eq!(state.active, Color::White, "still white's move to finish");

        // No other command is accepted in this phase.
        assert!(matches!(
            api.play("f5", "f6").unwrap_err(),
            ApiError::WrongPhase { .. }
        ));
        assert!(api.legal_moves("k6").unwrap().is_empty());

        let state = api.promote(Kind::Queen).unwrap();
        assert_eq!(state.phase, Phase::Normal);
        assert_eq!(state.active, Color::Black);
        let queen = state.pieces.iter().find(|p| p.square == "k6").unwrap();
        assert_eq!(queen.kind, Kind::Queen);

        // The promotion is its own history entry, but white has still only
        // played one move, so the move number does not advance because of it.
        assert_eq!(state.history.len(), 2);
        assert_eq!(state.history[1].notation, "k6=Q");
        assert_eq!(state.move_number, 1);

        assert!(matches!(
            api.promote(Kind::Rook).unwrap_err(),
            ApiError::WrongPhase { .. }
        ));
    }

    #[test]
    fn undo_restores_captured_pieces_and_promotions() {
        let mut pieces = kings();
        pieces.push(placed("f9", Kind::Pawn, Color::White));
        pieces.push(placed("g9", Kind::Bishop, Color::Black));

        let mut api = GameApi::from_pieces(&pieces, Color::White).unwrap();
        let before = api.state().unwrap();

        api.play("f9", "g9").unwrap();
        let state = api.undo().unwrap();

        assert_eq!(state, before, "undo returns to the exact prior state");
        assert_eq!(api.undo().unwrap_err(), ApiError::CannotUndo);
    }

    #[test]
    fn undo_steps_back_into_the_promotion_phase() {
        let mut pieces = kings();
        pieces.push(placed("k5", Kind::Pawn, Color::White));

        let mut api = GameApi::from_pieces(&pieces, Color::White).unwrap();
        api.play("k5", "k6").unwrap();
        api.promote(Kind::Queen).unwrap();

        let state = api.undo().unwrap();
        assert_eq!(state.phase, Phase::Promotion);
        let pawn = state.pieces.iter().find(|p| p.square == "k6").unwrap();
        assert_eq!(pawn.kind, Kind::Pawn);
    }

    #[test]
    fn check_is_reported_for_the_side_to_move() {
        let mut pieces = kings();
        pieces.push(placed("f1", Kind::Rook, Color::Black));

        let mut api = GameApi::from_pieces(&pieces, Color::White).unwrap();
        assert!(api.state().unwrap().check, "white king on f5 is attacked");

        let mut api = GameApi::from_pieces(&kings(), Color::White).unwrap();
        assert!(!api.state().unwrap().check);
    }

    #[test]
    fn checkmate_finishes_the_game_and_can_be_taken_back() {
        // Black rooks cover the f file and the ranks around the white king; the
        // last one arriving on h1 completes the net.
        let pieces = vec![
            placed("f5", Kind::King, Color::White),
            placed("a11", Kind::King, Color::Black),
            placed("f1", Kind::Rook, Color::Black),
            placed("e2", Kind::Rook, Color::Black),
            placed("g1", Kind::Rook, Color::Black),
            placed("d3", Kind::Rook, Color::Black),
            placed("i1", Kind::Rook, Color::Black),
        ];

        let mut api = GameApi::from_pieces(&pieces, Color::Black).unwrap();
        let state = api.play("i1", "h1").unwrap();

        assert_eq!(state.phase, Phase::Finished);
        assert_eq!(state.winner, Some(Color::Black));

        let state = api.undo().unwrap();
        assert_eq!(state.phase, Phase::Normal);
        assert_eq!(state.winner, None);
        assert_eq!(state.active, Color::Black);
    }

    #[test]
    fn a_position_without_kings_is_refused_rather_than_panicking() {
        let pieces = vec![placed("f5", Kind::Pawn, Color::White)];
        assert_eq!(
            GameApi::from_pieces(&pieces, Color::White).unwrap_err(),
            ApiError::MissingKing {
                color: Color::White
            }
        );

        let pieces = vec![placed("f5", Kind::King, Color::White)];
        assert_eq!(
            GameApi::from_pieces(&pieces, Color::White).unwrap_err(),
            ApiError::MissingKing {
                color: Color::Black
            }
        );
    }

    #[test]
    fn duplicate_placement_is_refused() {
        let mut pieces = kings();
        pieces.push(placed("f5", Kind::Rook, Color::Black));

        assert_eq!(
            GameApi::from_pieces(&pieces, Color::White).unwrap_err(),
            ApiError::DuplicateSquare {
                square: "f5".to_string()
            }
        );
    }

    #[test]
    fn en_passant_capture_names_the_square_the_pawn_stands_on() {
        let pieces = vec![
            placed("a11", Kind::King, Color::White),
            placed("k6", Kind::King, Color::Black),
            placed("j1", Kind::Pawn, Color::White),
            placed("i3", Kind::Pawn, Color::Black),
        ];

        let mut api = GameApi::from_pieces(&pieces, Color::White).unwrap();
        api.play("j1", "j3").unwrap();

        let moves = api.legal_moves("i3").unwrap();
        let capture = moves
            .iter()
            .find(|m| m.to == "j2")
            .expect("en passant available");

        assert_eq!(
            capture.action,
            MoveAction::Capture {
                piece: Kind::Pawn,
                color: Color::White,
                // The taken pawn is on j3, not on the square being moved to.
                square: "j3".to_string(),
            }
        );

        let state = api.play("i3", "j2").unwrap();
        assert!(!state.pieces.iter().any(|p| p.square == "j3"));
        assert_eq!(state.captured.black, [Kind::Pawn]);
    }

    #[test]
    fn state_serializes_to_the_shape_the_frontend_expects() {
        let mut api = GameApi::new();
        let json = serde_json::to_string(&api.state().unwrap()).unwrap();

        assert!(json.contains(r#""phase":"normal""#));
        assert!(json.contains(r#""active":"white""#));
        assert!(json.contains(r#""moveNumber":1"#));
        assert!(json.contains(r#""canUndo":false"#));
        assert!(json.contains(r#"{"square":"e2","kind":"king","color":"white"}"#));

        // Round-trips, so a caller can keep a state as a plain value.
        let parsed: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, api.state().unwrap());
    }
}
