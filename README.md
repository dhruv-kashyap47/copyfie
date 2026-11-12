# Copyfie

A fast, local-first clipboard manager for Windows, macOS, and Linux (X11).

> Closed-source commercial product. This repository is private. See `NOTICE`
> and `THIRD_PARTY.md` for attribution.

## Status

Milestone 1 (MVP) — text capture, hotkey palette, search, and paste.
The current tree is the scaffold: Tauri shell, `copyfie-core` crate, CI, and
attribution files.

## Development

Requirements: Node.js 22/24+, Rust stable (MSVC toolchain on Windows), and the
platform Tauri prerequisites.

```sh
npm install          # frontend dependencies
npm run dev          # Vite dev server on :1420
npm run tauri dev    # full app in dev mode
npm run build        # production frontend bundle
npm run check        # svelte-check + TypeScript
npm test             # vitest
```

Rust:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Licenses and advisories:

```sh
cargo deny --config scripts/license/cargo-deny.toml check
```

## Layout

| Path | Purpose |
|---|---|
| `src-tauri/` | Tauri shell: commands, windows, hotkey, tray, state wiring. |
| `crates/copyfie-core/` | Models, hashing, SQLite store, search, retention. No Tauri, no OS APIs. |
| `src/palette/` | Palette webview (Svelte 5 + Vite). |
| `scripts/license/` | `cargo-deny` policy and license tooling. |
| `third_party/licenses/` | Upstream MIT texts for adapted code. |
| `docs/` | Architecture, platform, privacy, perf, and attribution docs. |
