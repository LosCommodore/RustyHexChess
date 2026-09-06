# Fix the incremental Zobrist update

## Context

The current working tree replaces `Game::moves: Vec<GameMove>` with `Game::plays: Vec<Play>`
(move + hash) and adds a running `Game::position_hash`, updated incrementally through
`PositionHash::update`.

As written, `position_hash` diverges from `PositionHash::from_board` of the actual position after
anything except a plain non-capturing move. `cargo test -p engine` fails `game::tests::test_undo`.
Nothing else fails because no test compares the incremental hash against a recomputed one.

Goal: `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)` at every
point, including after `undo`.

## Defects

| # | Where | Problem |
|---|-------|---------|
| 1 | `game.rs:139` | `from_board` hashes with `active_player` but sets `active_side: Side::White`. Hash and game disagree; `update_state` evaluates the wrong side. |
| 2 | `zobrist.rs:147` | `update` ignores `Action::Capture` — captured piece is never XOR'd out. For en passant the pawn is on `Capture.pos`, not `destination`. |
| 3 | `game.rs:473` | Promotion passes `origin == destination` with the *pawn* as `piece`, so the two `update_piece` calls cancel: pawn not removed, new piece not added. |
| 4 | `game.rs:398` | `is_promote_move` is dead — `validate_move` never yields `Action::Promote` (only `promote()` builds it). `change_player` is always `true`, so a promotion flips the side-to-move key twice for one turn change. |
| 5 | `game.rs:427` | `undo` rolls back the board but not `position_hash`; `Play::hash` is never read. This is the failing test. |
| 6 | `game.rs:399` | En-passant component never updated incrementally. `update_en_passant` has no non-test caller (warning masked by `#![allow(unused)]`). Positions differing only in en-passant availability collide, and `from_board`'s `en_passant_field` is never cleared. |
| 7 | `game.rs:172` | `plays()` returns `&[Play]` with private fields and no accessors — history unusable outside the crate. `api.rs` still calls `game.moves()` (603/606/618/639); no compile error only because it is commented out of `lib.rs`. |

`lib.rs` is only a module reordering — no issue.

## Fix

### `zobrist.rs` — give `update` full move semantics

Drop the `change_player` flag (the caller owns turn changes) and switch on `game_move.action`:

- `Move` — `update_piece(origin, piece)`, `update_piece(destination, piece)`
- `Capture { enemy, pos }` — the above plus `update_piece(*pos, enemy)`; using `pos` rather than
  `destination` is what makes en passant correct
- `Promote { to }` — `update_piece(origin, piece)` (pawn off) and `update_piece(destination, to)`
  (`origin == destination` here, fine once the pieces differ)

Expose `toggle_player()` and keep `update_en_passant` reachable (`pub(crate)` is enough — `Game` is
the only caller). XOR is its own inverse, so `update` doubles as the undo of a move.

### `game.rs` — own the en-passant field and the initial hash

- New field `en_passant_field: Option<Position>`, initialised from `from_board`'s parameter;
  `get_en_passant_field()` becomes the *recompute* after a move. Fixes the constructor case where an
  en-passant field is supplied while `plays` is empty.
- New field `initial_hash: PositionHash`, so `undo` can restore when the stack empties.
- `from_board`: set `active_side: active_player` (defect 1).

`make_move`, after `board.execute`:

```rust
self.position_hash.update(&game_move);
self.position_hash.toggle_player();
let new_ep = self.get_en_passant_field_for(&game_move);
self.position_hash.update_en_passant(self.en_passant_field, new_ep);
self.en_passant_field = new_ep;
self.plays.push(Play { game_move, hash: self.position_hash });
```

Compute the new en-passant field from `game_move` directly rather than from `plays.last()`, so the
order above works. Do **not** derive `change_player` from `Action::Promote`: a pawn reaching the
promotion rank still carries `Move`/`Capture`.

`promote()`: `self.position_hash.update(&game_move)` only — no `toggle_player`, `make_move` already
handed over the turn. Push the updated `Play` as above.

`undo()`, after `board.undo(&play.game_move)`:

```rust
self.position_hash = self.plays.last().map(|p| p.hash).unwrap_or(self.initial_hash);
self.en_passant_field = self.get_en_passant_field();
```

Restoring the stored hash beats replaying XORs and makes `Play::hash` load-bearing.

### Accessors (defect 7)

Add `Play::game_move(&self) -> &GameMove` and `Play::hash(&self) -> PositionHash`. `PositionHash`
sits behind the private `mod zobrist` — make it `pub mod zobrist` or `pub use zobrist::PositionHash;`
in `lib.rs`. Leave `api.rs` alone (already commented out); note that its `moves()` calls need
renaming when it is revived.

## Verification

- `cargo test -p engine` — `game::tests::test_undo` passes again.
- New test: play a scripted game covering a double pawn push, an en-passant capture, a normal
  capture and a promotion; after **every** `make_move`, `promote` and `undo` assert
  `game.position_hash == PositionHash::from_board(&game.board, game.active_side, game.en_passant_field)`.
  This single test catches defects 1–6.
- Second test: move-then-undo returns the exact prior hash, and two move orders reaching the same
  position hash equal (transposition).
- `cargo clippy -p engine` — consider removing `#![allow(unused)]` in `game.rs` so dead code such as
  defect 4 becomes visible again.
