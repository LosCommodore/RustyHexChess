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

## Status (checked 2026-09-07, at `d72cd18`)

`cargo test -p engine`: **26 passed, 1 failed** — `test_undo`, on defect 9.

The `d72cd18` refactor moved the en-passant machinery onto `Board` and got the hard part right:

- `Board::get_en_passant_field(&game_move)` takes the move directly instead of reading history —
  this is the `get_en_passant_field_for` helper earlier revisions of this plan asked for.
- `Board::get_en_passant_moves(active_player, last_move)` walks *backwards* from the skipped field
  (`pawn_capture_moves_reversed`) to find pawns that could actually take there.
- `Game::en_passant_possible(player)` filters those through `move_leaves_king_in_check` and returns
  the field only if a legal capture exists. **This closes defect 6b**: it is exactly the "a pawn of
  the side to move can really capture there" test that `PositionHash::from_board`'s contract
  (`zobrist.rs:82-86`) demands, so the incremental key and a recomputed hash now agree on when an
  en-passant key belongs in the hash at all.
- `make_move` folds the field in and out around each move (`game.rs:351`, `359`, `362`).

`Position::new`/`from_human` returning `Option` instead of `CoordinateError`, and `Position::add`,
are a clean simplification — no hash impact.

A green suite would still be weak evidence: no test compares `position_hash` against a freshly
computed `from_board`, so defects 6c and 8 would survive one. `test_undo` catches `undo` only
because `Game` derives `Serialize` and `position_hash` is a serialized field.

## Defects

Numbering is stable across revisions; resolved entries have been dropped, so there are gaps.

| # | Where | Problem |
|---|-------|---------|
| 6c | `game.rs:117`, `game.rs:134` | `from_board`'s `en_passant_field` is folded into the starting hash but never stored. The first `make_move` computes `old_en_passant` via `en_passant_possible`, which reads `plays.last()` and returns `None` on an empty history — so that key is never XOR'd out and rides along in every hash for the rest of the game. Move generation ignores the parameter for the same reason, so the capture it promises is never offered. |
| 7 | `game.rs:173`, `lib.rs:13` | `plays()` returns `&[Play]` with private fields and no accessors, and `PositionHash` sits behind a private `mod zobrist` — history and hash are unusable outside the crate. There is no `position_hash()` accessor at all. Now visible as a clippy warning: `PositionHash::hash` is "never used". `api.rs` still calls `game.moves()` (603/606/618/639); no compile error only because it is commented out of `lib.rs`. |
| 8 | `game.rs:284` | `with_active_side` assigns `active_side` and calls `update_state`, but never toggles the side key. Handing the turn over this way — which `test_stale_mate_on_setup` does twice — silently desynchronises the hash from the position. It must also refresh `position_hash_initial`, or undoing back to an empty stack restores a hash built from the *original* `active_player`. |
| 9 | `game.rs:354-363`, `game.rs:429-432`, `game.rs:383` | Two conventions for `Play::hash` in one crate. `make_move` pushes *before* `update_move`, so its plays carry the hash of the position played **from** (`plays[0].hash == position_hash_initial` in the `test_undo` output proves it). `promote` still pushes after, and `undo` still walks back to `plays.last()` — both the "position produced" reading. The failing test. |
| 10 | `game.rs:232-245` | `get_movement_options(pos)` appends **every** en-passant move available to the side to move, regardless of which piece sits on `pos`. The returned moves have a different `origin` and a different `piece` than the caller asked about. |

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

Fix: filter the appended moves to `origin == pos`, or pass `pos` into `get_en_passant_moves` and let
`Board` do it. `en_passant_possible` is a separate caller and wants the unfiltered list, so the
filter belongs at the `get_movement_options` end.

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

**Defect 9.** `make_move` pushes the `Play` early only because `en_passant_possible` reads
`plays.last()`. It no longer has to: `Board::get_en_passant_moves` already takes the move as an
argument. Give `en_passant_possible` a sibling that takes `&GameMove` (or inline the board call),
compute `new_en_passant` from `game_move` directly, and move the push back to the end:

```rust
self.board.execute(&game_move);
self.position_hash.update_move(&game_move);
if !does_promote {
    self.position_hash.toggle_player();
}
let new_ep = self.en_passant_possible_after(&game_move, !self.active_side);
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

**Defect 6c.** Store the constructor's field: `en_passant_field: Option<Position>` on `Game`,
initialised from `from_board`'s parameter and updated wherever the hash is. `old_en_passant` then
reads the field instead of recomputing from `plays.last()`, which fixes both halves — the key comes
back out, and `get_movement_options` can offer the capture on move one. `undo` recomputes it from
the restored history.

**Defect 8.** `with_active_side`: `if side != self.active_side { self.position_hash.toggle_player() }`
before the assignment, and refresh `position_hash_initial` alongside it.

**Defect 10.** Filter the appended en-passant moves by `origin == pos`, as described above.

### Accessors (defect 7)

Add `Play::game_move(&self) -> &GameMove` and `Play::hash(&self) -> PositionHash`, plus
`Game::position_hash(&self) -> PositionHash`. Make `zobrist` reachable — `pub mod zobrist` or
`pub use zobrist::PositionHash;` in `lib.rs`. Leave `api.rs` alone (already commented out); note
that its `moves()` calls need renaming to `plays()` when it is revived.

### Cleanups

Not defects, but loose ends from the refactor:

- `UserError::OutsideBoard` (`game.rs:21`) is declared and never constructed — `CoordinateError`'s
  removal left it behind.
- `GameMove::is_en_passant` (`board.rs:88`) has no caller.
- `cargo clippy`: `clone_on_copy` on `position_hash.clone()` at `game.rs:139` (`PositionHash` is
  `Copy`), and `mvs.len() > 0` at `game.rs:336`.

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
- Second test: move-then-undo returns the exact prior hash, and two move orders reaching the same
  position hash equal (transposition).
- `cargo clippy -p engine` — clean.
