# Fix the incremental Zobrist update

Goal: `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)` at every
point, including after `undo`.

## Scope

Engine only — `board.rs`, `game.rs`, `zobrist.rs`, `lib.rs`. **`api.rs` and `wasm.rs` are parked**
until the engine is finished: `api.rs` is commented out of `lib.rs` (`lib.rs:5`), `wasm.rs` depends
on it, so neither is compiled or type-checked and both have drifted from current signatures. They
do not count as callers when judging whether an engine change is safe.

## Status (2026-09-11, working tree on top of `2de920c`)

`cargo test -p engine`: **26 passed, 1 failed** — `test_en_passant`, panicking at `game.rs:303` on
defect 14, which is the only defect left.

The XOR bookkeeping, the starting hash, `undo`'s restore, the en-passant semantics and the
`Play::hash` convention ("the position this play produced") are right. En passant now validates and
generates moves for double pushes by **both** sides, verified against hand-built positions (White
J1→J3 with ep field J2; Black B11→B9 with ep field B10, on each of the two capture squares).

Two coverage holes remain, and both need closing regardless of the fix: no test passes `Some(..)` to
`from_board`, so `validate_en_passant` has no coverage at all; and no test compares `position_hash`
against a freshly computed `from_board`, so a hash desync survives a green suite. `test_undo`
catches `undo` only because `Game` derives `Serialize` and `position_hash` is a serialized field.

## Defect 14 — the stale history read

Defect numbering is stable across revisions; 1-13, 15 and 16 are resolved and have been dropped.

`Game::get_en_passant_moves` (`game.rs:288-307`) derives its field from `self.plays.last()`, falling
back to `en_passant_field_initial`. `make_move` reads it twice: `old_en_passant` (`game.rs:413`,
before `board.execute`) is fine either way, but `new_en_passant` (`game.rs:417`) must describe the
move *just made* — and with the push at `game.rs:427`, `plays.last()` is still the previous move
against an already-mutated board. It records a stale ep key, or panics at the `expect`.

**Playing any en passant panics through the public API today.** Set up a position by hand with a
legal en passant — Black B11→B9, ep field B10, a White pawn on C9 — and play it:

```
game.make_move(C9, B10)  ->  panicked at 'En passant field not valid ???', game.rs:303
```

`make_move` executes the capture, then reads `new_en_passant` before the push. With an empty history
that falls back to `en_passant_field_initial`, still B10, and looks for the pawn behind it — the one
the capture just removed. Use this as the regression test: a plain sequence of public calls.
`test_en_passant` reproduces the same panic mid-game.

Ordering alone cannot fix it — restoring the old push order makes `test_en_passant` pass and
`test_undo` fail again. The history read has to go.

### Fix

Give `Game::get_en_passant_moves` a sibling that takes the move instead of reading history —
`Board::get_en_passant_field(&game_move)` already derives the field from a move, and
`Board::get_en_passant_moves` already takes it as an argument:

```rust
self.board.execute(&game_move);
self.position_hash.update_move(&game_move);
if !does_promote {
    self.position_hash.update_active_player();
}
let new_ep = self.valid_en_passant_field_after(&game_move, !self.active_side);
self.position_hash.update_en_passant(old_en_passant, new_ep);
self.plays.push(Play { game_move, hash: self.position_hash });
```

Keeps `Play::hash` as the position produced, without the stale read, and makes the
`// The GameMove stores the hash after the move` comment true again.

## Cleanups

- **One definition of "forward."** `Side::move_direction()` and `get_pawn_moves` (`board.rs:169-170`)
  build the same value independently. They drifted apart once already — that was defect 11, where
  White's direction sat on the wrong axis. Have `get_pawn_moves` call `move_direction()`.
- `validate_en_passant` discards `dy` (`game.rs:174`, `let (_, dx)` with `field.add(0, dx)`). Correct
  only because every direction currently has `dy == 0`; silently breaks if that changes. Restore
  `let (dy, dx)` with `field.add(dy, dx)` and `field.add(-dy, -dx)`, which also clears the `-1 * dx`
  at `game.rs:189` — the one clippy warning the lib emits.
- `validate_en_passant` check (3) routes through the `expect` at `game.rs:303`, and is safe only
  because checks (1)/(2) screen out every `None` case first — both paths happen to go through
  `get_moved_pawn_position_from_en_passant`. Invisible coupling, one edit from a panic in a
  constructor; comment it at minimum.
- `Game::position_hash` has no accessor. `plays().last().hash` is not a substitute — there is
  nothing there before the first move. Add `pub fn position_hash(&self) -> PositionHash`, a method
  rather than a `pub` field, since it is a derived value with an invariant tying it to the board.
- `Play` could take `#[non_exhaustive]`: its public fields let outside code build one whose `hash`
  does not match its `game_move`. Harmless until something like `Game::from_plays` appears.
- `UserError::OutsideBoard` (`game.rs:20`) is never constructed; `GameMove::is_en_passant`
  (`board.rs:87`) has no caller.

## Verification

- `cargo test -p engine` — all 27 pass; `cargo clippy -p engine --all-targets` — clean.
- **Defect 14 regression test**: the hand-set-up repro above — `from_board` with a legal ep field,
  then `make_move` of the capture. It panics today and needs nothing else fixed first.
- **Hash invariant test**: play a scripted game covering a double push, an en-passant capture, a
  normal capture and a promotion; after *every* `make_move`, `promote` and `undo` assert
  `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)`. Add the same
  assertion for `Game::new()`, for `from_board(.., Side::Black, ..)`, and for
  `from_board(.., Some(field))` followed by one unrelated move. Nothing checks a freshly constructed
  game today, which is how the old inverted starting hash went unnoticed through a green suite.
- **`validate_en_passant` cases**, since it has none: both sides accepted, on each of the two capture
  squares; and rejected with `Err(UserError::InvalidBoard(..))` — never a panic — for a piece on the
  ep field, no pawn behind it, the pawn's origin not a starting square, the origin occupied, and a
  field no pawn of the side to move can capture on.
- **`get_movement_options`**, whose fix has no test: in a position with a live en passant, assert it
  returns only moves with `origin == pos` for *every* piece of the side to move, and that
  `make_move(rook, en_passant_field)` is `Err(IllegalMove)`.
- Transposition: two move orders reaching the same position hash equal, and move-then-undo returns
  the exact prior hash.
