<template>
  <div class="board-container">
    <svg :width="SVG_WIDTH" :height="SVG_HEIGHT" class="board-svg" :viewBox="`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`">
      <!-- Hexagons -->
      <g class="hexagons">
        <polygon
          v-for="hex in hexagons"
          :key="`hex-${hex.q}-${hex.r}`"
          :points="getHexagonPoints(hex)"
          :class="['hexagon', getHexClass(hex)]"
          @click="selectHex(hex)"
        />
      </g>

      <!-- Pieces -->
      <g class="pieces">
        <text
          v-for="(piece, index) in pieces"
          :key="`piece-${index}`"
          :x="piece.pixelX"
          :y="piece.pixelY"
          class="piece"
          :class="piece.color"
          :style="{ cursor: 'grab' }"
          @mousedown="startDrag($event, index)"
          @click.stop="selectPiece(piece.x, piece.y)"
        >
          {{ pieceSymbols[piece.type] }}
        </text>
      </g>

      <!-- Selection highlight -->
      <polygon
        v-if="selectedHex && hexagons.find(h => h.q === selectedHex!.q && h.r === selectedHex!.r)"
        :points="getHexagonPoints(hexagons.find(h => h.q === selectedHex!.q && h.r === selectedHex!.r)!)"
        class="selected-hex"
      />

      <!-- Column labels (a-k) -->
      <g class="labels column-labels">
        <text
          v-for="label in columnLabels"
          :key="`col-${label.text}`"
          :x="label.x"
          :y="label.y"
          class="label"
        >
          {{ label.text }}
        </text>
      </g>

      <!-- Rank labels (1-11) -->
      <g class="labels rank-labels">
        <g v-for="label in rankLabels" :key="`rank-${label.text}`">
          <!-- ↙ is drawn at 45°; a rank actually runs 30° below horizontal -->
          <text
            :x="label.x - 9"
            :y="label.y"
            :transform="`rotate(15, ${label.x - 9}, ${label.y - 5})`"
            class="label rank-arrow"
          >↙</text>
          <text :x="label.x + 8" :y="label.y" class="label">{{ label.text }}</text>
        </g>
      </g>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

const HEX_RADIUS = 40;
const SVG_WIDTH = 800;
const SVG_HEIGHT = 900;
const CENTER_X = 400;
const CENTER_Y = 450;

interface HexCoord {
  q: number;
  r: number;
}

interface Piece {
  x: number;
  y: number;
  type: string;
  color: string;
  pixelX: number;
  pixelY: number;
}

const selectedHex = ref<HexCoord | null>(null);
let draggedPieceIndex: number | null = null;
let dragOffsetX = 0;
let dragOffsetY = 0;

const pieceSymbols: Record<string, string> = {
  pawn: '♟',
  rook: '♜',
  knight: '♞',
  bishop: '♝',
  queen: '♛',
  king: '♚',
};

// Generate hexagonal chess board shape (hexagon of hexagons)
const hexagons = computed(() => {
  const hexes: HexCoord[] = [];
  for (let q = -5; q <= 5; q++) {
    for (let r = -5; r <= 5; r++) {
      if (Math.abs(q) <= 5 && Math.abs(r) <= 5 && Math.abs(q + r) <= 5) {
        hexes.push({ q, r });
      }
    }
  }
  return hexes;
});

// Helper to create piece with pixel coordinates
function createPiece(x: number, y: number, type: string, color: string): Piece {
  const pixel = getHexCenterPixel(x, y);
  return { x, y, type, color, pixelX: pixel.x, pixelY: pixel.y };
}

// Piece starting positions (pawns only - should form V-shape)
const pieces = ref<Piece[]>([
  // White pawns
  createPiece(1, 4, 'pawn', 'white'),
  createPiece(2, 4, 'pawn', 'white'),
  createPiece(3, 4, 'pawn', 'white'),
  createPiece(4, 4, 'pawn', 'white'),
  createPiece(5, 4, 'pawn', 'white'),
  createPiece(6, 3, 'pawn', 'white'),
  createPiece(7, 2, 'pawn', 'white'),
  createPiece(8, 1, 'pawn', 'white'),
  createPiece(9, 0, 'pawn', 'white'),

  // Black pawns
  createPiece(1, 10, 'pawn', 'black'),
  createPiece(2, 9, 'pawn', 'black'),
  createPiece(3, 8, 'pawn', 'black'),
  createPiece(4, 7, 'pawn', 'black'),
  createPiece(5, 6, 'pawn', 'black'),
  createPiece(6, 6, 'pawn', 'black'),
  createPiece(7, 6, 'pawn', 'black'),
  createPiece(8, 6, 'pawn', 'black'),
  createPiece(9, 6, 'pawn', 'black'),
]);

function engineToHex(engineX: number, engineY: number): HexCoord {
  const q = engineY - 5;
  const r = engineX - 5;
  return { q, r };
}

function hexToPixel(q: number, r: number) {
  const x = HEX_RADIUS * (3 / 2 * q);
  const y = HEX_RADIUS * (Math.sqrt(3) / 2 * q + Math.sqrt(3) * r);
  return { x: CENTER_X + x, y: CENTER_Y + y };
}

function getHexCenterPixel(engineX: number, engineY: number) {
  const hex = engineToHex(engineX, engineY);
  return hexToPixel(hex.q, hex.r);
}

function getHexagonPoints(hex: HexCoord): string {
  const { x, y } = hexToPixel(hex.q, hex.r);
  const points = [];
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 3) * i;
    const px = x + HEX_RADIUS * Math.cos(angle);
    const py = y + HEX_RADIUS * Math.sin(angle);
    points.push(`${px},${py}`);
  }
  return points.join(' ');
}

function getHexClass(hex: HexCoord): string {
  const colorIndex = ((hex.q + 2 * hex.r) % 3 + 3) % 3;
  const colors = ['color1', 'color2', 'color3'] as const;
  return colors[colorIndex]!;
}

function selectHex(hex: HexCoord) {
  if (selectedHex.value?.q === hex.q && selectedHex.value?.r === hex.r) {
    selectedHex.value = null;
  } else {
    selectedHex.value = { q: hex.q, r: hex.r };
  }
}

function selectPiece(engineX: number, engineY: number) {
  const hex = engineToHex(engineX, engineY);
  selectHex(hex);
}

function pixelToHex(pixelX: number, pixelY: number): { q: number; r: number } {
  const x = pixelX - CENTER_X;
  const y = pixelY - CENTER_Y;

  const q = (2 / 3) * x / HEX_RADIUS;
  const r = (-1 / 3) * x / HEX_RADIUS + (Math.sqrt(3) / 3) * y / HEX_RADIUS;

  return { q: Math.round(q), r: Math.round(r) };
}

function startDrag(event: MouseEvent, index: number) {
  draggedPieceIndex = index;
  const piece = pieces.value[index];
  if (!piece) return;
  dragOffsetX = event.clientX - piece.pixelX;
  dragOffsetY = event.clientY - piece.pixelY;

  document.addEventListener('mousemove', handleDrag);
  document.addEventListener('mouseup', endDrag);
  (event.target as SVGTextElement).style.cursor = 'grabbing';
}

function handleDrag(event: MouseEvent) {
  if (draggedPieceIndex === null) return;

  const piece = pieces.value[draggedPieceIndex];
  if (!piece) return;
  piece.pixelX = event.clientX - dragOffsetX;
  piece.pixelY = event.clientY - dragOffsetY;
}

function endDrag() {
  if (draggedPieceIndex === null) return;

  const piece = pieces.value[draggedPieceIndex];
  if (!piece) return;
  const hex = pixelToHex(piece.pixelX, piece.pixelY);

  const validHex = hexagons.value.find(h => h.q === hex.q && h.r === hex.r);
  if (validHex) {
    piece.x = validHex.r + 5;
    piece.y = validHex.q + 5;
    const newPixel = getHexCenterPixel(piece.x, piece.y);
    piece.pixelX = newPixel.x;
    piece.pixelY = newPixel.y;
  }

  draggedPieceIndex = null;
  document.removeEventListener('mousemove', handleDrag);
  document.removeEventListener('mouseup', endDrag);
}

const LABEL_GAP = HEX_RADIUS + 22;
// Hex height is √3·R, so a quarter of it is half the y-step between columns.
const QUARTER_HEX = (Math.sqrt(3) * HEX_RADIUS) / 4;
// Pulls the rank labels back toward the board; the ↙ glyph widens them.
const RANK_SHIFT_X = -10;

// A visual column is all hexes sharing the same q — pixel x depends only on q.
// Each label sits just under that column's lowest hexagon, so the row of
// labels follows the board's V-shaped bottom edge.
const columnLabels = computed(() =>
  Array.from({ length: 11 }, (_, i) => {
    const q = i - 5;
    const rMax = Math.max(...hexagons.value.filter(h => h.q === q).map(h => h.r));
    const bottom = hexToPixel(q, rMax);
    return {
      text: String.fromCharCode(97 + i),
      x: bottom.x,
      y: bottom.y + LABEL_GAP,
    };
  })
);

// A rank is the up-right diagonal: all hexes sharing s = -(q + r). Each label
// sits outside that rank's rightmost hex. Ranks 1-6 hang off the right column
// (which is 6 hexes tall, so they share an x); ranks 7-11 anchor on the
// upper-right edge, so their x steps left as they climb.
const rankLabels = computed(() =>
  Array.from({ length: 11 }, (_, i) => {
    const s = i - 5;
    const rank = hexagons.value.filter(h => -(h.q + h.r) === s);
    const anchor = rank.reduce((a, h) => (h.q > a.q ? h : a));
    const p = hexToPixel(anchor.q, anchor.r);
    const [dx, dy] = s <= 0 ? [1, 0] : [0.5, -Math.sqrt(3) / 2];
    // Ranks hanging off the right column rise two quarter-hexes so each number
    // lands on its own ↙ diagonal instead of level with the hex centre.
    const rise = s <= 0 ? 2 * QUARTER_HEX : 0;
    // Ranks 7-11 anchor on the upper-right edge with a different offset
    // direction, which overshoots by half a row. Step them back half a
    // hexagon along that edge so the run stays contiguous with rank 6.
    const back = s > 0 ? 0.6 : 0;
    return {
      text: String(i + 1),
      x: p.x + dx * LABEL_GAP + RANK_SHIFT_X + back * HEX_RADIUS * 1.5,
      y: p.y + dy * LABEL_GAP - rise + 7 + back * 2 * QUARTER_HEX,
    };
  })
);
</script>

<style scoped>
.board-container {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  background: #f5f5f5;
}

.board-svg {
  filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.15));
  background: white;
}

.hexagon {
  stroke: #999;
  stroke-width: 2;
  cursor: pointer;
  transition: opacity 0.2s;
}

.hexagon.color1 {
  fill: #e8d5c4;
}

.hexagon.color2 {
  fill: #d18b47;
}

.hexagon.color3 {
  fill: #c9a876;
}

.hexagon:hover {
  opacity: 0.9;
  stroke-width: 2.5;
}

.selected-hex {
  fill: none;
  stroke: #ff6b6b;
  stroke-width: 3;
  pointer-events: none;
}

.piece {
  font-size: 40px;
  text-anchor: middle;
  dominant-baseline: middle;
  cursor: pointer;
  user-select: none;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.piece.white {
  fill: #f0f0f0;
  stroke: #333;
  stroke-width: 0.8;
  paint-order: stroke;
}

.piece.black {
  fill: #333;
  stroke: #f0f0f0;
  stroke-width: 0.8;
  paint-order: stroke;
}

.label {
  font-size: 20px;
  font-weight: bold;
  text-anchor: middle;
  fill: #666;
  pointer-events: none;
  user-select: none;
}

.label.rank-arrow {
  font-size: 15px;
  font-weight: normal;
  opacity: 0.55;
}
</style>
