# Hexagon chess in Rust

Hexagonal chess — [Gliński's variant](https://en.wikipedia.org/wiki/Hexagonal_chess) —
with the game logic written in Rust. The same engine compiles to WebAssembly to
drive a web frontend and also runs in the terminal.

## Play online

### 👉 [loscommodore.github.io/RustyHexChess](https://loscommodore.github.io/RustyHexChess/)

Runs entirely in the browser — the Rust engine is compiled to WASM, so there is
no server. Works on desktop, tablet and phone: drag a piece, or tap to select and
tap a highlighted hex to move. A green dot marks a move, a red ring the piece a
capture takes.

## Project layout

- **`engine/`** — the Rust engine: rules, move generation, Zobrist hashing, a
  terminal CLI (`src/bin/main.rs`), and the browser API layer (`api.rs` +
  `wasm.rs`). `cargo test` runs the whole suite natively.
- **`frontend/`** — a Quasar 2 / Vue 3 / TypeScript single-page app that loads
  the WASM engine. Not part of the Cargo workspace; it builds independently.

## Running locally

Prerequisites: a Rust toolchain with the `wasm32-unknown-unknown` target,
[`wasm-pack`](https://wasm-bindgen.github.io/wasm-pack/) (`cargo install wasm-pack`), and Node 22+.

```bash
cd frontend && npm install && cd ..   # once
npm run dev                           # builds the WASM engine, then starts Quasar dev
```

The dev server prints its URL (default <http://localhost:9000/>). Other scripts,
from the repo root:

```bash
npm run build:engine   # wasm-pack build engine --target web --release
npm run build          # build the engine, then the production frontend
cargo test -p engine   # the engine test suite
```

## Deployment

A push to `main` runs [`.github/workflows/deploy-pages.yml`](.github/workflows/deploy-pages.yml),
which builds the WASM engine and the frontend and publishes the static site to
GitHub Pages. Enable it once under **Settings → Pages → Source: GitHub Actions**.
The build sets `PUBLIC_PATH=/RustyHexChess/` so assets resolve under the project
site; change that (in the workflow and `frontend/quasar.config.ts`) if the repo
is renamed or served from a custom domain.

## Display in the Terminal

As a first step the game will be rendered in the terminal. It is easier to display rotated 90 degrees:


<pre style="font-family: monospace; white-space: pre; line-height: 1.2; background-color: #000; color: #fff; padding: 10px;">                    A  [ . ][ . ][ . ][ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ]
                 B  <span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ][ . ]
              C  [ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ][ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ][ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>
           D  <span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ]<span style='background:var(--yellow,#a60)'>[ . ][ . ][ . ]</span>[ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ][ . ]
        E  [ . ][ . ]<span style='background:var(--yellow,#a60)'>[ . ][ . ][ . ][ . ]</span>[ . ][ . ][ . ][ . ]
     F  <span style='background:var(--yellow,#a60)'>[ . ][ . ][ . ][ . ]</span><span style='color:var(--bright-red,#f55)'>[ Q ]</span><span style='background:var(--yellow,#a60)'>[ . ][ . ]</span><span style='background:var(--yellow,#a60)'><span style='color:var(--bright-blue,#55f)'>[ K ]</span></span>[ . ][ . ][ . ]
        G  [ . ][ . ]<span style='background:var(--yellow,#a60)'>[ . ][ . ][ . ][ . ]</span>[ . ][ . ][ . ][ . ] \
           H  <span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ]<span style='background:var(--yellow,#a60)'>[ . ][ . ][ . ]</span>[ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ][ . ] \  11
              I  [ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ][ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ][ . ]<span style='color:var(--bright-red,#f55)'>[ K ]</span> \  10
                 J  <span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ][ . ] \  9
                    K  [ . ][ . ][ . ][ . ]<span style='background:var(--yellow,#a60)'>[ . ]</span>[ . ] \  8
                         \    \    \    \    \    \   7
                         1    2    3    4    5    6  
</pre>