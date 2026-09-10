# Fix the incremental Zobrist update

## Context

The current working tree replaces `Game::moves: Vec<GameMove>` with `Game::plays: Vec<Play>`
(move + hash) and adds a running `Game::position_hash`, updated incrementally through
`PositionHash::update_move` and `update_en_passant`.

The per-move XOR bookkeeping, the starting hash, `undo`'s restore and the en-passant *semantics*
are now right. What remains is the plumbing around them: one convention collision that breaks
`undo`, a constructor parameter that leaks a key into every later hash, and a move-generation
bug the refactor introduced.

Goal: `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)` at every
point, including after `undo`.

## Status (checked 2026-09-10, working tree on top of `05d6041`)

`cargo test` aborts before running, so the last known result stands: **25 passed, 2 failed** —
`test_undo` on defect 9, and `test_en_passant`, which panics on defect 11.

Defect 7 is resolved: `pub mod zobrist` makes `PositionHash` nameable, `Play`'s fields are public,
and `update_move`/`update_en_passant` were narrowed to `pub(crate)` so the wider module visibility
did not hand outside callers the incremental mutators. `from_board` and `hash()` stay `pub`, which
is the right surface. One small piece is carried into Cleanups below.

The refactor moved the en-passant machinery onto `Board` and got the hard part right:

- `Board::get_en_passant_field(&game_move)` takes the move directly instead of reading history.
- `Board::get_en_passant_moves(active_player, en_passant_field)` walks *backwards* from the skipped
  field (`pawn_capture_moves_reversed`) to find pawns that could actually take there, and now also
  verifies that the double-stepped pawn is really sitting behind that field.
- `Game::get_valid_en_passant_field(side)` filters those through `is_pinned` and returns the field
  only if a legal capture exists. **This closes defect 6b**: it is exactly the "a pawn of the side
  to move can really capture there" test that `PositionHash::from_board`'s contract
  (`zobrist.rs:82-86`) demands, so the incremental key and a recomputed hash now agree on when an
  en-passant key belongs in the hash at all.
- `en_passant_field_initial` on `Game` closes the movement-generation half of defect 6c: with an
  empty history the constructor's field is now consulted, so the capture it promises is offered.
- `impl Not for Side` moved to `lib.rs` with its `use std::ops::Not;`, so the crate compiles again.

`Position::new`/`from_human` returning `Option` instead of `CoordinateError`, and `Position::add`,
are a clean simplification — no hash impact.

A green suite would still be weak evidence: no test compares `position_hash` against a freshly
computed `from_board`, so defects 6c and 8 would survive one. `test_undo` catches `undo` only
because `Game` derives `Serialize` and `position_hash` is a serialized field.

## Defects

Numbering is stable across revisions; resolved entries have been dropped, so there are gaps.

| # | Where | Problem |
|---|-------|---------|
| 6c | `game.rs:134` | Half fixed — `en_passant_field_initial` now feeds move generation. The hash half remains: `from_board` folds the caller's raw `en_passant_field` in, but `old_en_passant` on the first move reads it back through the *validity* filter. Pass a field where no legal capture exists and the two disagree: the key goes in and never comes out. `from_board` has to normalise its parameter through the same test it will later be compared against. |
| 8 | `game.rs:284` | `with_active_side` assigns `active_side` and calls `update_state`, but never toggles the side key. Handing the turn over this way — which `test_stale_mate_on_setup` does twice — silently desynchronises the hash from the position. It must also refresh `position_hash_initial`, or undoing back to an empty stack restores a hash built from the *original* `active_player`. |
| 9 | `game.rs:354-363`, `game.rs:429-432`, `game.rs:383` | Two conventions for `Play::hash` in one crate. `make_move` pushes *before* `update_move`, so its plays carry the hash of the position played **from** (`plays[0].hash == position_hash_initial` in the `test_undo` output proves it). `promote` still pushes after, and `undo` still walks back to `plays.last()` — both the "position produced" reading. The failing test. |
| 10 | `game.rs:255-260` | Still open. `get_movement_options(pos)` appends **every** en-passant move available to the side to move, regardless of which piece sits on `pos`. The returned moves have a different `origin` and a different `piece` than the caller asked about. The refactor moved this code but kept the unfiltered `mv.extend(self.get_en_passant_moves(self.active_side))`. |
| 11 | `lib.rs:27-32` | `Side::move_direction()` returns `(1, 0)` for White — the wrong axis. Pawns advance along **x**: `get_pawn_moves` uses `(0, orientation)` with `orientation = ±1` (`board.rs:169`), and `get_en_passant_field` moves `origin.x` while holding `y` fixed. Black's `(0, -1)` is right; White's should be `(0, 1)`. Its one caller, `get_moved_pawn_position_from_en_passant`, therefore looks for the double-stepped white pawn on the wrong square, finds nothing, and `Board::get_en_passant_moves` returns `None`. That is the `test_en_passant` panic. |
| 12 | `game.rs:246` | `.expect("En passant field not valid ???")` turns a `None` from `Board::get_en_passant_moves` into a panic. For a genuine invariant that is the right call, but this path is reachable from `Game::from_board`'s public `en_passant_field` parameter: hand it a field with no double-stepped pawn behind it and the first move generation crashes. Validate in `from_board` and return `UserError::InvalidBoard`; keep the panic for the post-`plays.last()` path, where it really is an invariant. |

`lib.rs` is otherwise only a module reordering — no issue.

### Defect 10 in detail

This one is a correctness bug in play, not just in the hash, and it is new in `d72cd18`.

`validate_move` picks its move with `options.iter().find(|o| o.destination == destination)`. The
piece at `origin` is checked for ownership, but never that the chosen option actually belongs to it.
So: White double-pushes a pawn, creating en-passant field `F` that a Black pawn can legally take.
Black then asks to move some *other* piece — a rook, the king — to `F`. That piece cannot reach `F`,
so `board.get_movement_options` does not offer it, but the appended en-passant list does. `find`
matches the pawn's capture, `make_move` executes it, and the pawn takes en passant while the rook
never moves. The engine silently plays a different move than the one requested.

Fix: filter the appended moves to `origin == pos`. `get_valid_en_passant_field` is the other caller
of `Game::get_en_passant_moves` and it wants the unfiltered list, so the filter belongs at the
`get_movement_options` end, not inside the helper.

## Fix

### `zobrist.rs`

`update_en_passant` is `pub`, `update` is renamed `update_move`, and `#![allow(unused)]` is gone.
What is left:

- Drop the `change_player` flag from `update_move` and expose `update_active_player` as
  `toggle_player()`. The flag is driven correctly today, but a caller that owns its turn changes
  explicitly cannot get it wrong again, and defect 8 needs `toggle_player()` regardless.
- `pub(crate)` is enough for these — `Game` is the only caller.
- XOR is its own inverse, so `update_move` doubles as the undo of a move; no separate path needed.

### `game.rs` — one ordering fix, one stored field

**Defect 9.** `make_move` pushes the `Play` early only because `get_valid_en_passant_field` reaches
`plays.last()` through `Game::get_en_passant_moves`. It no longer has to:
`Board::get_en_passant_field(&game_move)` derives the field from the move, and
`Board::get_en_passant_moves` takes that field as an argument. Give `Game::get_en_passant_moves` a
sibling that takes the move instead of reading history, and move the push back to the end:

```rust
self.board.execute(&game_move);
self.position_hash.update_move(&game_move);
if !does_promote {
    self.position_hash.toggle_player();
}
let new_ep = self.valid_en_passant_field_after(&game_move, !self.active_side);
self.position_hash.update_en_passant(old_en_passant, new_ep);
self.plays.push(Play { game_move, hash: self.position_hash });
```

Then `Play::hash` means "the position this play produced" in all three places, `promote` and `undo`
need no change, and the `// The GameMove stores the hash after the move` comment at `game.rs:382`
becomes true again.

The alternative is to adopt the flipped convention everywhere: push before the update in `promote`
too, and reduce `undo` to `self.position_hash = play.hash`. That also drops
`position_hash_initial`, since the starting hash becomes `plays[0].hash`. It is coherent and
cheaper. Prefer "hash of the resulting position" anyway, so a `Play` answers "which position did
this move reach" — the question a threefold-repetition scan asks of the history.

**Defect 6c.** `en_passant_field_initial` has the movement-generation half. For the hash half,
normalise in `from_board`: run the caller's parameter through the same validity test
`get_valid_en_passant_field` applies, and hash the normalised value. Then what goes into the
starting hash is exactly what the first `make_move` will XOR back out.

**Defect 11.** `Side::move_direction()` → `(0, 1)` for White. Better still, have `get_pawn_moves`
use it too (`board.rs:169-170` builds the same value inline as `(0, orientation)`), so there is one
definition of "forward" and the next mismatch cannot happen.

**Defect 12.** Validate `en_passant_field` in `from_board` and reject a bad one with
`UserError::InvalidBoard`, so the `expect` at `game.rs:246` is only ever reached on a genuine
internal invariant.

**Defect 8.** `with_active_side`: `if side != self.active_side { self.position_hash.toggle_player() }`
before the assignment, and refresh `position_hash_initial` alongside it.

**Defect 10.** Filter the appended en-passant moves by `origin == pos`, as described above.

**Defect 13.** Restore the name `move_leaves_king_in_check`.

### Cleanups

Not defects, but loose ends from the refactor:

- **Delete `use insta::comparator;` at `zobrist.rs:9`** — it breaks the library build (see Status).
- `Game::position_hash` has no accessor, so the *current* position's hash cannot be read from
  outside. `plays().last().hash` is not a substitute: it is the wrong end of the move under defect
  9, and there is nothing there at all before the first move. Add
  `pub fn position_hash(&self) -> PositionHash`. A method rather than a `pub` field — it is a
  derived value with an invariant tying it to the board.
- `Play` could take `#[non_exhaustive]`. Its fields are public, which suits a passive record and
  matches `GameMove`, but all-public fields also let outside code build a `Play` whose `hash` does
  not match its `game_move`. Harmless while nothing consumes an externally-built one; worth the
  attribute before anything like `Game::from_plays` appears.
- `api.rs` still calls `game.moves()` (603/606/618/639); no compile error only because it is
  commented out of `lib.rs`. Those need renaming to `plays()` when it is revived.
- `UserError::OutsideBoard` (`game.rs:21`) is declared and never constructed — `CoordinateError`'s
  removal left it behind.
- `GameMove::is_en_passant` (`board.rs:88`) has no caller.


## Verification

- `cargo test -p engine` — all 27 tests pass.
- New test, the one that would catch defects 6c and 8: play a scripted game covering a double pawn
  push, an en-passant capture, a normal capture and a promotion; after **every** `make_move`,
  `promote` and `undo` assert
  `game.position_hash == PositionHash::from_board(&game.board, game.active_side, game.en_passant_field)`.
  With `change_player` gone the invariant holds at every step, promotion included.
- Add the same assertion to a `Game::new()` and a `from_board(.., Side::Black, ..)` case. Nothing
  checks a freshly constructed game today, which is how the old inverted starting hash went
  unnoticed through an otherwise-green suite.
- For 6c specifically: construct with `from_board(.., Some(field))`, play one unrelated move, and
  assert the hash matches a fresh `from_board` of the new position. No existing test passes a
  `Some(..)` there at all.
- For 10: after a double pawn push that a pawn can answer en passant, assert that
  `get_movement_options(pos)` returns only moves with `origin == pos`, and that
  `make_move(other_piece, en_passant_field)` is rejected rather than silently moving the pawn.
- For 11: `test_en_passant` already covers White; add the mirror case so a Black double push is
  answered too. The current asymmetry — Black's direction right, White's wrong — is exactly what a
  one-sided test lets through.
- For 12: `from_board(.., Some(field))` with no pawn behind `field` must return
  `Err(UserError::InvalidBoard(..))`, not panic on the next move generation.
- Second test: move-then-undo returns the exact prior hash, and two move orders reaching the same
  position hash equal (transposition).
- `cargo clippy -p engine` — clean.
