<template>
  <q-page class="game-page">
    <div class="row q-pa-lg full-width">
      <div class="col-12">
        <q-card class="q-mb-lg">
          <q-card-section>
            <div class="text-h4">Hexagonal Chess</div>
            <p class="text-subtitle1">Phase 2: Board Rendering with Static Pieces</p>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <div class="row justify-center q-pa-lg full-width">
      <div class="col-12 col-lg-8">
        <Board />
      </div>
    </div>

    <div class="row q-pa-lg full-width">
      <div class="col-12">
        <q-card>
          <q-card-section>
            <div class="text-subtitle2">Click on hexagons to select them</div>
            <p class="text-caption text-grey-7">Drag & drop coming in Phase 2</p>
          </q-card-section>
        </q-card>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import Board from '../components/Board.vue';
</script>

<style scoped>
.game-page {
  background: linear-gradient(135deg, #f9f9f9 0%, #f0f0f0 100%);
  min-height: 100vh;
}
</style>
