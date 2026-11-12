# Copyfie — Progress Log

> Read this file and `task.md` at the start of every session.
> Append a new dated section per session. Never rewrite history; correct it with a new entry.

---

## 2026-11-12 — Session 1 (Phase 0: docs and setup)

**Scope:** Phase 0 only — no app code, no scaffolding.

### Done

- **Paddle docs MCP:** added `paddle-docs` (`https://paddlehq.mcp.kapa.ai`) to the global OpenCode config via `opencode mcp add paddle-docs --global --url …`. Status: **needs authentication** — sign in with `/mcps` (Google/GitHub) when convenient. No Paddle account or API key was created.
- **PLAN.md edits:**
  - Naming clearance moved to v1.0 task 0; release gate, site, Paddle onboarding, and all later tasks shifted by one (v1.0 now **0–29**).
  - Locked mode now allows **Clear history** with a confirmation (clears all items, including pinned, because pins are hidden while locked). Updated the decisions table, §7 locked-mode table, §14 matrix, acceptance criteria, and the manual checklist.
- **Task tracking:** created `task.md` (all milestone tasks, IDs M1/M2/MX/M3/M4) and this `progress.md`.
- **Git:** `.gitignore` created; repo initialized; `origin` set to `https://github.com/dhruv-kashyap47/copyfie.git`; PLAN.md, task.md, progress.md, .gitignore committed and pushed.

### Decisions

- Milestone IDs: `M1` = MVP, `M2` = v1.0, `MX` = deferred v1.x, `M3` = v1.1, `M4` = v1.2.
- `logo/` is deliberately **not** committed yet (untracked local asset); decide later whether it belongs in the repo.
- Paddle work (account, backend, checkout) stays deferred to M2; consult the docs MCP first when it starts.

### Blockers

- `paddle-docs` MCP awaits OAuth sign-in (`/mcps`). Not blocking Phase 0.

### Measurements

- None yet (no app build).

### Next up

- Stop and wait for **"start Milestone 1"**.
- When approved: **M1-01** (repo/workspace, Tauri hello, CI skeleton, `cargo-deny`, NOTICE/THIRD_PARTY), then M1-02 onward in order.

---

## 2026-11-12 — Session 2 (Milestone 1, M1-01)

**Scope:** M1-01 only — repo/workspace, Tauri hello, CI skeleton, `cargo-deny`, NOTICE/THIRD_PARTY.

### Done

- **Workspace:** root `Cargo.toml` with `src-tauri` + `crates/copyfie-core`; release profile per PLAN §8 (`opt-level="s"`, `lto="fat"`, `codegen-units=1`, `panic="abort"`, `strip="symbols"`, `copyfie-core` at `opt-level=3`). `rust-toolchain.toml` pins stable with `rustfmt` + `clippy`. `rust-version` is intentionally unpinned; CI tracks stable.
- **Tauri hello:** `src-tauri` builder with one command, `app_info` (name, app version, core version, bundle id). Frameless, always-on-top, 680×480 palette window (visible for now; M1-07 makes it resident/hidden). Capabilities: `core:event:default` only, no fs/shell/http. Strict CSP with a dev-only variant for Vite WS/HMR.
- **Frontend:** Svelte 5 + Vite 8 + TypeScript 6, plain CSS with the token set from PLAN §7. Hello screen shows brand, bridge status, and versions. `svelte-check` clean. `vitest` wired with `passWithNoTests` until the palette tests land.
- **`copyfie-core`:** crate scaffold, no dependencies yet, with the "no Tauri, no OS APIs" rule documented; one unit test.
- **CI skeleton (`.github/workflows/ci.yml`):** `frontend` (check, test, build, bundle budget), `rust` matrix (`ubuntu-22.04`, `windows-latest`, `macos-14`: fmt, clippy `-D warnings`, tests, debug build), `licenses` (`cargo-deny` allowlist, `npm audit --omit=dev`, attribution files present).
- **Bundle budget gate:** `scripts/bundle/check_frontend_size.mjs` — 50 KB gzipped JS+CSS limit; current total **14.0 KB**.
- **Attribution:** `NOTICE`, `THIRD_PARTY.md`, and verbatim MIT texts at `third_party/licenses/{ortu,ropy}-MIT.txt`. Pinned provenance: Ortu `v2.1.0` @ `b5747d3…`, Ropy `0.6.2` @ `0d7ac38…`. No reference code copied yet, so the adapted-files table is empty by design.
- **`cargo-deny` policy:** `scripts/license/cargo-deny.toml`. Verified locally with `cargo-deny 0.20.2`: **advisories ok, bans ok, licenses ok, sources ok**.

### Decisions

- **cargo-deny config lives at `scripts/license/cargo-deny.toml`** per PLAN §4; CI passes `--config` explicitly (verified to work with the CLI).
- **Allowed licenses** extended with **Unicode-3.0** (Unicode License v3, OSI-approved) because Tauri's `idna`/`icu_*` chain requires it. **MPL-2.0** is allowed for unmodified dependencies only; no MPL code is copied into our sources.
- **Wildcard ban:** the path dependency is declared as `copyfie-core = { path = …, version = "0.1.0" }` so `wildcards = "deny"` passes without weakening the policy.
- **Placeholder app icon** generated at `app-icon.png` (1024², purple gradient + “C”) and expanded via `tauri icon` into `src-tauri/icons/`. Final branding is gated on M2-00 (naming clearance) and M2-21 (packaging). `android/` and `ios/` icon sets were deleted — desktop only.
- **`logo/` remains untracked** (wide wordmark, not an icon source). The icon source is `app-icon.png`, committed.
- **TypeScript 6, not 7:** `svelte-check` 4.7 peers on TS `^5 || ^6`, so TS 7 would break the type-check job.
- **`dist/` is built before Rust jobs in CI** because `tauri::generate_context!` needs the frontend assets at compile time.

### Verification

- `npm run check` → 0 errors, 0 warnings.
- `npm test` → no test files (passWithNoTests), exits 0.
- `npm run build` → 35.8 KB JS / 2.65 KB CSS (14.0 KB gzipped total).
- `npm run size:check` → 14.0 KB / 50 KB budget.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` → clean.
- `cargo build -p copyfie` and `npm run tauri build -- --debug --no-bundle` → success.
- Launched `target/debug/copyfie.exe`, window titled “Copyfie” rendered the hello screen and reported **Tauri bridge connected / App 0.1.0 / Core 0.1.0 / com.copyfie.app**. Process stopped after the check.
- `cargo deny --config scripts/license/cargo-deny.toml check` → all four checks ok. `npm audit --omit=dev` → 0 vulnerabilities.

### Measurements

- Frontend bundle: **14.0 KB gzipped** (JS 13.0 KB + CSS 1.0 KB) vs 50 KB budget.
- Debug build time: ~1m50s cold, ~24s incremental. Release/LTO timing deferred to M1-11.
- RSS: **not measured yet** (M1-12). No RAM gate concerns at scaffold stage.

### Notes / follow-ups

- `cargo deny` reports a `cssparser` duplicate (warn only) from Tauri's HTML parsing chain; revisit if the warnings grow.
- Dev builds need `npm run dev` running (Tauri `devUrl`); `cargo run` alone shows a blank window unless Vite is up.
- CI bundle budget (50 KB) and the `rust` matrix are the first items to revisit when the palette UI lands (M1-06) and when installers are added (M2-21).

### Next up

- **M1-02** — `copyfie-core`: models, BLAKE3 hashing/canonicalization, SQLite schema + migrations, dedupe, basic retention, unit tests.

---

## 2026-11-12 — Session 3 (Milestone 1, M1-02)

**Scope:** M1-02 — `copyfie-core` models, BLAKE3 hashing/canonicalization, SQLite schema + migrations, dedupe, basic retention, tests.

### Done

- **`model.rs`:** `ItemKind` (text/image/files), `Item` (full row), `ItemSummary` (palette payload, camelCase serde), `SourceApp`, `NewItem` with `text()` / `files()` constructors, `make_preview` (400 chars, code-point safe), `ListQuery`.
- **`hash.rs`:** CRLF→LF + trailing-NUL canonicalization, BLAKE3-256 `hash_text`/`hash_bytes`, path normalization (keeps `/` and `C:\` roots, trims trailing separators), order-insensitive `hash_files`, `hash_image_rgba` (pixels + dimensions, ready for M2-04).
- **`retention.rs`:** `Retention::{Days, Forever}` with `0 = Forever` sentinel, 1–365 clamping, saturating expiry math, `DAY_MS`.
- **`store/`:** `Database` (file + in-memory) with the PLAN §5 pragma set (WAL, NORMAL, foreign keys, 8 MB cache, mmap, secure_delete, incremental auto_vacuum + one-time VACUUM on upgrade). Forward-only migrations via `PRAGMA user_version`, future schema refused. Atomic dedupe upsert on `(kind, content_hash)` with pin/expiry preservation and `is_new`. Listing with kind/pin filters and paging, pin/unpin with expiry recompute, `touch_used`, delete, clear (keep pins), meta key/value, per-kind counts, age cleanup and incremental vacuum.
- **Migrations:** `0001_init.sql` (meta + items), `0002_fts.sql` (external-content FTS5 + insert/delete/update triggers, `unicode61 remove_diacritics 2`, `prefix='2 3'`, `fts_built` flag), `0003_indexes.sql` (unique dedupe index, recency, pins, partial expiry, source app).
- **Tests:** **38 passing** — 21 unit (hashing, preview, retention, migration, FTS triggers) + 17 integration (dedupe, canonicalization, recopy source/use_count, pins, retention boundary, list order/filters/paging, clear, meta, counts, file dedupe).

### Decisions

- **Migrations live in `crates/copyfie-core/migrations/`**, not `src-tauri/migrations/` as sketched in PLAN §4. The store crate owns the schema; `include_str!` stays inside one crate and core remains free of Tauri. File names still match the plan (`0001_init`, `0002_fts`, `0003_indexes`).
- **rusqlite 0.42:** the separate `fts5` feature no longer exists; FTS5 is compiled into the `bundled` build. Verified by FTS trigger tests. `rusqlite` requires Rust ≥1.89.
- **`is_new` uses `RETURNING ... (first_seen_at = captured_at)`**, not `excluded.captured_at`: SQLite's `RETURNING` cannot reference the `excluded` pseudo-table. Edge case: a recopy in the same millisecond as first capture reports `is_new = true`; no data effect (single-writer actor in M1-03 serializes writes).
- **Files payload keeps display order, hash ignores order.** `NewItem::files` stores the JSON array in selection order (Explorer/Nautilus order is meaningful) while `hash_files` sorts + trims for identity.
- **`Retention` lives at crate root** (policy), DB cleanup in `store/retention.rs` (SQL).
- **`secure_delete=ON`** from day one so deleted text is zeroed on disk.

### Verification

- `cargo test -p copyfie-core` → 38 passed, 0 failed.
- `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo deny --config scripts/license/cargo-deny.toml check` → advisories/bans/licenses/sources all ok.
- FTS5 confirmed working in the bundled SQLite via schema-level tests (insert → match, recopy reindex, delete, retention cleanup).

### Measurements

- Core crate test suite: ~0.11 s total (21 unit + 17 integration), in-memory and temp-file DBs.
- Schema at v3; no migration cost yet. Insert/search benchmarks deferred to M1-11.

### Notes / follow-ups

- `store::items::upsert` is synchronous and takes `&self`; M1-03 wraps it in the single-writer actor and adds the coalesced `history:changed` event.
- Search query building (secure FTS `MATCH` escaping) is still to come; the FTS table + triggers are ready and tested.
- Pinned items are excluded from age cleanup by `expires_at = NULL`, so retention SQL never needs a pin join.

### Next up

- **M1-03** — store actor + custom URI protocol for thumbnails (stub).

---

## 2026-11-12 — Session 4 (Milestone 1, M1-03)

**Scope:** M1-03 — store actor + custom URI protocol for thumbnails (stub).

### Done

- **Store actor (`src-tauri/src/store/mod.rs`):** one dedicated `copyfie-store` thread owns the `Database`. Cloneable `StoreHandle` sends boxed closures over an `mpsc` channel and awaits `tokio::sync::oneshot` replies, so there is no `Mutex<Connection>` and no blocking of the async runtime. `ActorError::{Stopped, Store}` keeps SQLite failures typed.
- **App wiring:** `setup` resolves `app_data_dir()`, creates the data root and `thumbs/`, opens `clipboard.db` (migrations run), and manages `AppState` with the actor handle.
- **Thumbnail protocol (`src-tauri/src/thumbs.rs`):** `copyfie://localhost/<64-hex>.png` plus the Windows `http://copyfie.localhost/...` form. Strict lowercase-hex key validation (no path traversal possible), `image/png`, `Cache-Control: public, max-age=31536000, immutable`, `X-Content-Type-Options: nosniff`, typed 400/404/500 responses. Images themselves are M2-04; the URL contract and caching are settled now.
- **Commands:** `app_info` (moved into `commands/mod.rs`) and `store_stats` (`commands/history.rs`) prove the actor end-to-end from the webview.
- **Hello screen:** now shows store counts through the actor.
- **CSP:** added `http://copyfie.localhost` to `img-src` (Windows custom-protocol origin) in both prod and dev policies.

### Decisions

- **Actor lives in `src-tauri`, not `copyfie-core`.** Core stays a synchronous, thread-agnostic library; the shell owns threads and async glue (matches PLAN §3/§4).
- **Closure-based job queue** instead of a large request enum. Commands stay a few lines and the queue does not grow into a god-enum; the single thread still serializes every operation.
- **Thumbnails resolve the data dir per request via `PathResolver`**, so the protocol handler needs no captured state and avoids any setup/webview ordering hazard. Tauri caches base dirs.
- **`AppState.data_dir` deferred** until the capture pipeline (M1-04) consumes it, keeping `-D warnings` clean.
- **Content-addressed thumbnails are immutable for a year**, so the cache header is safe even during dev hot reload.

### Verification

- `cargo test -p copyfie` → **10 passed** (actor serialization/concurrency/error recovery; thumbnail validation, serving, Windows URL form, 404s).
- `cargo test --workspace` → **48 passed** total.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo deny --config scripts/license/cargo-deny.toml check` → advisories, bans, licenses, sources all ok.
- Built and launched `target/debug/copyfie.exe`: the window showed **Store 0 items · 0 text · 0 image · 0 files**, and `%APPDATA%\com.copyfie.app\` contained `clipboard.db` (WAL) plus `thumbs/`.

### Measurements

- Actor test suite: 0.01 s. First-run data dir: `clipboard.db` 4 KB + 53 KB WAL at schema v3.
- No RSS measurement yet (M1-12).

### Notes / follow-ups

- The store thread exits when the last `StoreHandle` drops at process exit; add an explicit flush/shutdown at M1-13 if needed.
- `store_stats` is temporary diagnostics; real history commands arrive at M1-06.
- Screenshot check showed square corners on the frameless Windows window — DWM rounding is scheduled in M1-07.

### Next up

- **M1-04** - capture pipeline: privacy gate, text capture, source app, suppression, coalesced `history:changed`.

---

## 2026-11-12 — Session 5 (Milestone 1, M1-04)

**Scope:** M1-04 — capture pipeline: privacy gate, text capture, source app, suppression, coalesced `history:changed`.

### Done

- **`platform/common` (`copyfie-platform`):** new workspace crate with no OS APIs. Traits `ClipboardWatcher`, `ClipboardReader` (two-phase `probe(sequence)` then `read_text(sequence)`), `ForegroundApp`, a `ClipboardProbe` (text/image/files/concealed), `PlatformError`, and blanket `Arc`/`Box` impls so tests can share fake backends.
- **Capture pipeline (`src-tauri/src/capture/pipeline.rs`):** probe → snapshot config + foreground app → privacy gate → read payload → canonicalize → empty check → echo check → store upsert → emit. Platform and clock are injectable. Outcomes distinguish `Stored { id, is_new }` from `Skipped(reason)`; platform/store failures return typed `CaptureError`.
- **Privacy gate (`capture/privacy.rs`):** `CaptureConfig` (paused, skip_concealed, retention, blocklist), `PrivacyGate::evaluate` with order paused → concealed → blocked app → has-text, and per-OS blocklist matching: Windows case-insensitive full path/basename (slash-normalized), macOS exact bundle id, Linux `WM_CLASS` or executable basename. Rules are platform-scoped.
- **Echo suppression (`capture/suppress.rs`):** one-shot, content-addressed token with a 2 s TTL; consuming clears it; re-arming replaces; expired tokens never match; `arm_text` canonicalizes so CRLF copies match.
- **Coalesced events (`capture/events.rs`):** `EventCoalescer` emits at most once per 80 ms window with a monotonic revision; `tauri_history_events(app)` bridges to the `history:changed` Tauri event; `HistoryEvents` trait keeps the pipeline testable.
- **Source-app helpers (`capture/source_app.rs`):** two-separator basename and display-name fallback (`name` → executable basename → "Unknown").
- **`StoreHandle::call_blocking`:** blocking actor call for dedicated threads (capture/cleanup), alongside the async command path.
- **Dedupe correction (M1-02 follow-up):** `is_new` is now decided by *which statement matched* — `INSERT ... ON CONFLICT DO NOTHING RETURNING id`, else `UPDATE ... RETURNING id` — instead of `first_seen_at = captured_at`. The timestamp trick misreported a same-millisecond recopy as new; a regression test now covers it.

### Decisions

- **Two-phase read is enforced by types**, not convention: the gate only receives a `ClipboardProbe`, and `read_text` is callable only after `Allow`. Pipeline tests assert read count stays 0 for concealed, paused, and blocked-app changes.
- **Pause is evaluated before concealed/blocklist** to avoid needless foreground queries; all three paths are skip-only and never read the payload.
- **Whitespace-only text is skipped** (`EmptyText`) in addition to truly empty text — stray whitespace copies are noise, and this avoids near-duplicate rows in the palette.
- **Suppression TTL is monotonic (`Instant`)**, not wall-clock, so a clock change cannot extend an echo window.
- **Events fire only after a successful write.** A failed `upsert` leaves no event and no armed suppression, so the next OS notification retries the same content.
- **`history:changed` carries only `{ revision }`.** Coalescing discards per-row detail by design; the palette refetches and can ignore stale revisions.
- **`capture` is a public module** in the app lib until the platform runner (M1-05) consumes it; this keeps the library API honest and `-D warnings` clean without allows.

### Verification

- `cargo test --workspace` → **83 passed** (44 app + 21 core unit + 18 core integration).
- Pipeline coverage: allowed text stored, concealed/paused/blocked never read, empty/whitespace skipped, armed echo skipped once and then captured, same-millisecond recopy not `is_new`, retention applied, source app persisted, live config updates, probe failure surfaced.
- Coalescer coverage: 25-event burst → one emit; separate bursts advance revision; 10 000 `notify()` calls stay under 500 ms.
- `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny --config scripts/license/cargo-deny.toml check` → clean.

### Measurements

- Test suite (all crates): **83 tests in ~0.55 s**.
- Coalescer cost: 10 000 notifications < 500 ms (channel send only, no lock, no webview call).

### Notes / follow-ups

- The pipeline is not started in the app yet: platform watch loops arrive in **M1-05**, which also wires `tauri_history_events` and `AppState`'s config/suppressor handles.
- Paste orchestration (M1-08) must call `Suppressor::arm_text` **before** writing to the clipboard.
- Image/file probes exist in the type but the gate skips them (`NoText`) until M2-04/M2-05.

### Next up

- **M1-05** — platform text capture: Windows `WM_CLIPBOARDUPDATE`, macOS `changeCount` polling, Linux X11 XFixes; foreground app identity; concealed markers; runner wiring.

---

## 2026-11-12 — Session 6 (Milestone 1, M1-05)

**Scope:** M1-05 — platform text capture on Windows, macOS, and Linux X11; foreground app identity; concealed markers; runner wiring.

### Done

- **Shared adapter (`platform/common/src/system.rs`):** `clipboard-rs` (0.3.5, `default-features = false`) powers watch/read on all three platforms. Two-phase `probe` (formats via `available_formats` + `has`) and `read_text`, a `ConcealedProbe` trait for OS-specific markers, and `SystemWatcher` which starts the watcher thread and **confirms startup** before returning (so failures are reported, not silent).
- **Windows backend (`platform/windows`):** concealed detection for `ExcludeClipboardContentFromMonitorProcessing`, `ClipboardViewerIgnore`, `KeePassClipFormat`, `com.bitwarden.concealed`, plus DWORD-valued `CanIncludeInClipboardHistory`/`CanUploadToCloudClipboard` (present but `0` → concealed; unreadable → fail closed, 3×10 ms retries). Foreground app via `GetForegroundWindow` → PID → `QueryFullProcessImageNameW`, plus window title; friendly name from the exe stem.
- **Linux X11 backend (`platform/linux`):** `x-kde-passwordManagerHint` presence detection (conservative — the value is not exposed by `clipboard-rs`), `_NET_ACTIVE_WINDOW` → `_NET_WM_PID` → `/proc/<pid>/comm`, `WM_CLASS` fallback, `_NET_WM_NAME` title, all over a short-lived `x11rb` connection. `is_wayland_session()` gate.
- **macOS backend (`platform/macos`):** concealed/transient marker list (`org.nspasteboard.*`, 1Password, TypeIt4Me, etc.) and `NSWorkspace.frontmostApplication` (bundle id + localized name) via `objc2` 0.6. Window title intentionally omitted (needs Accessibility; that is paste-time, M1-08/M2-10).
- **Runner (`src-tauri/src/capture/runner.rs`):** dedicated `copyfie-capture` thread selects the platform backend, builds the pipeline, and blocks on `wait_for_change`. Stored items log id/new (no content); skips log at debug; platform errors log at warn and back off 250 ms. On Wayland it logs a capability warning and does not attempt X11 capture.
- **Logging:** `tauri-plugin-log` (Info) + `log`, so capture failures are visible in release builds without a console. No clipboard content is ever logged.
- **Frontend proof:** the scaffold listens for `history:changed` and refreshes store stats, showing live capture and event coalescing.

### Real end-to-end verification (Windows 11, this machine)

1. Launched the app, copied `"Copyfie end-to-end capture test 2026-11-12"` → UI showed **1 item · 1 text**, **1 history change**; log: `clipboard capture started` + captured line.
2. Copied a **concealed** item (Windows Forms `DataObject` with `ExcludeClipboardContentFromMonitorProcessing`) → **not stored** (still 1 item). Confirms the gate blocks before payload read.
3. Re-copied the original text → still **1 item**, **2 history changes**: dedupe bumped the row and the palette was notified.
4. Minimized Copyfie, copied from Windows Terminal → new row with `source_app_name = WindowsTerminal`, `source_app_id = C:\Program Files\WindowsApps\…\WindowsTerminal.exe`. Confirms foreground identity and background capture.
5. DB inspection (Python sqlite3): row 1 text/preview correct, expiry `captured_at + 30 days`, `is_pinned = 0`.

### Verification

- `cargo test --workspace` → **90 passed** (40 app + 21 core unit + 18 core integration + 4 common + 3 Windows + 4 Linux).
- `cargo check --workspace --all-targets` on Windows type-checks the **Linux X11 backend** too (`x11rb` is pure Rust); the macOS crate is cfg-gated and compiled by the macOS CI job.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo deny` initially failed on **BSL-1.0** (`clipboard-win`'s support crates). Added BSL-1.0 to the allowlist (permissive, OSI-approved, closed-source compatible); policy is now all green.

### Decisions

- **One shared `clipboard-rs` adapter, per-OS specifics only.** Watch/read behaviour is identical across platforms; only concealed markers and foreground identity differ. Avoids three copies of the watcher code.
- **File selections are skipped (`NoText`) until M2-05** rather than stored as path text, preserving the files → image → text payload priority.
- **Windows value markers fail closed** if unreadable: better to skip a copy than store a password.
- **KDE hint presence = concealed** on X11. Conservative, documented.
- **Fresh X11 connection per capture** (~1 ms) instead of shared mutable state; source app is read only after the gate, so this is not on the hot path.
- **Wayland does not start X11 capture.** The capability notice/copy-only UX is M2-12.
- **`tauri-plugin-log` added now** (planned in §9) so capture failures are diagnosable in release; logs never include content.

### Measurements

- Test suite: **90 tests**, ~0.6 s.
- E2E capture latency observed < 2 s including the test's fixed sleeps; real latency measurement is M1-11/M1-12.
- Data dir now holds 2 captured rows; no RSS measurement yet.

### Notes / follow-ups

- macOS backend is written but **not compiled on this machine**. The CI `macos-14` job is the gate; if `objc2-app-kit` API details differ, fix there before M1-13.
- Linux runtime behaviour (XFixes events, `/proc` lookup) is **not** exercised here; needs the Xvfb/Ubuntu CI job or a manual X11 run. Tracked for M1-13 acceptance.
- Windows source-app attribution records whatever window is foreground at clipboard-change time; if Copyfie itself is focused, it records `copyfie.exe`. Paste writes are suppressed (M1-08), so this is cosmetic.
- The runner keeps the `CaptureConfig`/`Suppressor` handles private until settings (M1-10) and paste (M1-08) need them.

### Next up

- **M1-06** — palette UI: Svelte shell, list, keyboard, filters, empty/error states.

---

## 2026-11-12 — Session 7 (Milestone 1, M1-06)

**Scope:** M1-06 — palette UI: Svelte shell, list, keyboard, filters, empty/error states.

### Done

- **Search backend (`copyfie-core/src/search`):** FTS5 query builder that quotes every token, doubles embedded quotes, drops punctuation-only tokens, caps input at 200 chars, and uses `"token"*` prefixes. Ranked query: `pinned DESC, bm25(8/3/1), captured_at DESC`, kind filter, 100-row cap. 14 tests cover prefix/substring, source-app matches, pins, kind filters, CJK/emoji, injection strings, limits.
- **History commands:** `history_list` (kind/pin filters, paging, 100 cap), `history_search`, `history_pin`, `history_delete`, `history_clear`, plus `copy_item` and `hide_palette`. Copy arms echo suppression **before** the clipboard write and clears it if the write fails.
- **Palette shell:** `SearchBar`, `FilterBar`, `ResultList`, `ResultRow`, `Footer`, and `App`; global tokens; strict CSP unchanged.
- **Instant + authoritative search:** each keystroke filters the loaded 100 summaries synchronously (`localSearch.ts`, ranked prefix → word-start → substring → subsequence, pins first) and a 50 ms debounce then asks FTS5. Empty FTS results keep local fuzzy matches. Stale responses are dropped by request id.
- **Keyboard (`keymap.ts`, pure reducer):** arrows wrap, PageUp/Down ±8 clamp, Home/End, Enter, Shift+Enter, bare `1–9` only when the query is empty, `Cmd/Ctrl+1–9` always, Esc, Tab/Shift+Tab filter cycling, `Cmd/Ctrl+P` pin, `Cmd/Ctrl+Backspace` delete (confirm for pinned), `Cmd/Ctrl+,` settings placeholder, `Cmd/Ctrl+B` block placeholder. IME composition ignores all shortcuts.
- **States/a11y:** combobox + listbox + option roles, `aria-activedescendant`, focus ring, reduced-motion support, loading skeleton, empty history, no-matches, error + retry, footer toast, live `history:changed` refresh.
- **Copy-only selection:** Enter copies the selected text item back to the clipboard with suppression; full focus-restore + synthetic paste lands in M1-08.

### Plan conflict resolved

PLAN §7 lists `Ctrl+P`/`Ctrl+N` as arrow alternatives and also `Cmd/Ctrl+P` as pin/unpin. On Windows both are the same chord. **Pin wins**; Ctrl+N/P navigation is dropped. Arrows, PageUp/Down, Home/End cover navigation. Tested explicitly.

### End-to-end verification (Windows 11, UI Automation)

- Palette renders the two captured items with source avatars, relative times, type labels, quick-paste numbers, and footer hints.
- Typing `Captured` in the search box (UIA ValuePattern) narrowed the list to **1 item**; Enter copied it: `Get-Clipboard` returned `Captured from another app 2026-11-12`, and the footer showed **“Copied to clipboard”**.
- `Ctrl+P` pinned the selected row (`is_pinned = 1` in SQLite); the **Pinned** filter showed exactly that row with the pin glyph.
- Copy did **not** create a duplicate or bump `captured_at`; `use_count` went to 1 — echo suppression works through the real OS clipboard path.
- Tab cycling updated the selected filter chip (UIA `SelectionItem.IsSelected`: Text → … → Pinned).

### Verification

- `npm run check` → 0 errors, 0 warnings; `npm test` → **22 passed** (keymap, local search, formatting).
- `npm run build` → **23.5 KB gzipped** JS+CSS (50 KB budget).
- `cargo test --workspace` → **104 passed** (40 app + 35 core unit + 18 core integration + 4 common + 3 Windows + 4 Linux).
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` → clean.

### Measurements

- Frontend: JS 21.6 KB gz + CSS 1.8 KB gz = **23.5 KB** (target <50 KB).
- Local filter on a 10k synthetic list: test asserts <100 ms (actual single-digit ms on this machine).
- UI render: two rows, no layout shift; palette window 680×480.

### Notes / follow-ups

- The palette is still the visible launch window; **M1-07** makes it hidden/pre-warmed and positions it under the cursor monitor.
- Enter copies but does not hide/paste; **M1-08** owns focus restore and synthetic input.
- Delete was verified at the store and keymap level but not end-to-end: UIA focus moved to another window before the synthetic keystroke, so it was not a reliable assertion. Re-check in M1-13 manual pass.
- Image/file rows render placeholders; their capture is M2-04/M2-05.

### Next up

- **M1-07** — pre-warm + show/focus + multi-monitor positioning.
