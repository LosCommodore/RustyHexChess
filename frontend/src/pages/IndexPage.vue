<template>
  <q-layout view="hHh LpR fFf">
    <q-header bordered class="bg-white text-dark">
      <q-toolbar>
        <q-btn
          flat
          dense
          round
          icon="menu"
          aria-label="Toggle history"
          @click="drawer = !drawer"
        />
        <q-toolbar-title class="text-subtitle1 text-weight-bold">
          Hexagonal Chess
        </q-toolbar-title>
      </q-toolbar>
    </q-header>

    <!-- show-if-above docks the drawer on wide screens and turns it into an
         overlay on narrow ones, which is why no manual pin is needed. -->
    <q-drawer v-model="drawer" show-if-above side="left" bordered :width="260">
      <div class="column full-height">
        <q-tabs v-model="tab" dense no-caps narrow-indicator active-color="primary">
          <q-tab name="history" label="History" />
          <q-tab name="setup" label="Setup" />
        </q-tabs>
        <q-separator />
        <q-tab-panels v-model="tab" animated class="col scroll">
          <q-tab-panel name="history" class="q-pa-none">
            <MoveHistory />
          </q-tab-panel>
          <q-tab-panel name="setup" class="q-pa-none">
            <PiecePalette />
          </q-tab-panel>
        </q-tab-panels>
      </div>
    </q-drawer>

    <q-page-container>
      <q-page class="game-page">
        <div class="game-body">
          <Board />
          <div class="side-column">
            <GameControls />
            <GameInfo />
          </div>
        </div>
      </q-page>
    </q-page-container>

    <PromotionDialog />
  </q-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import Board from '@/components/Board.vue';
import GameControls from '@/components/GameControls.vue';
import GameInfo from '@/components/GameInfo.vue';
import MoveHistory from '@/components/MoveHistory.vue';
import PiecePalette from '@/components/PiecePalette.vue';
import PromotionDialog from '@/components/PromotionDialog.vue';

// Closed by default; `show-if-above` on the drawer keeps it docked open on wide
// screens, so this only affects phones/tablets, where it starts out of the way.
const drawer = ref(false);
const tab = ref('history');
</script>

<style scoped>
.game-page {
  padding: 24px;
  background: #f5f5f5;
}

/* Panels sit beside the board on wide screens and wrap below it when the row no
   longer fits, so the board and controls stack on tablets and phones. */
.game-body {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 24px;
}

/* Beside the board it stays a tidy fixed width; once wrapped below, it grows to
   the full row rather than leaving a 220px column stranded on the left. */
.side-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
  flex: 1 1 220px;
  min-width: 200px;
  max-width: 360px;
}

/* Tighten the margins on phones so the board gets the width it needs. */
@media (max-width: 600px) {
  .game-page {
    padding: 12px;
  }

  .game-body {
    gap: 16px;
  }

  .side-column {
    max-width: none;
  }
}
</style>
