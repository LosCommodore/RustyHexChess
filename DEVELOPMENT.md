# Development notes

Environment and workflow things that are not visible in the code and cost time
every time they are rediscovered. Deep dives live in [doc/](doc/); this is the
index of traps.

## Layout

A Cargo workspace with one member, `engine` — the library plus `engine/src/bin/main.rs`,
a terminal CLI. `frontend/` is a separate Quasar/Vue app and is **not** part of
the workspace; the two build independently. `engine/pkg/` is wasm-pack output:
generated, self-ignoring, never edited and never committed.

## Rust

Edition 2024, so a 1.85+ toolchain. Nothing pins it — no `rust-toolchain.toml`,
currently building on 1.97.1.

`cargo test` builds natively. The `wasm` module and the wasm dependencies are
cfg'd out of that build, and `display` plus crossterm are cfg'd out of the wasm
build; see the `[target.'cfg(...)'.dependencies]` blocks in
[engine/Cargo.toml](engine/Cargo.toml).

## rust-analyzer analyzes exactly one target

Which means one of `wasm.rs` or `display.rs` always has no IDE support — the
dimmed one is the one currently cfg'd out. The workspace is set to
`wasm32-unknown-unknown`; comment that line out in
[.vscode/settings.json](.vscode/settings.json) to switch back.
Full explanation in [doc/wasm-commands.md](doc/wasm-commands.md).

## Snapshot tests

`insta`, with `cargo insta review` to accept changes (needs `cargo install cargo-insta`,
1.48.0 here). The `.snap` files are a debug dump of the move list.

Less obvious: the board tests *also* write HTML renders of the position to
`engine/src/snapshots/*.html` through `display::save_board_to_html_file`. Open
one in a browser — reading the picture beats reading the debug dump when a
movement test fails. They are committed alongside the `.snap` files.

## Frontend

npm, and Node `>=22.12` per `frontend/package.json` (on 24.x here).

The root `package.json` scripts drive both halves: `npm run dev` does a `--dev`
engine build then starts Quasar, `npm run build` is the release pair. They call
`quasar` directly, so the global `@quasar/cli` is a prerequisite.

**There is no hot reload for Rust changes.** Vite's HMR covers `frontend/` only.
`engine/pkg/` is written by a separate build that Quasar knows nothing about, and
the `.wasm` binary is fetched at runtime by the generated `init()` rather than
being part of the module graph. So after editing Rust:

```bash
npm run build:engine:dev   # in a second terminal; leave `npm run dev` running
```

then reload the browser. Rebuilding while the dev server runs is safe —
`engine/pkg/` is just files on disk to Vite. Restarting `npm run dev` for a Rust
change works too but rebuilds the frontend for nothing.

(As of now `frontend/src/` does not import the engine at all — `state.ts` and
`Board.vue` are still placeholders. This is the rule for once it is wired up.)

## VS Code

- Rust inlay hints stay hidden until `Ctrl+Alt` is held
  (`"editor.inlayHints.enabled": "offUnlessPressed"`).
- [.vscode/launch.json](.vscode/launch.json) labels its configs
  `'hexagon_logic'`, the crate's old name. Cosmetic — the cargo args are
  correct. The same dead name survives in `engine/Cargo.lock`, a leftover the
  workspace ignores; the root `Cargo.lock` is the real one.

## WASM

Everything about the Rust → browser pipeline — the wasm-bindgen/wasm-pack split,
build commands, watch loop, gotchas, current documentation links — is in
[doc/wasm-commands.md](doc/wasm-commands.md).
