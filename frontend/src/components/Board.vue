<template>
  <div class="board-container">
    <svg :width="svgWidth" :height="svgHeight" class="board-svg">
      <!-- Hexagons -->
      <g class="hexagons">
        <polygon
          v-for="hex in hexagons"
          :key="`hex-${hex.y}-${hex.x}`"
          :points="getHexagonPoints(hex.x, hex.y)"
          :class="['hexagon', getHexClass(hex.x, hex.y)]"
          @click="selectHex(hex.x, hex.y)"
        />
      </g>

      <!-- Pieces -->
      <g class="pieces">
        <text
          v-for="piece in pieces"
          :key="`piece-${piece.y}-${piece.x}`"
          :x="getHexCenter(piece.x, piece.y).cx"
          :y="getHexCenter(piece.x, piece.y).cy"
          class="piece"
          :class="piece.color"
          @click="selectPiece(piece.x, piece.y)"
        >
          {{ pieceSymbols[piece.type] }}
        </text>
      </g>

      <!-- Selection highlight -->
      <polygon
        v-if="selectedHex"
        :points="getHexagonPoints(selectedHex.x, selectedHex.y)"
        class="selected-hex"
      />
    </svg>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface Hex {
  x: number;
  y: number;
}

interface Piece {
  x: number;
  y: number;
  type: string;
  color: string;
}

const hexSize = 40;
const svgWidth = 700;
const svgHeight = 700;

// Board dimensions from engine
const BOARD_DIM = 11;
const X_RANGE: [number, number][] = [
  [5, 10],
  [4, 10],
  [3, 10],
  [2, 10],
  [1, 10],
  [0, 10],
  [0, 9],
  [0, 8],
  [0, 7],
  [0, 6],
  [0, 5],
];

const pieceSymbols: Record<string, string> = {
  pawn: '♟',
  rook: '♜',
  knight: '♞',
  bishop: '♝',
  queen: '♛',
  king: '♚',
};

const selectedHex = ref<Hex | null>(null);

// Generate all valid hexagons
const hexagons = computed(() => {
  const hexes: Hex[] = [];
  for (let y = 0; y < BOARD_DIM; y++) {
    const [xMin, xMax] = X_RANGE[y];
    for (let x = xMin; x < xMax; x++) {
      hexes.push({ x, y });
    }
  }
  return hexes;
});

// Starting positions for hexagonal chess
const pieces = computed<Piece[]>(() => [
  // White pieces (bottom)
  { x: 5, y: 0, type: 'pawn', color: 'white' },
  { x: 6, y: 0, type: 'pawn', color: 'white' },
  { x: 7, y: 0, type: 'pawn', color: 'white' },
  { x: 8, y: 0, type: 'pawn', color: 'white' },
  { x: 9, y: 0, type: 'pawn', color: 'white' },

  { x: 4, y: 1, type: 'rook', color: 'white' },
  { x: 5, y: 1, type: 'knight', color: 'white' },
  { x: 6, y: 1, type: 'bishop', color: 'white' },
  { x: 7, y: 1, type: 'queen', color: 'white' },
  { x: 8, y: 1, type: 'king', color: 'white' },
  { x: 9, y: 1, type: 'bishop', color: 'white' },
  { x: 10, y: 1, type: 'knight', color: 'white' },
  { x: 11, y: 1, type: 'rook', color: 'white' },

  // Black pieces (top)
  { x: 0, y: 9, type: 'pawn', color: 'black' },
  { x: 1, y: 9, type: 'pawn', color: 'black' },
  { x: 2, y: 9, type: 'pawn', color: 'black' },
  { x: 3, y: 9, type: 'pawn', color: 'black' },
  { x: 4, y: 9, type: 'pawn', color: 'black' },

  { x: 0, y: 10, type: 'rook', color: 'black' },
  { x: 1, y: 10, type: 'knight', color: 'black' },
  { x: 2, y: 10, type: 'bishop', color: 'black' },
  { x: 3, y: 10, type: 'queen', color: 'black' },
  { x: 4, y: 10, type: 'king', color: 'black' },
  { x: 5, y: 10, type: 'bishop', color: 'black' },
  { x: 6, y: 10, type: 'knight', color: 'black' },
  { x: 7, y: 10, type: 'rook', color: 'black' },
]);

function getHexCenter(x: number, y: number) {
  const offsetX = 100 + x * hexSize * 0.75;
  const offsetY = 100 + y * hexSize * Math.sqrt(3) / 2;
  const adjustedX = offsetX + (y % 2) * hexSize * 0.375;
  return { cx: adjustedX, cy: offsetY };
}

function getHexagonPoints(x: number, y: number): string {
  const { cx, cy } = getHexCenter(x, y);
  const points = [];
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 3) * i;
    const px = cx + hexSize * Math.cos(angle);
    const py = cy + hexSize * Math.sin(angle);
    points.push(`${px},${py}`);
  }
  return points.join(' ');
}

function getHexClass(x: number, y: number): string {
  return (x + y) % 2 === 0 ? 'light' : 'dark';
}

function selectHex(x: number, y: number) {
  if (selectedHex.value?.x === x && selectedHex.value?.y === y) {
    selectedHex.value = null;
  } else {
    selectedHex.value = { x, y };
  }
}

function selectPiece(x: number, y: number) {
  selectHex(x, y);
}
</script>

<style scoped>
.board-container {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  background: linear-gradient(135deg, #f5f5f5 0%, #e8e8e8 100%);
  border-radius: 8px;
}

.board-svg {
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.1));
}

.hexagon {
  stroke: #999;
  stroke-width: 1;
  cursor: pointer;
  transition: opacity 0.2s;
}

.hexagon.light {
  fill: #f0e6d2;
}

.hexagon.dark {
  fill: #d2b48c;
}

.hexagon:hover {
  opacity: 0.8;
}

.selected-hex {
  fill: none;
  stroke: #ff6b6b;
  stroke-width: 3;
  pointer-events: none;
}

.piece {
  font-size: 32px;
  text-anchor: middle;
  dominant-baseline: middle;
  cursor: pointer;
  user-select: none;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.3));
}

.piece.white {
  fill: #f0f0f0;
  stroke: #333;
  stroke-width: 0.5;
}

.piece.black {
  fill: #333;
  stroke: #f0f0f0;
  stroke-width: 0.5;
}
</style>
