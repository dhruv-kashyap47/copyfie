# Third-Party Notices — Copyfie

Copyfie is proprietary, closed-source software. This file records the
third-party code and dependencies used by Copyfie and the licenses they are
distributed under. It ships with every build (About → Open-source notices) and
with the installers.

Maintainer rule: when a dependency is added, removed, or updated, update this
file in the same pull request. When code is copied or adapted from a reference
project, add a row to the table below and keep the upstream MIT header in the
adapted file.

---

## 1. Adapted source code

Every copied or adapted file carries a header such as:

```rust
// Adapted from Ortu (MIT), <commit>, src-tauri/src/clipboard.rs.
// Copyright (c) 2025-2026 Abhijith P Subash. See THIRD_PARTY.md.
```

| Our file | Upstream repo | Upstream commit | License | What changed |
|---|---|---|---|---|
| `platform/macos/src/concealed.rs` | Ortu | v2.1.0 `b5747d3195fd2c52faf296d7489857499fc3021f` | MIT | Pasteboard marker list informed by Ortu's clipboard handling; implementation written from scratch. |
| `src/lib/icons.ts` | Lucide (lucide-static) | v1.72.0 | ISC; some paths MIT (Feather) | Inner SVG paths for search, x, text, image, files and pin; adapted for Svelte rendering. |
| _(other UI code)_ | — | — | — | Windows/X11 backends and the palette/settings UI are original; only prior-art ideas from PLAN §2 are used. |

Reference projects and their pinned revisions used for study and adaptation:

| Project | Version | Commit | License | Text |
|---|---|---|---|---|
| Ortu | v2.1.0 | `b5747d3195fd2c52faf296d7489857499fc3021f` | MIT | `third_party/licenses/ortu-MIT.txt` |
| Ropy | 0.6.2 | `0d7ac38fcc6883205d400d6849d8977e4f1572e4` | MIT | `third_party/licenses/ropy-MIT.txt` |
| Lucide (lucide-static) | 1.72.0 | npm release | ISC; Feather-derived paths MIT | `third_party/licenses/lucide-ISC.txt` |

Both projects are MIT-licensed. MIT permits sublicensing, modification, and
closed distribution provided the copyright and permission notices are retained.
Neither project's name, logo, or branding is used.

## 2. Direct Rust dependencies

Snapshot at scaffold time (M1-01). The authoritative, full transitive list is
generated from `Cargo.lock` by `cargo-about` in CI and appended to the release
copy of this file.

| Crate | License | Purpose |
|---|---|---|
| `tauri` | Apache-2.0 OR MIT | Application shell, webview, IPC |
| `tauri-build` | Apache-2.0 OR MIT | Build-time context generation |
| `serde` | MIT OR Apache-2.0 | Serialization |
| `serde_json` | MIT OR Apache-2.0 | JSON IPC, settings, file-path payloads |
| `rusqlite` | MIT | SQLite bindings (bundles public-domain SQLite + FTS5) |
| `blake3` | CC0-1.0 OR Apache-2.0 | Content hashing and dedupe identity |
| `thiserror` | MIT OR Apache-2.0 | Error types |
| `tokio` | MIT | Async reply channels for the store actor (Tauri provides the runtime) |
| `log` | MIT OR Apache-2.0 | Diagnostics (never logs clipboard content) |
| `tauri-plugin-log` | MIT OR Apache-2.0 | Log file + stdout, so capture failures are visible in release |
| `tauri-plugin-global-shortcut` | MIT OR Apache-2.0 | Register/rebind the global palette shortcut |
| `tauri-plugin-single-instance` | MIT OR Apache-2.0 | One process; second launch reveals the palette |
| `tauri-plugin-autostart` | MIT OR Apache-2.0 | Launch at login (registry / LaunchAgent / XDG autostart) |
| `clipboard-rs` | MIT | Clipboard watch/read on all desktop platforms (no image feature) |
| `windows-sys` | MIT OR Apache-2.0 | Win32 foreground process, window title, clipboard format DWORDs |
| `x11rb` | MIT OR Apache-2.0 | X11 `_NET_ACTIVE_WINDOW`, `_NET_WM_PID`, `WM_CLASS` |
| `objc2` | MIT | Objective-C runtime bridge (macOS) |
| `objc2-app-kit` | MIT | `NSWorkspace` / `NSRunningApplication` (macOS) |
| `objc2-foundation` | MIT | `NSString` interop (macOS) |
| `enigo` | MIT | Synthetic Ctrl/Cmd+V on Windows, macOS, and X11 |
| `tempfile` | MIT OR Apache-2.0 | Atomic staging and tests (dev) |
| `copyfie-core` | Proprietary (internal) | Models, store, search — not published |
| `copyfie-platform` | Proprietary (internal) | Platform traits shared by OS backends — not published |
| `copyfie-platform-windows` | Proprietary (internal) | Windows clipboard/foreground backend — not published |
| `copyfie-platform-macos` | Proprietary (internal) | macOS clipboard/foreground backend — not published |
| `copyfie-platform-linux` | Proprietary (internal) | Linux X11 clipboard/foreground backend — not published |

## 3. Direct JavaScript dependencies

| Package | License | Purpose |
|---|---|---|
| `svelte` | MIT | UI runtime |
| `@tauri-apps/api` | Apache-2.0 OR MIT | IPC and events |
| `vite` | MIT | Build tool (dev only) |
| `@sveltejs/vite-plugin-svelte` | MIT | Svelte compiler (dev only) |
| `typescript` | Apache-2.0 | Type checking (dev only) |
| `svelte-check` | MIT | Svelte type checking (dev only) |
| `vitest` | MIT | Unit tests (dev only) |
| `@tauri-apps/cli` | Apache-2.0 OR MIT | Tauri CLI (dev only) |

Lucide is **not** a runtime dependency: only the paths used by `src/lib/icons.ts` are copied,
with the notice in `third_party/licenses/lucide-ISC.txt`.

### Bundled fonts

| Font | Version | License | Purpose | Text |
|---|---|---|---|---|
| Inter | 5.2.8 (Latin subset) | SIL OFL 1.1 | Palette and settings UI typeface | `third_party/licenses/inter-OFL.txt` |

The font is a self-hosted Latin-subset variable WOFF2 in `src/assets/fonts/` (33 KB); no
network request is made at runtime. macOS uses the system SF stack first; Windows/Linux use
Inter.

## 4. License policy

- Allowed: **MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0-1.0,
  Unlicense, Unicode-3.0** (Unicode License v3, OSI-approved; required by the
  `icu_*` crates used for IDN handling), **BSL-1.0** (Boost Software License,
  OSI-approved; required by `clipboard-win`'s support crates).
- Allowed for **unmodified dependencies only**: **MPL-2.0** (file-level
  copyleft). No MPL-2.0 code is copied into Copyfie source files.
- Denied: **GPL, AGPL, LGPL (static linking), SSPL, BUSL, Commons Clause, and
  any non-commercial or source-available-only license**.
- `cargo-deny` enforces this in CI against the full transitive graph:
  `scripts/license/cargo-deny.toml`.
- Linux system libraries (WebKitGTK, GTK) are **dynamically linked distro
  packages**, not bundled. They are LGPL and are listed for counsel review
  before v1.0.

## 5. Distribution obligation

MIT requires the copyright and permission notices to appear in all copies and
substantial portions. Copyfie satisfies this by:

1. Shipping `NOTICE`, `THIRD_PARTY.md`, and `third_party/licenses/*.txt` in the
   installer and release assets.
2. Rendering `THIRD_PARTY.md` in **About → Open-source notices**.
3. Keeping the MIT header on every adapted source file.
