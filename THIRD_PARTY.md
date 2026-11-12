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
| _(none yet)_ | — | — | — | Scaffold only; no reference code has been copied at M1-01. |

Reference projects and their pinned revisions used for study and adaptation:

| Project | Version | Commit | License | Text |
|---|---|---|---|---|
| Ortu | v2.1.0 | `b5747d3195fd2c52faf296d7489857499fc3021f` | MIT | `third_party/licenses/ortu-MIT.txt` |
| Ropy | 0.6.2 | `0d7ac38fcc6883205d400d6849d8977e4f1572e4` | MIT | `third_party/licenses/ropy-MIT.txt` |

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
| `tempfile` | MIT OR Apache-2.0 | Atomic staging and tests (dev) |
| `copyfie-core` | Proprietary (internal) | Models, store, search — not published |

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

## 4. License policy

- Allowed: **MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0-1.0,
  Unlicense, Unicode-3.0** (Unicode License v3, OSI-approved; required by the
  `icu_*` crates used for IDN handling).
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
