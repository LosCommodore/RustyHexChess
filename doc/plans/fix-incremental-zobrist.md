# Fix the incremental Zobrist update

Goal: `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)` at every
point, including after `undo`.

**Every known defect is fixed, and the suite now holds them down.** Only cleanups remain.

## Scope

Engine only — `board.rs`, `game.rs`, `zobrist.rs`, `lib.rs`. **`api.rs` and `wasm.rs` are parked**
until the engine is finished: `api.rs` is commented out of `lib.rs` (`lib.rs:5`), `wasm.rs` depends
on it, so neither is compiled or type-checked and both have drifted from current signatures. They
do not count as callers when judging whether an engine change is safe.

## Status (2026-09-12, working tree on top of `fea2fe5`)

- `cargo test -p engine` — **34 passed, 0 failed** (27 before, plus 7 new).
- `cargo clippy -p engine` — lib clean. `--all-targets` reports 9 warnings, all inside
  `#[cfg(test)]` code and all pre-existing.
- Defect numbering was stable across revisions; 1-16 are resolved and have been dropped.

The coverage holes that let defects 11, 15 and 16 hide behind a green suite are closed:
`validate_en_passant` now has tests, and `position_hash` is compared against a freshly computed
`from_board` after every kind of move.

### Which test guards what

Each of these was checked by reintroducing the defect and confirming the suite goes red — a test
that has never failed proves nothing.

| Break this | Caught by |
|---|---|
| `new_en_passant` reading `plays.last()` instead of the move just played (defect 14) | `en_passant_can_be_played_from_a_set_up_position`, both `hash_invariant_*` |
| `Side::move_direction()` for White (defect 11) | `a_playable_en_passant_field_is_accepted`, `hash_invariant_across_a_scripted_game` |
| `validate_en_passant`'s origin-empty check (defect 15) | `a_playable_en_passant_field_is_accepted`, `en_passant_can_be_played_from_a_set_up_position` |
| the `origin == pos` filter in `get_movement_options` (defect 10) | `en_passant_is_only_offered_to_the_pawn_that_can_take` |
| `update_en_passant` or `update_active_player` in `make_move` | all three hash-invariant tests |
| `undo`'s hash restore | `hash_invariant_across_a_scripted_game`, `transpositions_and_undo_agree` |

One trap worth knowing if these tests are ever edited: the defect-10 test needs a *pawn* that cannot
take (G5) as well as a rook. The pawn-type guard at `game.rs:315-321` blocks the rook on its own, so a
rook-only position stays green with the `origin` filter removed.

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
