<template>
  <aside class="game-info">
    <section class="panel">
      <h2 class="panel-title">Active player</h2>
      <div class="active-player">
        <span class="swatch" :class="game.activePlayer" />
        <span class="player-name">{{ playerName }}</span>
      </div>
    </section>

    <section class="panel">
      <h2 class="panel-title">Game state</h2>
      <span class="status" :class="game.status">{{ STATUS_LABELS[game.status] }}</span>
    </section>

    <section class="panel">
      <h2 class="panel-title">Move</h2>
      <p class="move-number">{{ game.moveNumber }}</p>
    </section>

    <section class="panel">
      <h2 class="panel-title">Captured</h2>
      <div v-for="side in (['white', 'black'] as PlayerColor[])" :key="side" class="captured-row">
        <span class="swatch" :class="side" />
        <span v-if="game.captured[side].length" class="captured-pieces" :class="opponentOf(side)">
          <span v-for="(type, i) in game.captured[side]" :key="`${side}-${i}`">
            {{ PIECE_SYMBOLS[type] }}
          </span>
        </span>
        <span v-else class="captured-none">—</span>
      </div>
    </section>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  game,
  PIECE_SYMBOLS,
  STATUS_LABELS,
  type PlayerColor,
} from '@/game/state';

const playerName = computed(() => (game.activePlayer === 'white' ? 'White' : 'Black'));

/** Captured pieces belong to the other side, so they render in its colour. */
function opponentOf(side: PlayerColor): PlayerColor {
  return side === 'white' ? 'black' : 'white';
}
</script>

<style scoped>
.game-info {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

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

.active-player {
  display: flex;
  align-items: center;
  gap: 10px;
}

.player-name {
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

/* Mirrors the board's piece styling so the two read as the same game. */
.swatch {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  flex-shrink: 0;
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

.status {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 14px;
  font-weight: 600;
  background: #eee;
  color: #555;
}

.status.active {
  background: #e3f2e6;
  color: #2e6e35;
}

.status.check {
  background: #fdf0d5;
  color: #8a5a00;
}

.status.checkmate {
  background: #fbe3e3;
  color: #a52222;
}

.status.stalemate,
.status.draw {
  background: #e8e8ef;
  color: #4a4a70;
}

.move-number {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: #333;
}

.captured-row {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 26px;
}

.captured-pieces {
  font-size: 20px;
  line-height: 1;
}

.captured-pieces.white {
  color: #f0f0f0;
  text-shadow: 0 0 1px #333, 0 0 1px #333, 0 0 1px #333;
}

.captured-pieces.black {
  color: #333;
}

.captured-none {
  color: #bbb;
}
</style>
