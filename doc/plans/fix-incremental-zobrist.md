# Fix the incremental Zobrist update

## Context

The current working tree replaces `Game::moves: Vec<GameMove>` with `Game::plays: Vec<Play>`
(move + hash) and adds a running `Game::position_hash`, updated incrementally through
`PositionHash::update`.

The per-move XOR bookkeeping, the starting hash and `undo` are now correct. What is still missing
is the surrounding state: `with_active_side` changes the turn behind the hash's back, and the
en-passant component is not maintained at all.

Goal: `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)` at every
point, including after `undo`.

## Status (checked 2026-09-06)

`cargo test -p engine`: **26 passed, 1 failed** — `test_undo`, on defect 9. Defect 6 is in flight:
`update_en_passant` is now `pub` and called from `make_move` (`game.rs:396`, `404`, `407`), and
`update` has been renamed `update_move`. Wiring it up required `plays.last()` to already hold the
new move, so the `Play` push moved ahead of the hash update — which flipped what `Play::hash`
means in `make_move` while `promote` and `undo` kept the old meaning. That collision is defect 9.

A green suite would be weaker evidence than it looks here: no test compares `position_hash` against
a freshly computed `from_board`, which is why defects 6b/6c and 8 would all survive one. `test_undo`
catches `undo` only because `Game` derives `Serialize` and `position_hash` is a serialized field, so
its JSON round-trip comparison happens to cover the hash.

`cargo clippy -p engine` reports three warnings. Two are cosmetic: a `clone_on_copy` on
`position_hash.clone()` at `game.rs:137`, and a `let...else` in `get_en_passant_field` that could
be `?`. The third is worth keeping: dropping `#![allow(unused)]` from `zobrist.rs` has already
started earning its keep — it now reports `PositionHash::hash` as never used, which is defect 7
showing up as a warning rather than as a silence.

## Defects

Numbering is stable across revisions; resolved entries have been dropped, so there are gaps.

| # | Where | Problem |
|---|-------|---------|
| 6b | `game.rs:247`, `zobrist.rs:82-86` | `get_en_passant_field` returns the skipped field after *any* double pawn push. `PositionHash::from_board`'s contract is stricter: `None` unless a pawn of the side to move can really capture there. Now that `update_en_passant` is live, every double push folds a key in, so the same position reached with and without an unanswerable double push hashes differently and a real threefold repetition goes unseen — precisely what that doc comment warns against. Either add the "can any enemy pawn actually take it" test, or amend the contract to the FEN-style rule and accept the missed repetitions. |
| 6c | `game.rs:117`, `game.rs:132` | `from_board`'s `en_passant_field` is folded into the starting hash but never stored. The first `make_move` reads `old_en_passant` from an empty `plays` and gets `None`, so that key is never XOR'd out — it is stuck in every hash for the rest of the game. Move generation ignores the parameter too, for the same reason. |
| 7 | `game.rs:172`, `lib.rs:13` | `plays()` returns `&[Play]` with private fields and no accessors, and `PositionHash` sits behind a private `mod zobrist` — history and hash are unusable outside the crate. There is no `position_hash()` accessor at all. `api.rs` still calls `game.moves()` (603/606/618/639); no compile error only because it is commented out of `lib.rs`. |
| 8 | `game.rs:348` | `with_active_side` assigns `active_side` and calls `update_state`, but never toggles the side key. Handing the turn over this way — which `test_stale_mate_on_setup` does twice — silently desynchronises the hash from the position. It must also refresh `position_hash_initial`, or undoing back to an empty stack restores a hash built from the *original* `active_player`. |
| 9 | `game.rs:399-406`, `game.rs:474`, `game.rs:428` | Two conventions for `Play::hash` in one crate. `make_move` now pushes *before* `update_move`, so its plays carry the hash of the position played **from** (`plays[0].hash == position_hash_initial` in the `test_undo` output proves it). `promote` still pushes after, and `undo` still walks back to `plays.last()` — both the "position produced" reading. The current failing test. |

`lib.rs` is otherwise only a module reordering — no issue.

## Fix

### `zobrist.rs` — visibility and one signature

`update_en_passant` is now `pub` and `update` has been renamed `update_move`. Also
`#![allow(unused)]` is gone from the file. What is left:

- Drop the `change_player` flag from `update_move`
  ([zobrist.rs:145](../../engine/src/zobrist.rs#L145)) and expose `update_active_player`
  ([zobrist.rs:123](../../engine/src/zobrist.rs#L123)) as `toggle_player()`. The flag is driven
  correctly today, but a caller that owns its turn changes explicitly cannot get it wrong again,
  and defect 8 needs `toggle_player()` regardless.
- `pub(crate)` is enough for all of these — `Game` is the only caller.
- XOR is its own inverse, so `update_move` doubles as the undo of a move; no separate path needed.

### `game.rs` — own the en-passant field and the initial hash

- New field `en_passant_field: Option<Position>`, initialised from `from_board`'s parameter;
  `get_en_passant_field()` becomes the *recompute* after a move. Fixes the constructor case where an
  en-passant field is supplied while `plays` is empty.
- `with_active_side`: `if side != self.active_side { self.position_hash.toggle_player() }` before
  the assignment, and refresh `position_hash_initial` alongside it (defect 8).

`make_move`'s hash block becomes:

```rust
self.board.execute(&game_move);
self.position_hash.update_move(&game_move);
if !does_promote {
    self.position_hash.toggle_player();
}
let new_ep = self.get_en_passant_field_for(&game_move);
self.position_hash.update_en_passant(self.en_passant_field, new_ep);
self.en_passant_field = new_ep;
self.plays.push(Play { game_move, hash: self.position_hash });
```

The `get_en_passant_field_for(&game_move)` helper is the point of defect 9: deriving the new field
from the move itself, rather than from `plays.last()`, is what lets the push stay last and keeps
`Play::hash` meaning "the position this play produced" in all three places. Reading `plays.last()`
forces the push to happen first, which is how the two conventions got mixed. Keep the turn-key
decision tied to `does_promote()` — the pending-promotion ply is the one case where the position
changes without the turn changing.

The alternative is to adopt the flipped convention everywhere: push before the update in `promote`
too, and reduce `undo` to `self.position_hash = play.hash`. That also drops `position_hash_initial`,
since the starting hash becomes `plays[0].hash`. It is coherent, and cheaper than writing the
helper. Prefer "hash of the resulting position" anyway, so a `Play` answers "which position did
this move reach" — the question a threefold-repetition scan asks of the history — and so the
`// The GameMove stores the hash after the move` comment at `game.rs:427` stays true.

`undo()` already restores the hash correctly. It gains one line for the en-passant field:

```rust
self.en_passant_field = self.get_en_passant_field();
```

### Accessors (defect 7)

Add `Play::game_move(&self) -> &GameMove` and `Play::hash(&self) -> PositionHash`, plus
`Game::position_hash(&self) -> PositionHash`. Make `zobrist` reachable — `pub mod zobrist` or
`pub use zobrist::PositionHash;` in `lib.rs`. Leave `api.rs` alone (already commented out); note
that its `moves()` calls need renaming to `plays()` when it is revived.

## Verification

- `cargo test -p engine` — all 27 tests pass.
- New test, the one that would catch defects 6b/6c and 8: play a scripted game covering a
  double pawn push, an en-passant capture, a normal capture and a promotion; after **every**
  `make_move`, `promote` and `undo` assert
  `game.position_hash == PositionHash::from_board(&game.board, game.active_side, game.en_passant_field)`.
  With `change_player` gone the invariant holds at every step, promotion included.
- Add the same assertion to a `Game::new()` and a `from_board(.., Side::Black, ..)` case. Nothing
  checks a freshly constructed game today, which is how the old inverted starting hash went
  unnoticed through a green-but-for-`test_undo` suite; a regression there would be just as silent.
- For 6c specifically: construct with `from_board(.., Some(field))`, play one unrelated move, and
  assert the hash matches a fresh `from_board` of the new position. That is the case where the
  stale key never comes back out, and no existing test passes a `Some(..)` here at all.
- Second test: move-then-undo returns the exact prior hash, and two move orders reaching the same
  position hash equal (transposition).
- `cargo clippy -p engine` — consider removing `#![allow(unused)]` from `zobrist.rs:1` so dead code
  such as an uncalled `update_en_passant` becomes visible again.
