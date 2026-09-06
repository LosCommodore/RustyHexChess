# Fix the incremental Zobrist update

## Context

The current working tree replaces `Game::moves: Vec<GameMove>` with `Game::plays: Vec<Play>`
(move + hash) and adds a running `Game::position_hash`, updated incrementally through
`PositionHash::update`.

The per-move XOR bookkeeping is now correct. What is still missing is the surrounding state: the
hash `from_board` starts from is inverted, `undo` never rolls it back, `with_active_side` changes
the turn behind its back, and the en-passant component is not maintained at all.

Goal: `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)` at every
point, including after `undo`.

## Status (checked 2026-09-06)

`cargo test -p engine`: **26 passed, 1 failed** — only `test_undo`, and it fails precisely on
defect 5. The board, `active_side` and `plays` all roll back correctly; `position_hash` stays at
`3949434852486983072` (the undone move) where `plays.last().hash` holds the right value
`4372491301682485172`. Since `Game` derives `Serialize` and `position_hash` is a serialized field,
the test's JSON round-trip comparison is already a hash-consistency check on `undo`.

No test compares `position_hash` against a freshly computed `from_board`, which is why defects 1,
6 and 8 are all invisible to the suite.

## Defects

Numbering is stable across revisions; resolved entries have been dropped, so there are gaps.

| # | Where | Problem |
|---|-------|---------|
| 1 | `game.rs:134`, `game.rs:139` | Two bugs in two lines. The hash is built from `!active_player`, so it claims the *other* side is to move; and `active_side` is hardcoded to `Side::White`, so the `active_player` parameter is ignored outright. They cancel only when the caller passes `Side::Black`. `Game::new()` passes `White`: the game opens with White on turn and a hash that says Black. |
| 5 | `game.rs:415` | `undo` rolls back the board, `active_side` and `state`, but not `position_hash`; `Play::hash` is stored and never read. The one failing test. |
| 6 | `game.rs:247`, `zobrist.rs:135` | En-passant component never updated incrementally. `update_en_passant` has no non-test caller (warning masked by `#![allow(unused)]` at `zobrist.rs:1`). Positions differing only in en-passant availability collide, and the `en_passant_field` handed to `from_board` is never stored, so it is never cleared from the hash either. |
| 7 | `game.rs:172`, `lib.rs:13` | `plays()` returns `&[Play]` with private fields and no accessors, and `PositionHash` sits behind a private `mod zobrist` — history and hash are unusable outside the crate. There is no `position_hash()` accessor at all. `api.rs` still calls `game.moves()` (603/606/618/639); no compile error only because it is commented out of `lib.rs`. |
| 8 | `game.rs:348` | `with_active_side` assigns `active_side` and calls `update_state`, but never toggles the side key. Handing the turn over this way — which `test_stale_mate_on_setup` does twice — silently desynchronises the hash from the position. |

`lib.rs` is otherwise only a module reordering — no issue.

## Fix

### `zobrist.rs` — visibility and one signature

- Drop the `change_player` flag from `update` ([zobrist.rs:147](../../engine/src/zobrist.rs#L147))
  and expose `update_active_player` as `toggle_player()`. The flag is driven correctly today, but a
  caller that owns its turn changes explicitly cannot get it wrong again, and defect 8 needs
  `toggle_player()` regardless.
- Make `update_en_passant` reachable. `pub(crate)` is enough for both — `Game` is the only caller.
- XOR is its own inverse, so `update` doubles as the undo of a move; no separate path is needed.

### `game.rs` — own the en-passant field and the initial hash

- `from_board`: hash with `active_player`, not `!active_player`, and set
  `active_side: active_player` rather than the hardcoded `Side::White` (defect 1).
- New field `en_passant_field: Option<Position>`, initialised from `from_board`'s parameter;
  `get_en_passant_field()` becomes the *recompute* after a move. Fixes the constructor case where an
  en-passant field is supplied while `plays` is empty.
- New field `initial_hash: PositionHash`, so `undo` can restore when the stack empties.
- `with_active_side`: `if side != self.active_side { self.position_hash.toggle_player() }` before
  the assignment (defect 8).

`make_move` keeps its current shape; only the hash block changes:

```rust
self.board.execute(&game_move);
self.position_hash.update(&game_move);
if !does_promote {
    self.position_hash.toggle_player();
}
let new_ep = self.get_en_passant_field_for(&game_move);
self.position_hash.update_en_passant(self.en_passant_field, new_ep);
self.en_passant_field = new_ep;
self.plays.push(Play { game_move, hash: self.position_hash });
```

Compute the new en-passant field from `game_move` directly rather than from `plays.last()`, so the
order above works. Keep the turn-key decision tied to `does_promote()` — the pending-promotion ply
is the one case where the position changes without the turn changing.

`promote()`: `update(&game_move)` then `toggle_player()`, which is what its current
`update(&game_move, true)` already does. Push the updated `Play` as above.

`undo()`, after `board.undo(&play.game_move)`:

```rust
self.position_hash = self.plays.last().map(|p| p.hash).unwrap_or(self.initial_hash);
self.en_passant_field = self.get_en_passant_field();
```

Restoring the stored hash beats replaying XORs and makes `Play::hash` load-bearing. This alone
turns `test_undo` green.

### Accessors (defect 7)

Add `Play::game_move(&self) -> &GameMove` and `Play::hash(&self) -> PositionHash`, plus
`Game::position_hash(&self) -> PositionHash`. Make `zobrist` reachable — `pub mod zobrist` or
`pub use zobrist::PositionHash;` in `lib.rs`. Leave `api.rs` alone (already commented out); note
that its `moves()` calls need renaming to `plays()` when it is revived.

## Verification

- `cargo test -p engine` — all 27 tests pass.
- New test, the one that would have caught defects 1, 6 and 8: play a scripted game covering a
  double pawn push, an en-passant capture, a normal capture and a promotion; after **every**
  `make_move`, `promote` and `undo` assert
  `game.position_hash == PositionHash::from_board(&game.board, game.active_side, game.en_passant_field)`.
  With `change_player` gone the invariant holds at every step, promotion included.
- Add the same assertion to a `Game::new()` and a `from_board(.., Side::Black, ..)` case — defect 1
  survives today only because nothing checks a freshly constructed game.
- Second test: move-then-undo returns the exact prior hash, and two move orders reaching the same
  position hash equal (transposition).
- `cargo clippy -p engine` — consider removing `#![allow(unused)]` from `zobrist.rs:1` so dead code
  such as an uncalled `update_en_passant` becomes visible again.
