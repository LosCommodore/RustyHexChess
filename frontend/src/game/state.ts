import { computed, reactive, ref } from 'vue';
import { Notify } from 'quasar';
import {
  errorCode,
  errorMessage,
  gameFromPieces,
  legalMoves as engineLegalMoves,
  newGame,
  play as enginePlay,
  promote as enginePromote,
  undo as engineUndo,
  type GameState,
  type PlacedPiece,
} from './engine';

export type PlayerColor = 'white' | 'black';

export type PieceType = 'pawn' | 'rook' | 'knight' | 'bishop' | 'queen' | 'king';

/** Game lifecycle, as far as the engine distinguishes it. */
export type GameStatus = 'active' | 'check' | 'checkmate' | 'stalemate' | 'draw';

/**
 * `game` plays through the engine and enforces the rules. `free` is board
 * setup: pieces go anywhere with no validation, and the position is only handed
 * to the engine when leaving setup.
 */
export type BoardMode = 'game' | 'free';

/** Axial hex coordinate. q selects the visual column, r the row within it. */
export interface HexCoord {
  q: number;
  r: number;
}

export interface Piece extends HexCoord {
  type: PieceType;
  color: PlayerColor;
}

/** One played move, denormalised so the move list renders without a lookup. */
export interface HistoryEntry {
  from: HexCoord;
  to: HexCoord;
  type: PieceType;
  color: PlayerColor;
  /** The side that made the move — drives the `.`/`…` ply suffix. */
  activePlayer: PlayerColor;
  moveNumber: number;
}

/** A hex highlighted as a move option: a dot for a step, a ring for a capture. */
export interface Marker extends HexCoord {
  kind: 'move' | 'capture';
}

// U+FE0E, the text-presentation variation selector. Without it iOS/Safari
// renders these chess symbols as colour emoji, which ignore the CSS `fill`
// (so "white" pieces come out dark) and size themselves differently (too big).
// Appending it forces the monochrome text glyph everywhere the symbols appear.
const TEXT_GLYPH = '︎';

export const PIECE_SYMBOLS: Record<PieceType, string> = {
  pawn: `♟${TEXT_GLYPH}`,
  rook: `♜${TEXT_GLYPH}`,
  knight: `♞${TEXT_GLYPH}`,
  bishop: `♝${TEXT_GLYPH}`,
  queen: `♛${TEXT_GLYPH}`,
  king: `♚${TEXT_GLYPH}`,
};

export const STATUS_LABELS: Record<GameStatus, string> = {
  active: 'In progress',
  check: 'Check',
  checkmate: 'Checkmate',
  stalemate: 'Stalemate',
  draw: 'Draw',
};

export const MODE_LABELS: Record<BoardMode, string> = {
  game: 'Game',
  free: 'Free placement',
};

// --- Notation -----------------------------------------------------------
// A file is a visual column (constant q); a rank is an up-right diagonal
// (constant s = -(q + r)). So rank = 6 - q - r, and the two convert cleanly.
// This is the engine's own square notation ("f5"), so it doubles as the wire
// format — nothing else needs to translate coordinates.

export function hexName({ q, r }: HexCoord): string {
  return `${String.fromCharCode(97 + q + 5)}${6 - q - r}`;
}

export function fromName(name: string): HexCoord {
  const q = name.charCodeAt(0) - 97 - 5;
  return { q, r: 6 - q - Number(name.slice(1)) };
}

// --- Engine <-> UI mapping ----------------------------------------------
// `PlayerColor`/`PieceType` are spelled identically to the engine's `Color`/
// `Kind`, so the only real translation is the square notation above.

function toPieces(placed: PlacedPiece[]): Piece[] {
  return placed.map((piece) => ({
    ...fromName(piece.square),
    type: piece.kind,
    color: piece.color,
  }));
}

function toStatus(state: GameState): GameStatus {
  if (state.phase === 'finished') {
    // The API reports a winner for checkmate and none for a draw; it does not
    // distinguish stalemate from a repetition/50-move draw, so both read "draw".
    return state.winner ? 'checkmate' : 'draw';
  }
  return state.check ? 'check' : 'active';
}

function toHistory(state: GameState): HistoryEntry[] {
  let halfMoves = 0;
  let lastNumber = 1;

  return state.history.map((move) => {
    // A promotion is a second entry on the same move as its pawn push, so it
    // shares that move's number and does not advance the count.
    const isPromotion = move.action.type === 'promote';
    const moveNumber = isPromotion ? lastNumber : Math.floor(halfMoves / 2) + 1;
    if (!isPromotion) {
      halfMoves += 1;
      lastNumber = moveNumber;
    }

    return {
      from: fromName(move.from),
      to: fromName(move.to),
      type: move.kind,
      color: move.color,
      activePlayer: move.color,
      moveNumber,
    };
  });
}

// --- Reactive state -----------------------------------------------------

/**
 * Single source of truth for the UI. In `game` mode it is a projection of the
 * engine's last returned state; in `free` mode it is a scratch buffer the
 * palette edits until the position is committed.
 */
export const game = reactive({
  activePlayer: 'white' as PlayerColor,
  status: 'active' as GameStatus,
  mode: 'game' as BoardMode,
  moveNumber: 1,
  captured: { white: [], black: [] } as Record<PlayerColor, PieceType[]>,
  pieces: [] as Piece[],
  history: [] as HistoryEntry[],
  /** Moves played in the position on screen; equals history.length when live. */
  viewIndex: 0,
});

/** The phase the engine reports, kept separate from the coarser `status`. */
const phase = ref<GameState['phase']>('normal');

// The board after each ply, indexed by move count. Browsing reads from here
// rather than replaying moves, so captures and promotions rewind correctly.
let positions: Piece[][] = [];

/** Projects an engine snapshot onto the reactive state and records it. */
function apply(state: GameState) {
  const pieces = toPieces(state.pieces);
  const played = state.history.length;

  // Keep [0 .. played-1] (already recorded), drop any now-diverged future, and
  // store this position at its move count.
  positions = positions.slice(0, played);
  positions[played] = pieces;

  game.mode = 'game';
  game.pieces = pieces;
  game.activePlayer = state.active;
  game.status = toStatus(state);
  game.moveNumber = state.moveNumber;
  game.captured = {
    white: [...state.captured.white],
    black: [...state.captured.black],
  };
  game.history = toHistory(state);
  game.viewIndex = played;
  phase.value = state.phase;
}

function notifyError(error: unknown) {
  Notify.create({ type: 'negative', message: errorMessage(error), position: 'top' });
}

/** Boot entry point: start the standard game once the WASM module is ready. */
export function startGame() {
  apply(newGame());
}

export const canUndo = computed(() => game.mode === 'game' && game.history.length > 0);

/** True while looking at a past position rather than the live one. */
export const isBrowsing = computed(
  () => game.mode === 'game' && game.viewIndex < game.history.length,
);

export const isPromoting = computed(() => phase.value === 'promotion');

/** The position on screen: a recorded snapshot while browsing, else live. */
export const viewedPieces = computed<Piece[]>(() => {
  if (game.mode === 'free') return game.pieces;
  return positions[game.viewIndex] ?? game.pieces;
});

/** Shows the position after `index` moves; 0 is the starting position. */
export function viewMove(index: number) {
  game.viewIndex = Math.max(0, Math.min(index, game.history.length));
}

export function viewLive() {
  game.viewIndex = game.history.length;
}

// --- Move options -------------------------------------------------------

/**
 * Where the piece on `hex` may legally go, as board markers. A capture rings
 * the square the taken piece stands on, which for en passant is not the
 * destination; a step dots its destination.
 */
export function markersFor(hex: HexCoord | null): Marker[] {
  if (!hex || game.mode !== 'game' || isBrowsing.value) return [];

  return engineLegalMoves(hexName(hex)).flatMap((move) => {
    if (move.action.type !== 'capture') {
      return [{ ...fromName(move.to), kind: 'move' as const }];
    }

    // Ring the square the taken piece stands on. For a normal capture that is
    // the destination, so one ring is enough. For en passant the taken pawn is
    // elsewhere, so also dot the (empty) square the pawn actually moves to —
    // otherwise the destination shows no marker at all.
    const captured: Marker = { ...fromName(move.action.square), kind: 'capture' };
    if (move.action.square === move.to) return [captured];
    return [captured, { ...fromName(move.to), kind: 'move' as const }];
  });
}

// --- Playing ------------------------------------------------------------

function indexAt(hex: HexCoord): number {
  return game.pieces.findIndex((piece) => piece.q === hex.q && piece.r === hex.r);
}

/**
 * Moves the piece at `index` to `to`. In game mode this plays through the
 * engine, which validates; a rejected move leaves the board untouched (the
 * caller snaps the drag back). In free mode it just relocates the piece.
 */
export function movePiece(index: number, to: HexCoord) {
  if (isBrowsing.value) return;

  const piece = game.pieces[index];
  if (!piece) return;
  if (piece.q === to.q && piece.r === to.r) return;

  if (game.mode === 'free') {
    // Hold the reference before splicing so a shifted index cannot bite us.
    const occupant = game.pieces.findIndex(
      (p, i) => i !== index && p.q === to.q && p.r === to.r,
    );
    if (occupant >= 0) game.pieces.splice(occupant, 1);
    piece.q = to.q;
    piece.r = to.r;
    return;
  }

  try {
    apply(enginePlay(hexName(piece), hexName(to)));
  } catch (error) {
    // An illegal drop is ordinary interaction, not a failure worth a toast; the
    // board re-renders from the unchanged state, snapping the piece back.
    const code = errorCode(error);
    if (code !== 'illegal_move' && code !== 'wrong_player') notifyError(error);
  }
}

export function undo() {
  if (game.mode !== 'game' || game.history.length === 0) return;
  try {
    apply(engineUndo());
  } catch (error) {
    notifyError(error);
  }
}

/** Replaces the pawn waiting on the far rank. */
export function choosePromotion(type: PieceType) {
  try {
    apply(enginePromote(type));
  } catch (error) {
    notifyError(error);
  }
}

// --- Modes and free placement -------------------------------------------

/** What a click on the board does while setting a position up. */
export type PlacementTool =
  | { kind: 'piece'; type: PieceType; color: PlayerColor }
  | { kind: 'erase' }
  | null;

export const placementTool = ref<PlacementTool>(null);

/** Enters free-placement mode, seeding the buffer with `pieces`. */
function enterFree(pieces: Piece[]) {
  game.mode = 'free';
  game.pieces = pieces.map((piece) => ({ ...piece }));
  game.history = [];
  game.viewIndex = 0;
  game.moveNumber = 1;
  game.activePlayer = 'white';
  game.status = 'active';
  phase.value = 'normal';
}

/** Commits the free-placement buffer to a new game. Reports why if refused. */
function commitFreePosition() {
  const pieces: PlacedPiece[] = game.pieces.map((piece) => ({
    square: hexName(piece),
    kind: piece.type,
    color: piece.color,
  }));

  try {
    apply(gameFromPieces(pieces, game.activePlayer));
    placementTool.value = null;
  } catch (error) {
    // Stay in free mode so the user can fix the position (add a king, remove a
    // duplicate) rather than losing their setup.
    notifyError(error);
  }
}

export function setMode(mode: BoardMode) {
  if (mode === game.mode) return;
  if (mode === 'free') {
    enterFree(viewedPieces.value);
  } else {
    commitFreePosition();
  }
}

export function selectTool(tool: PlacementTool) {
  // Placing pieces is setup: switch into free mode, seeding from what is shown.
  if (tool && game.mode !== 'free') enterFree(viewedPieces.value);
  placementTool.value = tool;
}

/** Applies the selected tool to a hex: places a piece, or clears the hex. */
export function applyTool(hex: HexCoord) {
  const tool = placementTool.value;
  if (!tool) return;
  if (game.mode !== 'free') enterFree(viewedPieces.value);

  const existing = indexAt(hex);
  if (existing >= 0) game.pieces.splice(existing, 1);

  if (tool.kind === 'piece') {
    game.pieces.push({ q: hex.q, r: hex.r, type: tool.type, color: tool.color });
  }
}

export function clearBoard() {
  if (game.mode !== 'free') enterFree([]);
  else game.pieces = [];
}

/** Loads the standard starting layout as an editable buffer. */
export function resetPosition() {
  enterFree(toPieces(newGame().pieces));
}
