# TODO — finishing the game logic

What the engine still owes a complete game of Gliński hexagonal chess. Written
against the state of `engine/src/` as of this file; line references may drift.
The game logic now lives in [game.rs](engine/src/game.rs), not `lib.rs`.

Suggested order: **C1 → C2 → A3 → A2 → A4 → C3 → E.**
Sections A1 (stalemate) and B (the bugs that blocked everything else) are done
and gone. Nothing in the engine is *wrong* today; what is left is unimplemented
rules, and an API layer that is not compiled at all — see the note opening
section C, which blocks everything the UI can see.

---

## A. End-of-game rules

### A2. Threefold repetition — no infrastructure at all

There is no position history; [`moves: Vec<GameMove>`](engine/src/game.rs#L86)
records moves, not positions.

- [ ] Define a position key: piece placement + side to move + en-passant
      availability. Castling rights are N/A — Gliński has no castling, as already
      noted at [board.rs:37](engine/src/board.rs#L37).
- [ ] Only let the en-passant part of the key differ when a capture is *actually
      available*, otherwise identical positions will not match.
- [ ] Decide: replay `moves` on demand, or maintain a `HashMap<PositionKey, u8>`.
      Zobrist hashing is the better long-term choice given
      [doc/plans/reinforcement-learning-player.md](doc/plans/reinforcement-learning-player.md).
- [ ] Decrement the counter in [`undo`](engine/src/game.rs#L396).
- [ ] Decide claimed vs automatic: threefold is claimable, fivefold is automatic.
      Needs the new command in C3.

### A3. 50-move rule — no halfmove clock

- [ ] Add a counter, reset on any pawn move or capture, incremented otherwise.
- [ ] Do not double-count promotions: `promote` pushes a *second* `GameMove` for
      the same half-move ([game.rs:435](engine/src/game.rs#L435)), which
      `snapshot` already has to filter out at
      [api.rs:620](engine/src/api.rs#L620).
- [ ] Store the clock per move (parallel `Vec`, or a field on the history entry)
      so `undo` restores it instead of recomputing it.
- [ ] Same split as A2: 50 claimable, 75 automatic.

### A4. Insufficient material — not implemented, and the FIDE table does not transfer

[`OutCome`](engine/src/game.rs#L47) already has the variant and
[`GameResult`](engine/src/game.rs#L69) already models a draw, but nothing ever
produces one.

- [ ] Implement the safe cases first: K vs K, K+N vs K, K+B vs K.
- [ ] **Decide the hex-specific cases before coding.** Hex bishops are confined to
      one of *three* colour complexes
      ([movement.rs:72](engine/src/movement.rs#L72)), so "K+B vs K+B, same
      complex" and the multi-bishop endings do not behave as they do in orthodox
      chess. This is the item most likely to end up silently wrong.
- [ ] Write the chosen table down in [doc/](doc/) with its reasoning.
- [ ] Hook the test into [`update_state`](engine/src/game.rs#L302) — it is the one
      place that decides whether the game is over.

---

## C. Plumbing the new rules need

**Blocker for all of C: the API layer is not in the build.**
[api.rs is commented out](engine/src/lib.rs#L4) (`// pub mod api; // todo:
uncomment later and adjust api to the code changes`), and
[wasm.rs](engine/src/wasm.rs) is a thin wrapper over it, so the whole
browser-facing surface is dark. It does not compile as written:
[api.rs:628](engine/src/api.rs#L628) calls `game.winner()`, which no longer
exists — the equivalent is now
[`game_result()`](engine/src/game.rs#L149) returning
[`GameResult`](engine/src/game.rs#L69) with public `winner` and `outcome`.
Re-enabling it is the first step of C1.

### C1. A draw reason, not just `winner: None`

- [x] Outcome enum: [`OutCome`](engine/src/game.rs#L47) covers checkmate,
      stalemate, threefold, fifty-move, insufficient material, agreement and
      resignation, and rides on [`GameState::GameOver`](engine/src/game.rs#L75).
- [ ] Carry it through [`Phase::Finished`](engine/src/api.rs#L50) — whose doc
      comment still says "Checkmate" — into
      [`api::GameState`](engine/src/api.rs#L143), which today exposes only
      `winner: Option<Color>` and cannot say *why* the game ended.
- [ ] The frontend already expects it: `GameStatus` in
      [state.ts:8](frontend/src/game/state.ts#L8) has `'stalemate'` and `'draw'`
      with nothing feeding them.

### C2. `undo` must restore all the new state

[`undo`](engine/src/game.rs#L396) reconstructs state from `mv.action` alone,
which will not survive the additions. It is correct for mate and stalemate —
the position you return to is one the mover had a legal move in, so it can
never be terminal — but that stops being true once counters are involved.

- [ ] Restore the halfmove clock and the repetition counts.
- [ ] Decide what `undo` does to a claimed/agreed draw, which is *not* derivable
      from the position.

### C3. New API commands

In [api.rs](engine/src/api.rs) and [wasm.rs](engine/src/wasm.rs):

- [ ] `claim_draw()`, `offer_draw()` / `accept_draw()`, `resign()`.
- [ ] A `draw_claimable` flag on the snapshot so the UI can enable the button.
- [ ] Matching `ErrorCode` variants.

### C4. Compute the terminal state once

- [x] [`update_state`](engine/src/game.rs#L302) decides the terminal state once
      and caches it in `Game::state`. It is called from
      [`from_board`](engine/src/game.rs#L109) (a hand-set-up position can already
      be finished), [`with_active_side`](engine/src/game.rs#L324) and
      [`next_turn`](engine/src/game.rs#L317). Repetition and material checks
      belong here too.
- [ ] It writes `GameState::Normal` in the non-terminal case, so it must never be
      called while a promotion is pending. All three call sites are safe today;
      a fourth would need checking.
- [ ] [`snapshot`](engine/src/api.rs#L591) still calls `king_in_check` on every
      state read for the `check` flag. Cache that alongside the outcome.

---

## D. Other gaps

- [ ] **Notation is incomplete** ([`played_move`](engine/src/api.rs#L643)): no
      `+`/`#` suffix, no disambiguation when two knights reach the same square, en
      passant renders as an ordinary capture, and the promotion entry renders as
      `f11=Q` with no origin.
- [ ] **Move generation performance.** Legal move generation clones the piece map
      per check test and mutates/undoes the board for every candidate — O(n²) with
      allocations, and [`next_turn`](engine/src/game.rs#L317) now runs a full
      legal-move search every ply instead of only when in check. Measured at
      ~0.3 ms per ply for a random playout including move selection: fine for a
      human game, and still the bottleneck for the RL plan.
- [ ] **`get_movement_options` requires `&mut self`** because it executes moves to
      test for check. A copy-based or attack-map approach would make the query
      `&self`, and would let [`player_has_movement_options`](engine/src/game.rs#L201)
      stop cloning the piece map.
- [ ] Optional, decide if in scope: clocks and time forfeit; a FEN-equivalent
      position import/export, which would also hand A2 a compact position key.

### D1. Panic policy — prio 2, not yet analysed

Crashing on a broken invariant is the intended policy. What is missing is making
the *deliberate* panics distinguishable from the unexamined ones.

- [ ] Audit the `expect`/`panic!` sites and convert the ones that assert a
      structural invariant to `unreachable!` with a message naming the invariant
      and what enforces it — as done in
      [`king_in_check`](engine/src/game.rs#L171). The `??? `-style messages
      ([game.rs:106](engine/src/game.rs#L106),
      [game.rs:186](engine/src/game.rs#L186),
      [game.rs:251](engine/src/game.rs#L251),
      [game.rs:426](engine/src/game.rs#L426),
      [game.rs:436](engine/src/game.rs#L436)) are the tell: each is either an
      invariant worth stating or a real error that belongs in `UserError`. Not
      analysed yet — decide per site.
- [ ] Panic handling at the wasm boundary. `wasm32-unknown-unknown` builds
      `panic = "abort"`, so `catch_unwind` is not available and there is no
      supervisor to restart anything: the module traps, the panic message is lost
      in the browser, and wasm-bindgen's internal `RefCell`s can be left borrowed
      so later calls fail with borrow errors instead of the original cause. Add
      `console_error_panic_hook` so the crash is legible, and decide the recovery
      unit on the JS side — most likely "this `Game` is dead, construct a fresh
      one" rather than trying to continue. Keeps the crash-on-bugs policy; just
      picks the failing unit deliberately.

---

## E. Tests

- [ ] **Perft-style node counts** from the start position to a fixed depth. The
      single highest-value test for a move generator, and there are none.
- [ ] One regression test per rule. Stalemate has one
      ([`test_stale_mate_on_setup`](engine/src/game.rs#L760)); still missing:
      threefold is claimable on the third occurrence and not the second; the
      50-move counter resets on a pawn move and on a capture and survives `undo`;
      each insufficient-material combination.
- [ ] A stalemate reached *by a move* rather than by setup — the sweep that
      validated the rule found plenty, e.g. bK A6, wK C7, wQ B8→D4 (stalemate)
      against wQ B8→B7 (mate) from the same position.
- [ ] Regression test for the en-passant leak that was fixed: `legal_moves` on a
      rook while an en passant is available must not list the pawn's square.
- [ ] Extend the `test_undo` round-trip at [game.rs:608](engine/src/game.rs#L608)
      to cover the new counters — it compares full serialized states, so it
      catches restore bugs for free.
- [ ] The `KingState::Check { .. }` patterns in `test_check_mate`
      ([game.rs:710](engine/src/game.rs#L710) and three more) are struct patterns
      on a unit variant — clippy warns, and `assert_eq!` would assert more.
