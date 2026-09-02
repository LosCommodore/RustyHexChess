# Rust → WASM: commands and references

## Setup (once)

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
```

## Daily

```bash
wasm-pack build engine --target web --dev       # → engine/pkg/; fast, keeps debug info
wasm-pack build engine --target web --release   # optimized, for a real build

cd frontend && quasar dev                       # dev server, HMR
cd frontend && quasar build                     # → frontend/dist/
cd frontend && npm run typecheck                # vue-tsc; checks your use of engine.d.ts
```

Two independent builds: Quasar watches `frontend/` only, so **a Rust change needs
a `wasm-pack build` before the dev server sees it**. Rebuilding while `quasar dev`
runs is fine — `engine/pkg/` is just files on disk to Vite.

`--target web` gives native ES modules; the alternatives are `bundler`, `nodejs`,
`no-modules`.

## Checking without a full build

```bash
cargo check -p engine --lib --target wasm32-unknown-unknown   # does it compile for wasm?
cargo test  -p engine                                         # native; the wasm module is cfg'd out
```

Checking against wasm32 catches the usual failure early — a dependency that does
not build for the target. `cargo test` will not, because it builds natively; that
is why `display` and the terminal dependencies are gated to non-wasm targets in
[engine/Cargo.toml](../engine/Cargo.toml).

After an API change, read [engine/pkg/engine.d.ts](../engine/pkg/engine.d.ts) —
it is the contract the frontend compiles against.

## Gotchas

- `wasm-pack clean` and `wasm-pack build --watch` **do not exist** — the `clean`
  and `dev:watch` scripts in the root `package.json` will fail. Use `cargo clean`
  and `rm -rf engine/pkg`. For a watch loop:
  `cargo watch -w engine/src -s 'wasm-pack build engine --target web --dev'`.
- `engine/pkg/` is generated and self-ignoring. Never edit it, never commit it.
- Panics abort. Add `console_error_panic_hook` for real stack traces in the browser.
- `--release` build time is mostly `wasm-opt`. Use `--dev` while iterating.

## Documentation

The whole toolchain moved out of the dormant `rustwasm` org into its own
`wasm-bindgen` org. Anything on `rustwasm.github.io` is stale or 404.

- **[wasm-bindgen Guide](https://wasm-bindgen.github.io/wasm-bindgen/)** — the
  reference. Two pages carry most of it: *Supported Types* (what can cross the
  boundary) and *Attributes* (`constructor`, `js_name`, `unchecked_return_type`,
  `typescript_custom_section`).
- **[wasm-pack book](https://wasm-bindgen.github.io/wasm-pack/book)** — actively
  maintained; 0.15.0 released May 2026.
- **[examples](https://github.com/wasm-bindgen/wasm-bindgen/tree/main/examples)**
  — ~30 small programs, usually faster than prose.
- **[js-sys](https://docs.rs/js-sys) / [web-sys](https://docs.rs/web-sys)** — API
  reference for the JS stdlib and the DOM. Search, don't browse.
- **[tsify](https://docs.rs/tsify)** — derives TypeScript from Rust structs; the
  alternative to the hand-written `typescript_custom_section` block in
  [engine/src/wasm.rs](../engine/src/wasm.rs).
