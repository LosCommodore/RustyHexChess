# Fix the incremental Zobrist update

Goal: `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)` at every
point, including after `undo`.

**Every known defect is fixed.** What remains is the test suite that should have caught them, plus
cleanups.

## Scope

Engine only — `board.rs`, `game.rs`, `zobrist.rs`, `lib.rs`. **`api.rs` and `wasm.rs` are parked**
until the engine is finished: `api.rs` is commented out of `lib.rs` (`lib.rs:5`), `wasm.rs` depends
on it, so neither is compiled or type-checked and both have drifted from current signatures. They
do not count as callers when judging whether an engine change is safe.

## Status (2026-09-12, working tree on top of `cf9282e`)

- `cargo test -p engine` — **27 passed, 0 failed**.
- `cargo clippy -p engine` — lib clean. `--all-targets` still reports 9 warnings, all inside
  `#[cfg(test)]` code (`board.rs:383+`, `game.rs:554+`) and all pre-existing.
- Defect 14, the last one, is fixed: `Game::get_en_passant_moves` and `get_valid_en_passant_field`
  now take the relevant move as an `Option<GameMove>` parameter (`game.rs:288`, `game.rs:406`)
  instead of reading `self.plays.last()`, so `make_move` can hand `new_en_passant` the move it just
  played (`game.rs:431`) while the push stays at the end.

The invariant was checked by hand across a double push, an en-passant capture, a promotion, and an
`undo` of each — it holds at every step. **None of that is in the suite.** That is now the whole
risk: every fix in this plan rests on ad-hoc verification that no longer exists.

The two standing coverage holes are the reason defects 11, 15 and 16 sat unnoticed behind a green
suite: no test passes `Some(..)` to `from_board`, so `validate_en_passant` has no coverage at all;
and no test compares `position_hash` against a freshly computed `from_board`. `test_undo` catches
`undo` only because `Game` derives `Serialize` and `position_hash` is a serialized field.

Defect numbering was stable across revisions; 1-16 are resolved and have been dropped.

## Tests to write

These are regression tests for code that is already fixed — not reproductions of live bugs.

- **Hash invariant.** Play a scripted game covering a double push, an en-passant capture, a normal
  capture and a promotion; after *every* `make_move`, `promote` and `undo` assert
  `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)`. Add the same
  assertion for `Game::new()`, for `from_board(.., Side::Black, ..)`, and for
  `from_board(.., Some(field))` followed by one unrelated move. Nothing checks a freshly constructed
  game today, which is how the old inverted starting hash went unnoticed.
- **En passant through the public API** — the defect 14 regression test: `from_board` with a legal ep
  field, then `make_move` of the capture. This panicked at `game.rs:304` until the fix, with no
  other defect involved.
- **`validate_en_passant`**, which has no coverage: both sides accepted, on each of the two capture
  squares; and rejected with `Err(UserError::InvalidBoard(..))` — never a panic — for a piece on the
  ep field, no pawn behind it, the pawn's origin not a starting square, the origin occupied, and a
  field no pawn of the side to move can capture on.
- **`get_movement_options`**: in a position with a live en passant, assert it returns only moves with
  `origin == pos` for *every* piece of the side to move, and that `make_move(rook, en_passant_field)`
  is `Err(IllegalMove)` rather than silently playing the pawn's capture.
- **Transposition**: two move orders reaching the same position hash equal, and move-then-undo
  returns the exact prior hash.

## Cleanups

- **One definition of "forward."** `Side::move_direction()` and `get_pawn_moves` (`board.rs:169-170`)
  build the same value independently. They drifted apart once already — that was defect 11, where
  White's direction sat on the wrong axis. Have `get_pawn_moves` call `move_direction()`.
- `validate_en_passant` discards `dy` (`game.rs:174`, `let (_, dx)` with `field.add(0, dx)`). Correct
  only because every direction currently has `dy == 0`; silently breaks if that changes. Restore
  `let (dy, dx)` with `field.add(dy, dx)` and `field.add(-dy, -dx)`.
- **Document what `last_move: None` means** on `get_en_passant_moves` (`game.rs:288`). It is not "no
  en-passant field" but "no previous move — fall back to `en_passant_field_initial`". All four call
  sites are right today, but each decides independently, and a stray `None` mid-game would silently
  resurrect the position's initial field instead of yielding nothing.
- Passing `Option<Position>` — the ep field itself — instead of `Option<GameMove>` would drop both
  the clone at each call site and the overloaded `None` above. `self.plays` and `self.board` are
  separate fields, so a caller can borrow both immutably in one expression.
- The `expect` at `game.rs:304` is safe only because every caller hands in a move that cannot
  produce a field with no pawn behind it — a promoting move is never a double push, and after `undo`
  the preceding play is likewise never one. That argument now spans four call sites and is written
  down nowhere; comment it at minimum.
- `Game::position_hash` has no accessor. `plays().last().hash` is not a substitute — there is
  nothing there before the first move. Add `pub fn position_hash(&self) -> PositionHash`, a method
  rather than a `pub` field, since it is a derived value with an invariant tying it to the board.
- `Play` could take `#[non_exhaustive]`: its public fields let outside code build one whose `hash`
  does not match its `game_move`. Harmless until something like `Game::from_plays` appears.
- `UserError::OutsideBoard` (`game.rs:20`) is never constructed; `GameMove::is_en_passant`
  (`board.rs:87`) has no caller.
