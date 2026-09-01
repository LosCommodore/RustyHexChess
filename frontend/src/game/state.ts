import { computed, reactive } from 'vue';

export type PlayerColor = 'white' | 'black';

export type PieceType = 'pawn' | 'rook' | 'knight' | 'bishop' | 'queen' | 'king';

/** Game lifecycle, mirroring what the engine will eventually report. */
export type GameStatus = 'active' | 'check' | 'checkmate' | 'stalemate' | 'draw';

/**
 * `game` enforces turn order and counts moves. `free` is board setup: pieces
 * go anywhere, and neither the active player nor the move number changes.
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

/** Enough to put a move back exactly as it was, including the turn it ended. */
export interface HistoryEntry {
  pieceIndex: number;
  from: HexCoord;
  to: HexCoord;
  /** Denormalised so the move list renders without looking the piece up. */
  type: PieceType;
  color: PlayerColor;
  activePlayer: PlayerColor;
  moveNumber: number;
}

export const PIECE_SYMBOLS: Record<PieceType, string> = {
  pawn: '♟',
  rook: '♜',
  knight: '♞',
  bishop: '♝',
  queen: '♛',
  king: '♚',
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

export function hexName({ q, r }: HexCoord): string {
  return `${String.fromCharCode(97 + q + 5)}${6 - q - r}`;
}

export function fromName(name: string): HexCoord {
  const q = name.charCodeAt(0) - 97 - 5;
  return { q, r: 6 - q - Number(name.slice(1)) };
}

function at(name: string, type: PieceType, color: PlayerColor): Piece {
  return { ...fromName(name), type, color };
}

const INITIAL_PIECES: Piece[] = [
  // White pawns, forming the V in front of the back diagonal
  at('b5', 'pawn', 'white'),
  at('c5', 'pawn', 'white'),
  at('d5', 'pawn', 'white'),
  at('e5', 'pawn', 'white'),
  at('f5', 'pawn', 'white'),
  at('g4', 'pawn', 'white'),
  at('h3', 'pawn', 'white'),
  at('i2', 'pawn', 'white'),
  at('j1', 'pawn', 'white'),
  // White pieces
  at('k1', 'rook', 'white'),
  at('h1', 'rook', 'white'),
  at('j2', 'knight', 'white'),
  at('i1', 'knight', 'white'),
  at('f1', 'bishop', 'white'),
  at('f2', 'bishop', 'white'),
  at('f3', 'bishop', 'white'),
  at('e2', 'queen', 'white'),
  at('g1', 'king', 'white'),

  // Black pawns
  at('b11', 'pawn', 'black'),
  at('c10', 'pawn', 'black'),
  at('d9', 'pawn', 'black'),
  at('e8', 'pawn', 'black'),
  at('f7', 'pawn', 'black'),
  at('g7', 'pawn', 'black'),
  at('h7', 'pawn', 'black'),
  at('i7', 'pawn', 'black'),
  at('j7', 'pawn', 'black'),
  // Black pieces
  at('a11', 'rook', 'black'),
  at('d11', 'rook', 'black'),
  at('b10', 'knight', 'black'),
  at('c11', 'knight', 'black'),
  at('f11', 'bishop', 'black'),
  at('f10', 'bishop', 'black'),
  at('f9', 'bishop', 'black'),
  at('g8', 'queen', 'black'),
  at('e11', 'king', 'black'),
];

/**
 * Single source of truth for the game. `Board.vue` owns only geometry and
 * reads pieces from here, so the controls can act on the same data.
 */
export const game = reactive({
  activePlayer: 'white' as PlayerColor,
  status: 'active' as GameStatus,
  mode: 'game' as BoardMode,
  /** Full move number as shown in chess notation, starting at 1. */
  moveNumber: 1,
  /** Pieces taken, keyed by the player who captured them. */
  captured: { white: [], black: [] } as Record<PlayerColor, PieceType[]>,
  pieces: INITIAL_PIECES.map(piece => ({ ...piece })),
  history: [] as HistoryEntry[],
  /**
   * Which point in the history is on screen: the number of moves played in
   * the viewed position. Equal to history.length means "live".
   */
  viewIndex: 0,
});

export const canUndo = computed(() => game.history.length > 0);

/** True while looking at a past position rather than the live one. */
export const isBrowsing = computed(() => game.viewIndex < game.history.length);

/**
 * The position on screen. Browsing never mutates the game: it rewinds a copy
 * by replaying the recorded origins backwards from the live position.
 */
export const viewedPieces = computed<Piece[]>(() => {
  const snapshot = game.pieces.map(piece => ({ ...piece }));
  for (let i = game.history.length - 1; i >= game.viewIndex; i--) {
    const entry = game.history[i];
    const piece = entry && snapshot[entry.pieceIndex];
    if (entry && piece) {
      piece.q = entry.from.q;
      piece.r = entry.from.r;
    }
  }
  return snapshot;
});

/** Shows the position after `index` moves; 0 is the starting position. */
export function viewMove(index: number) {
  game.viewIndex = Math.max(0, Math.min(index, game.history.length));
}

export function viewLive() {
  game.viewIndex = game.history.length;
}

/** Moves a piece and records what it takes to put it back. */
export function movePiece(index: number, to: HexCoord) {
  // Browsing is read-only: rewind to the live position before playing on.
  if (isBrowsing.value) return;

  const piece = game.pieces[index];
  if (!piece) return;
  if (piece.q === to.q && piece.r === to.r) return;

  game.history.push({
    pieceIndex: index,
    from: { q: piece.q, r: piece.r },
    to: { q: to.q, r: to.r },
    type: piece.type,
    color: piece.color,
    activePlayer: game.activePlayer,
    moveNumber: game.moveNumber,
  });

  piece.q = to.q;
  piece.r = to.r;
  game.viewIndex = game.history.length;

  // Setting up a position should not consume turns.
  if (game.mode === 'game') {
    if (game.activePlayer === 'black') game.moveNumber += 1;
    game.activePlayer = game.activePlayer === 'white' ? 'black' : 'white';
  }
}

/** Reverts the last move, restoring the turn it was made on. */
export function undo() {
  const last = game.history.pop();
  if (!last) return;

  const piece = game.pieces[last.pieceIndex];
  if (piece) {
    piece.q = last.from.q;
    piece.r = last.from.r;
  }
  game.activePlayer = last.activePlayer;
  game.moveNumber = last.moveNumber;
  game.viewIndex = game.history.length;
}

export function setMode(mode: BoardMode) {
  game.mode = mode;
}

// --- Demo data ----------------------------------------------------------
// Prototype scaffolding so the panels have something to show. These moves are
// not legal chess, only plausible-looking. Delete the seedDemoGame() call to
// start from the initial position with an empty history.

const DEMO_MOVES: [from: string, to: string][] = [
  ['f5', 'f6'],
  ['c10', 'c9'],
  ['e5', 'e6'],
  ['d9', 'd8'],
  ['g4', 'g5'],
  ['e8', 'e7'],
  ['h1', 'h2'],
  ['g7', 'g6'],
  ['i1', 'h4'],
  ['f9', 'f8'],
  ['e2', 'd4'],
  ['b10', 'c8'],
];

function seedDemoGame() {
  for (const [from, to] of DEMO_MOVES) {
    const origin = fromName(from);
    const index = game.pieces.findIndex(p => p.q === origin.q && p.r === origin.r);
    if (index >= 0) movePiece(index, fromName(to));
  }

  game.captured.white.push('pawn', 'knight');
  game.captured.black.push('pawn');
  game.status = 'check';
}

seedDemoGame();
