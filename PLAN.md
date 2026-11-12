# Copyfie — Implementation Plan

> Plan only. No code, scaffolding, or repository changes are included.

I inspected both reference repos read-only in a temp directory (Ropy `0.6.2`, Ortu `2.1.0`) and read their LICENSE files, manifests, architecture docs, clipboard/paste/window code, storage layer, packaging, and open/closed issues. Below is the implementation plan, revised for a paid, closed-source, single-edition product with a 14-day trial.

---

# Decisions

| Area | Decision | Notes / status |
|---|---|---|
| **Business model** | **One-time purchase. No subscription. One product, all features.** | No free/Pro tier split. A **14-day** trial is the only gate. |
| **Trial** | **14 days from first launch, all features.** Calendar-based, not usage-based. | No account. Signed trial record, redundant anchors, clock-rollback detection; tamper-resistant, not tamper-proof. One gentle, dismissible reminder on days **10, 13, and 14** — at most one per day, never blocking. |
| **After trial** | Capture continues for **all** types; palette shows the **25 most recent text items**. | Older/locked rows are dimmed with one line: “N more items waiting, unlock to see them.” Nothing is deleted. Purchase unlocks instantly. |
| **Locked after trial** | Images, files, pins, full-history search, snippets, OCR, custom themes, sync, and settings. **Clear history stays available.** | Settings opens in a locked state: only **License**, **About**, **Privacy**, **Quit**, and **Clear history** work. License entry is always reachable; deletion is always possible. |
| **Footer banner** | One small footer line only. | Trial countdown and at most one reminder per day. No popups, no modal dialogs, no blocking screens. |
| **Pricing** | One-time **$15–19** target; launch price **$12–15**. | Includes **3 devices** and free updates within **v1.x**; **v2.0 is a paid upgrade**. Competitor-price research is a task before finalizing. Refund window matches the 14-day trial. |
| **Payments (MoR)** | **Paddle** is the Merchant of Record. | Checkout, tax/VAT, receipts, refunds, and chargebacks are Paddle’s responsibility. Indian seller onboarding; monthly payouts by wire or Payoneer, $100 minimum. |
| **License delivery** | **Own Ed25519 key service**, driven by Paddle webhooks. | Paddle’s license-key feature is not a good fit (no robust 3-device/offline story), so we issue the key ourselves. Webhook → signed key → email + success page; offline verification in the app; restore-purchase by email; revoked-key list. |
| **Backend** | **One small serverless service + static marketing site.** | Cloudflare Workers + KV (free/low-cost tier); no accounts, no clipboard data, no telemetry. Swappable MoR behind a provider-agnostic interface. |
| **Architecture** | **No separate Pro process, no IPC contract crate, no Pro build.** | One isolated in-app licensing module (`license/`), one small `PaymentProvider` seam, and no clipboard logic in either. |
| **Source model** | **Closed-source commercial product. Private repo permanently.** | Proprietary `LICENSE` + EULA. MIT attribution for adapted Ropy/Ortu code retained from day one. |
| **Sync** | **Not in v1.0.** Optional **v1.2**: E2E-encrypted sync through a user-chosen synced folder (iCloud/Dropbox/OneDrive). | No server, no relay, no managed sync backend. |
| **History length** | User setting: **default 30 days**, options 7/30/90/365 days or **Forever**. | No plan-based history limits — **the trial length never truncates history**. Image cache cap 1 GB, user-adjustable. |
| **RAM target** | Core process **< 40 MB**; total with webview expected **60–100 MB**. | Keep the pre-warmed palette. Measure real RSS per OS in Milestone 1. Decision gate if total **> 120 MB**. |
| **Default hotkey** | `CommandOrControl+Shift+Space`. | First-run conflict detection + simple rebind flow. `Ctrl+Shift+V` is plain-text paste; `Ctrl+Alt+V` is Paste Special in Office. |
| **Quick paste `1–9`** | Bare digits only when the query is empty; `Cmd/Ctrl+1–9` always. | Applies to the 25 visible rows in limited mode. |
| **`Shift+Enter`** | Text → plain text; files → newline-joined paths; images → normal paste + toast. | Defined fallback instead of a silent no-op. |
| **Source app display** | Name + colored initial in v1.0. | Icons in v1.1. |
| **Blocklist matching** | Exact executable/bundle ID + basename. No regex. | Wayland cannot enforce it; documented. |
| **Updates in v1.0** | **No auto-updater.** Manual “Check for updates” link to the release page. | v1.x adds opt-out daily check + signed auto-updater. v1.x updates are free. |
| **Linux v1.0** | **X11 only.** | Keep Wayland capability notice + copy-only fallback. Full Wayland support in v1.x. |
| **v1.0 packaging** | NSIS (Windows), DMG (macOS), deb + rpm (Linux). | MSI, AppImage, Flatpak move to v1.x. |
| **Distribution** | **One-line installers** (`irm \| iex`, `curl \| sh`) plus **winget** and a **`copyfie/tap` Homebrew cask**; signed apt/rpm repo in v1.x. | Scripts detect OS/arch, verify SHA-256 + signature, install per-user where possible, enable autostart, launch, and print the uninstall command (§12). |
| **macOS artifact shape** | Per-arch DMGs first. | Universal DMG later. |
| **AppImage** | Deferred to v1.x. | If shipped later, exempt from the 10 MB budget. |
| **OCR on Linux** | Optional system Tesseract. | No bundled OCR models. |
| **Clear history** | Keeps pinned items by default. | Separate “clear including pinned” confirmation. **Allowed in locked mode** with confirmation: capture continues after the trial, so the user must be able to delete it. |
| **Minimum OS** | Windows 10 1809+, macOS 12+, Ubuntu 22.04+/glibc 2.35. | |
| **Product identity** | **Final name: Copyfie.** Bundle id `com.copyfie.app` (provisional until the domain is confirmed); download domain `copyfie.app` (placeholder). | Before packaging/signing: verify domain availability, trademark conflicts, and GitHub/winget/Homebrew/package handles, then lock the bundle id. |
| **Repo visibility** | **Private permanently** (closed source). | Source is not published; EULA governs use. |
| **Wayland / Flatpak / MSI / auto-updater** | Deferred to **v1.x later**. | v1.0 ships X11 + NSIS/DMG/deb/rpm, no auto-update. |

---

# 1. License audit (reference repos) and dependency policy

## Bottom line

| Repo | LICENSE file | Manifest license | Copy into our product? | Copy into the closed binary? |
|---|---|---|---|---|
| **Ropy** | MIT — “Copyright 2025-2026 The Ropy Authors” | `Cargo.toml`: `license = "MIT"` | **Yes**, with the MIT notice retained | **Yes**, with the MIT notice retained |
| **Ortu** | MIT — “Copyright (c) 2025-2026 Abhijith P Subash” | `package.json`: `MIT`; `src-tauri/Cargo.toml`: `license = "MIT"` | **Yes**, with the MIT notice retained | **Yes**, with the MIT notice retained |

Neither repo is AGPL, non-commercial, source-available-only, or unlicensed. Both are permissive and compatible with a proprietary, closed-source product.

## What changes now that the product is closed source

- MIT permits sublicensing, modification, and closed distribution. Copying and adapting Ropy/Ortu code into our proprietary codebase is allowed **provided the MIT copyright and permission notices are retained** in `THIRD_PARTY.md` and in adapted source files.
- There is no copyleft contamination risk from our own code: the product is proprietary, and we are not adding GPL/AGPL headers anywhere.
- The remaining obligation is **attribution**, not source disclosure. MIT does not require us to publish our source or our modifications.
- Keep a provenance record per adapted file (upstream repo, commit, what changed). This protects us if a question arises later and makes license notices accurate.
- **Do not** reuse either project’s name, logo, or branding. Attribute them as prior art/inspiration only.
- The product needs a proprietary `LICENSE` (all rights reserved) and an **EULA** before any binary is distributed. Both are release gates, not Milestone 1 blockers.

## Dependency licenses

Direct dependencies of both repos are permissive. Spot-checked from crates.io:

| Dependency group | Licenses | Conflict with a closed product? |
|---|---|---|
| Tauri 2 + official plugins | Apache‑2.0 OR MIT | No |
| `rusqlite` | MIT | No |
| `clipboard-rs` | MIT | No |
| `enigo` | MIT | No |
| `image` | MIT OR Apache‑2.0 | No |
| `ashpd`, `zbus` | MIT | No |
| `x11rb`, `windows-sys`, `objc2`, `objc2-app-kit` | MIT / Apache‑2.0 / Zlib | No |
| `blake3` | CC0‑1.0 OR Apache‑2.0 | No |
| `seahash`, `redb`, `global-hotkey`, `tray-icon`, `arboard`, `cocoa`, `objc` | MIT / Apache‑2.0 | No |
| `aes-gcm`, `sha2`, `sysinfo`, `winreg` | MIT / Apache‑2.0 | No |
| SQLite + FTS5 (bundled via `rusqlite`) | Public domain | No |
| Svelte / Vite / TypeScript | MIT / MIT / Apache‑2.0 | No |

**Policy:** `cargo-deny` with an allowlist of **MIT, Apache‑2.0, BSD‑2/3, ISC, Zlib, CC0, Unlicense**; allow **MPL‑2.0** only for unmodified dependencies (file-level copyleft). **Deny GPL, AGPL, SSPL, and non-commercial licenses** — a proprietary product cannot ship them. Re-check on every dependency update, including transitive crates.

**System libraries on Linux:** WebKitGTK and GTK are **LGPL**, dynamically linked as distro packages and not bundled by us. That is the normal Tauri arrangement, but list them in `THIRD_PARTY.md` and have counsel confirm dynamic-linking compliance before v1.0. Do not statically link LGPL libraries.

## Attribution obligations

- MIT requires the copyright notice and permission notice in “all copies or substantial portions.”
- Add both copyright lines to `NOTICE` / `THIRD_PARTY.md` from day one, even where we only adapt ideas, and definitely where we copy code.
- Keep MIT headers on copied files; add an “adapted from” header with upstream repo, commit, and a summary of changes.
- Do not add GPL/AGPL headers. Our own files carry no license header until the proprietary `LICENSE`/EULA are finalized; after that, a short “Copyright © 2026 … All rights reserved.” header is sufficient.
- `THIRD_PARTY.md` ships inside the app (About → Open-source notices) and in the installer/release assets, satisfying the “all copies” requirement for a binary-only distribution.

---

# 2. Reuse map

Legend: **Adopt** = copy with minor changes; **Adapt** = reuse structure/approach, rewrite against our schema/stack; **Rewrite** = study only; **N/A** = incompatible with our fixed stack.

| # | Component / files | Source | License | Call | Effort saved | Risk |
|---|---|---|---|---|---|---|
| 1 | Tauri 2 scaffold, `tauri.conf.json`, capabilities, tauri-action release workflow | Ortu | MIT | **Adapt** | S | Low |
| 2 | SQLite schema, FTS5 external-content triggers, fuzzy re-rank | Ortu `db.rs` | MIT | **Adapt** | M | Medium — schema must change for source app, files, pins, blobs-on-disk |
| 3 | macOS concealed/transient pasteboard markers + tests | Ortu `clipboard.rs` | MIT | **Adopt/Adapt** | S | Low — proven marker list |
| 4 | Popup centering on cursor monitor with mixed-DPI math | Ortu `lib.rs` | MIT | **Adopt** | S | Low — fixes Ropy #113 class of bugs |
| 5 | Windows DWM rounded corners + titlebar theming | Ortu `lib.rs` | MIT | **Adapt** | S | Low |
| 6 | macOS Accessibility status check + Settings deep link | Ortu `commands.rs` | MIT | **Adopt** | S | Low |
| 7 | macOS paste keycode fix (`Key::Other(9)`) + held-modifier release | Ortu `commands.rs`, `docs/macos-direct-paste-investigation.md` | MIT | **Adopt** | S | Low — avoids a confirmed release crash |
| 8 | CopyTracker: capture acknowledged only after persistence, retryable generation | Ropy `clipboard/capture.rs` | MIT | **Adapt** | M | Medium — port to SQLite/lock model |
| 9 | Clipboard watcher payload priority + single-slot newest-wins image queue | Ropy `clipboard/listener.rs` | MIT | **Adapt** | M | Low |
| 10 | Image files on disk, atomic staging, hash-named thumbnails, sidecar GC | Ropy `repository/assets.rs`, `cleanup.rs` | MIT | **Adapt** | M | Low |
| 11 | In-memory literal search + `memchr::memmem` + benchmark discipline | Ropy `search.rs`, `docs/search-performance.md` | MIT | **Adapt** | M | Low — we add FTS5 |
| 12 | Resident vs transient popup lifetime; X11 focus query | Ropy `gui/window.rs` | MIT | **Adapt** | M | Medium — Tauri/GTK differs from GPUI |
| 13 | Wayland GlobalShortcuts portal, Hyprland runtime binding, activation token | Ropy `gui/hotkey/{portal,hyprland}.rs`, `linux/activation.rs`, `docs/wayland.md` | MIT | **Adapt — v1.x later** | L | **High** — Tauri exposes less native surface |
| 14 | Linux enigo backend selection to avoid duplicate X11+Wayland input | Ropy `gui/paste.rs` | MIT | **Adopt** (X11 now; Wayland later) | S | Low |
| 15 | Hotkey update transactionality (new registration must succeed before persisting) | Ropy `gui/hotkey.rs` | MIT | **Adapt** | M | Medium |
| 16 | Updater rollback pattern, Homebrew cask generator, AUR recipe | Ropy `updater/`, `scripts/`, `packaging/` | MIT | **Adapt — Homebrew cask in v1.0; updater/AUR v1.x** | M | Packaging only; no clipboard logic. |
| 17 | History cleanup buffering (don’t scan on every save) | Ropy `cleanup.rs` | MIT | **Adapt** | S | Low |
| 18 | Auto-group classifier (~30 detectors) | Ortu `clipboard.rs` | MIT | **Skip v1** | — | Not in scope; possible v1.x |
| 19 | Snippets, transforms, AES-GCM sensitive items, backup/restore | Ortu | MIT | **v1.1** (snippets); later for the rest | M | Snippets are in the single product, not a paid tier |
| 20 | GPUI board, masonry grid, themes, i18n | Ropy | MIT | **N/A** | — | Stack conflict; do not port |

## Where the two repos disagree, and my recommendation

| Topic | Ropy | Ortu | Recommendation |
|---|---|---|---|
| Storage | redb KV + custom time index | SQLite WAL + FTS5 | **Keep SQLite + FTS5** (fixed decision). Adopt Ropy’s cleanup buffering and Ropy’s atomic payload discipline. |
| Images | Files on disk, atomic staging, 180 px thumbs | BLOBs + thumb BLOBs in SQLite | **Files on disk.** Avoids WAL bloat, large DB copies, and VACUUM pain; better for future sync. |
| Capture | `clipboard-rs`, event-driven Windows/X11, polling macOS/Wayland | `arboard`, 350 ms polling with full reads | **Ropy’s approach.** Event-driven where possible; read only after change. |
| Dedup | Content hash + persistence acknowledgement | `raw_content` equality, bump timestamp | **Ropy’s approach**, with BLAKE3 and a proper unique index. |
| Search | In-memory `memchr` literal search | FTS5 candidate set + fuzzy re-rank | **Hybrid**: FTS5 for ranked/full-text candidates, in-memory preview scan for instant substring/fuzzy. |
| Popup lifetime | Resident/pre-warmed (except Wayland transient) | Lazily created on first hotkey | **Ropy’s approach** for the 100 ms target. Core RSS budget is 40 MB; total is tracked against the 120 MB gate. |
| Default hotkey | `Ctrl+Shift+D` | `Alt+V` | **Adopt `CommandOrControl+Shift+Space`** (decided). Neither reference uses it. `Ctrl+Shift+V` is plain-text paste; `Ctrl+Alt+V` is Paste Special in Office. Add first-run conflict detection + rebind. |
| macOS concealed items | Not implemented | Implemented and tested | **Ortu wins**; extend to Windows/Linux. |
| Source app | Not tracked | Not tracked | **Neither covers it.** We add it; it is required for the blocklist and the UX. |
| Wayland | Deep, documented, native | Broken/partial; issue #14 | **Deferred to v1.x.** v1.0 is X11-only, with a capability notice and copy-only fallback. |
| Paste delay | Fixed 50 ms | 250–450 ms + AppleScript reactivation | **Neither.** Restore focus deterministically, poll until target is active, then paste; target ~60–120 ms. |
| Packaging | cargo-dist, custom updater with rollback, AUR/Homebrew | tauri-action, Tauri updater, universal DMG | **Tauri packaging for v1.0: NSIS, DMG, deb/rpm.** MSI, AppImage, Flatpak, and auto-updater are v1.x. |
| UI stack | GPUI native | SvelteKit + Tailwind | **Keep Tauri + Svelte + Vite.** Ortu proves the stack; use plain Svelte 5 (no SvelteKit, no Tailwind) for size and paint speed. |

## Reference facts worth keeping

- **Ortu artifacts (v2.1.0):** NSIS 3.65 MB, MSI 4.99 MB, `.deb` 5.89 MB, universal DMG 9.82 MB, **AppImage 79.85 MB**.
- **Ropy artifacts (0.6.2):** Linux x86_64 `tar.xz` 9.80 MB, Windows zip 9.08 MB, macOS DMG 8.20–9.21 MB; uncompressed Linux executable 37.37 MiB.
- **Ropy 0.5.7 benchmark:** isolated hidden-window RSS 57.28 MiB; 10,000-record ASCII search ~1.08 ms after optimization.
- These numbers are the basis for the size/RAM expectations in §8. They show that **core RSS < 40 MB is realistic**, that **total RSS with a pre-warmed webview is expected at 60–100 MB**, and that **AppImage cannot meet `< 10 MB`** (hence deferred).

---

# 3. Architecture overview

## Components

| Component | Responsibility |
|---|---|
| `copyfie-core` (Rust lib) | Models, hashing, SQLite store, FTS5 search, retention, settings model. No Tauri, no OS APIs. |
| `platform/common` | Traits: `ClipboardBackend`, `ForegroundApp`, `FocusRestore`, `InputSimulator`, `HotkeyBackend`, `PermissionProbe`. |
| `platform/windows` | `WM_CLIPBOARDUPDATE` watcher, clipboard formats, foreground process, focus restore, SendInput, DWM. |
| `platform/macos` | `NSPasteboard` changeCount polling, concealed types, `NSWorkspace` frontmost app, `NSRunningApplication` activation, CGEvent, Accessibility. |
| `platform/linux` | X11 XFixes capture, `_NET_ACTIVE_WINDOW` focus, XTEST. Wayland data-control/portal/virtual-keyboard is v1.x. |
| `src-tauri` | Tauri shell: commands, window manager, tray, hotkey registration, single instance, autostart, state wiring. |
| Store actor | One writer thread owns the SQLite connection; commands send requests over a channel. Serializes writes, cleanup, blob GC. |
| Search cache | In-memory summaries + previews for instant typeahead; FTS5 for full-text/ranked results. |
| Svelte palette | Search-first UI, keyboard navigation, rendering. |
| Svelte settings | Hotkey recorder, history length, blocklist, theme, about. Lazily created second webview; destroyed on close. |
| License module (`license/`) | Trial record (14 days), offline license verification, optional activation/seat count, entitlement state. Isolated behind one interface; no clipboard logic. |
| Payments seam | `PaymentProvider` trait (checkout URL, webhook verification, event types). Paddle is the v1.0 implementation; the app never imports Paddle types directly. |
| Feature modules | Snippets (v1.1), OCR (v1.1), custom themes (v1.1), folder sync (v1.2). In-process, gated by a single entitlement check. |

## Flow: capture → dedupe → store → search → paste

```
┌──────────────── OS clipboard ────────────────┐
│ Windows: WM_CLIPBOARDUPDATE (event)          │
│ macOS:   changeCount poll (400 ms, cheap)    │
│ X11:     XFixes selection notify (event)     │
│ Wayland: data-control MIME poll (v1.x)    │
└──────────────────────┬───────────────────────┘
                       │ raw candidate
                       ▼
              ┌─────────────────┐
              │ Privacy gate    │
              │ concealed marks │──drop──► nothing read / nothing stored
              │ app blocklist   │
              │ pause flag      │
              └────────┬────────┘
                       │ allowed
                       ▼
              ┌─────────────────┐
              │ Canonicalizer   │ normalize line endings, paths, RGBA
              │ BLAKE3 hash     │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐      existing hash?
              │ Deduper         │──── yes ──► bump captured_at, keep pin/expiry, no re-encode
              └────────┬────────┘
                       │ new
                       ▼
        ┌──────────────────────────────────┐
        │ Store actor (single writer)      │
        │ SQLite WAL + FTS5 triggers       │
        │ image files: images/<hash>.png   │
        │              thumbs/<hash>.png   │
        └──────────────┬───────────────────┘
                       │
          ┌────────────┴────────────┐
          ▼                         ▼
  Summary cache update      history:changed event (coalesced)
          │                         │
          └────────────┬────────────┘
                       ▼
┌────────────── Palette (pre-warmed) ──────────────┐
│ hotkey → show/focus → cached rows instantly      │
│ entitlement → full history or 25-item view       │
│ keystroke → local filter (0–3 ms)                │
│           → debounced FTS5 query (30–60 ms)      │
│ Enter → paste_item(id, mode)                     │
└──────────────────────┬───────────────────────────┘
                       ▼
        ┌──────────────────────────────────┐
        │ Paste orchestrator               │
        │ hide palette                     │
        │ restore previous target window   │
        │ wait until target is active      │
        │ write clipboard (+ suppress echo)│
        │ synthesize Ctrl/Cmd+V            │
        └──────────────────────────────────┘
```

## Key design rules

- **Never read clipboard payload before the privacy gate.** Enumerate formats/types first.
- **Persistence acknowledgement drives dedupe.** A failed DB write must allow the same content to be captured again.
- **One writer, many readers conceptually.** The store actor owns writes; search reads from the cache and FTS. No `Mutex<Connection>` spread across commands.
- **Suppress our own writes.** Writing an item back to the clipboard must not create a new history row or bump the item. Use a hash+generation suppression token with a 2 s TTL.
- **The palette never blocks on the DB at open.** It renders cached summaries, then refreshes.
- **Entitlement never gates capture.** Trial expiry only changes what the palette displays; the store keeps writing everything.

---

# 4. Repository and folder structure

```
copyfie/
├─ package.json
├─ vite.config.ts
├─ tsconfig.json
├─ index.html                      # palette entry
├─ settings.html                   # settings entry
├─ src/
│  ├─ palette/
│  │  ├─ App.svelte
│  │  ├─ SearchBar.svelte
│  │  ├─ FilterBar.svelte
│  │  ├─ ResultList.svelte
│  │  ├─ ResultRow.svelte
│  │  └─ Footer.svelte
│  ├─ settings/
│  │  ├─ App.svelte
│  │  ├─ General.svelte
│  │  ├─ HotkeyRecorder.svelte
│  │  ├─ Blocklist.svelte
│  │  ├─ About.svelte
│  │  ├─ UnlockPanel.svelte        # license key entry, trial status, purchase
│  │  └─ LockedHistory.svelte      # 25-item limit + "N more items waiting"
│  ├─ lib/
│  │  ├─ ipc.ts                   # typed invoke wrappers
│  │  ├─ keymap.ts                # keyboard state machine
│  │  ├─ localSearch.ts           # instant preview filter
│  │  ├─ format.ts                # Intl time/size formatting
│  │  ├─ theme.ts
│  │  └─ types.ts
│  └─ styles/
│     ├─ tokens.css
│     └─ app.css
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  ├─ capabilities/
│  │  ├─ palette.json
│  │  └─ settings.json
│  ├─ migrations/
│  │  ├─ 0001_init.sql
│  │  ├─ 0002_fts.sql
│  │  └─ 0003_indexes.sql
│  ├─ icons/
│  └─ src/
│     ├─ main.rs
│     ├─ lib.rs                   # builder, plugins, setup
│     ├─ state.rs                 # AppState, StoreHandle, SettingsState
│     ├─ commands/
│     │  ├─ mod.rs
│     │  ├─ history.rs            # list, search, pin, delete, clear
│     │  ├─ paste.rs              # paste/copy/plain modes
│     │  ├─ settings.rs
│     │  ├─ window.rs
│     │  └─ license.rs            # trial, unlock, activation, entitlement
│     ├─ capture/
│     │  ├─ mod.rs
│     │  ├─ pipeline.rs           # gate → canonicalize → dedupe → store
│     │  ├─ suppress.rs           # echo suppression
│     │  └─ source_app.rs
│     ├─ store/
│     │  ├─ mod.rs                # StoreActor
│     │  ├─ db.rs                 # connection, pragmas
│     │  ├─ items.rs
│     │  ├─ fts.rs
│     │  ├─ blobs.rs
│     │  ├─ retention.rs
│     │  └─ migrate.rs
│     ├─ search/
│     │  ├─ mod.rs
│     │  ├─ query.rs
│     │  ├─ rank.rs
│     │  └─ cache.rs
│     ├─ window/
│     │  ├─ mod.rs
│     │  ├─ palette.rs            # pre-warm, position, show/hide
│     │  ├─ settings.rs
│     │  └─ tray.rs
│     ├─ hotkey/
│     │  ├─ mod.rs                # backend selection
│     │  └─ accelerator.rs        # parse/validate/display
│     ├─ paste/
│     │  ├─ mod.rs                # orchestrator
│     │  └─ restore.rs
│     └─ license/
│         ├─ mod.rs               # AppEntitlement: trial | active | locked
│         ├─ trial.rs             # signed trial record, clock-rollback guard
│         ├─ verify.rs            # Ed25519 license verification
│         ├─ activate.rs          # optional 3-device activation client
│         └─ machine.rs           # stable device fingerprint
├─ crates/
│  ├─ copyfie-core/               # no Tauri, no platform crates
│  │  ├─ src/
│  │  │  ├─ model.rs
│  │  │  ├─ hash.rs
│  │  │  ├─ store/
│  │  │  ├─ search/
│  │  │  ├─ settings/
│  │  │  └─ retention.rs
│  │  └─ benches/                 # 10k/100k search, insert, dedupe
│  └─ copyfie-license/            # standalone verifier: key format, Ed25519, trial record
│     └─ src/lib.rs               # no OS APIs; unit-testable in isolation
├─ platform/
│  ├─ common/
│  │  └─ src/lib.rs               # traits + shared types
│  ├─ windows/
│  │  ├─ Cargo.toml
│  │  └─ src/{clipboard,formats,foreground,focus,input,permissions,dwm}.rs
│  ├─ macos/
│  │  ├─ Cargo.toml
│  │  └─ src/{pasteboard,concealed,workspace,focus,input,accessibility}.rs
│  └─ linux/
│     ├─ Cargo.toml
│     └─ src/
│        ├─ x11/{clipboard,active_window,focus,input}.rs
│        ├─ wayland/{data_control,portal,activation,capabilities}.rs
│        └─ tray.rs
├─ backend/                        # Paddle webhook + license issuance (§14)
│  ├─ wrangler.toml
│  ├─ src/{webhook,license,restore,revocations,reconcile}.ts
│  ├─ test/
│  └─ README.md
├─ site/                           # marketing + purchase pages (§12, §14)
│  ├─ index.html
│  ├─ pricing.html
│  ├─ buy.html                     # Paddle hosted checkout entry
│  ├─ welcome.html                 # key display + copyfie://activate
│  ├─ restore.html
│  ├─ download.html
│  └─ legal/{terms,privacy,refund,eula}.md
├─ scripts/
│  ├─ perf/{open.ps1,open.sh,rss.py,seed_10k.rs}
│  ├─ bundle/{verify_sizes.py,macos_sign.sh,windows_sign.ps1}
│  ├─ install/{install.ps1,install.sh,uninstall.sh}   # one-line install (§12)
│  ├─ license/{cargo-deny.toml,gen_notices.sh,keygen.rs,issue_key.rs,verify_key.rs}
│  └─ release/{bump_version.sh,generate_latest_json.sh,checksums.sh}
├─ docs/
│  ├─ ARCHITECTURE.md
│  ├─ PLATFORMS.md
│  ├─ PRIVACY.md
│  ├─ PERF.md
│  ├─ ATTRIBUTION.md
│  ├─ LICENSING.md                 # trial, purchase, activation, refunds, payouts
│  └─ TESTING.md
├─ tests/
│  ├─ integration/
│  └─ fixtures/
├─ .github/workflows/
├─ LICENSE                         # Proprietary, all rights reserved (EULA gate)
├─ EULA.md
├─ NOTICE
├─ THIRD_PARTY.md
└─ Cargo.toml                      # workspace
```

**Why `platform/` is outside `src-tauri`:** it keeps OS code testable without spinning up Tauri, and it makes the “no OS APIs in core” rule enforceable in CI. `src-tauri` wires the crates together.

**Licensing boundary:** `license/` (plus the small `copyfie-license` verifier crate) is the only code that decides entitlement. Commands ask `AppEntitlement` for `trial | active | locked`; they do not scatter license checks. The module has no clipboard, store, or platform dependencies, so it is unit-testable and replaceable. There is **no separate Pro process, no IPC, and no second build** — the single binary contains every feature. Trial expiry only filters what the palette exposes (§14); capture and storage stay unconditional.

**Closed-source note:** the repository is private and the shipped binary is proprietary. The `copyfie-license` verifier is an internal crate, not a separately published open-source contract. There is no open-source license obligation created by our own code.

---

# 5. Data model

## SQLite setup

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA temp_store = MEMORY;
PRAGMA cache_size = -8192;          -- 8 MB
PRAGMA mmap_size = 67108864;        -- 64 MB virtual mapping
PRAGMA wal_autocheckpoint = 256;
PRAGMA secure_delete = ON;          -- deleted text is zeroed
PRAGMA auto_vacuum = INCREMENTAL;   -- set before first table; one VACUUM after
```

`auto_vacuum` must be set before tables exist. On an existing DB, run `VACUUM` once in a migration.

## Tables

```sql
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE items (
  id                 INTEGER PRIMARY KEY,
  kind               INTEGER NOT NULL,        -- 0 text, 1 image, 2 files
  content_hash       BLOB NOT NULL,           -- BLAKE3-256
  text               TEXT NOT NULL DEFAULT '',-- text payload or JSON path array
  preview            TEXT NOT NULL,           -- first ~400 chars, display/search
  blob_name          TEXT,                    -- "<hex>.png" for images
  thumb_name         TEXT,                    -- "<hex>.png"
  byte_size          INTEGER NOT NULL DEFAULT 0,
  width              INTEGER,
  height             INTEGER,
  source_app_id      TEXT,                    -- exe path / bundle id / WM_CLASS
  source_app_name    TEXT,
  source_window_title TEXT,
  is_pinned          INTEGER NOT NULL DEFAULT 0,
  captured_at        INTEGER NOT NULL,        -- unix ms
  first_seen_at      INTEGER NOT NULL,
  last_used_at       INTEGER,
  use_count          INTEGER NOT NULL DEFAULT 0,
  expires_at         INTEGER,                 -- NULL = never
  ocr_text           TEXT,                    -- populated by the OCR feature (v1.1)
  CHECK (kind IN (0,1,2))
);

CREATE UNIQUE INDEX items_kind_hash ON items(kind, content_hash);
CREATE INDEX items_captured ON items(captured_at DESC);
CREATE INDEX items_pinned_captured ON items(is_pinned DESC, captured_at DESC);
CREATE INDEX items_expiry ON items(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX items_source ON items(source_app_id);
```

### Why `content_hash` is a BLOB

BLAKE3 output is 32 bytes. Storing raw bytes halves index size versus hex and avoids case issues. Use hex only in filenames.

### Kind values

- `0` text
- `1` image
- `2` files (JSON array in `text`, normalized paths)

Do not store rich text/HTML in v1, per decision.

## FTS5

```sql
CREATE VIRTUAL TABLE items_fts USING fts5(
  text,
  preview,
  source_app_name,
  content='items',
  content_rowid='id',
  tokenize="unicode61 remove_diacritics 2",
  prefix='2 3'
);
```

Triggers keep it in sync:

```sql
CREATE TRIGGER items_fts_ai AFTER INSERT ON items BEGIN
  INSERT INTO items_fts(rowid, text, preview, source_app_name)
  VALUES (new.id, new.text, new.preview, COALESCE(new.source_app_name,''));
END;

CREATE TRIGGER items_fts_ad AFTER DELETE ON items BEGIN
  INSERT INTO items_fts(items_fts, rowid, text, preview, source_app_name)
  VALUES ('delete', old.id, old.text, old.preview, COALESCE(old.source_app_name,''));
END;

CREATE TRIGGER items_fts_au AFTER UPDATE OF text, preview, source_app_name ON items BEGIN
  INSERT INTO items_fts(items_fts, rowid, text, preview, source_app_name)
  VALUES ('delete', old.id, old.text, old.preview, COALESCE(old.source_app_name,''));
  INSERT INTO items_fts(rowid, text, preview, source_app_name)
  VALUES (new.id, new.text, new.preview, COALESCE(new.source_app_name,''));
END;
```

- External-content FTS5 avoids duplicating content.
- `unicode61` handles Unicode word tokenization; `prefix='2 3'` makes 2- and 3-character prefix queries fast.
- Fuzzy/substring search is **not** delegated to FTS5; see §5.4.
- If OCR ships, either extend this table and rebuild, or use a separate `items_ocr_fts` table to avoid rebuilding the main index when OCR is enabled. Prefer a separate table.

## Image storage: files, not BLOBs

**Decision: files on disk.**

```
<data_dir>/copyfie/
  clipboard.db
  clipboard.db-wal
  images/
    <hash>.png
    .tmp/<random>.png
  thumbs/
    <hash>.png
  settings.json
  logs/
```

Reasons:

- A 40 MB image in SQLite bloats the WAL, page cache, and every DB copy.
- Deletes require `VACUUM` to reclaim space; file delete is immediate.
- Random access to thumbnails is a small file read; WebView can cache it via a custom protocol.
- Future sync can move content-addressed blobs without exporting a DB.

Pipeline:

1. Read RGBA + dimensions; reject `> 64 MB` raw or `> 50 MP` unless downscaled.
2. Hash **raw RGBA + width + height** with BLAKE3 (dedupe identical pixels even if encoders differ).
3. If hash exists: bump `captured_at`, no encode.
4. Else encode on a background thread:
   - Cap the longest side at 2560 px (configurable later).
   - Store PNG with `CompressionType::Fast` (alpha-safe). PNG is compressed; avoid lossy JPEG artifacts on screenshots.
   - Thumbnail: 256 px longest side, PNG.
5. Write to `images/.tmp/`, `fsync`, rename into place.
6. Insert row; if insert fails, remove the new files.
7. Serve thumbnails through a custom Tauri URI protocol, e.g. `copyfie://thumb/<hash>` with strict hash validation and `Cache-Control: immutable`. **Do not base64 images over IPC** (Ortu does this; it is wasteful).

## Dedupe

```sql
INSERT INTO items (kind, content_hash, text, preview, blob_name, thumb_name,
                   byte_size, width, height, source_app_id, source_app_name,
                   captured_at, first_seen_at, expires_at)
VALUES (...)
ON CONFLICT(kind, content_hash) DO UPDATE SET
  captured_at      = excluded.captured_at,
  source_app_id    = excluded.source_app_id,
  source_app_name  = excluded.source_app_name,
  use_count        = 0,
  expires_at       = CASE WHEN items.is_pinned = 1 THEN NULL ELSE excluded.expires_at END
RETURNING id, (first_seen_at = excluded.captured_at) AS is_new;
```

- Recopy moves the item to the top but preserves pin and existing payload.
- `is_new` is derived from `first_seen_at`; no second SELECT.
- Normalize text before hashing: CRLF → LF, strip trailing NUL. Store the normalized text.
- Normalize file paths: trim trailing separators, sort for hashing, preserve display order separately if needed.
- Hash collision is astronomically unlikely; if a hash exists with different bytes (verify on insert if paranoid), log and drop.

## Expiry and retention

- `expires_at = captured_at + retention_days`, computed on insert from the user setting. Default **30 days**; options 7/30/90/365 days or **Forever** (`expires_at = NULL`).
- There is **no plan-based item cap** and no hidden retention limit. If the user chooses Forever, nothing is deleted by age. The settings UI shows the item count and warns above 50,000 items; it never silently truncates.
- Pinned: `expires_at = NULL`.
- When unpinning: recompute `expires_at = captured_at + retention_days`; delete immediately if already past.
- Cleanup:
  ```sql
  DELETE FROM items
  WHERE is_pinned = 0 AND expires_at IS NOT NULL AND expires_at <= ?now;
  ```
- Run at startup, then every 5 minutes, and after each write only if the “buffered cleanup” threshold is crossed (Ropy’s idea: don’t scan on every insert).
- Blob GC:
  - Build a set of referenced `blob_name`/`thumb_name`.
  - Remove files not in the set, plus `.tmp` files older than 1 hour.
  - Guard with the store actor lock; never run concurrently with capture commit.
- Image disk cap: default 1 GB, user-adjustable up to 20 GB. When exceeded, evict the oldest **unpinned** image items (row + blobs) and show one non-blocking footer notice; never evict pinned images. Document this in settings next to the cap.
- `VACUUM` after “Clear all”, and optionally monthly via `PRAGMA incremental_vacuum`.

## Migrations

- Use `PRAGMA user_version`.
- `migrations/NNNN_*.sql` files, `include_str!`, applied in a transaction.
- Each migration is forward-only. Never edit a shipped migration.
- On startup: if `user_version` is newer than the binary supports, refuse to write and show a “downgrade not supported” dialog rather than corrupting data.
- Back up the DB once before the first migration of a major version (`clipboard.db.bak`).
- FTS5: create/rebuild in `0002_fts.sql`; set `meta.fts_built=1`. If a future SQLite build lacks FTS5, fail fast at startup rather than silently degrading (we bundle SQLite with FTS5).

## Settings and blocklist

Use a JSON file, not the DB:

```json
{
  "version": 1,
  "hotkey": "CommandOrControl+Shift+Space",
  "theme": "system",
  "accent": "#5B8DEF",
  "retention_days": 30,
  "launch_at_login": true,
  "skip_concealed": true,
  "paste_mode": "paste",
  "show_source_app": true,
  "max_image_cache_mb": 1024,
  "capture_paused": false,
  "blocklist": [
    { "platform": "windows", "id": "C:\\Program Files\\KeePassXC\\KeePassXC.exe", "label": "KeePassXC" },
    { "platform": "macos", "id": "com.1password.1password", "label": "1Password" },
    { "platform": "linux", "id": "keepassxc", "label": "KeePassXC" }
  ]
}
```

- Atomic write: temp file, `fsync`, rename, keep `.bak`.
- On parse error: load defaults into memory, show a banner, **do not overwrite** the broken file (Ropy’s recovery behavior).
- Clamp values (`retention_days` 1–365; `max_image_cache_mb` 128–20480; accent validated hex). `retention_days = 0` is the internal “Forever” sentinel, shown as “Forever” in the UI.
- First run: attempt to register `CommandOrControl+Shift+Space`. If it is already taken, show a blocking setup step that lets the user record a replacement before the palette is considered ready; never silently substitute a different hotkey.
- Unknown fields are preserved where practical, so a future-version round-trip does not destroy settings.
- Blocklist matching:
  - Windows: case-insensitive full path or basename.
  - macOS: bundle identifier.
  - Linux X11: `WM_CLASS` or `/proc/<pid>/exe` basename.
  - Wayland: **not reliably available**; document that the blocklist cannot be enforced for native Wayland apps.

## Trial and license state

Trial and license data live in a dedicated directory, not in the clipboard DB:

```
<data_dir>/copyfie/
  license/
    trial.json        # signed trial-start record + last-seen anchor
    license.key       # the user's Ed25519 license key (0600)
    activation.json   # optional activation result / seat cache (0600)
```

Signed trial record:

```json
{
  "v": 1,
  "product": "copyfie",
  "install_id": "<random 128-bit>",
  "started_at": 1767225600000,
  "days": 14,
  "version": "1.0.0",
  "max_seen_at": 1767225600000,
  "last_reminder_day": 0,
  "sig": "<base64 Ed25519 signature>"
}
```

- **Calendar-based:** the trial ends at `started_at + 14 × 86_400_000` in wall-clock time. It is not 14 days of app usage and does not pause while the app is closed.
- Signed with a build-embedded Ed25519 key at first launch. This is **obfuscation plus tamper evidence**, not a cryptographic guarantee: the signing key is inside the binary and can be extracted. It stops casual editing of the JSON, nothing more.
- Anchors are duplicated so one deleted file does not reset the trial, and any missing anchor is rewritten on the next launch:
  - `trial.json` in the app data directory.
  - `meta` row `trial_started_at` in `clipboard.db`.
  - A platform anchor: `HKCU\Software\Copyfie` value on Windows, Keychain item on macOS (fallback file if Keychain is unavailable), and `~/.config/copyfie/trial` on Linux.
- **Clock-rollback detection:** persist `max_seen_at` on every launch, every 6 hours while running, and on clean exit. Effective time is `max(now, max_seen_at)`. A backwards jump of more than 5 minutes is recorded in `clock_anomalies` and never extends the trial — a rollback can at most freeze the remaining days, never add them.
- Trial = `started_at + 14 days` using effective time. Expiry is evaluated at startup, on palette open, hourly while running, and after sleep/wake; it never runs on a per-keystroke path.
- **Tamper resistance, stated honestly:** signed record + three independent anchors + monotonic clock. Deleting every anchor and the database resets the trial; accept that leakage and document it. No machine fingerprinting and no phone-home — both would contradict the privacy promise for a temporary trial.
- **Reminders:** days **10, 13, and 14** of the trial, computed from `started_at`. At most one per calendar day, dismissible, non-blocking, and shown in the footer area — never during an active paste or search. `last_reminder_day` prevents repeats; day 14 reads “Last day”.
- **History is unaffected by the trial.** Retention remains the user’s setting (7/30/90/365/Forever); the trial never shortens it.
- **Limited-mode count grows with a longer trial.** After 14 days more items have accumulated, so the “N more items waiting” line is larger and more persuasive. Compute it from the real number of hidden text items (total text items − 25); never hard-code 25.
- License key is a single text string (`COPYFIE-…`, base64url payload + Ed25519 signature) that the user pastes into the unlock panel. Store it in `license.key` with `0600`; optionally mirror the key itself to Keychain/Credential Manager later.
- **Reinstall:** the license key is re-entered and re-validated offline; license validity does not depend on install identity. Deleting *all* app data and every anchor resets the trial — unavoidable without an account. Document this honestly in `docs/LICENSING.md`; do not pretend the trial is unbreakable.
- No clipboard content, window titles, or history is stored in this directory.

---

# 6. Per-OS strategy

| Concern | Windows | macOS | Linux X11 (v1.0) | Linux Wayland (v1.x) |
|---|---|---|---|---|
| Clipboard watch | `WM_CLIPBOARDUPDATE` via `clipboard-rs`/`clipboard-win`; fallback 500 ms polling if listener registration fails | `NSPasteboard.changeCount` poll every 400 ms; read only on change | XFixes selection notify (event-driven) | `wlr-data-control`/`ext-data-control` MIME poll 700 ms; compositor must support it |
| Change detection | Event | Cheap integer poll | Event | Sorted MIME list + text comparison |
| Payload priority | files → image → text | files (`public.file-url`/`NSFilenamesPboardType`) → image → text | files (`text/uri-list`) → image → text | files → image → text |
| Concealed/password-manager detection | `ExcludeClipboardContentFromMonitorProcessing`, `ClipboardViewerIgnore`, `CanIncludeInClipboardHistory=0`, `CanUploadToCloudClipboard=0`, `KeePassClipFormat`, `com.bitwarden.concealed` | `org.nspasteboard.ConcealedType`, `TransientType`, `AutoGeneratedType`, `com.agilebits.onepassword`, `de.petermaurer.TransientPasteboardType`, `com.typeit4me.clipping`, `Pasteboard generator type`, `application/x-nspasteboard-concealed-type` | `x-kde-passwordManagerHint=secret` (KeePassXC, KWallet) | No standard; KDE may expose the hint; GNOME does not |
| Source app | `GetForegroundWindow` → PID → `QueryFullProcessImageNameW`; friendly name cache | `NSWorkspace.frontmostApplication` or `org.nspasteboard.source` | `_NET_ACTIVE_WINDOW` → `_NET_WM_PID` → `/proc/<pid>/comm`; `WM_CLASS` fallback | **Not reliably available**; store “Unknown” |
| Global hotkey | `RegisterHotKey` via `global-hotkey`; first-run conflict check + rebind | Carbon `RegisterEventHotKey` via `global-hotkey`; first-run conflict check + rebind | `XGrabKey` via `global-hotkey`; NumLock/CapsLock handling; rebind | XDG GlobalShortcuts portal (`ashpd`), Hyprland IPC binding, or manual `copyfie --show` |
| Focus restore | Store HWND+PID; `SetForegroundWindow`; poll until active; `AttachThreadInput` fallback | Store `NSRunningApplication` PID; activate after hiding; poll `isActive` | Store `_NET_ACTIVE_WINDOW`; send EWMH activate message; poll; `XSetInputFocus` fallback | Compositor decides; activation token required; often copy-only |
| Paste simulation | `SendInput` Ctrl+V with scancodes; release held modifiers first; detect elevated target | `CGEvent` Cmd+V with **keycode 9**, not `Key::Unicode('v')`; requires Accessibility | enigo XTEST Ctrl+V; release modifiers | enigo virtual-keyboard protocol; compositor must support it; else copy-only |
| Permissions | None; SmartScreen/UAC only | Accessibility (paste); Input Monitoring not normally needed | None; XTEST required | Portal approval for shortcuts; no permission model for global focus |
| Tray/autostart | Tray; HKCU Run key, repair stale path | Tray/menu bar; LaunchAgent with `--hidden`; Accessory policy | AppIndicator/KSNI; `~/.config/autostart/*.desktop` | Tray support spotty; autostart via portal/`.desktop` |
| Biggest risk | Hotkey conflicts (default handled by first-run check); focus theft; elevated targets; RDP clipboard | Accessibility trust differs per install identity/path; secure input; hidden WebView throttling | WM differences; XTEST unavailable; tray absent | Deferred to v1.x; GNOME unsupported; no layer shell; portal/focus/paste vary by compositor |

## Riskiest implementation details

1. **Global hotkey conflicts.** The selected default is `CommandOrControl+Shift+Space`. It is less collision-prone than `Ctrl+Shift+V` (plain-text paste) or `Ctrl+Alt+V` (Paste Special in Office), but it can still be taken by an IME, launcher, or app. Do a first-run registration test and provide a one-step rebind flow. Never silently fall back to another chord.
2. **macOS Accessibility.** Permission is per app identity and path. A dev build, an ad-hoc signed build, and an installed `.app` are different trust entries. Detect with `AXIsProcessTrusted`, deep-link to Privacy → Accessibility, and **fall back to copy-only** rather than failing silently.
3. **macOS `enigo` crash.** Ortu documented a release-only SIGTRAP from `Key::Unicode('v')` on a tokio worker thread. Use the fixed virtual keycode (`Key::Other(9)`) or direct `CGEvent`. Do not repeat Ropy’s `Key::Unicode('v')` path on macOS.
4. **Focus restore is a race.** Hiding the palette, activating the target, writing the clipboard, and sending the key must be ordered. Poll for target activation with a bounded timeout; if the wrong window is foreground, abort paste and leave the item on the clipboard.
5. **Windows elevated windows.** UIPI blocks synthetic input from a medium-integrity process to a high-integrity window. Detect likely elevation and show “Copied — paste into the elevated app manually.”
6. **Wayland (v1.x later).** Capture, shortcut, focus, and paste are four independent capabilities. A working window does not imply any of them. GNOME does not expose data-control, so background capture is impossible without a Shell extension. In v1.0, keep the capability notice and copy-only fallback; do not promise full Wayland support.
7. **Wayland window placement (v1.x later).** No global positioning and no layer-shell in Tauri/GTK. Tiling compositors will tile the palette unless the user adds a float rule. Document this when Wayland support ships.
8. **Flatpak (v1.x later).** Background clipboard monitoring is likely blocked by the sandbox. Validate before promising Flatpak parity; deb/rpm remain the primary Linux artifacts.
9. **Windows clipboard held by another app.** `OpenClipboard` can fail transiently. Retry 3× with 10 ms backoff; never block the UI.
10. **Secure input on macOS.** If `IsSecureEventInputEnabled()` is true, synthetic paste may be blocked. Skip paste and copy only.

## Platform notes that change the design

- **Use `clipboard-rs`, not `arboard`, on Linux.** Ortu’s `arboard` path is X11-centric and its Wayland issue (#14) is real. `clipboard-rs` has a `wayland` feature we will use when Wayland support ships in v1.x.
- **Do not use Tauri’s global-shortcut plugin on Wayland.** It wraps `global-hotkey`/X11. Use it on Windows/macOS/X11; use `ashpd` when Wayland support ships in v1.x.
- **v1.0 Linux is X11-only.** If `WAYLAND_DISPLAY`/`XDG_SESSION_TYPE=wayland` is detected, show a capability notice (“Wayland support is coming; copy-only fallback”) and keep the copy-only path working. Do not partially register X11 hotkeys or claim capture works.
- **Do not create the palette lazily.** Ortu creates it on first hotkey, which cannot meet 100 ms. Pre-create hidden.
- **Do not rely on focus returning automatically.** Ropy hides the window and waits 50 ms; that works sometimes but is not deterministic. Restore explicitly.

---

# 7. Frontend plan

## Stack choices

- **Svelte 5 + Vite + TypeScript**, plain Svelte, no SvelteKit. Two HTML entry points: `index.html` (palette) and `settings.html`.
- **No Tailwind.** Plain CSS with design tokens. This keeps CSS small, avoids purge/config overhead, and gives precise control over the palette’s paint path. Ortu’s Tailwind is fine but adds build complexity for little benefit here.
- **No icon library.** Hand-pick ~12 inline SVG icons (~5 KB total) instead of `lucide-svelte`.
- **No date library.** Store Unix ms; format with `Intl.DateTimeFormat` / `Intl.RelativeTimeFormat`.
- **No state library.** Svelte runes + a typed IPC module.

## Components

| Component | Responsibility |
|---|---|
| `palette/App.svelte` | Shell, open/reset lifecycle, keyboard listener, focus management. |
| `SearchBar.svelte` | Input, IME composition guard, clear button, query state. |
| `FilterBar.svelte` | All / Text / Images / Files / Pinned chips; Tab cycles. |
| `ResultList.svelte` | Renders ≤50 rows, scroll selected into view, empty/loading states. |
| `ResultRow.svelte` | Type icon, 2-line preview, image thumb, source app, relative time, pin, shortcut number. |
| `Footer.svelte` | Key hints; platform-aware `⌘` vs `Ctrl`; the single trial/unlock footer line. |
| `settings/App.svelte` | Tabbed settings shell. |
| `HotkeyRecorder.svelte` | Captures modifiers correctly (including Win/Super); validates conflicts before saving. |
| `FirstRunHotkey.svelte` | Blocking first-run check: verifies `CommandOrControl+Shift+Space`, offers a one-step rebind if taken, links to settings. |
| `Blocklist.svelte` | List, add current app, remove. |
| `General.svelte` | Theme, accent, history length, image cap, launch at login, pause capture, updates. |
| `About.svelte` | Version, license status, attribution, open-source notices, diagnostics. |
| `UnlockPanel.svelte` | License key entry, trial status, purchase link, activation state. Always reachable, even when settings are locked. |
| `LockedHistory.svelte` | Dimmed “N more items waiting, unlock to see them” list row with an Unlock action. One line, no dialog. |

## State model

```ts
type PaletteState = {
  openId: number;
  query: string;
  filter: "all" | "text" | "image" | "files" | "pinned";
  items: ItemSummary[];
  selected: number;
  status: "idle" | "searching" | "error";
  requestId: number;
  warm: boolean;
  entitlement: "trial" | "active" | "locked";
  lockedCount: number;          // older items hidden in limited mode
};
```

- `items` is replaced atomically per response; never mutate in place.
- `selected` is clamped on every result change.
- A stale response is ignored by `requestId`.
- Settings are loaded once at prewarm and cached in `localStorage` for instant theme paint; reconciled with Rust after open.

## Keyboard handling

Use one window-level `keydown` listener in capture phase:

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection; wrap at ends. Also `Ctrl+P` / `Ctrl+N`. |
| `PageUp` / `PageDown` | Jump 8 rows. |
| `Home` / `End` | First/last. |
| `Enter` | Paste selected item. |
| `Shift+Enter` | Paste as plain text (text items); for images/files, copy path/text representation; show toast if not applicable. |
| `1`–`9` | Quick paste by position **only when the query is empty**; `Cmd/Ctrl+1–9` always works. |
| `Esc` | If a filter/panel is open, back out; otherwise close. |
| `Tab` / `Shift+Tab` | Cycle type filters. |
| `Cmd/Ctrl+P` | Pin/unpin selected. |
| `Cmd/Ctrl+Backspace` | Delete selected (with confirm for pinned). |
| `Cmd/Ctrl+,` | Open settings. |
| `Cmd/Ctrl+B` | Block the source app of the selected item. |
| `Space` | Optional preview (hold); defer to v1.1 to keep paint fast. |

Rules:

- Ignore shortcuts while `event.isComposing` or during IME composition.
- Do not intercept plain typing in the search input except the navigation keys above.
- On open, reset `query`, `filter`, `selected=0`, focus the input in the same frame.
- On blur, hide after a short guard unless a native menu/dialog is open.

## Limited mode (after trial)

Entitlement is `trial`, `active`, or `locked`. Rust returns it with every `palette:open`; the UI never guesses.

**During trial:** all features, full history, and a single compact footer line: “Trial · N days left”. Clicking it opens the unlock panel.

### Trial reminders

| Day | Treatment |
|---|---|
| 0 | Footer reads “Trial started · 14 days left”. No toast, no modal. |
| 1–9 | Footer countdown only: “Trial · N days left”. No banner. |
| 10 | First gentle reminder, dismissible: “4 days left in your trial”. |
| 13 | Second reminder: “Tomorrow is the last day”. |
| 14 | Final reminder: “Last day · Unlock Copyfie”. |

- One reminder **per calendar day at most**, tracked by `last_reminder_day`; dismissing one suppresses it for the rest of that day.
- Reminders never block: no modal, no overlay, no focus steal, no interruption of a paste or an open palette. They replace the footer line, they do not stack.
- The day-14 state is not a lockout: capture and storage keep running, and the palette switches to limited mode only when the trial clock actually expires.
- The footer countdown is the only always-present trial UI; the reminder is the same line with warmer copy, not a second element.

**Locked:** capture and storage continue exactly as before. Only presentation and certain actions change:

| Area | Locked behavior |
|---|---|
| Result list | Renders the **25 most recent text items** only. Images, files, and pinned items are not listed. |
| Older items | One dimmed placeholder row at the end of the list: “**N more items waiting, unlock to see them**”. `Enter` on it (or clicking) opens the unlock panel. This is a row, not a dialog. `N` is the real number of hidden text items, so a 14-day trial produces a noticeably larger count. |
| Search | Local substring/fuzzy filter over the 25 visible items only. No FTS query, no full-history search. Placeholder: “Search recent 25 · unlock for everything”. |
| Filters | Only **All** and **Text** are active. Images / Files / Pinned chips are visibly disabled. |
| Paste / copy | Works normally on the 25 visible items, including `Enter`, `Shift+Enter`, and `1–9`. |
| Pin, delete, block-app | Disabled (`Cmd/Ctrl+P`, `Cmd/Ctrl+Backspace`, `Cmd/Ctrl+B`). |
| Settings | Opens in a locked state. **License**, **About**, **Privacy**, **Quit**, and **Clear history** work; all other settings are visible but disabled with an “Unlock to change settings” note. History length, hotkey, blocklist, and theme cannot be changed. |
| Clear history | **Allowed**, with a confirmation dialog. In locked mode it clears **all** items including pinned ones (pins are not manageable while locked); in licensed mode it keeps pinned items by default. |
| Snippets / OCR / custom themes / sync | Not available. Their entry points are hidden, not shown as broken. |
| Footer | A single line: “**N more items waiting** · Unlock”. No popups, no modal dialogs, no overlays. |

- **Purchase unlock is instant:** successful verification emits `entitlement:changed`; the palette reloads the full summary cache and the banner disappears without a restart.
- **Expiry is quiet:** if the trial ends while the app is running, nothing interrupts the current paste; the next palette open renders limited mode.
- **Nothing is deleted at expiry.** The retention setting chosen during trial keeps applying.
- The unlock panel is a settings pane, not a separate window, and is reachable from the footer and the About/License tab.

## Window pre-warming (the 100 ms path)

1. At app startup, create the palette window **hidden**:
   - `visible: false`, `focus: false`, `decorations: false`, `resizable: false`, `always_on_top: true`, `skip_taskbar: true`, `shadow: true`.
   - `transparent` on macOS/Linux; **opaque on Windows** with DWM rounded corners (Ortu’s finding: transparency causes corner artifacts/rendering issues on Windows).
   - Set the native background color to the theme background to avoid a white flash.
2. Load the frontend and wait for `palette:ready`.
3. Load the first 50 summaries, settings, theme, and precompute relative-time labels. Keep the webview renderer alive.
4. On Windows, add browser args to prevent renderer throttling:
   `--disable-background-timer-throttling --disable-renderer-backgrounding --disable-backgrounding-occluded-windows`.
5. On hotkey:
   - Query the monitor under the cursor, compute position **before** showing.
   - `show()`, `set_focus()`, activate the app on macOS.
   - Emit `palette:open` with an `openId`; the frontend resets and renders from cache.
   - Emit `palette:shown` after the first `requestAnimationFrame` for measurement.
6. If hotkey is pressed before `palette:ready`, show with a skeleton and let the ready handler fill it.

**Fallback if hidden webviews are throttled:** during prewarm, show the window off-screen for one frame, then move it off-screen/hide. This risks appearing in screenshots/alt-tab; test on each OS before adopting. Prefer the hidden path.

## Theming

- CSS variables: `--bg`, `--bg-elevated`, `--fg`, `--fg-muted`, `--border`, `--accent`, `--row-hover`, `--row-selected`, `--danger`.
- `data-theme="light|dark"` on `<html>`, set by an inline script from `localStorage` before first paint; reconcile with Rust.
- Follow system via `prefers-color-scheme` and Tauri theme events.
- One accent color from settings; validate contrast.
- No blur/vibrancy in v1. `backdrop-filter` is inconsistent and can cost paint time. Use a near-opaque solid surface.
- Windows: DWM caption color for the settings window; palette is frameless with `DWMWA_WINDOW_CORNER_PREFERENCE=ROUND`.
- macOS: settings window uses native titlebar with `setTitlebarAppearsTransparent` and matching appearance; palette is a borderless floating panel.

## Accessibility and polish

- Roles: `combobox` input, `listbox` results, `option` rows; `aria-activedescendant`.
- Announce “N results” on change.
- Visible focus ring; do not remove outlines.
- Respect reduced motion.
- High-contrast token set; test with Windows High Contrast and macOS Increase Contrast.
- No layout shift on image load: fixed 40×40 thumb box, `loading="lazy"`, `decoding="async"`.

---

# 8. Performance plan

## Targets and reality check

| Target | Mechanism | How measured | Realistic expectation |
|---|---|---|---|
| Installer < 10 MB | Tauri uses system WebView; `lto=fat`, `codegen-units=1`, `panic=abort`, `strip`, minimal features; no bundled fonts/icons | `scripts/bundle/verify_sizes.py` on release artifacts | v1.0: NSIS ~4–5.5 MB, deb/rpm ~6–7.5 MB, per-arch DMG ~6–8 MB. MSI/AppImage/Flatpak are v1.x; AppImage would be ~70–90 MB. |
| Core RSS < 40 MB; total 60–100 MB | Bounded DB/cache/image budgets; pre-warmed palette retained | RSS/working set per process **and** total process tree; 10-min idle; M1 measurement | Core likely 20–35 MB; total with WebView2/WKWebView/WebKitGTK 60–100 MB. **Decision gate if total >120 MB.** |
| Palette opens < 100 ms | Pre-created hidden window; cached first page; position before show; no DB read on open | Rust `Instant` at hotkey → frontend `performance.mark` at first rAF | 30–80 ms p95 when warm. Cold first open 250–700 ms. |
| Search 10k+ instant | FTS5 + in-memory summary scan; 50-result cap; 30–60 ms debounce; stale request cancellation | Criterion bench + seeded 10k DB + UI timing | FTS5 <5 ms; local filter 1–3 ms; total <50 ms p95. |
| Idle CPU ~0 | Event-driven Windows/X11; changeCount poll on macOS; MIME poll on Wayland (v1.x); no UI timers when hidden; no daily updater in v1.0 | 10-min sample; `Get-Counter` / `pidstat` | <0.1% average. |

## How each is achieved

### Size

- `rusqlite` bundled with FTS5 is the biggest single add (~0.6–0.9 MB compressed) and is non-negotiable.
- `image` only `png` + `jpeg`; no WebP/AVIF/TIFF codecs.
- No `chrono`/`time` in Rust; timestamps are `i64` ms and formatted in JS.
- No `toml` crate; JSON settings.
- No `tokio` “full”; Tauri supplies its runtime. Use `tokio` only if needed.
- Avoid duplicate clipboard crates (`clipboard-rs` only).
- Frontend: no Tailwind, no icon library, no router; target <50 KB gzip JS/CSS.
- Release profile:
  ```toml
  [profile.release]
  opt-level = "s"        # benchmark "z"/"3"; pick based on search results
  lto = "fat"
  codegen-units = 1
  panic = "abort"
  strip = "symbols"
  ```
  Ropy found global `opt-level=2/z` badly hurt search; benchmark before choosing. Consider per-package optimization: `3` for `copyfie-core`/search, `s` for the rest.
- Run `cargo bloat --release --crates` and `cargo llvm-lines` monthly; fail CI if a single dependency adds >300 KB unexpectedly.
- `cargo-dist`-style artifact size recording is not needed; a simple script + release JSON is enough.

### RAM

- **Budget:** core process private working set **< 40 MB**; total process tree **60–100 MB expected**, with a hard **decision gate at > 120 MB**.
- Rust core: 20–35 MB target (DB cache 8 MB, summary cache 2–5 MB, image decode transient).
- Summary cache cap: store first 256 chars per item, not full text, for 10k items (~3 MB). Full text stays in SQLite; FTS handles deep matches.
- Image thumbnails: lazy, 256 px, WebView cache; never keep all thumbnails in Rust memory.
- Decode full images only on paste; cache at most one decoded image (or zero).
- Drop the settings window on close; only the palette webview is resident.
- **Milestone 1 measurement task:** record per-process and total RSS on Windows, macOS, and Linux X11 after 10 minutes idle, with 10k seeded items and the palette pre-warmed. Store results in `docs/perf/`.
- **Decision gate:** if total RSS exceeds 120 MB on any target, choose one before M1 exit: tighten caches, disable pre-warm for that OS, or formally raise the total budget. Do not silently ship a regression.

### Open time

- Pre-warm is mandatory.
- Position before `show()`; do not move a visible window (Ortu’s fix).
- Render cached rows; do not await IPC in the open handler.
- Avoid font loading, layout thrash, and image decoding on first paint.
- Keep the palette DOM small: 50 rows max, fixed row height, CSS containment.
- Measure with an `open_metrics` log line (no content): `hotkey_to_show_us`, `show_to_first_paint_us`, `items_rendered`.

### Search

- **Instant layer:** on each keystroke, filter the in-memory summaries with a lowercase substring/subsequence scan. Return first 50 sorted by pinned, match quality, recency. This is synchronous and <3 ms for 10k.
- **Authoritative layer:** after 40–60 ms idle, run FTS5:
  ```sql
  SELECT i.id
  FROM items_fts f
  JOIN items i ON i.id = f.rowid
  WHERE items_fts MATCH ?1
    AND (?2 IS NULL OR i.kind = ?2)
  ORDER BY i.is_pinned DESC, bm25(items_fts, 8.0, 3.0, 1.0), i.captured_at DESC
  LIMIT 50;
  ```
- Merge: FTS results replace local results only if the request is current and the query has not changed.
- If FTS returns nothing and the query looks like a substring (e.g., `ello`), keep local fuzzy matches.
- Escape the MATCH query: quote each token, escape `"`, append `*`. Never interpolate raw user input.
- Cap query length at 200 chars; cap results at 100 for IPC safety.
- Benchmark with 1k/10k/100k seeded rows; record p50/p95 in `docs/PERF.md`.
- OCR and very large histories (100k+) may require trigram FTS or a disk-backed index; measure at 100k before shipping OCR.

### Idle CPU

- Windows: `WM_CLIPBOARDUPDATE`, zero polling.
- macOS: `changeCount` read every 400 ms; no payload read unless changed. This is far cheaper than Ortu’s 350 ms full read.
- X11: XFixes events.
- Wayland: MIME list poll every 700 ms only when Wayland support ships (v1.x).
- No `setInterval` in the hidden webview; relative-time labels update only on open.
- v1.0 has no auto-updater or daily network check. v1.x adds an opt-out daily check; do not run it before then.
- Retention cleanup every 5 min is a single indexed DELETE; acceptable, or trigger on write threshold.
- Tray/menu only updates on demand.

### Measurement harness

- `scripts/perf/seed_10k.rs`: deterministic text/image/file fixtures.
- `scripts/perf/open.ps1` / `open.sh`: send hotkey, read the app’s metrics log, repeat 100×, report p50/p95.
- `scripts/perf/rss.py`: sample process tree RSS every second for 10 minutes on Windows, macOS, and Linux; report per-process and total. This is the Milestone 1 RAM gate.
- `scripts/perf/idle_cpu.py`: sample CPU over 10 minutes with the palette hidden.
- Criterion benches in `copyfie-core` for insert, dedupe, FTS query, local scan.
- Store results as JSON in `docs/perf/<version>-<os>-<arch>.json`; compare in CI on a self-hosted reference machine. GitHub-hosted runners are too noisy for absolute benchmarks; use them only for functional tests.

---

# 9. Dependency list

Size impact is **estimated compressed installer delta** unless noted. “Baseline” is a hello-world Tauri 2 app (~2.5 MB NSIS) and Ortu’s 3.65 MB NSIS build as a reference.

## Rust — core/app

| Crate | Purpose | License | Est. size | Notes |
|---|---|---|---|---|
| `tauri` 2 | App shell, webview, IPC | Apache‑2.0/MIT | 2.5–3.5 MB baseline | Disable unused features; no devtools in release. |
| `tauri-plugin-global-shortcut` | Hotkey on Win/mac/X11 | Apache‑2.0/MIT | +50–100 KB | Do not use on Wayland (v1.x). |
| `tauri-plugin-autostart` | Launch at login | Apache‑2.0/MIT | +30 KB | Repair stale entries on launch. |
| `tauri-plugin-single-instance` | One instance | Apache‑2.0/MIT | +20 KB | Forward `--show`. |
| `tauri-plugin-updater` | Signed updates | Apache‑2.0/MIT | +250–400 KB | **v1.x only.** Not in v1.0. |
| `tauri-plugin-process` | Relaunch | Apache‑2.0/MIT | +10 KB | v1.x, used by updater. |
| `tauri-plugin-opener` | Open URLs/folders | Apache‑2.0/MIT | +20 KB | Settings links / manual update link. |
| `tauri-plugin-dialog` | File picker | Apache‑2.0/MIT | +80–120 KB | Blocklist import; can defer. |
| `rusqlite` (bundled, `fts5`) | SQLite + FTS5 | MIT | **+0.6–0.9 MB** | Biggest necessary add; bundles SQLite public domain. |
| `clipboard-rs` | Clipboard I/O + watcher | MIT | +150–250 KB | `wayland` feature only when v1.x support ships. |
| `enigo` | Synthetic input | MIT | +80–150 KB | `wayland` feature only in v1.x. |
| `image` (`png`,`jpeg`) | Encode/decode/thumbnails | MIT/Apache‑2.0 | +250–400 KB | Avoid all other codecs. |
| `blake3` | Content hashing | CC0/Apache‑2.0 | +50–100 KB | Faster than SHA‑256 for images. |
| `serde` + `serde_json` | IPC/config | MIT/Apache‑2.0 | +150–250 KB | Tauri already pulls serde. |
| `thiserror` | Error types | MIT/Apache‑2.0 | +10 KB | |
| `log` + `tauri-plugin-log` | Local logs | MIT/Apache‑2.0 | +80–120 KB | Never log content. |
| `windows-sys` (minimal features) | Win32 focus/DWM/process | MIT/Apache‑2.0 | +80–200 KB | Prefer over full `windows`. |
| `winreg` (Windows) | Trial/license registry anchor | MIT | +15 KB | HKCU only. |
| `objc2` + `objc2-app-kit` + `objc2-foundation` | macOS workspace/window | MIT/Zlib/Apache‑2.0 | +200–350 KB | Minimal features. |
| `security-framework` (macOS) | Keychain trial anchor | MIT/Apache‑2.0 | +30 KB | Trial/license anchors only. |
| `x11rb` (minimal) | X11 focus/EWMH | MIT/Apache‑2.0 | +150–250 KB | Or `x11-dl` for smaller; x11rb is safer. |
| `ashpd` (`async-io`, `global_shortcuts`) + `zbus` | Wayland portal | MIT | **+1.0–1.8 MB Linux only** | **v1.x only.** Feature-gate. |
| `tempfile` | Atomic staging/tests | MIT/Apache‑2.0 | +20 KB | |
| `memchr` | Fast substring scanning | Unlicense/MIT | +20 KB | Search cache. |
| `ed25519-dalek` + `base64` | License and trial signatures | BSD‑3‑Clause / MIT/Apache‑2.0 | +80–120 KB | Core licensing; offline verification. |
| `ureq` + `rustls` (optional) | Activation / purchase check | MIT/Apache‑2.0 | +1.0–1.8 MB if enabled | Only for the optional 3-device activation; feature-gate. Offline mode must not need it. |
| `x25519-dalek` + `chacha20poly1305` + `hkdf` | Folder sync encryption (v1.2) | BSD‑3‑Clause / MIT/Apache‑2.0 | +150–250 KB | Not in v1.0. |

**Explicitly avoided:** `chrono`, `toml`, `arboard` (duplicate clipboard), `tray-icon` (Tauri handles), `gpui`, `redb`, `rust-i18n`, `keyring` (defer), `reqwest` (too large; use `ureq` for the few activation calls), `notify`, `walkdir`.

**Backend (not shipped in the app):** TypeScript on Cloudflare Workers, `wrangler` for deploy, `zod` for webhook payload validation, `jose`/`@noble/ed25519` for signing, and a transactional email API. It has no access to clipboard data and no dependency on the app’s Rust crates.

## JavaScript — build/runtime

| Package | Purpose | License | Est. size | Notes |
|---|---|---|---|---|
| `svelte` 5 | UI runtime | MIT | ~10–15 KB gzip | |
| `vite` | Build/dev | MIT | dev only | |
| `typescript` | Types | Apache‑2.0 | dev only | |
| `@tauri-apps/api` | IPC/events | Apache‑2.0/MIT | ~8–12 KB gzip | Use `core` + `event` only. |
| `vitest` + `@testing-library/svelte` | Tests | MIT | dev only | |
| `svelte-check` | Type check | MIT | dev only | |

**Avoided:** SvelteKit, Tailwind, icon libraries, date libraries, state libraries, animation libraries, router.

## Size summary

| Artifact | Expected | Budget | Status |
|---|---|---|---|
| Windows NSIS | 4.0–5.5 MB | <10 MB | **v1.0** |
| Windows MSI | 5.5–7.0 MB | <10 MB | v1.x later |
| macOS DMG (per arch) | 6.0–8.0 MB | <10 MB | **v1.0** |
| macOS DMG (universal) | 9.0–11 MB | borderline | v1.x later |
| Linux `.deb`/`.rpm` | 6.0–7.5 MB | <10 MB | **v1.0** |
| Linux AppImage | 70–90 MB | cannot meet 10 MB | v1.x later; exempt if shipped |
| Flatpak delta | 5–15 MB + shared runtime | depends on runtime | v1.x later |

These are estimates; the CI size gate is the authority. v1.0 size gate covers only NSIS, DMG (per arch), deb, and rpm. Licensing adds ~100–200 KB (Ed25519 + verifier); the optional activation HTTP client (`ureq`/`rustls`) is feature-gated and can be excluded from an offline-only build if it threatens the budget.

---

# 10. Testing strategy

## Unit tests (Rust)

- Hash/canonicalization: CRLF/LF, trailing NUL, Unicode, empty, huge input.
- Dedupe: new vs recopy, pin/expiry preservation, re-copy after delete, hash collision simulation.
- FTS query builder: quotes, punctuation, CJK, emoji, 1-char tokens, injection attempts, max length.
- Search ranking: pinned first, exact vs prefix vs fuzzy, type filter, source filter.
- Retention: age boundary, pinned exemption, image cap eviction, blob GC leaves no orphans.
- Migrations: fresh DB, v1→v2, unknown future version refusal, corrupt settings recovery.
- Settings: clamp, invalid hotkey, unknown fields, atomic write, `.bak` recovery.
- Path normalization: Windows drive/UNC, macOS POSIX, Linux, trailing slashes, duplicate paths.
- Image limits: 0×0, 1×1, 50 MP, 64 MB, alpha, grayscale.
- Suppression: own write does not create/bump; TTL expiry; concurrent capture.
- Trial: first-run idempotence, earliest anchor wins, missing-anchor recovery, tampered/future record, clock rollback (freezes, never extends), **14-day boundary**, reminder schedule (day 10/13/14, once per day, dismissible), no extension by re-running.
- License: key parse/normalize, valid/invalid signature, wrong product/edition, revocation list, 14-day grace, seat activation token, masked display, never logged.
- Entitlement matrix: locked mode exposes exactly 25 text items and no FTS; capture/store paths behave identically in trial, licensed, and locked states.

## Integration tests

- Temp data dir + real SQLite + FTS5:
  - Insert 10k text items; search latency and correctness.
  - Insert images/files; verify files, thumbs, DB refs, GC.
  - Simulate DB failure; verify capture retry and no dedupe poisoning (Ropy’s `fail_next_batch` pattern).
  - Kill process mid-write; reopen; WAL recovery; orphan blob cleanup.
  - Concurrent capture + clear history; no stale rows resurrect.
- Platform clipboard harness:
  - Windows: PowerShell `Set-Clipboard`, WinForms clipboard with custom formats; assert concealed formats are skipped.
  - macOS: `pbcopy`, `osascript` for image/file pasteboard; private pasteboard for concealed tests (Ortu’s technique).
  - Linux X11: `xclip`/`xsel` under Xvfb; assert XFixes capture.
  - Wayland (v1.x): Sway/Hyprland VM + `wl-copy`; portal D-Bus fixture.
  - v1.0 Wayland fallback: simulate `WAYLAND_DISPLAY`, assert capability notice and copy-only path.
- Paste integration:
  - Launch a test editor and verify pasted content; automate on Windows/Linux; manual on macOS.
  - Focus restore: two windows, open palette from A, paste, assert A receives.
  - Elevated target on Windows: assert copy-only fallback.
  - macOS Accessibility denied: assert copy-only and banner.
- Frontend:
  - Keymap reducer: arrows, wrap, Enter, Shift+Enter, digits, IME, stale queries.
  - Local search: 10k fixture, expected ordering, performance sanity.
  - IPC mock: error states, empty states, request cancellation.
  - Locked mode: 25-row cap, dimmed “N more waiting” row, disabled shortcuts/settings tabs, instant unlock transition, single footer banner (no dialog).
- Licensing integration:
  - Fresh install → trial active for exactly 14 calendar days; system clock set back → no extension; reminder only on days 10/13/14, once per day.
  - Delete one anchor → trial not reset; delete all → new trial (documented behavior).
  - Paste a valid test key → instant unlock; invalid/expired/revoked key → clear error, no lockout.
  - Refund/revocation list → 14-day grace then limited mode; no data deletion.
  - Reminder state: day boundaries, one per day, dismissal persists for the day, no reminder mid-paste.
- License backend (unit + integration against the Worker):
  - Webhook signature valid/invalid/stale timestamp; replay ignored; duplicate `transaction.completed` issues exactly one key.
  - Key payload/signature round-trip with the app verifier; 3-device activation and seat release.
  - Restore-purchase returns a key for a known email and a generic response for an unknown one; rate limits enforced.
  - Refund event revokes; revocation list is signed and verifies; cron reconciliation reports zero drift.
  - Email failure leaves the key available on `/welcome`; retry succeeds.
- E2E: `tauri-driver` + WebdriverIO on Windows/Linux X11; smoke only (open, type, select, close). macOS lacks WebDriver support in Tauri; use manual tests.

## Performance tests

- Criterion: insert, dedupe, FTS, local scan, retention.
- Seeded 10k/100k DB; report p50/p95.
- Open-to-first-paint: 100 iterations per OS, report p50/p95.
- RSS: 10-minute idle, process tree, per OS; compare core <40 MB, total 60–100 MB, gate >120 MB.
- Idle CPU: 10-minute sample.
- Installer size: release artifacts.
- Soak: 24 h with periodic copies, images, searches; track RSS slope and errors.

## Manual cross-OS checklist

| # | Test | Windows | macOS | Linux X11 | Linux Wayland (v1.x) |
|---|---|---|---|---|---|
| 1 | Copy text in browser/editor/terminal/chat → appears | ☐ | ☐ | ☐ | ☐ |
| 2 | Copy image (screenshot/browser) → thumbnail + paste | ☐ | ☐ | ☐ | ☐ |
| 3 | Copy files (Explorer/Finder/Nautilus) → paste files | ☐ | ☐ | ☐ | ☐ |
| 4 | 1Password/Bitwarden/KeePassXC password copy → not stored | ☐ | ☐ | ☐ (KDE hint) | ☐ / documented gap |
| 5 | Blocklist app → not captured; remove → captured | ☐ | ☐ | ☐ | ☐ / gap |
| 6 | Hotkey default, rebind, conflict error, persists after restart | ☐ | ☐ | ☐ | portal |
| 7 | Open from VS Code/browser, Enter pastes back into same app | ☐ | ☐ | ☐ | ☐ / copy-only |
| 8 | Multi-monitor, mixed DPI, monitor under cursor | ☐ | ☐ | ☐ | compositor |
| 9 | Shift+Enter plain text; 1–9; arrows; Esc; IME | ☐ | ☐ | ☐ | ☐ |
| 10 | Pin survives expiry; unpin expires correctly | ☐ | ☐ | ☐ | ☐ |
| 11 | Launch at login opens hidden; single instance; tray | ☐ | ☐ | ☐ | partial |
| 12 | Theme auto/light/dark, accent, no flash | ☐ | ☐ | ☐ | ☐ |
| 13 | Accessibility denied → copy-only banner; grant → paste | n/a | ☐ | n/a | n/a |
| 14 | Elevated target → copy-only notice | ☐ | n/a | n/a | n/a |
| 15 | Sleep/wake, monitor unplug, RDP reconnect | ☐ | ☐ | ☐ | ☐ |
| 16 | 24 h soak; RSS growth; idle CPU | ☐ | ☐ | ☐ | ☐ |
| 17 | Wayland detected on v1.0 → capability notice + copy-only works | n/a | n/a | n/a | ☐ |
| 18 | Auto-update from previous version (v1.x) | ☐ | ☐ | ☐ | ☐ |
| 19 | Clear all history: rows, FTS, blobs, WAL truncated | ☐ | ☐ | ☐ | ☐ |
| 20 | Fresh install → 14-day trial; all features; footer countdown; reminders only on days 10/13/14 | ☐ | ☐ | ☐ | ☐ |
| 21 | Locked mode: 25 text items, “N more waiting” row, images/files/pins hidden, capture continues | ☐ | ☐ | ☐ | ☐ |
| 22 | Locked settings: only License/About/Privacy/Quit/Clear history usable; one footer banner | ☐ | ☐ | ☐ | ☐ |
| 23 | Valid key unlocks instantly offline; reinstall + re-enter works; revoked key returns after grace | ☐ | ☐ | ☐ | ☐ |
| 24 | One-line install (`irm \| iex`, `curl \| sh`), winget, and brew cask on a clean machine | ☐ | ☐ | ☐ | ☐ |
| 25 | Tampered artifact rejected; autostart created; uninstall keeps history; `--purge` removes it | ☐ | ☐ | ☐ | ☐ |
| 26 | Paddle purchase → `/welcome` key → `copyfie://activate` → unlocked; restore by email works | ☐ | ☐ | ☐ | ☐ |

---

# 11. CI/CD and packaging

## Workflows

### `ci.yml` (every PR/push)

- `frontend`: `npm ci`, `svelte-check`, `vitest run`, production build, bundle-size check.
- `rust`: matrix `ubuntu-22.04`, `windows-latest`, `macos-14`; `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `cargo test -p copyfie-core --benches` (no run on PR).
- `license`: `cargo-deny` with the proprietary-product allowlist (MIT, Apache‑2.0, BSD, ISC, Zlib, CC0, Unlicense; MPL‑2.0 only for unmodified deps; **deny GPL/AGPL/SSPL/non-commercial**), `cargo-about`, `npm audit --omit=dev`, verify `THIRD_PARTY.md` is current and ships with the app.
- `platform-smoke`: Linux Xvfb clipboard test; Windows clipboard format test; macOS compile-only.
- `size`: weekly/manual release-profile build, compare artifact sizes to budget.

### `release.yml` (tag `v*`)

Matrix:

| OS | Target | Bundles |
|---|---|---|
| `windows-latest` | `x86_64-pc-windows-msvc` | NSIS |
| `ubuntu-22.04` | `x86_64-unknown-linux-gnu` | deb, rpm |
| `macos-14` | `aarch64-apple-darwin` | DMG |
| `macos-13` | `x86_64-apple-darwin` | DMG |

Steps:

1. Checkout, setup Node LTS, Rust stable, cache Cargo/npm.
2. Linux install: `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf libxdo-dev`.
3. `npm ci`, `npm run build`.
4. `tauri-apps/tauri-action@v0`:
   - `tagName: v__VERSION__`, draft release.
   - Upload NSIS/DMG/deb/rpm artifacts.
5. Generate SHA-256 sums, SBOM (`cargo cyclonedx`), and a size report. Bundle `LICENSE`, `EULA.md`, `THIRD_PARTY.md`, and the signed revocation list into the installers. **No `latest.json` and no updater manifest in v1.0.**
6. Code signing (**required before charging; unsigned builds are beta-only**):
   - **Windows:** `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_CODE_SIGNING_ACCOUNT`, `AZURE_CERTIFICATE_PROFILE`; or SignPath OSS (`SIGNPATH_API_TOKEN`). Sign the NSIS installer and the executable.
   - **macOS:** `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_APP_SPECIFIC_PASSWORD`, `APPLE_TEAM_ID`; `notarytool` after signing, then staple. **Do not sell an unsigned/unnotarized DMG.**
7. Publish draft for manual approval. Record the manual update path in the release notes. The license-signing private key is **never** in CI; only the public key is compiled in.
8. Publish the install assets: `install.ps1`, `install.sh`, `uninstall.sh`, `SHA256SUMS`, `SHA256SUMS.sig`, and the release key fingerprint. The website serves the same files for the stable one-liner URLs.

### `install-test.yml` (release candidates, nightly, manual)

- Runs the **published one-liners** on clean `windows-2022`/`windows-latest`, `macos-13`/`macos-14`, `ubuntu-22.04`/`ubuntu-24.04`, and Fedora 40/41 containers.
- Asserts install location, `--version`, autostart entry, `.desktop`/icon presence on Linux, uninstall behavior, and that history survives uninstall.
- Negative cases: tampered artifact, bad checksum, invalid signature, unsupported arch, no-admin user. All must fail safely or install per-user.
- Static checks: `shellcheck` for the shell scripts, `PSScriptAnalyzer` for the PowerShell script.

### `site.yml` / `backend.yml`

- `site.yml`: build and deploy `site/` to Cloudflare Pages on changes; verify links, CSP headers, and that `/buy`, `/welcome`, `/restore`, `/download`, and legal pages return 200.
- `backend.yml`: typecheck, unit tests, and deploy the Worker on changes to `backend/`; run against a staging KV namespace first. Deployments use a scoped API token, never the Ed25519 private key.
- Both are independent of the app release; the app never blocks on a site deploy.

### Later packaging workflows

- `v1.x` auto-updater: `TAURI_SIGNING_PRIVATE_KEY`, `latest.json`, update banner, failed-update recovery.
- `v1.x` MSI, AppImage, Flatpak.
- `v1.x` signed apt/rpm repository: GPG-signed `InRelease`/`repomd.xml`, package signing, `copyfie` package.
- `v1.x` `aur.yml`: `copyfie-bin` PKGBUILD, like Ropy’s.
- `v1.x` Homebrew core cask submission if warranted; the `copyfie/tap` cask ships in v1.0.
- winget manifest PRs are manual in v1.0 and automated in v1.x; the installer carries the EULA and requires acceptance.

## Packaging configuration notes

- **Windows NSIS:** `installMode: "currentUser"` to avoid UAC; `webviewInstallMode: { type: "downloadBootstrapper" }` (Windows 10/11 have WebView2). **MSI is v1.x.** Include an EULA page and a “View license” link.
- **macOS:** `minimumSystemVersion: "12.0"`; `LSUIElement=true`; `macOSPrivateApi` for transparency. Accessibility does not use a usage string; add a clear About/Help link. Per-arch DMGs in v1.0; universal build later. Notarize before selling.
- **Linux:** v1.0 is **X11-only**. deb/rpm dependencies on `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`; `.desktop` with `Exec=copyfie --hidden`, `StartupWMClass=copyfie`, `Keywords=clipboard;history;`. Ship `LICENSE`/`EULA` under `/usr/share/doc/copyfie/`.
- **Flatpak / AppImage:** v1.x later; Flatpak’s sandbox likely blocks background capture.
- **CSP:** strict; no remote script; `img-src 'self' asset: http://asset.localhost copyfie:`; `connect-src ipc: http://ipc.localhost` plus `https://copyfie.app` and Paddle’s checkout/API hosts only when activation is enabled; no telemetry endpoints.
- **Capabilities:** one per window; only `core:event:default` and the custom commands; no fs/shell/http permissions in the webview. Activation HTTP calls happen in Rust, not the webview.

---

# 12. One-line install and distribution

Plan only: none of the scripts below exist yet. They are scheduled in the v1.0 packaging milestone (§13).

## Goals

- A new user can install, launch, and later remove Copyfie by pasting **one command per OS**.
- Every path downloads a **signed, notarized (macOS), checksum-verified** artifact from the official release — never an unverified payload.
- Per-user install with **no admin/root wherever the OS allows it**.
- The scripts stay small, auditable, and boring: no telemetry, no analytics, no shell-profile edits, no silent privilege escalation, no `curl | sh` of a moving target.

## Install matrix

| Platform | One-liner | Package manager | Install location |
|---|---|---|---|
| Windows 10/11 | `irm https://copyfie.app/install.ps1 \| iex` | `winget install Copyfie.Copyfie` | `%LOCALAPPDATA%\Programs\Copyfie` (per-user, no UAC) |
| macOS 12+ | `curl -fsSL https://copyfie.app/install.sh \| sh` | `brew install --cask copyfie/tap/copyfie` | `/Applications`, else `~/Applications` |
| Linux (X11) | `curl -fsSL https://copyfie.app/install.sh \| sh` | `.deb` / `.rpm` direct; signed apt/rpm repo in v1.x | `~/.local/opt/copyfie` + `~/.local/bin/copyfie` (per-user) |

- One `install.sh` covers macOS and Linux; the branching is internal, not user-facing.
- The domain `copyfie.app` is assumed, consistent with `com.copyfie.app`; confirm at the identity gate.
- `irm … | iex` and `curl … | sh` only bootstrap. The script resolves a **pinned version**, downloads the artifact, verifies it, then installs. `--version x.y.z` pins an older release; the bare URL serves the latest stable script.
- The scripts are published as release assets and mirrored on the website, so the URL is stable while the code is versioned with the release.

## Script contract

Every installer script must:

1. **Detect** OS, architecture (`x86_64`/`arm64`), distro family on Linux, session type (X11/Wayland), and whether stdin is interactive.
2. **Resolve** artifact URLs from the GitHub `releases/latest/download/` path — no API call, no rate limit, no JSON parsing.
3. **Download** into a private temp directory: the artifact, `SHA256SUMS`, and `SHA256SUMS.sig`.
4. **Verify** the artifact's SHA-256 against `SHA256SUMS`, then verify `SHA256SUMS.sig` against the embedded release public key. Abort before extracting or executing anything.
5. **Verify the OS signature** where one exists: Authenticode on Windows, `codesign --verify` + `spctl` on macOS. A failed OS signature is fatal even when the hash matches.
6. **Install** per-user; `--system` is the explicit opt-in to a system-wide install.
7. **Enable launch at login** through the app's own autostart writer, unless `--no-autostart` is passed.
8. **Launch** the app once (`--no-launch` skips) so the tray icon and the trial start are visible.
9. **Print** the exact uninstall command and the data directory for this machine.

Rules: `set -euo pipefail` (`$ErrorActionPreference = 'Stop'` on PowerShell); no partial installs; temp files removed on exit; actionable error text; non-zero exit on failure; never prompts (stdin is unavailable under `irm | iex`); never edits shell profiles; never installs system packages without `--system`. Flags: `--version`, `--dir`, `--no-autostart`, `--no-launch`, `--system`, `--quiet`, `--dry-run`.

## Per-OS behavior

### Windows (`install.ps1`)

- Download `Copyfie_<ver>_x64-setup.exe`, verify SHA-256 and Authenticode, then run NSIS silently: `... /S`.
- NSIS `currentUser` mode installs to `%LOCALAPPDATA%\Programs\Copyfie`, so no elevation.
- Use `Get-FileHash -Algorithm SHA256`; refuse to run the installer if `Get-AuthenticodeSignature` is not `Valid`.
- Launch `%LOCALAPPDATA%\Programs\Copyfie\copyfie.exe --install-complete`.
- Uninstall command: `"%LOCALAPPDATA%\Programs\Copyfie\uninstall.exe" /S` (or `winget uninstall Copyfie.Copyfie`).
- Document the inspect-first variant at the top of the script and on the download page: `irm https://copyfie.app/install.ps1 -OutFile install.ps1`, review, then `.\install.ps1`.

### macOS (`install.sh`)

- Detect `arm64` vs `x86_64` and pick the matching DMG.
- Verify SHA-256, then `codesign --verify --deep --strict` and `spctl --assess --type install`; **abort if either fails**.
- `hdiutil attach`, copy `Copyfie.app` to `/Applications` when writable, else `~/Applications`, then `hdiutil detach`.
- Never run `xattr -cr` in the normal flow. A notarized app needs no quarantine surgery; an unsigned beta fails with an explanation instead of silently stripping the flag.
- Because `curl` does not set the quarantine attribute, a `curl | sh` install would otherwise bypass Gatekeeper's usual prompt. That is exactly why step 5 must run `codesign`/`spctl` explicitly rather than relying on the OS prompt.

### Linux (`install.sh`)

- Detect `/etc/os-release` (`ID`, `ID_LIKE`) and arch; choose `.deb` (Debian/Ubuntu/Mint/Pop) or `.rpm` (Fedora/RHEL/openSUSE).
- Default is per-user, no root:
  - Extract the package into `~/.local/opt/copyfie` (`dpkg-deb -x`, or `rpm2cpio … | cpio -idm`).
  - Symlink `~/.local/bin/copyfie`.
  - Install the `.desktop` file into `~/.local/share/applications` and icons under `~/.local/share/icons/hicolor`, then run `update-desktop-database` when present.
- `--system` uses the package manager (`apt install ./copyfie_x.y.z_amd64.deb`, `dnf install ./copyfie-…rpm`) with `sudo`.
- Check runtime dependencies with `ldd` (`libwebkit2gtk-4.1`, `libgtk-3`, `libayatana-appindicator3`); on failure print the distro-specific hint and change nothing.
- If `XDG_SESSION_TYPE=wayland`, print the v1.0 capability notice (copy-only) but continue installing.
- The signed apt/rpm **repository** is v1.x. v1.0 installs a directly downloaded package.

## Uninstall

- One command per OS, printed by the installer and documented on the download page:
  - Windows: `"%LOCALAPPDATA%\Programs\Copyfie\uninstall.exe" /S` or `winget uninstall Copyfie.Copyfie`.
  - macOS: `curl -fsSL https://copyfie.app/uninstall.sh | sh` or `brew uninstall --cask copyfie/tap/copyfie`.
  - Linux: `curl -fsSL https://copyfie.app/uninstall.sh | sh`, or `sudo apt remove copyfie` for a `--system` install.
- `uninstall.sh` removes the app, autostart entry, desktop entry, icons, and the macOS login item. It never deletes history unless `--purge` is passed.
- **History is kept on uninstall by default.** The app's data directory (under `%APPDATA%`, `~/Library/Application Support`, or `~/.local/share`) is left intact, and the script prints the exact path. `--purge` also removes `license/` and `trial.json`, and the script warns that the key must be re-entered.
- The app exposes the same behavior itself — `copyfie --uninstall` (keep data) and `copyfie --uninstall --purge` — so package-manager installs have a non-script path.
- Uninstall never touches the network and never revokes the license.

## Manual download page

`https://copyfie.app/download` is a real page, not a redirect to GitHub:

- Artifacts per OS/arch with version, size, minimum OS, and SHA-256.
- `SHA256SUMS`, `SHA256SUMS.sig`, and the release public-key fingerprint.
- Per-OS manual steps (the same steps the scripts automate) plus the exact uninstall command.
- Release notes and the supported-platform matrix (Linux = X11 in v1.0).
- Linked from the README, the GitHub release page, the app's About → Installation, and every script error message.

## Checksums and signatures

- `release.yml` emits `SHA256SUMS` covering **every** artifact, plus a detached `SHA256SUMS.sig` (minisign/Ed25519) signed with an offline release key — distinct from the license-signing key and from the OS code-signing certificates.
- Checksums are published on the GitHub release **and** the download page. Scripts verify against the signed checksum file, not a hash baked into the script, so they do not need to change per version.
- The public key and fingerprint live in the repo, on the website, and in the docs; rotation is documented.
- Linux v1.0 trust = SHA-256 + detached signature. The v1.x apt/rpm repo adds GPG-signed metadata and package signatures.

## CI install tests

`install-test.yml` (release candidates, nightly, and `workflow_dispatch`) runs the **real published one-liners** on clean runners:

| Runner | Asserts |
|---|---|
| `windows-2022`, `windows-latest` | Exit 0; per-user path exists; hash matches; `copyfie.exe --version` works; uninstall removes the app; data survives uninstall |
| `macos-13` (x86_64), `macos-14` (arm64) | Exit 0; `spctl` accepts the app; launches; uninstall removes the bundle; data survives |
| `ubuntu-22.04`, `ubuntu-24.04` | Per-user extract; `copyfie --version`; `.desktop` present; uninstall removes symlink and entry |
| `fedora:40`, `fedora:41` containers | `.rpm` extract path; dependency check; uninstall |
| Tampered artifact fixture | Script exits non-zero, installs nothing, leaves no temp files |
| Unsupported arch (`armv7`) | Clear error, non-zero exit |
| No-admin user (Windows/macOS/Linux) | Per-user install succeeds |

Static checks run on every PR: `shellcheck` for `install.sh`/`uninstall.sh` and `PSScriptAnalyzer` for `install.ps1`. Scripts live in `scripts/install/` and are also published as release assets.

## How signing changes the install paths

| Path | Unsigned | Signed (+ notarized on macOS) |
|---|---|---|
| Windows `irm \| iex` | Script runs, but SmartScreen shows “Unknown publisher” on the setup, Defender may quarantine it, and enterprise policy can block the download. | Authenticode (OV/EV or Azure Trusted Signing) + reputation → warnings disappear for most users; `Get-AuthenticodeSignature` passes. |
| Windows `winget` | Manifest validation rejects unsigned installers. | Required: winget checks hash and signature. |
| macOS `curl \| sh` | Script must abort because `codesign`/`spctl` fail. Never fall back to `xattr -cr`. | Passes; notarization also covers the first browser download. |
| macOS Homebrew | A cask for an unsigned app fails Gatekeeper on first launch. | Standard cask; no `--no-quarantine` needed. |
| Linux script / deb / rpm | No OS gate; trust comes from the checksum + signature. | Same, plus package signing in v1.x. |

Consequence: **the one-liners are advertised only after signing and notarization exist.** Until then the download page offers dated beta builds behind an explicit “unsigned, expect warnings” banner with manual steps.

## Scope

- **v1.0:** `install.ps1`, `install.sh`, `uninstall.sh`, download page, `SHA256SUMS` + `.sig`, winget manifest, `copyfie/tap` Homebrew cask, `install-test.yml`.
- **v1.x:** signed apt/rpm repository, AUR `copyfie-bin`, Homebrew core cask submission if warranted, and installer support for the auto-updater.

---

# 13. Milestone breakdown

Effort: **S** ≈ 1–3 days, **M** ≈ 1–1.5 weeks, **L** ≈ 2–3 weeks for one experienced dev. Parallelizable across platform owners.

## MVP — text capture, hotkey popup, search, paste

| Order | Task | Effort | Reuse |
|---|---|---|---|
| 1 | Repo/workspace, Tauri hello, CI skeleton, `cargo-deny`, NOTICE/THIRD_PARTY | S | Adapt Ortu CI |
| 2 | `copyfie-core`: models, BLAKE3, SQLite schema/migrations, dedupe, basic retention, unit tests | M | Adapt Ortu FTS5, Ropy dedupe |
| 3 | Store actor + custom URI protocol for thumbnails (stub) | M | Rewrite |
| 4 | Capture pipeline: privacy gate, text capture, source app, suppression, coalesced `history:changed` | L | Adapt Ropy watcher/capture, Ortu concealed |
| 5 | Platform text capture: Windows, macOS, Linux X11 (parallel) | L | Adapt Ropy clipboard-rs + Ortu macOS |
| 6 | Palette UI: Svelte shell, list, keyboard, filters, empty/error states | L | Rewrite (Ortu UX reference) |
| 7 | Pre-warm + show/focus + multi-monitor positioning | M | Adapt Ropy resident window, Ortu centering |
| 8 | Paste orchestrator: hide, restore focus, write, suppress echo, synthesize paste | L | Adapt Ortu macOS fix, Ropy enigo |
| 9 | Hotkey + first-run conflict detection + rebind flow + tray + single instance + autostart | M | Adapt Ortu/Ropy |
| 10 | Minimal settings + theme | M | Rewrite |
| 11 | Perf harness, 10k fixture, baseline report | M | Adapt Ropy benchmarks |
| 12 | **Measure real RSS per OS** (core + total process tree, 10k items, 10 min idle) and record in `docs/perf/` | M | New |
| 13 | MVP acceptance pass + docs | M | — |

**MVP acceptance criteria**

- Text copied in Notepad, TextEdit, and gedit appears within 500 ms p95.
- Hotkey default is `CommandOrControl+Shift+Space`; first-run conflict detection offers a rebind and never silently changes it.
- Hotkey opens the palette at p95 <100 ms with 10k seeded items (pre-warmed, reference hardware).
- Typing filters instantly (<16 ms local); FTS results arrive <50 ms p95.
- `↑/↓`, `Enter`, `Shift+Enter`, `1–9`, `Esc` all work; IME does not break Enter.
- Enter pastes into the app that was focused before opening; wrong-target paste never happens silently.
- Copying an item back does not create a duplicate or bump the item.
- Password-manager copy is not stored on macOS; Windows/Linux markers covered where available.
- **Core process RSS < 40 MB; total process tree 60–100 MB.** If total > 120 MB, the M1 exit is blocked until the decision gate is resolved.
- No telemetry, no account, no background network, and no update checker in the MVP.
- Release builds succeed on all three OSes; installer sizes recorded.

## v1.0 — images, files, pins, privacy, licensing, trial, installers

| Order | Task | Effort | Reuse |
|---|---|---|---|
| 0 | **Naming clearance (before everything else):** confirm `copyfie.app` availability and ownership, run a trademark search on the name and logo, and reserve the GitHub org/repo, winget publisher ID, Homebrew tap, AUR/package name, and npm/crate names; then lock `com.copyfie.app` and the copyright line. | S | New |
| 1 | **Release gate:** proprietary `LICENSE`, EULA, terms of service, privacy policy, and refund policy finalized before any binary is distributed. | S | — |
| 2 | **Marketing site (early):** landing, pricing, product description, `/buy`, `/welcome`, `/restore`, and legal pages. Required before Paddle seller approval. | M | New |
| 3 | **Paddle seller onboarding:** apply as an Indian business/individual; submit the live site, pricing, refund policy, terms, privacy policy, and product description. Start early; approval is external latency. | M | New |
| 4 | Image capture: hash, downscale, PNG, thumbnail, atomic files | M | Adapt Ropy assets |
| 5 | File-path capture: Windows CF_HDROP, macOS file URLs, Linux URI list | M | Adapt Ropy files; Ortu lacks non-macOS files |
| 6 | Pin/unpin + pin-aware expiry + retention setting + image disk cap + blob GC | M | Adapt Ortu/Ropy cleanup |
| 7 | Fuzzy search + type filters + source-app name/initial display | M | Adapt Ortu fuzzy, Ropy search |
| 8 | Blocklist capture gate + settings UI + “block current app” | M | New |
| 9 | Full settings: hotkey recorder, history length (7/30/90/365/Forever), image cap, launch at login, theme, pause capture | M | Adapt Ortu/Ropy |
| 10 | macOS Accessibility onboarding + copy-only fallback | M | Adapt Ortu |
| 11 | Windows elevated-target detection + copy-only fallback | S | New |
| 12 | **Wayland capability notice + copy-only fallback** (no full Wayland support) | S | New |
| 13 | **Trial module (14 days):** signed trial-start record, redundant anchors, clock-rollback detection, entitlement state, day 10/13/14 reminder schedule | M | New |
| 14 | **License module:** Ed25519 verification, key format, persistent key, 3-device activation (optional online), grace period | M | New |
| 15 | **Limited-mode UI:** 25 most recent text items, real “N more items waiting” count, locked settings except Clear history, one footer line | M | New |
| 16 | **Paddle checkout + customer journey:** hosted checkout, `/welcome` success page, `copyfie://activate` deep link, emailed key, manual paste fallback | M | New |
| 17 | **License backend:** Worker webhook (signature verification, replay protection, idempotency), Ed25519 issuance, email delivery, restore-purchase by email, revocation list, cron reconciliation | L | New |
| 18 | **Download page + published checksums:** artifacts, `SHA256SUMS`, `SHA256SUMS.sig`, key fingerprint, manual steps, uninstall commands | M | New |
| 19 | **Payouts (India):** confirm wire vs Payoneer, schedule and $100 minimum, fee/FX model, FIRC/documentation with the accountant | S | New |
| 20 | **Pricing validation:** competitor scan for one-time clipboard-manager pricing; confirm launch and list price | S | New |
| 21 | Packaging: NSIS, DMG, deb/rpm, `.desktop`, icons, EULA/notices in installer | L | Adapt Ortu/Ropy |
| 22 | **One-line installers:** `install.ps1`, `install.sh`, `uninstall.sh` — OS/arch detection, SHA-256 + signature verification, Authenticode/`codesign`/`spctl` checks, per-user install, autostart, launch, printed uninstall command | L | New |
| 23 | **Uninstall flow:** app `--uninstall` / `--purge`, script removal of app, autostart, desktop entry, icons, login item; history kept by default | M | New |
| 24 | **Install channels:** winget manifest, `copyfie/tap` Homebrew cask | M | New |
| 25 | **Install CI:** `install-test.yml` on clean Windows/macOS/Ubuntu/Fedora runners, tampered-artifact and no-admin cases, `shellcheck` + `PSScriptAnalyzer` | M | New |
| 26 | Manual “Check for updates” link to the release page; document the update path | S | New |
| 27 | Perf/soak validation, size gate, privacy docs | M | Adapt Ropy |
| 28 | **Pre-launch verification:** sandbox + one live test purchase/refund, webhook replay and delivery check, restore-purchase test, payout test, accountant GST/export review (§14 checklist) | M | New |
| 29 | Attribution/THIRD_PARTY audit (maintained from day one) | S | — |

**v1.0 acceptance criteria**

- Images and files captured and pasted on Windows, macOS, and Linux **X11**.
- On Wayland, the app detects the session, shows a capability notice, and keeps the manual copy path working (write a selected item to the clipboard for the user to paste); it does not claim background capture or synthetic paste support.
- Pinned items survive expiry; unpinned items expire at the configured boundary; 7/30/90/365/Forever all behave correctly; the image cap is enforced and pinned images are never evicted.
- Blocklist prevents capture; concealed markers prevent capture; paused capture persists across restart.
- macOS Accessibility denied → clear banner, copy-only works; granted → direct paste works.
- **Trial:** starts on first launch; all features present in the build work for **14 calendar days**; survives restarts; resists naive JSON edits and clock rollback (a rollback freezes, never extends); deleting one anchor does not reset it.
- **Trial reminders:** appear only on days 10, 13, and 14; at most one per day; dismissible; footer-only; never block a paste or a search.
- **After expiry:** capture continues for text/images/files; palette shows the 25 most recent text items; older content is represented by one dimmed “N more items waiting, unlock to see them” row with the real hidden count; images/files/pins/full-history search/settings are locked; **Clear history remains available with a confirmation**; one footer banner; no popup or modal; nothing is deleted automatically.
- **Unlock:** a valid key unlocks instantly without restart; verification works fully offline; reinstall + re-enter key works; the 3-device activation limit is enforced when online; refund/chargeback revokes after the documented grace period.
- **Paddle purchase:** hosted checkout completes in sandbox and in one live low-value purchase; the `/welcome` page shows the key; `copyfie://activate?key=…` opens the app and activates; the key is emailed; manual paste works as a fallback.
- **License backend:** webhook signatures verified, replays ignored, duplicate events idempotent, delayed webhooks recovered, refunds revoked, restore-purchase by email works, and reconciliation shows no drift.
- **Payouts:** Paddle seller account approved for an Indian entity; payout route, fee, FX, and documentation confirmed; accountant has reviewed GST/export treatment.
- **One product:** while trial-active or licensed, no feature is gated, and no “Pro” upsell exists anywhere.
- Installers: NSIS <6 MB, DMG <10 MB, deb/rpm <8 MB. **No MSI, AppImage, or Flatpak in v1.0.**
- **One-line install works on clean machines** — Windows 10/11, macOS 12+ (arm64 and x86_64), Ubuntu 22.04/24.04, Fedora 40+ — without admin; autostart is enabled; the app launches; the uninstall command is printed.
- **Installers verify before executing:** a tampered artifact, a bad checksum, or an invalid signature aborts with a non-zero exit and installs nothing; the script never runs an unverified payload.
- **`winget install Copyfie.Copyfie` and `brew install --cask copyfie/tap/copyfie`** succeed and their uninstall counterparts work.
- **Uninstall keeps history by default**; `--purge` removes history, `license/`, and `trial.json` after a warning; autostart and desktop entries are removed in both cases.
- **Download page is live** with every artifact, its SHA-256, the signature, the size, and manual install/uninstall steps; published checksums match the release assets.
- Idle CPU <0.1% over 10 minutes; 24 h soak with no crash and <10% RSS growth; core <40 MB, total 60–100 MB, gate at 120 MB.
- No auto-updater in v1.0; the manual update link and release notes are correct.
- Privacy doc states exactly what is stored, where, and what is never read; the trial/license files contain no clipboard content.
- Proprietary `LICENSE`, EULA, and `THIRD_PARTY.md` are present in the installer and in About.

## v1.x — deferred platform and packaging work (not v1.0)

Schedule these separately from the numbered v1.1/v1.2 feature milestones. They are explicitly **not** v1.0 blockers; the order below is the suggested priority.

| Order | Task | Effort | Notes |
|---|---|---|---|
| 1 | Auto-updater: signed `latest.json`, update banner, failed-update recovery | M | Tauri updater; opt-out daily check. |
| 2 | Full Wayland support: data-control capture, GlobalShortcuts portal, Hyprland binding, activation token, virtual-keyboard paste | L | Ropy patterns; capability matrix; GNOME limitations. |
| 3 | MSI installer | S | Enterprise demand. |
| 4 | AppImage | S | Exempt from 10 MB budget; ~70–90 MB. |
| 5 | Flatpak | M | Validate sandbox/capture first; may be copy-only. |
| 6 | Universal macOS DMG | S | After per-arch is stable. |
| 7 | AUR `copyfie-bin`, signed apt/rpm repository, Homebrew core cask submission | M | winget and the `copyfie/tap` cask ship in v1.0. |

## v1.1 — snippets, OCR, custom themes, icons

All features ship in the single product; there is no separate build or entitlement tier beyond “licensed”.

| Order | Task | Effort | Notes |
|---|---|---|---|
| 1 | Snippets CRUD, variables, search integration, pin-to-top | M | `{{date}}`, `{{time}}`, `{{uuid}}`, `{{clipboard}}`, `{{cursor}}`, `{{input:Name}}` |
| 2 | OCR: Windows OCR API, macOS Vision, Linux optional Tesseract; OCR text index | L | Local only; no bundled models |
| 3 | Custom themes: JSON token packs, preview, contrast validation, import/export | S | No arbitrary CSS |
| 4 | Source-app icons (replaces colored initial) | S | Extracted locally, cached |
| 5 | Snippet/theme backup and restore | S | Simple JSON export |

**Acceptance:** snippets expand correctly at paste time and are searchable; OCR runs locally and never uploads; OCR is optional and can be disabled; themes validate contrast and never inject CSS; no telemetry; all features work offline under a valid license.

## v1.2 — optional encrypted folder sync

No server, no relay, no managed backend. Sync is a folder the user already syncs (iCloud Drive, Dropbox, OneDrive, Syncthing, a network share). If the user does not enable it, nothing changes.

| Order | Task | Effort |
|---|---|---|
| 1 | Device pairing (QR/manual sync key), X25519 identity, per-device keys | M |
| 2 | Content encryption (XChaCha20-Poly1305), metadata envelope | M |
| 3 | Folder transport: per-device append-only oplog + manifest in the chosen folder | L |
| 4 | Conflict resolution: HLC + LWW, tombstones, pinned precedence; conflict copies never silently overwrite | L |
| 5 | Image blob sync + thumbnail regeneration | M |
| 6 | Sync status UI, pause, conflict log, key rotation/revocation, “folder not available” handling | M |
| 7 | Security review + threat model (including cloud-provider access, rollback, and conflict oracles) | M |

**Acceptance:** two devices converge through a shared folder; the folder provider sees only ciphertext and opaque filenames; offline edits merge without data loss; a revoked device cannot decrypt new data; missing/offline folders degrade gracefully; no account and no relay; iCloud “optimize storage” and Dropbox conflict copies are handled explicitly.

## v2.0 — paid upgrade (not detailed here)

- A v1.x license keeps working and keeps receiving v1.x updates indefinitely.
- v2.0 requires a discounted upgrade key; `updates_through` in the v1 key prevents v2.0 from treating it as current.
- No v2.0 feature scope is committed in this plan. Decide after v1.x usage and revenue data.
- The v1 app remains installable and licensed after v2.0 ships; there is no forced migration.

---

# 14. Licensing, trial, and limited mode

## Architecture

One binary, one product, one entitlement module, one tiny backend. There is no Pro executable, no IPC, and no second build.

```
              first launch
                   │
                   ▼
             ┌───────────┐  14 days   ┌───────────┐
             │  Trial    │───────────►│  Locked   │
             └─────┬─────┘            └─────┬─────┘
                   │ valid key              │ valid key
                   └───────────┬────────────┘
                               ▼
                        ┌─────────────┐
                        │  Licensed   │
                        └─────────────┘
```

- `license/` exposes one state: `Entitlement { Trial { days_left }, Licensed { .. }, Locked }`.
- Command handlers ask the entitlement before exposing locked UI or actions. Capture, canonicalization, dedupe, storage, retention, and blob GC **never** check entitlement.
- No network call is required to launch, capture, search, paste, or verify a license. The network is used only at purchase, restore-purchase, and optional seat activation.
- The license module has no clipboard/store/platform dependencies; it is unit-testable with fixed clocks and fixtures.
- Payments sit behind a small `PaymentProvider` seam (create checkout session, verify webhook, normalize events). Paddle is the only implementation in v1.0; a different MoR can be swapped without touching the app.

## Trial record

Full storage details are in §5. Summary:

- **14 calendar days** from first launch (`started_at + 14 × 86_400_000`), not 14 days of usage.
- Payload: `{ v, product, install_id, started_at, days: 14, version, max_seen_at, last_reminder_day }` + Ed25519 `sig`.
- **Tamper resistance:** signed record, three independent anchors (app data, DB meta row, platform anchor), and a monotonic `max_seen_at`. A backwards clock jump is recorded and cannot extend the trial; it can only freeze the remaining days. Deleting every anchor resets the trial — accepted and documented, no machine fingerprinting.
- **Reminders:** days 10, 13, and 14, at most one per calendar day, dismissible, footer-only, never blocking and never during a paste.
- Expiry is evaluated at startup, palette open, hourly, and after wake — never on a keystroke or paste path.
- Expiry only changes presentation (§7). Nothing is deleted, and retention stays the user’s setting.

## License keys

- Format: `COPYFIE-<base64url(payload)>.<base64url(signature)>`; the unlock UI accepts paste and normalizes whitespace/hyphens.
- Payload: `{ v, product: "copyfie", license_id, edition: "standard", issued_at, seats: 3, updates_through: "1.x", txn_hash }`.
- Verification is **fully offline** with an embedded Ed25519 public key: format, signature, product, edition, seat field, and the signed revocation list bundled with the release.
- Persist to `license/license.key` with `0600`; display only a masked fingerprint (last 4 characters). Never log the key.
- **Reinstall:** re-enter the key, or use restore-purchase. License validity is independent of install identity.
- **3 devices:** activation is optional. The backend records `license_id → activated device hashes`; the app sends a salted hash, never clipboard data or hardware serials. A restore page can deactivate old devices. Offline, the key still unlocks — seat enforcement is best-effort, not a hard lock.
- **Refunds/chargebacks:** a Paddle webhook marks the license revoked and publishes it in the signed revocation list. If offline, the app keeps working for a **14-day grace period**, then returns to limited mode. No data deletion.
- **Updates:** a v1.x key unlocks all v1.x updates. v2.0 is a paid upgrade; `updates_through` records the major version.

## Payment provider: Paddle

- Paddle is the **Merchant of Record**: checkout, VAT/sales tax, invoices, receipts, refunds, and chargebacks are Paddle’s responsibility. We never see or store card data.
- One SKU: “Copyfie — one-time license, 3 devices”. No subscription, no seat add-ons in v1.0.
- Checkout uses **Paddle hosted checkout** opened in the system browser (an overlay on our own page is acceptable if it works cleanly, but hosted is the default because it handles 3DS/SCA, localization, and payment-method coverage with less code).
- Events consumed: `transaction.completed` (issue), `adjustment.created` with refund action and `transaction.refunded` (revoke). All others are ignored.
- Sandbox first: Paddle sandbox + test cards, then a real low-value test purchase before launch.
- The `PaymentProvider` seam keeps Paddle-specific fields (`transaction_id`, `customer_id`, signature format) out of the app and out of the license format.

## License backend (webhook → key → email)

Paddle’s built-in license keys are not sufficient (no clean 3-device/offline model), so we issue our own.

### Minimal components

| Piece | Choice | Notes |
|---|---|---|
| Static site | Cloudflare Pages | Landing, pricing, `/buy`, `/welcome`, `/restore`, legal pages, download page. |
| Webhook + API | Cloudflare Workers | `POST /api/paddle/webhook`, `GET /api/license`, `POST /api/restore`, `GET /api/revocations`. |
| Storage | Workers KV (or D1) | `txn → license`, `event_id → seen`, `license → status/seats`, `email_hash → license` for restore. |
| Signing | Ed25519, private key in Worker secret | Public key compiled into the app. Key id prefix enables rotation. |
| Email | Resend or Postmark free tier | Key + restore link; plain text, no tracking pixels. |
| Cron | Workers Cron Trigger | Retry pending emails, reconcile Paddle vs issued keys, refresh revocation list. |
| Support CLI | Local signed tool | Manual re-issue, revoke, seat reset. |

### Purchase flow

1. **Unlock** in the app (footer, Settings → License, or the locked-mode row) opens `https://copyfie.app/buy` in the system browser.
2. **Paddle hosted checkout** collects payment. `custom_data` carries only a random `checkout_id`; no clipboard data, no machine identifiers.
3. Paddle redirects to `https://copyfie.app/welcome?txn=txn_…`.
4. Paddle sends `transaction.completed`. The Worker verifies the signature, dedupes on `event_id`, issues the key once per transaction (idempotent), stores the record, and emails the key.
5. The welcome page shows the key, a **Copy** button, and an **Open Copyfie** button using `copyfie://activate?key=…`. It polls briefly if the webhook has not landed yet.
6. The deep link opens the app, validates offline, shows “License activated · 3 devices”, and the footer disappears. The key is also in the email.
7. **Fallbacks:** manual paste in License → Unlock; re-open `/welcome?txn=…`; **restore purchase** at `/restore` with the purchase email.

### Restore purchase

- `/restore` asks for the purchase email, then `POST /api/restore`.
- Primary lookup is the **Paddle API by email** (no PII stored by us); KV is a cache for resends.
- The response is always generic (“If a purchase exists, your key is on its way”) to prevent account enumeration; requests are rate-limited per IP and per email.
- The email contains the key plus a link to deactivate old devices.

### Revocation

- Refund/chargeback events mark the license revoked and add it to the published list.
- `GET /api/revocations` returns a **signed** JSON document; the app verifies it offline and also ships a snapshot in each release.
- Revocation is graceful: 14-day grace, then limited mode, never deletion.

### Security

- Verify `Paddle-Signature` (HMAC-SHA256 over `ts:raw_body`), reject timestamps older than 5 minutes, constant-time compare.
- **Replay protection:** `event_id` stored with a 30-day TTL; replayed events are acknowledged and ignored.
- **Idempotency:** one license per `transaction_id`; concurrent duplicates serialize on the same KV key.
- Secrets only in Worker secrets/secret store; the Ed25519 private key is never in the repo, CI, or the app. Yearly rotation with a key id.
- No card data, no raw emails in logs, no license keys in logs or analytics. Log line: `{event_id, txn, outcome}` only.
- CORS restricted to `copyfie.app`; strict CSP; the site is static except the API routes.
- Admin actions require a signed request from the local CLI; no public admin endpoint.

### Failure handling

| Failure | Behavior |
|---|---|
| Webhook delayed | Welcome page polls; email arrives late; key can be fetched by `txn`. |
| Webhook fails / 5xx | Paddle retries; the Worker returns 2xx only after a durable write. |
| Duplicate webhook | Ignored by `event_id`; same key returned. |
| Email provider down | Key is still shown on the success page; cron retries; support can resend. |
| Backend down at purchase | Paddle still charges; reconcile + manual issuance from the Paddle dashboard. |
| Customer offline | Existing keys verify offline forever; activation is optional. |
| Refund while offline | Grace period; revocation applies on the next successful check. |

### Hosting and cost

- **Default:** Cloudflare Pages + Workers + KV + Cron. Free tier covers launch volume; paid tier is ~$5/month.
- Alternatives: Vercel/Netlify functions + Upstash; AWS Lambda + DynamoDB. Same interface, more setup.
- Target fixed cost: **$0–5/month**, plus the email free tier. No servers to patch, no database to operate.
- A daily reconciliation job compares Paddle transactions against issued keys and alerts on drift.

## Customer journey (purchase → unlock)

| Step | User sees | System |
|---|---|---|
| 1 | Trial footer: “Trial · N days left” | Entitlement = `trial` |
| 2 | Clicks Unlock (footer or Settings → License) | Opens `copyfie.app/buy` |
| 3 | Paddle checkout, localized, card/wallet options | Paddle handles tax and 3DS |
| 4 | “Thanks — here is your key” page | Webhook issues key; page shows it |
| 5 | Clicks **Open Copyfie** | `copyfie://activate?key=…` |
| 6 | App shows “License activated · 3 devices” | Offline verification; instant unlock |
| 7 | Key also arrives by email | Resend / restore via `/restore` |
| 8 | Reinstall or second machine | Paste key, or restore by email |

- The deep link is strictly validated: scheme `copyfie`, single `key` parameter, no auto-paste, no silent clipboard write.
- If the user is on another device, the same key works (up to 3 activations).

## Pricing and refunds

- Target list price **$15–19**, launch price **$12–15**; includes **3 devices** and v1.x updates; v2.0 is a paid upgrade.
- Before finalizing, run the competitor scan (one-time clipboard managers and adjacent utilities on Paddle/Lemon Squeezy/Gumroad, Mac App Store, Microsoft Store, direct sites) and record it in `docs/LICENSING.md`.
- Refund policy: **14 days, no questions asked**, handled by Paddle — the same length as the trial, so the “try it free, then buy” promise is coherent. Revocation follows the 14-day grace.
- Marketing copy: “**14 days free, then a one-time purchase**” — never “subscription”, never “free forever”, never a countdown that implies data loss.
- Regional pricing / purchasing-power parity is a v1.x decision after launch data.

## Payouts (India)

- Paddle pays the seller **monthly**, with a **$100 minimum** payout. Confirm the exact current schedule and threshold before relying on it.
- Two routes to an Indian bank account:
  - **Bank wire (SWIFT):** Paddle sends USD; intermediary and receiving-bank fees (~$15–30) plus FX spread (~1–3%); needs FIRC/BRC documentation for export accounting.
  - **Payoneer:** USD receiving account with lower fixed fees; withdrawal to an Indian bank typically costs ~1–2%; better for small monthly volumes and simpler documentation.
- **Minimize fixed fees:** keep one payout method, let the monthly balance accumulate above the $100 minimum, avoid mid-cycle withdrawals, batch refunds before payout, and price in USD so FX is the only conversion.
- Taxes: export of services; GST zero-rated/LUT treatment and documentation depend on entity type. Get an accountant to confirm, and keep FIRCs.
- Track: gross sales, Paddle fee, payout fee, FX spread, net INR. Revisit pricing if net margin is squeezed.

## Pre-launch verification checklist

| Check | Pass condition |
|---|---|
| Seller approval | Paddle account approved as an Indian business/individual; live site, pricing, refund policy, terms, privacy policy, product description all present. |
| Test purchase | Sandbox and one live low-value purchase complete; key issued, emailed, and unlocks the app. |
| Test refund | Refund via Paddle dashboard marks the license revoked; app shows grace then limited mode; no data lost. |
| Webhook reliability | Replay, duplicate, delayed, and 5xx cases behave as specified; reconciliation finds no drift. |
| Restore purchase | Email-based restore re-sends the correct key; generic response for unknown emails. |
| Payout test | A test payout reaches the chosen route; fee, FX, and FIRC documented. |
| Paddle terms | Current payout schedule, $100 minimum, fees, refund/chargeback rules, and India availability confirmed with Paddle in writing. |
| Accounting | Accountant review of GST/export treatment and payout documentation. |
| Refund window | Policy matches the 14-day trial; wording on the site and in the EULA is consistent. |

## Limited mode matrix (post-trial)

| Capability | Trial / Licensed | Locked |
|---|---|---|
| Background capture (text, images, files) | Yes | **Yes — continues unconditionally** |
| Retention, dedupe, privacy gate, blocklist | Yes | **Yes — unchanged** |
| 25 most recent text items | Yes | **Yes** |
| Older text, images, files, pinned items in the list | Yes | **No** |
| Full-history search / FTS | Yes | **No** — local filter over the 25 visible items only |
| Type filters | All | **All + Text only** |
| Paste/copy of visible items | Yes | **Yes** |
| Pin/unpin, delete, block current app | Yes | **No** |
| Settings | All tabs | **Only License, About, Privacy, Quit, and Clear history** (the rest of Settings is visible but disabled) |
| Hotkey, tray, autostart, single instance | Yes | **Yes** — configured values keep working; the tray always offers **Pause capture** and **Quit** |
| Snippets, OCR, custom themes, folder sync | Yes | **No** |
| Export/backup | Yes | **No** (data is not deleted; it is only hidden) |
| Clear history | Yes (keeps pins; optional “including pinned”) | **Yes, with confirmation** — clears everything, including pinned items, because pins are hidden and unmanageable in locked mode |
| Deletion at expiry | — | **None. Nothing is deleted.** |

UI details are in §7 (“Limited mode”). The only post-trial prompt is one footer line with an Unlock action. Because the trial now runs 14 days, “N more items waiting” is typically much larger than it would have been at 7 days — it is computed from hidden text items, not a constant, and it is the main conversion surface.

## Folder sync design (v1.2, optional)

- **No server, no relay, no account.** The transport is a folder the user already syncs: iCloud Drive, Dropbox, OneDrive, Syncthing, or a network share.
- Layout inside the chosen folder:
  ```
  Copyfie/
    manifest.enc
    devices/<device_id>/oplog-<seq>.jsonl.enc
    blobs/<hmac(master, content_hash)>.enc
  ```
- Pairing: device A displays a QR/manual **sync key** containing a random `sync_id` and a 32-byte master key. Optionally derive the master key from a passphrase with Argon2id. The key never leaves the devices in plaintext.
- Crypto: per-device X25519 identity, epoch keys via HKDF, content and metadata encrypted with XChaCha20-Poly1305. Blob filenames are keyed HMACs so the provider cannot test equality of plaintext hashes.
- Conflict handling: per-device append-only oplog + manifest merge; hybrid logical clock; last-write-wins for mutable fields; **pins win ties**; tombstones retained 30 days. Items are mostly immutable (content, timestamp), so the conflict surface is small.
- Provider quirks: atomic temp-write + rename; tolerate “conflicted copy” files; never delete unknown files; detect iCloud “optimize storage” placeholders; show “sync folder unavailable” instead of failing capture; expose a conflict log.
- Threat model: the provider can read sizes/timing, delete, or roll back files. Detect rollback with monotonic counters; never claim confidentiality against a compromised device. No plaintext ever leaves the device.
- Sync is off by default and requires explicit folder selection + pairing.

## Anti-piracy posture

- Every client-side check can be patched. The goal is to make paying easy and casual bypass annoying, not to deploy DRM.
- Do: centralized entitlement checks, stripped release symbols, obfuscated key material, signed licenses, optional activation, signed revocation lists, regular updates.
- Do not: kernel drivers, rootkits, background phone-home, clipboard telemetry, hardware-locking, or breaking offline use. Those conflict with the privacy promise and cost more than they recover.
- The trial is deliberately **calendar-based and locally enforced**: 14 days, signed record, redundant anchors, rollback detection. That is tamper-resistant enough for a $15 product without turning the app into spyware.
- Accept some leakage. Support and update quality are the real retention strategy for a one-time product.

## Operational tooling

- `scripts/license/keygen.rs`: generate the Ed25519 keypair **once**, offline. Private key never enters the repo, CI, or the app; it lives in the backend secret store.
- `scripts/license/issue_key.rs`: local issuance for support/refunds/manual orders when the backend is unavailable; same payload format as the Worker.
- `scripts/license/verify_key.rs`: support tool to validate a customer key without the private key.
- `backend/`: the Worker, its tests, and a local `wrangler dev` setup. Deployed from CI with a scoped token.
- Signed revocation list generated by the backend, bundled at release, and fetched by the app.
- CI has no private keys. Release/code-signing keys, license-signing keys, and Paddle secrets are all separate.
- A signed admin CLI handles: re-issue, revoke, seat reset, and “did this customer’s webhook arrive?”.

---

# 15. Risks and mitigations (ranked)

| Rank | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | **Limited mode feels punitive and kills conversion** | Medium | High | Show 25 real, useful items; one footer line; instant unlock; never delete data; measure trial→paid conversion. Keep 25 as the shipped behavior unless evidence strongly justifies a change. |
| 2 | **Total RSS exceeds the 120 MB gate** | Medium | High | M1 measures per-OS core + total RSS. Gate blocks M1 exit >120 MB. Tighten caches, reduce pre-warm scope, or formally raise the budget. |
| 3 | **macOS Accessibility/focus/paste reliability** | Medium | High | Explicit permission onboarding; deterministic target PID restore; `Key::Other(9)`; copy-only fallback; test installed/ad-hoc builds, not just dev. |
| 4 | **Windows focus restore / elevated targets** | Medium | High | Poll active window; `AttachThreadInput` fallback; detect elevation; copy-only with toast; never paste blind. |
| 5 | **Clipboard echo/dedupe races** | Medium | High | Suppression token + persistence acknowledgement + generation; integration tests for copy-after-delete/clear. |
| 6 | **Hidden WebView throttling kills 100 ms** | Medium | High | Windows browser args; test off-screen prewarm; measure p95 per OS; fallback plan. |
| 7 | **Trial is reset by deleting app data, or patched** | High | Medium | Redundant anchors, signed record, and monotonic clock stop casual resets. Accept that a determined user can bypass it; do not add invasive DRM or hardware locking. Keep the paid path cheaper than the workaround. |
| 8 | **Price is wrong for the market** | Medium | High | Run competitor research before launch; launch at $12–15 with a clear one-time promise; raise to $15–19 later; watch refund rate and conversion. |
| 9 | **Paddle seller approval delayed or rejected** | Medium | High | Build the site and legal pages first, apply early, keep the product description/pricing/refund policy complete and consistent, and treat approval as external latency in the schedule. The `PaymentProvider` seam allows a different MoR without app changes. |
| 10 | **Name, domain, or handle conflict forces a rename** | Medium | High | Clear naming before packaging: domain, trademark search, GitHub org/repo, winget publisher ID, Homebrew tap, AUR/package name. A late rename means new bundle id, installers, URLs, notarization and Paddle metadata. |
| 11 | **Key delivery fails (webhook missed, email lost)** | Medium | High | Show the key on `/welcome` from the webhook record, not only by email; poll briefly; idempotent re-fetch by `txn`; restore by email; cron retries; daily reconciliation against Paddle. |
| 12 | **India payout/FX costs and documentation** | Medium | Medium | Compare wire vs Payoneer, batch monthly above the $100 minimum, keep one route, track fees/FX, and have an accountant confirm GST/export treatment and FIRC records before launch. |
| 13 | **Payout holds, refund/chargeback abuse** | Medium | High | Use Paddle as MoR; keep offline key issuance as a manual fallback; signed revocation list; 14-day grace instead of instant lockout. |
| 14 | **One-time revenue cannot fund ongoing v1.x updates** | Medium | Medium | Scope v1.x tightly; v2.0 is a paid upgrade; 3-device limit; minimal backend cost (free tier); keep the team/overhead small. |
| 15 | **SmartScreen/Gatekeeper warnings block installs and kill conversion** | Medium | High | Advertise the one-liners only after Authenticode + notarization; scripts verify OS signatures and abort otherwise; beta page states “unsigned, expect warnings”; EV/OV or Azure Trusted Signing to shorten the reputation ramp. |
| 16 | **Install scripts become a supply-chain vector** | Medium | High | Small reviewed scripts published as release assets; linted in CI (`shellcheck`, `PSScriptAnalyzer`); SHA-256 + detached signature verified before anything executes; pinned version, `--dry-run`, no hidden downloads. |
| 17 | **Trial reminders annoy users or feel like nagging** | Medium | Medium | At most one reminder per day, dismissible, footer-only, none during a paste; days 10/13/14 only; measure dismissal rate and trial→paid conversion. |
| 18 | **Folder sync conflicts or provider eviction (v1.2)** | High | Medium | Append-only per-device oplog; never overwrite unknown files; conflict copies surfaced; iCloud “optimize storage” detection; sync is optional. |
| 19 | **v1.0 Linux is X11-only; Wayland users get no capture/paste** | High | Medium | State it clearly in the README/release notes. Detect Wayland, show capability notice, keep copy-only fallback. Prioritize full Wayland in v1.x. |
| 20 | **No auto-updater in v1.0 slows security fixes** | Medium | Medium | Manual “Check for updates” link + release notes; prioritize signed updater in v1.x; document the manual update path. |
| 21 | **Image storage growth** | Medium | Medium | Default 30-day retention, user-selectable Forever, 1 GB image cap with oldest-unpinned eviction, orphan GC, atomic files; pinned images never evicted. |
| 22 | **License/activation looks like telemetry** | Medium | Medium | No clipboard content in activation or purchase; document exactly what is sent (license id, salted device hash, txn id); offline unlock always works; activation is optional. |
| 23 | **Proprietary LICENSE/EULA not ready at release** | Medium | High | Release gate before any binary distribution; counsel review; EULA and notices in the installer and About. |
| 24 | **Wayland technical limitations when v1.x ships** | High | Medium | Accept toplevel + float rules; capability matrix; GNOME unsupported without extension; copy-only fallback. |
| 25 | **Source app identity unavailable on Wayland** | Certain | Medium | Show “Unknown”; disable blocklist enforcement there; document the gap. |
| 26 | **Flatpak sandbox blocks background capture** | High | Medium | Validate with a spike in v1.x before promising; make deb/rpm primary. |
| 27 | **Signing/notarization credentials not wired into CI in time** | Medium | High | Ship unsigned only for beta; wire Apple Developer ID + Windows signing secrets early; keep bundle IDs stable; block the paid launch, not the beta, on this. |
| 28 | **FTS5 query correctness/injection** | Low | Medium | Tokenize + quote + bind parameters; fuzz tests; no raw MATCH interpolation. |
| 29 | **Reference code provenance/attribution drift** | Medium | Medium | `THIRD_PARTY.md`, MIT file headers, `cargo-deny`, PR checklist; ship notices in the app and installer. |
| 30 | **Autostart stale path / first-run failures** (Ortu #20, #28) | Medium | Medium | Repair autostart path every launch; ignore debug builds; test login flow. |
| 31 | **Multi-monitor/DPI placement bugs** (Ropy #113, Ortu #21) | Medium | Low | Use Ortu’s cursor-monitor + work-area + DPI math; manual matrix. |
| 32 | **Hotkey rebinding records Win/Super as Ctrl** (Ortu #13) | Medium | Low | Capture each modifier independently; platform-specific tests. |
| 33 | **Test flakiness (clipboard/GUI)** | High | Low | Isolate data dirs; retries only for infrastructure; keep GUI tests smoke-level; manual checklist for paste. |

---

# 16. Decision log and release gates

All previously open questions are resolved in the **Decisions** table at the top of this document. The remaining items are gates, not open design questions:

| Gate | When | Owner action |
|---|---|---|
| **Proprietary `LICENSE` + EULA** | Before any binary distribution | Counsel-reviewed all-rights-reserved license, EULA, privacy policy, and refund policy. MIT notices remain intact. No copyleft headers. |
| **Product identity** | Before v1.0 packaging/signing | Name is final (**Copyfie**). Verify `copyfie.app` availability/ownership, trademark conflicts, and GitHub/winget/Homebrew/package handles; then lock `com.copyfie.app`, accent color, and copyright line. |
| **Naming clearance** | Before v1.0 packaging/signing | Domain secured, trademark search clean or name adjusted, and all handles reserved; a rename after packaging means new bundle id, installers, URLs, and a re-do of Paddle/notarization metadata. |
| **Code signing / notarization** | Before charging money | Apple Developer ID + notarization; Windows code-signing certificate. Unsigned builds are beta-only and must say so; the one-line installers are not advertised until this gate passes. |
| **Install + download path** | Before launch | One-liners work on clean VMs for every supported OS; `SHA256SUMS` + detached signature published; download page live with manual steps and uninstall commands; winget manifest and `copyfie/tap` cask accepted. |
| **Merchant of Record (Paddle)** | Before launch | Paddle seller account approved as an Indian business/individual; live site, pricing, refund policy, terms, privacy policy, and product description in place; test purchase and refund completed end to end. |
| **License backend** | Before launch | Webhook signature verification, replay protection, idempotent issuance, key email, restore-purchase, revocation list, and cron reconciliation all verified against sandbox and one live transaction. |
| **Payout + accounting** | Before launch | Payout route chosen (wire or Payoneer), schedule and $100 minimum confirmed, fee/FX documented, FIRC handled, accountant has reviewed GST/export treatment. |
| **Pricing validation** | Before launch | Competitor scan; confirm launch and list price; record in `docs/LICENSING.md`. |
| **Trial + limited-mode acceptance** | v1.0 exit | Test 14-day trial start, anchors, clock rollback, day 10/13/14 reminders, 25-item view with real hidden count, locked settings, instant unlock, offline verification, reinstall, refund grace. |
| **RAM gate** | Milestone 1 exit | Core <40 MB; total 60–100 MB expected. If total >120 MB, resolve before M1 exit. |
| **Folder-sync threat model** | v1.2 | Review the no-server design, provider rollback, conflict handling, and key rotation before shipping. |
| **Wayland/Flatpak/MSI/AppImage/auto-updater** | v1.x planning | Explicitly deferred from v1.0; do not let them creep into the v1.0 scope. |

No decision currently blocks **Milestone 1**. Milestone 1 may start once the user says “start Milestone 1”.

---

# 17. Attribution & compliance

- **`LICENSE`**: **proprietary, all rights reserved**, plus a **EULA**. The repo is private and the binary is closed source. No copyleft headers anywhere.
- **No contract crates and no second build**: `copyfie-license` is an internal verifier crate under the same proprietary license. Nothing is published as a shared SDK.
- **`NOTICE`**: project copyright, plus:
  - “Portions adapted from Ortu, Copyright (c) 2025-2026 Abhijith P Subash, MIT.”
  - “Portions adapted from Ropy, Copyright 2025-2026 The Ropy Authors, MIT.”
- **`THIRD_PARTY.md`**: maintained from day one, generated by `cargo-about`/`cargo-deny`, with a manual table for adapted code:
  | Our file | Upstream repo | Upstream commit | License | What changed |
- **Source headers**: every copied/adapted file gets:
  ```rust
  // Adapted from Ortu (MIT), <commit>, src-tauri/src/clipboard.rs.
  // Copyright (c) 2025-2026 Abhijith P Subash. See THIRD_PARTY.md.
  ```
- **License texts**: keep upstream MIT text under `third_party/licenses/ortu-MIT.txt` and `ropy-MIT.txt`.
- **PR checklist**: “If this copies reference code, did you update THIRD_PARTY.md and keep the MIT header? Did you avoid adding any GPL/AGPL code or dependency?”
- **Dependency hygiene**: CI runs `cargo-deny` against the proprietary-product allowlist and fails on GPL/AGPL/SSPL/non-commercial transitive dependencies. `npm audit --omit=dev` as well.
- **Notices in the product**: About → Open-source notices renders `THIRD_PARTY.md`; the installer and release assets include `LICENSE`, `EULA`, and `THIRD_PARTY.md`. This satisfies the MIT “all copies” obligation for a binary-only distribution.
- **No trademark use**: don’t reuse either project’s name/logo; describe them only as inspiration/attribution.

---

## What is decided before Milestone 1

Nothing blocks Milestone 1. The name (**Copyfie**), architecture, RAM target, hotkey, scope trim, trial/limited-mode design, and deferred-work list are settled. The binding gates are:

1. **Proprietary `LICENSE` + EULA** — before any binary distribution.
2. **Naming clearance + branding** — verify `copyfie.app`, trademark conflicts, and GitHub/winget/Homebrew/package handles; then lock `com.copyfie.app`, accent color, and copyright line before packaging/signing.
3. **RAM gate** — at M1 exit: core <40 MB, total 60–100 MB; resolve if total >120 MB.
4. **Pricing validation** — before launch; competitor scan and final price.
5. **Paddle seller approval + license backend** — before charging money: approved seller account, working webhook/key service, test purchase and refund.
6. **Code signing** — before charging money, and before the one-line installers are advertised.

**No implementation has started.** When you want implementation to begin, say **“start Milestone 1”**. Until then, this remains plan-only.
