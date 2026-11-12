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
npm run tauri dev -- -- --show
npm run build        # production frontend bundle
npm run check        # svelte-check + TypeScript
npm test             # vitest
```

The palette window is created **hidden and pre-warmed** (as in production). Pass
`--show` to the binary to display it at startup:

```sh
cargo run -p copyfie -- --show          # needs `npm run dev` running
# or, through the Tauri CLI:
npm run tauri dev -- -- --show
```

Without `--show`, the app runs with no visible window until the global hotkey
lands in M1-09; this is intentional pre-warm behaviour, not a failed launch.

### Hotkey and tray

- Default shortcut: `CommandOrControl+Shift+Space`. If it is already taken,
  Copyfie opens with a blocking recorder and will not silently pick another
  chord. The replacement is saved to `settings.json` in the app data directory.
- Tray menu: **Show palette**, **Pause capture**, **Start at login**, **Quit**.
  Left-clicking the tray icon also shows the palette.
- Launch at login is **off by default**; the tray menu toggles it.
- A second launch forwards to the running instance and reveals the palette.

### Settings

Open with `Cmd/Ctrl+,` or **Tray → Settings…**.

- **Appearance:** theme (System / Light / Dark) and accent color, applied live to
  both windows and cached for instant paint.
- **History:** retention (7/30/90/365 days or Forever; default 30) and pause
  capture. Retention applies to newly captured items; pinned items never expire.
- **Shortcut:** record a replacement global shortcut. A conflicting chord is
  rejected and the previous shortcut stays active.
- **Startup:** launch at login.
- **About:** version, privacy statement, and attribution.

Settings live in `settings.json` in the app data directory. Unknown fields are
preserved and the file is written atomically with a `.bak` copy.

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
