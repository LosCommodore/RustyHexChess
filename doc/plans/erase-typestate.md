# Erasing the Type-State Pattern

**Status: done.** `cargo test` 35/35, no snapshot drift, `engine/pkg/engine.d.ts` regenerated.
Rationale for the removal is in [type_state_pattern.md](../type_state_pattern.md); this file records the result and what it opened up.

---

## What each layer looks like now

| File | |
|---|---|
| [lib.rs](../../engine/src/lib.rs) | One `Game` with a `state: GameState` field. Transitions take `&mut self` and return `Result<()>`. |
| [api.rs](../../engine/src/api.rs) | `GameApi { game: Game }`. Commands are *check phase → call the engine → return `self.state()`*. Its 16 tests never changed. |
| [wasm.rs](../../engine/src/wasm.rs) | Unchanged except that `"poisoned"` is gone from the `HexChessError` union. |
| [bin/main.rs](../../engine/src/bin/main.rs) | Plain calls, no `NextTurn` destructuring. |

```rust
pub enum GameState {
    Normal,
    Promotion,
    GameOver { winner: Option<Side> },   // None = remis
}

pub fn new(board: Option<Board>) -> Self
pub fn state(&self) -> GameState
pub fn winner(&self) -> Result<Option<Side>>   // Err while the game is still running
pub fn make_move(&mut self, origin: Position, destination: Position) -> Result<()>
pub fn make_human_move(&mut self, origin: HumanNotation, destination: HumanNotation) -> Result<()>
pub fn promote(&mut self, piece_type: PieceType) -> Result<()>
pub fn undo(&mut self) -> Result<()>
pub fn with_active_side(&mut self, side: Side)
```

Gone: `Game<T>`, `NormalTurn`, `PromotePawn`, `GameOver`, `NextTurn`, `GameError<G>`, `GameResult<G>`, `transition`, `undo_next_turn`, the free `new_game` — and in `api.rs` the whole erasure layer: `Stage`, `Option<Stage>`, `take_normal`, `restore`, `ApiError::Poisoned`. Net ~190 lines.

`api.rs` kept everything that is translation rather than erasure: the wire types, `parse_square` / `square_name` (a name can be well-formed yet off a hexagon — `a1`), the stable `code()` strings, `require_kings`, and the notation. Moving any of it into `wasm.rs` would put it behind `cfg(target_family = "wasm")`, where it cannot be tested natively.

## Invariants worth not breaking again

Both are pinned by tests, and both were implicit in *which `impl` block a method lived in* before:

- **The mover stays on turn through a promotion.** `make_move` returns early when a pawn reaches the far rank; `promote` calls `next_turn`. The other order hands the new queen to the opponent.
- **`undo` reads its answer off the popped move** — `active_side = mv.piece.side`, `state` from `mv.action`. This is what returns a finished game to playable and a promotion to `Promotion`.

## Verifying a change to this surface

`cargo test -p engine` — the `api.rs` tests are the contract; they exercise the erased handle, which is the surface that must not change.
`cargo check -p engine --lib --target wasm32-unknown-unknown` — `--lib` matters: `bin/main.rs` draws with crossterm and has never built for wasm.
`npm run build:engine:dev`, then read `engine/pkg/engine.d.ts`.

---

## What this opened up

- **Remis.** `GameOver { winner: None }` is representable but unreachable: `check_king` returns `Nothing` when the king is not in check, so stalemate is never detected. `Game::winner` and the API's `winner` field already carry it through.
- **`#[derive(Deserialize)]` on `Game`, `Board`, `GameState`.** Trivial now that the generic is gone, and the reason the pattern had to go — nothing in a JSON payload tells you which `T` to build. Needed for RL replay buffers and any server; see [reinforcement-learning-player.md](reinforcement-learning-player.md).
- **`get_movement_options(&mut self)` → `&self`.** It takes `&mut self` only because legality filtering mutates the board temporarily. Blocks parallel position evaluation, so RL will want it; a `board.rs` concern.
