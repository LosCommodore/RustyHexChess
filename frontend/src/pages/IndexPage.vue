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
      <MoveHistory />
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
  </q-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import Board from '@/components/Board.vue';
import GameControls from '@/components/GameControls.vue';
import GameInfo from '@/components/GameInfo.vue';
import MoveHistory from '@/components/MoveHistory.vue';

const drawer = ref(true);
</script>

<style scoped>
.game-page {
  padding: 24px;
  background: #f5f5f5;
}

/* Panels sit beside the board, and drop below it when the viewport is
   narrower than the board's fixed 800px width. */
.game-body {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 24px;
}

.side-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
  width: 220px;
}
</style>
