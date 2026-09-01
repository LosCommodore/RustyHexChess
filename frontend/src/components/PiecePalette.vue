<template>
  <div class="palette">
    <div class="section-title">Place pieces</div>
    <p class="hint">
      Pick a piece, then click any hex. Setup switches to free placement and
      starts a fresh position.
    </p>

    <div v-for="color in COLORS" :key="color" class="group">
      <div class="group-label">
        <span class="swatch" :class="color" />
        {{ color }}
      </div>
      <div class="grid">
        <button
          v-for="type in TYPES"
          :key="`${color}-${type}`"
          type="button"
          class="slot"
          :class="{ selected: isSelected(color, type) }"
          :title="`${color} ${type}`"
          :aria-pressed="isSelected(color, type)"
          @click="pick(color, type)"
        >
          <span class="glyph" :class="color">{{ PIECE_SYMBOLS[type] }}</span>
        </button>
      </div>
    </div>

    <button
      type="button"
      class="slot erase"
      :class="{ selected: placementTool?.kind === 'erase' }"
      :aria-pressed="placementTool?.kind === 'erase'"
      @click="selectTool(placementTool?.kind === 'erase' ? null : { kind: 'erase' })"
    >
      <q-icon name="backspace" size="16px" />
      Erase hex
    </button>

    <q-banner v-if="placementTool" dense class="active-banner">
      {{ toolLabel }} — click a hex to apply.
      <template #action>
        <q-btn flat dense no-caps size="sm" label="Done" @click="selectTool(null)" />
      </template>
    </q-banner>

    <q-separator class="q-my-md" />

    <div class="actions">
      <q-btn outline no-caps size="sm" color="grey-8" label="Reset position" @click="resetPosition" />
      <q-btn outline no-caps size="sm" color="grey-8" label="Clear board" @click="clearBoard" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  clearBoard,
  PIECE_SYMBOLS,
  placementTool,
  resetPosition,
  selectTool,
  type PieceType,
  type PlayerColor,
} from '@/game/state';

const COLORS: PlayerColor[] = ['white', 'black'];
const TYPES: PieceType[] = ['king', 'queen', 'rook', 'bishop', 'knight', 'pawn'];

function isSelected(color: PlayerColor, type: PieceType): boolean {
  const tool = placementTool.value;
  return tool?.kind === 'piece' && tool.color === color && tool.type === type;
}

/** Clicking the selected piece again clears the tool. */
function pick(color: PlayerColor, type: PieceType) {
  selectTool(isSelected(color, type) ? null : { kind: 'piece', type, color });
}

const toolLabel = computed(() => {
  const tool = placementTool.value;
  if (!tool) return '';
  return tool.kind === 'erase' ? 'Erasing' : `Placing ${tool.color} ${tool.type}`;
});
</script>

<style scoped>
.palette {
  padding: 12px;
}

.section-title {
  margin-bottom: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #999;
}

.hint {
  margin: 0 0 12px;
  font-size: 11px;
  line-height: 1.4;
  color: #999;
}

.group {
  margin-bottom: 12px;
}

.group-label {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
  font-size: 11px;
  font-weight: 600;
  text-transform: capitalize;
  color: #777;
}

.swatch {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.swatch.white {
  background: #f0f0f0;
  border: 1.5px solid #333;
}

.swatch.black {
  background: #333;
  border: 1.5px solid #f0f0f0;
  box-shadow: 0 0 0 1px #999;
}

.grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 4px;
}

.slot {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px 0;
  border: 1px solid #ddd;
  border-radius: 5px;
  background: white;
  cursor: pointer;
}

.slot:hover {
  background: #f4f4f4;
}

.slot.selected {
  border-color: #2e6e35;
  background: #e3f2e6;
  box-shadow: inset 0 0 0 1px #2e6e35;
}

.glyph {
  font-size: 19px;
  line-height: 1;
}

/* Matches the board's piece rendering so the palette previews the result. */
.glyph.white {
  color: #f0f0f0;
  text-shadow: 0 0 1px #333, 0 0 1px #333, 0 0 1px #333, 0 0 1px #333;
}

.glyph.black {
  color: #333;
}

.erase {
  gap: 6px;
  width: 100%;
  padding: 7px 0;
  font-size: 12px;
  font-weight: 600;
  color: #666;
}

.active-banner {
  margin-top: 10px;
  border-radius: 6px;
  background: #f2f9f4;
  font-size: 11px;
  line-height: 1.4;
  color: #2e6e35;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
</style>
