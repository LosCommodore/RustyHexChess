# Fix the incremental Zobrist update

Goal: `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)` at every
point, including after `undo`.

## Status (2026-09-11, clean tree at `9a9cc3d`)

`cargo test -p engine`: **26 passed, 1 failed** — `test_en_passant`, panicking at `game.rs:302`.

The XOR bookkeeping, the starting hash, `undo`'s restore, the en-passant semantics and the
`Play::hash` convention ("the position this play produced") are right. What remains is plumbing.

**`validate_en_passant` rejects every en-passant field, for both sides.** Verified against
hand-built positions (White J1→J3, ep field J2, Black pawn on I3 to take — and the Black-pushed
mirror): both come back `Err(InvalidBoard(..))`. Defect 16 kills the White-pushed case, defect 15
the Black-pushed one. Its *structure* is sound — checks (1)-(3) exactly characterise a legal double
push (skipped field empty, pawn at `field + forward`, origin at `field - forward` an empty starting
square) — only the arithmetic is broken.

Two coverage holes let this through, and both need closing regardless of the fixes: no test passes
`Some(..)` to `from_board`, so `validate_en_passant` has no coverage at all; and no test compares
`position_hash` against a freshly computed `from_board`, so defect 8 would survive a green suite.
`test_undo` catches `undo` only because `Game` derives `Serialize` and `position_hash` is a
serialized field.

## Defects

Numbering is stable across revisions; resolved entries have been dropped, so there are gaps.

| # | Where | Problem |
|---|-------|---------|
| 8 | `game.rs:354-360` | `with_active_side` assigns `active_side` and calls `update_state`, but never toggles the side key. Handing the turn over this way — which `test_stale_mate_on_setup` does twice — silently desynchronises the hash from the position. It must also refresh `position_hash_initial`, or undoing back to an empty stack restores a hash built from the *original* `active_player`. |
| 10 | `game.rs:309-315` | `get_movement_options(pos)` appends **every** en-passant move available to the side to move, regardless of which piece sits on `pos`. The returned moves have a different `origin` and a different `piece` than the caller asked about. Detail below. |
| 11 | `lib.rs:28-33` | `Side::move_direction()` returns `(1, 0)` for White — the wrong axis, and in fact one of White's *capture* directions (`pawn_capture_moves(White) == [(1,0), (-1,1)]`). Pawns advance along **x**: `get_pawn_moves` builds `(0, orientation)` (`board.rs:169-170`) and `get_en_passant_field` moves `origin.x` holding `y` fixed. Black's `(0, -1)` is right; White's should be `(0, 1)`. `get_moved_pawn_position_from_en_passant` therefore looks for the double-stepped white pawn on the wrong square and `Board::get_en_passant_moves` returns `None`. Pre-existing on `main`, not from the refactor. |
| 13 | `game.rs:264` | Restore the name `move_leaves_king_in_check`. |
| 14 | `game.rs:410-423` | **New in `501d17b`.** `get_valid_en_passant_field` reads `self.plays.last()`. Moving the push to the end left `new_en_passant` at `game.rs:414` reading the **previous** move against an already-executed board. Detail below. |
| 15 | `game.rs:194-196` | **New in `501d17b`.** Check 2b's emptiness test is inverted: `if !self.board.pieces.get(&pawn_origin).is_none()` became `if !self.board.pieces.contains_key(&pawn_origin)`, negating twice over. It now errors when the origin square is *empty* — the only legal case — and accepts when it is occupied. Drop the `!`. The same rewrite at check (1) (`game.rs:166`) is correct. |
| 16 | `game.rs:173-174, 188` | **New in `9a9cc3d`.** `let (_, dx) = enemy.move_direction();` with `field.add(0, dx)` works around defect 11 by discarding `dy`. It does not help: for White `move_direction` is `(1, 0)`, so `dx == 0` and `expected_pawn_pos == field` — which check (1) has just proved empty, so check 2a always fails. |

### Defect 10 in detail

A correctness bug in play, not just in the hash; new in `d72cd18`.

`validate_move` picks its move with `options.iter().find(|o| o.destination == destination)`, never
checking that the chosen option belongs to the piece at `origin`. So: White double-pushes, creating
en-passant field `F` that a Black pawn can take. Black asks to move some *other* piece — a rook, the
king — to `F`. That piece cannot reach `F`, but the appended en-passant list offers a move there
anyway; `find` matches the pawn's capture and executes it while the rook never moves. The engine
silently plays a different move than the one requested.

### Defect 14 in detail

`Game::get_en_passant_moves` (`game.rs:287-306`) derives its field from `self.plays.last()`, falling
back to `en_passant_field_initial`. `make_move` reads it twice: `old_en_passant` (`game.rs:410`,
before `board.execute`) is fine either way, but `new_en_passant` (`game.rs:414`) must describe the
move *just made* — and with the push now at `game.rs:420`, `plays.last()` is still the previous move
against an already-mutated board. It records a stale ep key, or panics at the `expect`.

In `test_en_passant`: Black answers the double push with the en-passant capture, `new_en_passant`
re-derives White's old field J2, looks for the white pawn behind it — just captured — and feeds
`None` into `game.rs:302`. Confirmed by experiment: with defect 11 patched it still panics there;
additionally restoring the old push order makes it pass and `test_undo` fail again. Ordering alone
cannot satisfy both — the history read has to go.

## Fix

**`zobrist.rs`.** Drop the `change_player` flag from `update_move` (`zobrist.rs:145,165`) and expose
`update_active_player` as `toggle_player()` — defect 8 needs it regardless, and a caller that owns
its turn changes explicitly cannot get the flag wrong. `pub(crate)` is enough; `Game` is the only
caller. XOR is its own inverse, so `update_move` doubles as the undo of a move.

**Defect 14.** Give `Game::get_en_passant_moves` a sibling that takes the move instead of reading
history — `Board::get_en_passant_field(&game_move)` already derives the field from a move, and
`Board::get_en_passant_moves` already takes it as an argument:

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

Keeps `Play::hash` as the position produced, without the stale read, and makes the
`// The GameMove stores the hash after the move` comment true again.

**Defect 15.** `game.rs:194` → `if self.board.pieces.contains_key(&pawn_origin)`.

**Defect 11.** `Side::move_direction()` → `(0, 1)` for White. Better still, have `get_pawn_moves`
use it too (`board.rs:169-170` builds the same value inline), so there is one definition of
"forward" and the next mismatch cannot happen.

**Defect 16.** With 11 fixed, restore both components: `let (dy, dx) = enemy.move_direction();`,
`field.add(dy, dx)` for the pawn and `field.add(-dy, -dx)` for its origin. The current
`field.add(0, -1 * dx)` is also a clippy `neg_multiply`.

**Defect 8.** `with_active_side`: `if side != self.active_side { self.position_hash.toggle_player() }`
before the assignment, and refresh `position_hash_initial` alongside it.

**Defect 10.** Filter the appended en-passant moves by `origin == pos`. `get_valid_en_passant_field`
is the other caller of `Game::get_en_passant_moves` and wants the unfiltered list, so the filter
belongs at the `get_movement_options` end, not inside the helper.

**Defect 13.** Restore the name `move_leaves_king_in_check`.

### Cleanups

- `validate_en_passant` check (3) routes through the `expect` at `game.rs:302`, and is safe only
  because checks (1)/(2) screen out every `None` case first — both paths happen to go through
  `get_moved_pawn_position_from_en_passant`. Invisible coupling, one edit from a panic in a
  constructor; comment it at minimum.
- `Game::position_hash` has no accessor. `plays().last().hash` is not a substitute — there is
  nothing there before the first move. Add `pub fn position_hash(&self) -> PositionHash`, a method
  rather than a `pub` field, since it is a derived value with an invariant tying it to the board.
- `Play` could take `#[non_exhaustive]`: its public fields let outside code build one whose `hash`
  does not match its `game_move`. Harmless until something like `Game::from_plays` appears.
- `api.rs` still calls `game.moves()` (603/606/618/639) — renamed to `plays()` when it is revived.
  No compile error only because it is commented out of `lib.rs`.
- `UserError::OutsideBoard` (`game.rs:19`) is never constructed; `GameMove::is_en_passant`
  (`board.rs:87`) has no caller.

## Verification

- `cargo test -p engine` — all 27 pass; `cargo clippy -p engine` — clean (`neg_multiply` at
  `game.rs:188` today).
- **Positive cases for `validate_en_passant`, both sides** — the gap that let an always-rejecting
  validator ship. Defects 15 and 16 each fail one side, so a one-sided test is not enough.
- Negative cases, each `Err(UserError::InvalidBoard(..))` and never a panic: a piece on the ep
  field; no pawn behind it; the pawn's origin not a starting square; the origin occupied; a field no
  pawn of the side to move can capture on.
- **Hash invariant test** (catches defect 8): play a scripted game covering a double push, an
  en-passant capture, a normal capture and a promotion; after *every* `make_move`, `promote` and
  `undo` assert `position_hash == PositionHash::from_board(&board, active_side, en_passant_field)`.
  With `change_player` gone it holds at every step, promotion included. Add the same assertion for
  `Game::new()`, for `from_board(.., Side::Black, ..)`, and for `from_board(.., Some(field))`
  followed by one unrelated move — nothing checks a freshly constructed game today, which is how the
  old inverted starting hash went unnoticed through a green suite.
- Defect 10: after a double push a pawn can answer, assert `get_movement_options(pos)` returns only
  moves with `origin == pos`, and that `make_move(other_piece, en_passant_field)` is rejected.
- Defect 11: `test_en_passant` covers a White double push; add the Black mirror.
- Defect 14: the ep capture in `test_en_passant` is the regression test — it panics today. Add a
  transposition check too: two move orders reaching the same position hash equal, and move-then-undo
  returns the exact prior hash.
