<template>
  <div class="history">
    <div class="section-title">History</div>

    <div class="nav">
      <q-btn flat dense size="sm" icon="first_page" title="Start" :disable="atStart" @click="viewMove(0)" />
      <q-btn flat dense size="sm" icon="chevron_left" title="Previous" :disable="atStart" @click="viewMove(game.viewIndex - 1)" />
      <span class="nav-position">{{ game.viewIndex }} / {{ game.history.length }}</span>
      <q-btn flat dense size="sm" icon="chevron_right" title="Next" :disable="!isBrowsing" @click="viewMove(game.viewIndex + 1)" />
      <q-btn flat dense size="sm" icon="last_page" title="Live" :disable="!isBrowsing" @click="viewLive" />
    </div>

    <q-list dense class="moves">
      <q-item clickable v-ripple :active="atStart" active-class="viewing" @click="viewMove(0)">
        <q-item-section side class="ply">–</q-item-section>
        <q-item-section>Starting position</q-item-section>
      </q-item>

      <q-item
        v-for="(entry, i) in game.history"
        :key="i"
        clickable
        v-ripple
        :active="game.viewIndex === i + 1"
        active-class="viewing"
        @click="viewMove(i + 1)"
      >
        <q-item-section side class="ply">
          {{ entry.moveNumber }}{{ entry.activePlayer === 'white' ? '.' : '…' }}
        </q-item-section>
        <q-item-section>
          <span class="notation">
            <span class="symbol" :class="entry.color">{{ PIECE_SYMBOLS[entry.type] }}</span>
            {{ hexName(entry.from) }}–{{ hexName(entry.to) }}
          </span>
        </q-item-section>
      </q-item>
    </q-list>

    <q-banner v-if="isBrowsing" dense class="browsing-banner">
      Viewing move {{ game.viewIndex }} of {{ game.history.length }} — board is read-only.
      <template #action>
        <q-btn flat dense no-caps size="sm" label="Back to live" @click="viewLive" />
      </template>
    </q-banner>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  game,
  hexName,
  isBrowsing,
  PIECE_SYMBOLS,
  viewLive,
  viewMove,
} from '@/game/state';

const atStart = computed(() => game.viewIndex === 0);
</script>

<style scoped>
.history {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 12px;
}

.section-title {
  margin-bottom: 8px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #999;
}

.nav {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-bottom: 8px;
}

.nav-position {
  flex: 1;
  text-align: center;
  font-size: 11px;
  color: #999;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

/* The list is the only part that should scroll as the game grows. */
.moves {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.moves :deep(.viewing) {
  background: #e3f2e6;
  color: #2e6e35;
  font-weight: 600;
}

.ply {
  min-width: 30px;
  padding-right: 6px;
  font-size: 12px;
  color: #aaa;
  font-variant-numeric: tabular-nums;
}

.notation {
  font-size: 13px;
  font-variant-numeric: tabular-nums;
}

.symbol {
  margin-right: 4px;
  font-size: 15px;
}

.symbol.white {
  color: #f0f0f0;
  text-shadow: 0 0 1px #333, 0 0 1px #333, 0 0 1px #333;
}

.symbol.black {
  color: #333;
}

.browsing-banner {
  margin-top: 8px;
  border-radius: 6px;
  background: #f2f9f4;
  font-size: 11px;
  line-height: 1.4;
  color: #2e6e35;
}
</style>
