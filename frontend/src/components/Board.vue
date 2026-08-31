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
const SVG_WIDTH = 1200;
const SVG_HEIGHT = 1200;
const CENTER_X = 600;
const CENTER_Y = 600;

interface HexCoord {
  q: number;
  r: number;
}

const selectedHex = ref<HexCoord | null>(null);

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

function hexToPixel(q: number, r: number) {
  const x = HEX_RADIUS * (3 / 2 * q);
  const y = HEX_RADIUS * (Math.sqrt(3) / 2 * q + Math.sqrt(3) * r);
  return { x: CENTER_X + x, y: CENTER_Y + y };
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
  return (hex.q + hex.r) % 2 === 0 ? 'light' : 'dark';
}

function selectHex(hex: HexCoord) {
  if (selectedHex.value?.q === hex.q && selectedHex.value?.r === hex.r) {
    selectedHex.value = null;
  } else {
    selectedHex.value = { q: hex.q, r: hex.r };
  }
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

.hexagon.light {
  fill: #e8d5c4;
}

.hexagon.dark {
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
</style>
