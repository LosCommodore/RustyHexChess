# RustyHexChess: WASM Frontend Integration Plan

## Status

The frontend prototype exists and runs: a Quasar/Vue 3 SPA with an SVG hex board,
drag & drop, move history browsing, free placement, and side panels. The engine
now has a full WASM interface. What is missing is the wiring between them —
`frontend/src/game/state.ts` is still a self-contained reactive mock with no rules.

**Done** (frontend, see `frontend/src/`):

- `components/Board.vue` — SVG hex grid, flat-top hexes, axial `{q, r}` coords,
  drag & drop to any hex, click selection, file/rank labels, move markers
- `game/state.ts` — reactive `game` object: pieces, active player, move number,
  history, `viewIndex` browsing, undo, free-placement tools, `hexName`/`fromName`
- `components/GameControls.vue`, `GameInfo.vue`, `MoveHistory.vue`,
  `PiecePalette.vue`, `pages/IndexPage.vue` layout

**Done** (engine):

- `engine/src/api.rs` — the abstraction layer: `GameApi` plus the wire types.
  Plain Rust, no WASM, 14 tests
- `engine/src/wasm.rs` — the `HexChess` binding over it, wasm32-only
- `wasm-pack build --target web --release` produces `engine/pkg/` with hand-written
  TypeScript for every payload

**Not done**:

- Any engine call from the frontend; all rules are absent (any piece to any hex)
- Captures, promotion dialog, check/checkmate status (all faked or hardcoded)

---

## The interface (built)

Two layers, because the useful one should not need a browser to run:

- **`engine/src/api.rs`** — `GameApi`, a handle that owns the game and applies
  commands to it. Plain Rust, tested natively. It exists because the engine is a
  type-state machine whose transitions consume `self`, which cannot be held
  across a foreign function boundary; `GameApi` keeps the three states in one
  `Option<Stage>` and `take()`s it to transition. Its wire types are its own
  (`Color`, `Kind`, `PlacedPiece`, `GameState`, …), not the engine's, so the
  engine stays free to change without breaking anything that speaks the protocol.
- **`engine/src/wasm.rs`** — `HexChess`, the `#[wasm_bindgen]` shim. wasm32-only,
  and thin enough that nothing in it needs browser testing: it parses arguments,
  calls one `GameApi` method, and converts the result.

```ts
const game = new HexChess();          // or HexChess.fromPieces(pieces, "white")
game.state()                          // GameState
game.legalMoves("f5")                 // LegalMove[] — empty, never throws, on a dead square
game.play("f5", "f6")                 // GameState; throws on an illegal move
game.promote("queen")                 // GameState; only in the "promotion" phase
game.undo()                           // GameState; also undoes the mating move
game.reset()                          // GameState
```

Properties worth knowing when wiring the frontend:

- **Squares are notation strings** (`"f5"`), the one form both sides already
  speak. The engine's `(y, x)` and the frontend's `{q, r}` never cross.
- **Every command returns the whole state**, so the UI can render from the
  return value and keep nothing of its own.
- **A rejected command changes nothing.** `GameError<G>` hands the game back and
  the layer puts it away again, so a failed `play` leaves a playable game.
- **Errors are real JS `Error`s with a stable `code`** (`illegal_move`,
  `wrong_player`, `wrong_phase`, `missing_king`, …). Branch on `err.code`.
- **Payloads are real objects**, parsed on the Rust side, matching the
  TypeScript in `engine/pkg/engine.d.ts`.
- **`fromPieces` requires both kings.** The engine's check detection panics
  without them, so the layer refuses such a position instead.
- **Duplicate destinations are collapsed** — see the engine bugs below.

Build with `npm run build:engine` (needs `wasm-pack`, `cargo install wasm-pack`).

---

## Remaining Work

### Step 2: `useGameEngine.ts` composable

- `import init, { HexChess } from '../../../engine/pkg'` (add a Vite alias; the
  frontend build currently has no reference to `engine/pkg`)
- Async `init()` once — `frontend/src/boot/` is empty apart from `.gitkeep`; add a
  Quasar boot file so the module is ready before the board mounts
- Hold the `HexChess` handle in module scope and map each `GameState` onto the
  shape `state.ts` already exposes, so components keep their current props.
  Squares convert with the existing `hexName`/`fromName`

### Step 3: Replace the mock in `state.ts`

`state.ts` stays the single source of truth for the UI; its mutators become engine
calls. Specific things that must change:

- **Delete `seedDemoGame()` and `DEMO_MOVES`** — the prototype's fake game, plus
  the hardcoded `game.captured` pushes and `game.status = 'check'`
- **`movePiece`** — call `play`, replace the local piece array from the returned
  state, and catch the throw on an illegal move (Board.vue snaps the drag back).
  The engine has already validated, so no frontend rule checks
- **Remove the hardcoded `markers` ref in `Board.vue`** — feed it from
  `legalMoves` for the selected hex; the marker rendering (dot/ring) already
  works, and a `capture` action names the square to ring, which for en passant
  is not the destination
- **`undo`** — call the engine's undo. The current implementation only restores the
  moved piece's origin, which cannot restore a captured piece
- **`viewedPieces` / `viewIndex`** — history browsing replays `from` coordinates
  backwards and silently breaks with captures and promotions. Keep the `pieces`
  array of each returned state instead of reconstructing (cheap: ~40 pieces)
- **`status`** — from `phase`, `check` and `winner`, not a local field
- **`captured`** — the state carries it, computed from the real move history

### Step 4: Promotion

The game is blocked until `promote` is called. Add `components/PromotionDialog.vue`
(QDialog, four QBtn options) opened by `phase === "promotion"` on the returned
state, not by frontend-side rank checks. Note `active` still names the promoting
player while the dialog is up.

### Step 5: Free placement

`PiecePalette.vue` and `applyTool`/`clearBoard` build arbitrary positions; hand
them to `HexChess.fromPieces(pieces, active)` when leaving setup mode. It refuses
a position with a king missing (`code: "missing_king"`) or two pieces on one
square, so the palette needs to surface that rather than assume success.

---

## Engine bugs found while building the interface

Not fixed — these are move-generation rules, outside the boundary layer:

1. **A pawn can capture straight forward.** [`board.rs:147`] passes
   `Capability::Both` for the pawn's forward step, so `f7xf6` is accepted. The
   step should be `Capability::Move`; only the two diagonals capture.
2. **The double step re-emits the single step**, so a pawn on its starting square
   offers the same destination twice ([`board.rs:153`], the walk starts at the
   origin). `api.rs` collapses duplicates for the UI, but move counts elsewhere
   (perft, an RL policy's action space) would still double-count.
3. **`king_in_check(side)` ignores its argument** when locating the king
   ([`lib.rs:150`] filters on `self.active_side`), so it is only correct for the
   side to move. Every current caller passes the active side; the API layer does
   too, deliberately.
4. **`Position::from_human` does not check board bounds** — the board is a
   hexagon, so `a1` parses into an off-board position. `api::parse_square`
   re-checks through `Position::new`; direct engine callers do not.

## Notes

- Rendering is **SVG**, not the Canvas 2D originally planned — geometry lives in
  `Board.vue` (`HEX_RADIUS`, `hexToPixel`, `pixelToHex`) and needs no change here
- Stalemate is not modelled: `check_king` reports `Mate` only from check, so a
  player with no legal moves and no check is simply stuck
- `crossterm` cannot build for wasm, so `display` and the terminal dependencies
  are gated to non-wasm targets
- No networking; WASM runs entirely in the browser
- `npm run dev` from the root rebuilds WASM, then starts the Quasar dev server
- Check the Network tab for `engine_bg.wasm` and the console for init errors
