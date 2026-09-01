<template>
  <section class="panel">
    <h2 class="panel-title">Mode</h2>
    <div class="mode-switch" role="group">
      <button
        v-for="option in (['game', 'free'] as BoardMode[])"
        :key="option"
        type="button"
        class="mode-button"
        :class="{ selected: game.mode === option }"
        :aria-pressed="game.mode === option"
        @click="setMode(option)"
      >
        {{ MODE_LABELS[option] }}
      </button>
    </div>
    <p class="mode-hint">{{ modeHint }}</p>

    <button type="button" class="undo-button" :disabled="!canUndo" @click="undo">
      Undo
      <span v-if="canUndo" class="undo-count">{{ game.history.length }}</span>
    </button>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  canUndo,
  game,
  MODE_LABELS,
  setMode,
  undo,
  type BoardMode,
} from '@/game/state';

const modeHint = computed(() =>
  game.mode === 'game'
    ? 'Moves advance the turn.'
    : 'Place pieces freely; the turn does not change.'
);
</script>

<style scoped>
.panel {
  background: white;
  border-radius: 8px;
  padding: 14px 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.panel-title {
  margin: 0 0 10px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #999;
}

.mode-switch {
  display: flex;
  border: 1px solid #ddd;
  border-radius: 6px;
  overflow: hidden;
}

.mode-button {
  flex: 1;
  padding: 7px 8px;
  border: none;
  background: white;
  font-size: 12px;
  font-weight: 600;
  color: #666;
  cursor: pointer;
}

.mode-button + .mode-button {
  border-left: 1px solid #ddd;
}

.mode-button:hover:not(.selected) {
  background: #f6f6f6;
}

.mode-button.selected {
  background: #2e6e35;
  color: white;
}

.mode-hint {
  margin: 8px 0 14px;
  font-size: 11px;
  line-height: 1.4;
  color: #999;
}

.undo-button {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 9px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: white;
  font-size: 14px;
  font-weight: 600;
  color: #333;
  cursor: pointer;
}

.undo-button:hover:not(:disabled) {
  background: #f6f6f6;
}

.undo-button:disabled {
  color: #ccc;
  cursor: not-allowed;
}

.undo-count {
  padding: 1px 7px;
  border-radius: 999px;
  background: #eee;
  font-size: 11px;
  color: #777;
}
</style>
