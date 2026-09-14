# TODO — finishing the game logic

What the engine still owes a complete game of Gliński hexagonal chess. The game
logic lives in [game.rs](engine/src/game.rs); its tests are split into
[game/tests.rs](engine/src/game/tests.rs). Line references drift — trust the
function names.

**Focus: completeness, not speed.** The known performance gaps are collected at
the bottom under *Deferred*; leave them until the rules are complete.

Suggested order: **1 → 2 → 3 → 4 → 5.**


## 1. Insufficient material — the hex-specific cases

The safe cases are done; what remains is the part orthodox chess tables get
wrong on a hex board.

- [ ] **Decide the bishop-complex cases before coding.** Hex bishops are confined
      to one of *three* colour complexes ([movement.rs](engine/src/movement.rs)),
      so K+2B (same complex) is still drawn while different complexes may not be,
      and "K+B vs K+B" depends on the complexes. Today two same-side bishops are
      always ruled "playable" — safe (never a false draw) but incomplete.
- [ ] Write the chosen table down in [doc/](doc/) with its reasoning, so the
      choice is reviewable rather than buried in a match arm.
- [ ] Tests: one per combination, including the hex bishop-complex distinctions.
      There is currently **no** committed insufficient-material test.

## 2. Draw offers, resignation, and claim-vs-automatic

`OutCome` already has `Agreement` and `Resignation`, but nothing produces them,
and threefold / fifty-move are applied automatically rather than on claim.

- [ ] Decide the model: keep threefold and fifty-move **automatic** (simplest,
      current behaviour), or make them **claimable** per FIDE with fivefold /
      75-move as the automatic backstop. Claimable needs a command and a
      `draw_claimable` flag on the snapshot.
- [ ] Add the API commands in [api.rs](engine/src/api.rs) /
      [wasm.rs](engine/src/wasm.rs): `resign()`, `offer_draw()` / `accept_draw()`,
      and `claim_draw()` if going the claimable route, with matching `ErrorCode`s.
- [ ] `undo` of an agreed/claimed draw: that state is *not* derivable from the
      position, unlike mate/repetition, so decide what taking it back means.

## 3. Notation completeness

[`played_move`](engine/src/api.rs) produces a usable but incomplete algebraic:

- [ ] No `+` / `#` check and checkmate suffixes.
- [ ] No disambiguation when two like pieces (e.g. two knights) can reach the same
      square.
- [ ] En passant renders as an ordinary capture.
- [ ] The promotion entry renders as `f11=Q` with no origin square.

## 4. Panic policy at the boundaries

Crash-on-broken-invariant is the intended policy; what's missing is making the
*deliberate* panics legible and distinct from unexamined ones.

- [ ] Audit the `expect`/`panic!` sites. Convert the ones asserting a structural
      invariant to `unreachable!` naming the invariant and what enforces it (as in
      `king_in_check`). The `"??? "` messages are the tell — each is either an
      invariant worth stating or a real error that belongs in `UserError`.
- [ ] wasm boundary: `wasm32-unknown-unknown` builds `panic = "abort"`, so a panic
      traps the module, loses its message, and can leave wasm-bindgen's `RefCell`s
      borrowed so later calls fail with the wrong error. Add
      `console_error_panic_hook` so the crash is legible, and settle the JS-side
      recovery unit — most likely "this `Game` is dead, make a fresh one."

## 5. Tests the engine still lacks

- [ ] **Perft node counts** from the start position to a fixed depth — the single
      highest-value move-generator test, and there are none.
- [ ] **Fifty-move regression**: the clock increments on a quiet move, resets on a
      pawn move and on a capture, survives `undo`, and draws at 100 ply. (Verified
      ad-hoc; not committed.)
- [ ] **Insufficient-material regression** (see item 1).
- [ ] A **stalemate reached by a move** rather than by setup — e.g. bK a6, wK c7,
      wQ b8→d4 (stalemate) vs b8→b7 (mate) from the same position. Only the
      setup case (`test_stale_mate_on_setup`) exists.
- [ ] Clean up the `KingState::Check { .. }` struct patterns in `test_check_mate`
      (clippy warns on the struct pattern for a unit variant; `assert_eq!` would
      assert more).

---

## Deferred — optimization, out of scope for now

Known and deliberately parked until the rules are complete:

- Legal-move generation clones the piece map per check test and mutates/undoes
  the board per candidate — O(n²) with allocations; `next_turn` runs a full
  legal-move search every ply. An attack-map / copy-based approach would also let
  `get_movement_options` and `player_has_movement_options` take `&self` instead of
  `&mut self`.
- `snapshot` recomputes `king_in_check` on every state read; it could be cached
  alongside the outcome in `update_state`.
- A FEN-equivalent import/export: nice-to-have, and would let `from_board` seed a
  starting half-move clock and hand a compact position key to any external tool.
