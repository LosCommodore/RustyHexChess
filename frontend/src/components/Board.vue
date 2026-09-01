<template>
  <div class="board-container">
    <svg :width="SVG_WIDTH" :height="SVG_HEIGHT" class="board-svg" :viewBox="`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`">
      <!-- Hexagons -->
      <g class="hexagons">
        <polygon
          v-for="hex in hexagons"
          :key="`hex-${hex.q}-${hex.r}`"
          :points="getHexagonPoints(hex)"
          :class="['hexagon', getHexClass(hex), { marked: isMarked(hex) }]"
          @click="selectHex(hex)"
        />
      </g>

      <!-- Pieces -->
      <g class="pieces">
        <text
          v-for="piece in renderedPieces"
          :key="`piece-${piece.index}`"
          :x="piece.x"
          :y="piece.y"
          class="piece"
          :class="[piece.color, { dragging: piece.dragging, browsing: isBrowsing }]"
          @mousedown="startDrag($event, piece.index)"
          @click.stop="selectHex(piece)"
        >
          {{ PIECE_SYMBOLS[piece.type] }}
        </text>
      </g>

      <!-- Move markers sit above the pieces so a capture ring reads on top of
           the enemy piece it targets -->
      <g class="markers">
        <circle
          v-for="marker in renderedMarkers"
          :key="`marker-${marker.q}-${marker.r}`"
          :cx="marker.x"
          :cy="marker.y"
          :r="marker.radius"
          :class="['marker', marker.kind]"
        />
      </g>

      <!-- Selection highlight -->
      <polygon
        v-if="selectedHex"
        :points="getHexagonPoints(selectedHex)"
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
          <text
            :x="label.arrowX"
            :y="label.arrowY"
            :transform="`rotate(${ARROW_ROTATION}, ${label.arrowX}, ${label.arrowY - 5})`"
            class="label rank-arrow"
          >↙</text>
          <text :x="label.numberX" :y="label.numberY" class="label">{{ label.text }}</text>
        </g>
      </g>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  game,
  isBrowsing,
  movePiece,
  PIECE_SYMBOLS,
  viewedPieces,
  type HexCoord,
} from '@/game/state';

const HEX_RADIUS = 40;
const SVG_WIDTH = 800;
const SVG_HEIGHT = 900;
const CENTER_X = 400;
const CENTER_Y = 450;
const BOARD_RADIUS = 5;

// The board is a hexagon of hexagons: every axial coord whose three cube
// components stay within BOARD_RADIUS. Constant, so not a computed.
const hexagons: HexCoord[] = [];
for (let q = -BOARD_RADIUS; q <= BOARD_RADIUS; q++) {
  for (let r = -BOARD_RADIUS; r <= BOARD_RADIUS; r++) {
    if (Math.abs(q + r) <= BOARD_RADIUS) hexagons.push({ q, r });
  }
}

const selectedHex = ref<HexCoord | null>(null);

/** A hex highlighted as a move option: a dot for a step, a ring for a capture. */
type MarkerKind = 'move' | 'capture';
interface Marker extends HexCoord {
  kind: MarkerKind;
}

// Placeholder until the engine supplies real move generation — these are just
// a few hexes around the centre so the layer is visible.
const markers = ref<Marker[]>([
  { q: 0, r: -1, kind: 'move' },
  { q: 1, r: -1, kind: 'move' },
  { q: 0, r: 1, kind: 'move' },
  { q: -1, r: 0, kind: 'capture' },
]);

// Move options are meaningless while placing pieces freely.
const activeMarkers = computed(() => (game.mode === 'game' ? markers.value : []));

// Set of "q,r" keys so the per-hex lookup below stays O(1) across 91 hexes.
const markedHexes = computed(() => new Set(activeMarkers.value.map(m => `${m.q},${m.r}`)));

function isMarked(hex: HexCoord): boolean {
  return markedHexes.value.has(`${hex.q},${hex.r}`);
}

const MARKER_DOT = HEX_RADIUS * 0.28;
const MARKER_RING = HEX_RADIUS * 0.72;

const renderedMarkers = computed(() =>
  activeMarkers.value.map(marker => ({
    ...marker,
    ...hexToPixel(marker.q, marker.r),
    radius: marker.kind === 'capture' ? MARKER_RING : MARKER_DOT,
  }))
);

/** Live drag, if any. Holds the pixel position the piece is rendered at. */
const drag = ref<{ index: number; x: number; y: number; grabX: number; grabY: number } | null>(null);

// A piece sits at its hex centre, except while dragged: then the cursor wins.
// Pixel position is derived, never stored, so the two can never disagree.
const renderedPieces = computed(() =>
  viewedPieces.value.map((piece, index) => {
    const dragging = drag.value?.index === index;
    const pixel = dragging && drag.value
      ? { x: drag.value.x, y: drag.value.y }
      : hexToPixel(piece.q, piece.r);
    return { ...piece, index, dragging, ...pixel };
  })
);

function hexToPixel(q: number, r: number) {
  return {
    x: CENTER_X + HEX_RADIUS * ((3 / 2) * q),
    y: CENTER_Y + HEX_RADIUS * ((Math.sqrt(3) / 2) * q + Math.sqrt(3) * r),
  };
}

function pixelToHex(pixelX: number, pixelY: number): HexCoord {
  const x = pixelX - CENTER_X;
  const y = pixelY - CENTER_Y;
  const q = ((2 / 3) * x) / HEX_RADIUS;
  const r = ((-1 / 3) * x) / HEX_RADIUS + ((Math.sqrt(3) / 3) * y) / HEX_RADIUS;
  return { q: Math.round(q), r: Math.round(r) };
}

function getHexagonPoints(hex: HexCoord): string {
  const { x, y } = hexToPixel(hex.q, hex.r);
  return Array.from({ length: 6 }, (_, i) => {
    const angle = (Math.PI / 3) * i;
    return `${x + HEX_RADIUS * Math.cos(angle)},${y + HEX_RADIUS * Math.sin(angle)}`;
  }).join(' ');
}

function getHexClass(hex: HexCoord): string {
  const colors = ['color1', 'color2', 'color3'] as const;
  return colors[(((hex.q + 2 * hex.r) % 3) + 3) % 3]!;
}

function selectHex(hex: HexCoord) {
  const isSelected = selectedHex.value?.q === hex.q && selectedHex.value?.r === hex.r;
  selectedHex.value = isSelected ? null : { q: hex.q, r: hex.r };
}

function startDrag(event: MouseEvent, index: number) {
  // Past positions are for reading, not editing.
  if (isBrowsing.value) return;

  const piece = game.pieces[index];
  if (!piece) return;
  const pixel = hexToPixel(piece.q, piece.r);
  drag.value = {
    index,
    x: pixel.x,
    y: pixel.y,
    grabX: event.clientX - pixel.x,
    grabY: event.clientY - pixel.y,
  };
  document.addEventListener('mousemove', handleDrag);
  document.addEventListener('mouseup', endDrag);
}

function handleDrag(event: MouseEvent) {
  if (!drag.value) return;
  drag.value.x = event.clientX - drag.value.grabX;
  drag.value.y = event.clientY - drag.value.grabY;
}

function endDrag() {
  const current = drag.value;
  if (!current) return;

  const dropped = pixelToHex(current.x, current.y);
  const target = hexagons.find(h => h.q === dropped.q && h.r === dropped.r);
  if (target) movePiece(current.index, target);

  drag.value = null;
  document.removeEventListener('mousemove', handleDrag);
  document.removeEventListener('mouseup', endDrag);
}

// --- Coordinate labels -------------------------------------------------
// Files a-k are the visual columns (constant q). Ranks 1-11 are the up-right
// diagonals (constant s = -(q + r)), which run 30° below horizontal.

const LABEL_GAP = HEX_RADIUS + 22;
// Hex height is √3·R, so a quarter of it is half the y-step between columns.
const QUARTER_HEX = (Math.sqrt(3) * HEX_RADIUS) / 4;
// Pulls the rank labels back toward the board; the ↙ glyph widens them.
const RANK_SHIFT_X = -10;
// ↙ is drawn at 45°; rotating it clockwise lifts it toward the rank axis.
const ARROW_ROTATION = 8;
const ARROW_DX = -9;
// The number sits up-right of the arrow, on the rank axis: 24px apart
// horizontally and 24·tan(30°) ≈ 14 up.
const NUMBER_DX = 15;
const NUMBER_DY = -14;

/** Pixel x depends only on q, so a column is every hex sharing one q. */
const columnLabels = computed(() =>
  Array.from({ length: 11 }, (_, i) => {
    const q = i - BOARD_RADIUS;
    const rMax = Math.max(...hexagons.filter(h => h.q === q).map(h => h.r));
    const bottom = hexToPixel(q, rMax);
    // Sits under the column's lowest hexagon, so the row of letters follows
    // the board's V-shaped bottom edge.
    return { text: String.fromCharCode(97 + i), x: bottom.x, y: bottom.y + LABEL_GAP };
  })
);

/**
 * Each rank label hangs off its rank's rightmost hex. Ranks 1-6 anchor on the
 * right column (6 hexes tall, so they share an x); ranks 7-11 anchor on the
 * upper-right edge, so their x steps left as they climb.
 */
const rankLabels = computed(() =>
  Array.from({ length: 11 }, (_, i) => {
    const s = i - BOARD_RADIUS;
    const anchor = hexagons
      .filter(h => -(h.q + h.r) === s)
      .reduce((rightmost, h) => (h.q > rightmost.q ? h : rightmost));
    const p = hexToPixel(anchor.q, anchor.r);

    const [dx, dy] = s <= 0 ? [1, 0] : [0.5, -Math.sqrt(3) / 2];
    // Right-column ranks rise two quarter-hexes so each number lands on its own
    // diagonal rather than level with the hex centre. Edge ranks instead step
    // back along the edge, which their offset direction overshoots.
    const rise = s <= 0 ? 2 * QUARTER_HEX : 0;
    const back = s > 0 ? 0.6 : 0;
    const x = p.x + dx * LABEL_GAP + RANK_SHIFT_X + back * HEX_RADIUS * 1.5;
    const y = p.y + dy * LABEL_GAP - rise + 7 + back * 2 * QUARTER_HEX;

    return {
      text: String(i + 1),
      arrowX: x + ARROW_DX,
      arrowY: y,
      numberX: x + NUMBER_DX,
      numberY: y + NUMBER_DY,
    };
  })
);
</script>

<style scoped>
/* No padding or background: the page owns both, so the board's top edge
   lines up with whatever sits beside it. */
.board-container {
  display: flex;
  justify-content: center;
  align-items: flex-start;
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

/* Lighten the movement path. brightness() works off whatever base fill the
   hex has, so one rule covers all three board colours. Flip to a value below
   1 and invert the selector to darken everything else instead. */
.hexagon.marked {
  filter: brightness(1.35);
}

.selected-hex {
  fill: none;
  stroke: #ff6b6b;
  stroke-width: 3;
  pointer-events: none;
}

/* Markers never swallow clicks — the hex underneath stays selectable. */
.marker {
  pointer-events: none;
}

.marker.move {
  fill: rgba(30, 110, 50, 0.5);
}

.marker.capture {
  fill: none;
  stroke: rgba(190, 45, 45, 0.65);
  stroke-width: 5;
}

.piece {
  font-size: 40px;
  text-anchor: middle;
  dominant-baseline: middle;
  cursor: grab;
  user-select: none;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.piece.dragging {
  cursor: grabbing;
}

/* While browsing history the board is a read-only replay. */
.piece.browsing {
  cursor: default;
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
  font-size: 24px;
  font-weight: normal;
  opacity: 0.55;
}
</style>
