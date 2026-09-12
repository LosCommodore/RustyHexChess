<template>
  <!-- persistent: a pawn on the last rank must be replaced before play resumes,
       so there is no way to dismiss this without choosing. -->
  <q-dialog :model-value="isPromoting" persistent>
    <q-card class="promotion-card">
      <q-card-section class="text-subtitle1 text-weight-bold">
        Promote {{ game.activePlayer }} pawn
      </q-card-section>
      <q-card-section class="choices">
        <button
          v-for="type in CHOICES"
          :key="type"
          type="button"
          class="choice"
          :title="type"
          @click="choosePromotion(type)"
        >
          <span class="glyph" :class="game.activePlayer">{{ PIECE_SYMBOLS[type] }}</span>
          <span class="name">{{ type }}</span>
        </button>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import {
  choosePromotion,
  game,
  isPromoting,
  PIECE_SYMBOLS,
  type PieceType,
} from '@/game/state';

// King and pawn are never valid promotion targets.
const CHOICES: PieceType[] = ['queen', 'rook', 'bishop', 'knight'];
</script>

<style scoped>
.promotion-card {
  min-width: 280px;
}

.choices {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.choice {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 10px 0;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: white;
  cursor: pointer;
}

.choice:hover {
  border-color: #2e6e35;
  background: #e3f2e6;
}

.glyph {
  font-size: 30px;
  line-height: 1;
}

.glyph.white {
  color: #f0f0f0;
  text-shadow: 0 0 1px #333, 0 0 1px #333, 0 0 1px #333, 0 0 1px #333;
}

.glyph.black {
  color: #333;
}

.name {
  font-size: 11px;
  text-transform: capitalize;
  color: #666;
}
</style>
