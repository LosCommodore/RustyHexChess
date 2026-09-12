// The single seam between the UI and the Rust engine. Everything below imports
// the generated WASM package; nothing else in the app touches it directly.
//
// The module owns one `HexChess` handle. Every command returns the full
// `GameState`, so callers keep no state of their own — `state.ts` maps each
// snapshot onto the reactive object the components render from.

import init, {
  HexChess,
  type Color,
  type GameState,
  type Kind,
  type LegalMove,
  type PlacedPiece,
} from '../../../engine/pkg';

export type { Color, GameState, Kind, LegalMove, PlacedPiece };

let ready: Promise<void> | null = null;

/** Loads the WASM module. Idempotent: the fetch-and-instantiate happens once. */
export function initEngine(): Promise<void> {
  if (!ready) {
    ready = init().then(() => undefined);
  }
  return ready;
}

let handle: HexChess | null = null;

function active(): HexChess {
  if (!handle) {
    throw new Error('engine called before initEngine() resolved');
  }
  return handle;
}

/** Replaces the current game, freeing the old handle's WASM memory. */
function swap(next: HexChess): GameState {
  handle?.free();
  handle = next;
  return handle.state();
}

/** A fresh game from the standard starting position. */
export function newGame(): GameState {
  return swap(new HexChess());
}

/**
 * A game from a hand-built position. Throws (`code: "missing_king"` /
 * `"duplicate_square"`) if the position is not playable, leaving the current
 * game untouched — the new handle is only adopted once it is built.
 */
export function gameFromPieces(pieces: PlacedPiece[], color: Color): GameState {
  return swap(HexChess.fromPieces(pieces, color));
}

export function play(from: string, to: string): GameState {
  return active().play(from, to);
}

export function promote(kind: Kind): GameState {
  return active().promote(kind);
}

export function undo(): GameState {
  return active().undo();
}

export function reset(): GameState {
  return active().reset();
}

/** Where the piece on `square` may go. Empty for anything unmovable — safe on any click. */
export function legalMoves(square: string): LegalMove[] {
  return active().legalMoves(square);
}

/** The stable `code` of a thrown engine error, or `null` for anything else. */
export function errorCode(error: unknown): string | null {
  if (error && typeof error === 'object' && 'code' in error) {
    const code = (error as { code: unknown }).code;
    return typeof code === 'string' ? code : null;
  }
  return null;
}

/** A human-readable message for a thrown value, engine error or not. */
export function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
