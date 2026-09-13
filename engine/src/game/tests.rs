//! Tests for the parent `game` module.
//!
//! A `#[cfg(test)] mod tests;` child of `game.rs`, split out to keep that
//! file readable. Still a unit-test module: `use super::*;` reaches the
//! parent's private items exactly as an inline `mod tests` would.

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
    // insta resolves its `snapshots/` dir next to this source file; since the
    // tests moved into `src/game/`, point back at the shared `src/snapshots`
    // (one level up) where the committed snapshots and the board tests' live.
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_debug_snapshot!(snapshot_name, options);
    });
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

    // A spectator pawn tucked by the black king: it keeps the starting
    // position from being an insufficient-material draw (which would end the
    // game before it begins) without touching the mate over on the f-file.
    board
        .pieces
        .insert(human(('B', 11)).unwrap(), Piece::new(Pawn, Black));

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

    // A spectator pawn by the white king, so the bare-kings starting board is
    // not an insufficient-material draw before the en-passant play begins. It
    // sits far from the j-file and never moves.
    board
        .pieces
        .insert(human(('A', 10)).unwrap(), Piece::new(Pawn, White));

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

fn is_threefold(game: &Game) -> bool {
    matches!(
        game.state(),
        GameState::GameOver(GameResult {
            outcome: OutCome::ThreefoldRepetition,
            winner: None,
        })
    )
}

// A rook and a king shuffling: the same position on its third occurrence
// must end the game as a draw. The rook's opening step is never undone, so
// the starting position does not recur and P is the sole repeated position.
#[test]
fn threefold_repetition_ends_the_game() {
    use PieceType::*;
    use Side::*;

    let board = board_with(&[
        (('A', 11), King, White),
        (('K', 6), King, Black),
        (('C', 11), Rook, White),
    ]);
    let mut game = Game::from_board(board, White, None).expect("valid board");

    // Step the rook off its start to reach P (1st occurrence), then return to
    // P twice by shuffling the rook and the black king.
    game.make_move(pos(('C', 11)), pos(('C', 10))).unwrap();
    let cycle = [
        (('K', 6), ('K', 5)),
        (('C', 10), ('C', 9)),
        (('K', 5), ('K', 6)),
        (('C', 9), ('C', 10)),
    ];
    for _ in 0..2 {
        for &(origin, destination) in &cycle {
            game.make_move(pos(origin), pos(destination)).unwrap();
        }
    }

    assert!(
        is_threefold(&game),
        "P occurred three times; expected a threefold draw, got {:?}",
        game.state()
    );
}

// The starting position is a valid occurrence for the threefold rule, so
// returning to it twice is its third appearance and must draw.
#[test]
fn threefold_repetition_counts_the_starting_position() {
    use PieceType::*;
    use Side::*;

    // The lone kings would be an insufficient-material draw at setup, so a
    // static central pawn keeps the game alive; it never moves, so the
    // starting position still recurs as the kings shuffle.
    let board = board_with(&[
        (('A', 11), King, White),
        (('K', 6), King, Black),
        (('F', 5), Pawn, White),
    ]);
    let mut game = Game::from_board(board, White, None).expect("valid board");

    // One full cycle returns to the starting position; do it twice.
    let cycle = [
        (('A', 11), ('A', 10)),
        (('K', 6), ('K', 5)),
        (('A', 10), ('A', 11)),
        (('K', 5), ('K', 6)),
    ];
    for _ in 0..2 {
        for &(origin, destination) in &cycle {
            game.make_move(pos(origin), pos(destination)).unwrap();
        }
    }

    assert!(
        is_threefold(&game),
        "the starting position occurred three times; expected a threefold draw, got {:?}",
        game.state()
    );
}

// The position right after a capture is a valid occurrence: the scan back
// must count it before it stops at the (irreversible) capture.
#[test]
fn threefold_repetition_counts_the_position_after_a_capture() {
    use PieceType::*;
    use Side::*;

    let board = board_with(&[
        (('A', 11), King, White),
        (('K', 6), King, Black),
        (('C', 11), Rook, White),
        (('C', 10), Rook, Black),
    ]);
    let mut game = Game::from_board(board, White, None).expect("valid board");

    // White captures the black rook -> position X (1st occurrence), then the
    // kings shuffle back to X twice more.
    game.make_move(pos(('C', 11)), pos(('C', 10)))
        .expect("capture");
    let cycle = [
        (('K', 6), ('K', 5)),
        (('A', 11), ('A', 10)),
        (('K', 5), ('K', 6)),
        (('A', 10), ('A', 11)),
    ];
    for _ in 0..2 {
        for &(origin, destination) in &cycle {
            game.make_move(pos(origin), pos(destination)).unwrap();
        }
    }

    assert!(
        is_threefold(&game),
        "the post-capture position occurred three times; expected a threefold draw, got {:?}",
        game.state()
    );
}
