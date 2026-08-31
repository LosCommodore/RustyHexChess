# RustyHexChess Frontend

Quasar v5 + Vue 3 + TypeScript frontend for RustyHexChess hexagonal chess.

## Quick Start

```bash
cd frontend
quasar dev
```

The app runs on **http://localhost:5173**

## Development

### Install dependencies

```bash
npm install
```

### Start dev server

```bash
quasar dev
```

Hot module reloading enabled — changes auto-refresh in browser.

### Build for production

```bash
quasar build
```

### Type checking

```bash
npm run typecheck
```

## Project Structure

```
src/
├── pages/
│   ├── IndexPage.vue      (Game page with board)
│   ├── SecondPage.vue
│   └── ErrorNotFound.vue
├── components/
│   └── Board.vue          (Hexagonal chess board - Phase 2)
├── composables/           (Vue 3 composables - Phase 2+)
├── layouts/               (Page layouts)
├── router/                (Vue Router configuration)
├── stores/                (Pinia state management)
├── css/
│   └── app.scss           (Global styles)
└── App.vue                (Root component)
```

## Development Phases

- **Phase 1** ✅: Quasar project scaffolding
- **Phase 2** (in progress): Board rendering with pieces
- **Phase 3**: WASM bindings to Rust engine
- **Phase 4**: Game logic integration

## Technologies

- **Framework**: Quasar v5 (Vue 3)
- **Language**: TypeScript
- **Styling**: SCSS
- **Build Tool**: Vite
- **Package Manager**: npm
