# TODO — finishing the game logic

What the engine still owes a complete game of Gliński hexagonal chess. Written
against the state of `engine/src/` as of this file; line references may drift.

Suggested order: **B → A1 → C1/C2 → A3 → A2 → A4 → C3 → E.**
A1 is the one that affects real games today.

---

## A. End-of-game rules

### A1. Stalemate — not implemented

A player with no legal moves whose king is *not* attacked draws immediately.
Today [`check_king`](engine/src/lib.rs#L170) returns `KingState::Nothing`
whenever the king is unattacked, without asking whether a legal move exists, and
[`next_turn`](engine/src/lib.rs#L261) only ends the game on `Mate`. A stalemated
player is stuck: every `make_move` returns `IllegalMove` forever.

- [ ] Split "has legal moves" out of `check_king` into `legal_moves_for_side(side)` —
      mate, stalemate and the UI all want it.
- [ ] Add `KingState::Stalemate` (or return `(in_check, has_moves)`) and map
      `!in_check && no_moves` to `GameOver { winner: None }`.

### A2. Threefold repetition — no infrastructure at all

There is no position history; `moves: Vec<GameMove>` records moves, not positions.

- [ ] Define a position key: piece placement + side to move + en-passant
      availability. Castling rights are N/A — Gliński has no castling, as already
      noted at [board.rs:38](engine/src/board.rs#L38).
- [ ] Only let the en-passant part of the key differ when a capture is *actually
      available*, otherwise identical positions will not match.
- [ ] Decide: replay `moves` on demand, or maintain a `HashMap<PositionKey, u8>`.
      Zobrist hashing is the better long-term choice given
      [doc/plans/reinforcement-learning-player.md](doc/plans/reinforcement-learning-player.md).
- [ ] Decrement the counter in [`undo`](engine/src/lib.rs#L341).
- [ ] Decide claimed vs automatic: threefold is claimable, fivefold is automatic.
      Needs the new command in C3.

### A3. 50-move rule — no halfmove clock

- [ ] Add a counter, reset on any pawn move or capture, incremented otherwise.
- [ ] Do not double-count promotions: `promote` pushes a *second* `GameMove` for
      the same half-move ([lib.rs:377](engine/src/lib.rs#L377)), which `snapshot`
      already has to filter out at [api.rs:614](engine/src/api.rs#L614).
- [ ] Store the clock per move (parallel `Vec`, or a field on the history entry)
      so `undo` restores it instead of recomputing it.
- [ ] Same split as A2: 50 claimable, 75 automatic.

### A4. Insufficient material — not implemented, and the FIDE table does not transfer

`GameOver { winner: None }` at [lib.rs:58](engine/src/lib.rs#L58) already models a
draw, but nothing ever produces one.

- [ ] Implement the safe cases first: K vs K, K+N vs K, K+B vs K.
- [ ] **Decide the hex-specific cases before coding.** Hex bishops are confined to
      one of *three* colour complexes
      ([movement.rs:72](engine/src/movement.rs#L72)), so "K+B vs K+B, same
      complex" and the multi-bishop endings do not behave as they do in orthodox
      chess. This is the item most likely to end up silently wrong.
- [ ] Write the chosen table down in [doc/](doc/) with its reasoning.

---

## B. Bugs that block the above — fix first

### B2. `king_in_check` panics on a missing king

[lib.rs:148](engine/src/lib.rs#L148). `GameApi` guards it with `require_kings`,
but `Game` is public and the panic is reachable from the CLI and from tests.

- [ ] Return a `UserError` instead of panicking.

### B3. Move generation assumes the queried piece belongs to the side to move

Two places take the side from `self.active_side` rather than from the piece in
hand, so both filter against the wrong side when `get_movement_options` is called
for a piece of the side *not* to move. `GameApi` blocks that; `Game` is public and
does not.

- [ ] [`move_creates_check_on_active_king`](engine/src/lib.rs#L163) — tests the
      active king regardless of whose move it is validating.
- [ ] [lib.rs:228](engine/src/lib.rs#L228) — `pawn_capture_moves(self.active_side)`
      inside `get_en_passant_moves`, which already has `pawn.side` in hand.

### B4. `api` is commented out of the build

[`pub mod api;`](engine/src/lib.rs#L1) is disabled, so `GameApi` compiles against
nothing and its call sites have drifted from the engine.

- [ ] Re-enable the module and fix the stale constructor calls:
      [api.rs:443](engine/src/api.rs#L443) and
      [api.rs:466](engine/src/api.rs#L466) still call `Game::new(None)` /
      `Game::new(Some(board))`, a signature that no longer exists.
- [ ] Drop [`require_kings`](engine/src/api.rs#L575) in favour of the check now in
      `Game::from_board`, rather than keeping two implementations. Its four call
      sites in `api.rs` go with it.

---

## C. Plumbing the new rules need

### C1. A draw reason, not just `winner: None`

`GameOver { winner: Option<Side> }` cannot distinguish stalemate from a 50-move
draw.

- [ ] Add an outcome enum: checkmate, stalemate, threefold, fifty-move,
      insufficient material, agreement, resignation.
- [ ] Carry it through [`Phase::Finished`](engine/src/api.rs#L50) — whose doc
      comment still says "Checkmate" — into
      [`api::GameState`](engine/src/api.rs#L143).

### C2. `undo` must restore all the new state

[lib.rs:351](engine/src/lib.rs#L351) reconstructs state from `mv.action` alone,
which will not survive the additions.

- [ ] Restore the halfmove clock, the repetition counts and the `GameOver` outcome.

### C3. New API commands

In [api.rs](engine/src/api.rs) and [wasm.rs](engine/src/wasm.rs):

- [ ] `claim_draw()`, `offer_draw()` / `accept_draw()`, `resign()`.
- [ ] A `draw_claimable` flag on the snapshot so the UI can enable the button.
- [ ] Matching `ErrorCode` variants.

### C4. Compute the terminal state once

[`snapshot`](engine/src/api.rs#L623) calls `king_in_check` on every state read;
adding repetition and material checks there makes it worse.

- [ ] Determine the terminal state in `next_turn` and cache it on the game.

---

## D. Other gaps

- [ ] **Notation is incomplete** ([`played_move`](engine/src/api.rs#L642)): no
      `+`/`#` suffix, no disambiguation when two knights reach the same square, en
      passant renders as an ordinary capture, and the promotion entry renders as
      `f11=Q` with no origin.
- [ ] **Move generation performance.** Legal move generation clones the piece map
      per check test and mutates/undoes the board for every candidate — O(n²) with
      allocations. Fine for a human game; it is the bottleneck for the RL plan.
- [ ] **`get_movement_options` requires `&mut self`** because it executes moves to
      test for check. A copy-based or attack-map approach would make the query
      `&self`.
- [ ] Optional, decide if in scope: clocks and time forfeit; a FEN-equivalent
      position import/export, which would also hand A2 a compact position key.

---

## E. Tests

- [ ] **Perft-style node counts** from the start position to a fixed depth. The
      single highest-value test for a move generator, and there are none.
- [ ] One regression test per rule: stalemate ends the game; threefold is
      claimable on the third occurrence and not the second; the 50-move counter
      resets on a pawn move and on a capture and survives `undo`; each
      insufficient-material combination.
- [ ] Regression test for the en-passant leak that was fixed: `legal_moves` on a
      rook while an en passant is available must not list the pawn's square.
- [ ] Extend the `test_undo` round-trip at [lib.rs:548](engine/src/lib.rs#L548) to
      cover the new counters — it compares full serialized states, so it catches
      restore bugs for free.
