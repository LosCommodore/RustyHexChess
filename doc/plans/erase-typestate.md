# RustyHexChess: Plan for Erasing the Type-State Pattern

**Status:** proposal, no code written yet.
**Scope:** `lib.rs` and `api.rs`. `board.rs`, `movement.rs`, `coordinates.rs` and `piece.rs` are untouched.
**Goal:** one `Game` type with a `status` field, mutated through `&mut self`. Delete the machinery that exists only to hide the generic from callers.

---

## 1. Should the pattern go?

**Yes.** The learning goal is met and recorded in [type_state_pattern.md](../type_state_pattern.md); what remains is cost. Four findings, in descending order of how much they should bother you.

### It leaks a fabricated error into the public API

`GameApi` holds `Option<Stage>` ([api.rs:411-415](../../engine/src/api.rs#L411-L415)) because a consuming transition has to move the game *out* of a `&mut self`. The `None` window that opens between taking it and putting it back is unreachable in practice — its own comment says so — yet it must be handled, so it became `ApiError::Poisoned`, and from there `"poisoned"` in the `HexChessError` union in [wasm.rs:83](../../engine/src/wasm.rs#L83).

A TypeScript consumer is now told to branch on an error condition that describes an internal Rust ownership workaround. That is the pattern charging rent at the furthest possible point from where it provides value.

### Its guarantee is already re-implemented at runtime

The compile-time promise is "you cannot call `promote` outside the promotion phase." But `GameApi` must accept that call from JavaScript and answer it dynamically, so the same rule is written again as `ApiError::WrongPhase` ([api.rs:155](../../engine/src/api.rs#L155)) and shipped as `"wrong_phase"`. The check exists twice; only the runtime one is load-bearing.

### One caller collects the benefit

| File | References to `Game<T>` / `NextTurn` / `GameError` |
|---|---|
| [lib.rs](../../engine/src/lib.rs) | 32 — the definitions themselves |
| [api.rs](../../engine/src/api.rs) | 8 — erasing them again |
| [bin/main.rs](../../engine/src/bin/main.rs) | 2 |
| `wasm.rs`, `board.rs`, `display.rs`, … | 0 |

Every consumer that matters reaches the engine through `GameApi`.

### Every mutating method pays a take-and-restore tax

`play`, `promote` and `undo` ([api.rs:524-592](../../engine/src/api.rs#L524-L592)) are each a `stage.take()`, a match, and a hand-written restore on *every* error path — plus `take_normal` and `restore` as helpers. `GameError<G>` ([lib.rs:44](../../engine/src/lib.rs#L44)) exists solely to carry the game back out of a failed consuming call, which is why the tests are full of `.map_err(|e| e.error)` (13 occurrences).

Note that `board.rs` already does the honest thing: `execute` / `undo` mutate in place, and `move_creates_check_on_active_king` ([lib.rs](../../engine/src/lib.rs)) already relies on that. The type-state only wraps the outer layer.

### Against all three of your goals

- **Frontend** — sees `GameApi` only. Nothing to lose.
- **RL** — wants make/unmake in a hot loop and `Deserialize` for replay buffers. `Game<T>` derives `Serialize` but *not* `Deserialize`, and it can't: nothing in a JSON payload tells you which `T` to build. See [reinforcement-learning-player.md](reinforcement-learning-player.md).
- **Client/server** — same problem, harder. Any wire protocol forces erasure at the boundary, and erasure is exactly what `Stage` is.

### The honest counter-argument

You lose a compile-time guard against misordered calls *inside the engine*. Mitigation: it is already a runtime error at the only boundary that can violate it, and a test can pin it. Accept the trade.

---

## 2. Do it before the frontend, not after

[frontend/src/](../../frontend/src/) does not import the engine yet — `state.ts` and `Board.vue` are still placeholders. So removing `"poisoned"` from the error union costs nothing today and is a breaking change tomorrow. This refactor is cheapest right now.

---

## 3. The refactor

The invariant: **the tests in `api.rs` are the contract.** They exercise the erased handle, which is precisely the surface that must not change. If they pass untouched, the refactor is correct. Do not edit them.

### Step 1 — collapse `Game<T>` into `Game`

Replace the marker types with a field:

```rust
pub enum Status {
    Normal,
    Promotion,
    Finished { winner: Side },
}

pub struct Game {
    board: Board,
    active_side: Side,
    moves: Vec<GameMove>,
    status: Status,
}
```

Then convert the three transitions to in-place mutation:

```rust
pub fn make_move(&mut self, origin: Position, destination: Position) -> Result<()>
pub fn promote(&mut self, piece_type: PieceType) -> Result<()>
pub fn undo(&mut self) -> Result<()>
```

Each sets `self.status` where it previously returned a `NextTurn` variant. `promote` and `make_move` gain a `Status` guard at the top, returning `UserError::WrongPhase`.

Deletions: `NormalTurn`, `PromotePawn`, `GameOver`, `NextTurn`, `GameError<G>`, `GameResult<G>`, `transition`, `undo_next_turn`. `Result<T>` stays as-is.

### Step 2 — collapse `GameApi`

```rust
pub struct GameApi {
    game: Game,
}
```

- `Stage`, `impl From<NextTurn> for Stage`, `take_normal`, `restore` — deleted.
- `Stage::phase()` becomes a match on `game.status`; `Stage::label()` moves alongside it for the `WrongPhase` message.
- `play` / `promote` / `undo` become: check phase → call the engine → return `self.state()`. No take, no restore; a rejected call cannot corrupt anything because nothing was moved out.
- **Delete `ApiError::Poisoned`** and its `"poisoned"` arm in `code()`.

### Step 3 — the boundary

Remove `"poisoned"` from the `HexChessError` union in [wasm.rs:22-86](../../engine/src/wasm.rs#L22-L86). Nothing else in `wasm.rs` changes — it forwards to `GameApi`, whose signatures are unchanged.

### Step 4 — call sites

`bin/main.rs` (2 sites: the `NextTurn::Continued` destructuring becomes a plain call) and the `lib.rs` test module (the 13 `.map_err(|e| e.error)` sites collapse to `?`).

### Step 5 — verify

`cargo test` green with the `api.rs` tests unmodified, `cargo insta review` showing no snapshot drift, `cargo check -p engine --target wasm32-unknown-unknown`, then `npm run build:engine:dev` and confirm `"poisoned"` is gone from `engine/pkg/engine.d.ts`.

Expect a net loss of roughly 150 lines.

---

## 4. Deliberately not in scope

Two things this refactor makes easier and should *not* be bundled with:

- **`#[derive(Deserialize)]` on `Game`, `Board`, `Status`.** Trivial once the generic is gone, needed for RL replay buffers and any server. Separate commit.
- **`get_movement_options(&mut self)` → `&self`.** It takes `&mut self` only because legality filtering mutates the board temporarily. Blocks parallel position evaluation, so RL will want it — but it is a `board.rs` concern and independent of this work.
