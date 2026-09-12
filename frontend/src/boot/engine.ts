import { defineBoot } from '#q-app';
import { initEngine } from '@/game/engine';
import { startGame } from '@/game/state';

// Load and instantiate the WASM engine before the app mounts, then start the
// standard game, so the board has a real position on first paint.
export default defineBoot(async () => {
  await initEngine();
  startGame();
});
