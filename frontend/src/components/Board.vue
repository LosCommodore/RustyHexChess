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
          v-for="piece in pieces"
          :key="`piece-${piece.x}-${piece.y}`"
          :x="getHexCenterPixel(piece.x, piece.y).x"
          :y="getHexCenterPixel(piece.x, piece.y).y"
          class="piece"
          :class="piece.color"
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
}

const selectedHex = ref<HexCoord | null>(null);

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

// Piece starting positions (pawns only - should form V-shape)
const pieces = computed<Piece[]>(() => [
  // White pawns
  { x: 1, y: 4, type: 'pawn', color: 'white' },
  { x: 2, y: 4, type: 'pawn', color: 'white' },
  { x: 3, y: 4, type: 'pawn', color: 'white' },
  { x: 4, y: 4, type: 'pawn', color: 'white' },
  { x: 5, y: 4, type: 'pawn', color: 'white' },
  { x: 6, y: 3, type: 'pawn', color: 'white' },
  { x: 7, y: 2, type: 'pawn', color: 'white' },
  { x: 8, y: 1, type: 'pawn', color: 'white' },
  { x: 9, y: 0, type: 'pawn', color: 'white' },

  // Black pawns
  { x: 1, y: 10, type: 'pawn', color: 'black' },
  { x: 2, y: 9, type: 'pawn', color: 'black' },
  { x: 3, y: 8, type: 'pawn', color: 'black' },
  { x: 4, y: 7, type: 'pawn', color: 'black' },
  { x: 5, y: 6, type: 'pawn', color: 'black' },
  { x: 6, y: 6, type: 'pawn', color: 'black' },
  { x: 7, y: 6, type: 'pawn', color: 'black' },
  { x: 8, y: 6, type: 'pawn', color: 'black' },
  { x: 9, y: 6, type: 'pawn', color: 'black' },
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
  return colors[colorIndex];
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
</style>
