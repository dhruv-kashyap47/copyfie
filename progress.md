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

---

## 2026-11-12 — Session 8 (Milestone 1, M1-07)

**Scope:** M1-07 — pre-warm, show/focus, cursor-monitor positioning.

### Done

- **Hidden pre-warm:** the palette window is created `visible: false`, `focus: false` with `shadow: true`, skip-taskbar, always-on-top, opaque theme background (no white flash), plus Windows browser args that prevent renderer throttling while hidden.
- **`show_palette`:** positions on the monitor under the cursor **before** showing, then shows, focuses, emits `palette:open` with a monotonic `open_id`, and logs the open.
- **Positioning:** uses the cursor monitor's `work_area()` and physical pixels: horizontally centered, 18% down the work area. No manual DPI math — Tauri returns physical rects, so mixed-DPI monitors are handled by construction. Falls back to current/primary monitor.
- **Windows focus:** `set_focus()` alone is blocked by the foreground lock when the window is shown programmatically. Added the plan's deterministic path: attach to the foreground thread's input queue → `BringWindowToTop` + `SetForegroundWindow` → detach. Non-Windows keeps `set_focus()`.
- **`palette:open` handling:** the frontend clears query/filter/selection/toast, refreshes from the store, and refocuses the search input on every open. Window `focus` events also refocus the input.
- **`--show` escape hatch:** shows the palette at startup for development and manual testing (also the planned manual show path for Wayland). Documented in the README. Without it, the app runs hidden/pre-warmed by design.

### Verification

- Launch **without** `--show`: process runs, `MainWindowTitle` empty, screenshot contains no palette, log shows `clipboard capture started`.
- Launch with `--show`: palette appears centered on the primary monitor at ~18% height; `GetForegroundWindow() == palette hwnd` (**focused=True**).
- Typed `Copyfie` with real synthetic keystrokes: the query value updated and the list narrowed to the 2 matching rows — keyboard focus lands in the search input on show.
- Hidden pre-warm: the list was already populated the moment the window appeared (no DB wait in the open path).
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm run check`, `npm test` → clean.

### Decisions

- **`--show` is an app argument, not a debug hack.** It doubles as the documented manual-show fallback for Wayland and for users whose hotkey is unavailable.
- **Position before `show()`** and only once per open; no move-visible-window flicker.
- **Force foreground only on user-triggered show**, with input-queue attachment scoped to the call and always detached.
- **No `palette:shown` timing event yet**; open-time metrics land with the M1-11 harness.

### Measurements

- Hidden window: CPU idle during capture start (no polling beyond the OS watcher); no RSS measurement yet (M1-12).
- Open path: list came from the already-rendered webview; no IPC wait observed.

### Notes / follow-ups

- Multi-monitor and mixed-DPI behaviour cannot be exercised on this single-monitor machine; add to the M1-13 manual matrix.
- `palette:open` reset is wired but will get its real workout once M1-09 adds the hotkey.
- The window still lacks DWM rounded corners on Windows (deferred to M1-09/tray polish or M2-21 packaging).

### Next up

- **M1-08** — paste orchestrator: hide, restore focus, write with suppression, synthesize paste.

---

## 2026-11-12 — Session 9 (Milestone 1, M1-08)

**Scope:** M1-08 — hide, restore focus, write with suppression, synthesize paste. This session **stops after M1-08**; M1-09 continues next time.

### Done

- **Platform contracts (`copyfie-platform`):** `FocusTarget` (opaque Windows HWND/pid, macOS pid, X11 window), `FocusBackend { capture, restore, is_active }`, and `PasteInput { send_paste }`. `input` module wraps enigo 0.6: **Cmd+V on macOS** using `Key::Other(9)` (the documented keycode fix) and **Ctrl+V elsewhere**. Accessibility prompt is disabled for now, so a denied permission falls back to copy-only instead of an unexpected prompt (M2-10 owns onboarding).
- **Windows focus:** capture via `GetForegroundWindow`; restore restores minimized windows, attaches to the foreground thread's input queue, brings the window to top, calls `SetForegroundWindow`, then detaches; `is_active` compares handles.
- **Linux X11 focus:** `_NET_ACTIVE_WINDOW` capture/compare and an EWMH `_NET_ACTIVE_WINDOW` client message for restore, over `x11rb`.
- **macOS focus:** `NSWorkspace.frontmostApplication` pid, `NSRunningApplication::runningApplicationWithProcessIdentifier` + `activateWithOptions(ActivateAllWindows | ActivateIgnoringOtherApps)`, frontmost-pid check.
- **Orchestrator (`src-tauri/src/paste`):** `PasteService` remembers the target in `show_palette` only while the palette is hidden, so showing it twice never records itself as the target. On paste: hide palette **and wait until the OS reports it hidden**, restore target, poll `is_active` (700 ms, 10 ms steps), 15 ms input-queue settle, arm suppression, write clipboard, 25 ms clipboard settle, send chord, `touch_used`.
- **Copy-only fallback:** missing target, restore timeout, clipboard failure, or input failure leaves the text on the clipboard, re-shows the palette, logs the reason, and returns `{ status: "copied", reason }`. The UI shows “Copied — …” instead of pretending to paste.
- **Command/UI:** `paste_item(id)` returns `{ status, reason }`; Enter now pastes, footer says “Enter paste”; `copy_item` remains for explicit copy-only actions.

### Verification

- **Input layer isolated:** a windowless probe (`paste_probe` example, since removed) sent the chord to a focused WinForms textbox; the log captured `KeyDown Control`, `KeyDown V`, and the textbox received the clipboard text. Confirms enigo/SendInput and the clipboard write.
- **Full app paste:** launched a WinForms target textbox in a separate process, focused it, started Copyfie with `--show`, pressed Enter in the palette.
  - Target textbox received exactly `Captured from another app 2026-11-12`.
  - Log: `restoring paste target` → `target active after 0 ms` → `paste chord sent` → `pasted item id=2`.
  - Clipboard contained the item; the palette hid.
  - SQLite: row count unchanged, `captured_at` unchanged, `use_count` incremented → **no echo duplicate or bump**.
- **Race found and fixed:** the first attempts pasted nowhere because `window.hide()` returns before the OS hides the window, so focus restore raced the hide. Waiting for `is_visible() == false` (400 ms cap) plus the settle delays made it deterministic.
- **Harness lesson:** debug builds allocate a console window; launching a console probe stole focus and invalidated one test. The diagnostic binary is now `windows_subsystem = "windows"`. Release builds have no console.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (14 suites), `npm run check`, `npm test` (22), `npm run build` (23.6 KB gz), `cargo deny` → all clean.

### Decisions

- **One enigo-based input path** for all three OSes, with the macOS keycode exception; avoids three SendInput/CGEvent/XTEST implementations.
- **Suppression is armed immediately before the clipboard write**, after focus is confirmed. A failed write clears it.
- **No blind paste:** if `is_active` is not confirmed, nothing is sent; the item goes to the clipboard and the palette returns.
- **Shift+Enter is currently identical to Enter.** History is plain text only in v1, so there is no rich payload to strip; the images/files variants land with M2-04/M2-05.
- **Elevated-target detection is deferred to M2-11.** On Windows, UIPI can block the chord into an elevated window; the current behavior is a silent no-op in that target (clipboard still gets the item). Tracked.
- **`paste_probe` removed** after verification; it was a diagnostic, not product code.

### Measurements

- Paste path observed: focus active at 0 ms in the test, clipboard settle 25 ms, chord sent immediately after → roughly **40–80 ms** on Windows for an already-active target, plus the ~10–30 ms hide wait. Proper p50/p95 instrumentation is M1-11.
- No RSS measurement yet (M1-12).

### Notes / follow-ups

- macOS/Linux focus restore is **type-checked but not runtime-tested** here; the macOS CI job compiles it and the M1-13 manual matrix must cover both.
- `copy_item` is still exposed for copy-only actions; the Wayland fallback will use it.
- Paste log lines contain ids/reasons only — never content.
- Stop here per instruction. Next session starts at **M1-09**.

### Next up

- **M1-09** — hotkey + first-run conflict detection + rebind + tray + single instance + autostart.

---

## 2026-11-12 — Session 10 (Milestone 1, M1-09)

**Scope:** M1-09 — global hotkey, first-run conflict detection, rebind, tray, single instance, autostart.

### Done

- **Settings store (`settings.rs`):** `settings.json` in the app data root with `version` + `hotkey`, unknown fields preserved, atomic temp-file → fsync → rename writes, `.bak` of the previous version, and a broken file is never overwritten (defaults are used in memory). Five unit tests.
- **Hotkey manager (`hotkey.rs`):** parses `CommandOrControl` accelerators through the plugin, tracks the configured chord, and rebinds **transactionally** — the new chord must register before the old one is released. An `initialized` flag distinguishes “registration has not run yet” from “registration failed”, so the pre-warmed UI never flashes the conflict overlay.
- **Global shortcut plugin:** handler shows the palette on key press; the saved shortcut is registered during setup. Success and conflict are both logged (ids only, no content).
- **Conflict UI (`HotkeySetup.svelte` + `lib/hotkey.ts`):** blocking storage-agnostic overlay showing the conflicting chord, a chord recorder (modifier required; modifier-only presses ignored), validation from the backend, and a save action. Escape is intentionally ignored while blocked. Seven frontend tests cover building and formatting accelerators.
- **Tray (`tray.rs`):** **Show palette**, **Pause capture** (check item, shared `CaptureConfig`), **Start at login** (check item reflecting the autostart plugin), separator, **Quit**. Left-clicking the icon shows the palette.
- **Single instance:** plugin registered first; a second launch exits and reveals/focuses the palette in the running process.
- **Autostart:** plugin initialized with `MacosLauncher::LaunchAgent` and `--hidden`; **off by default**, toggled from the tray.
- **State wiring:** `AppState` now owns the capture config, hotkey manager, and settings store; the capture runner and tray share one config instance.

### Verification

- **Default hotkey:** launched hidden (no `--show`), pressed `Ctrl+Shift+Space` → log `global shortcut registered` + `palette shown (open_id=1)`, palette visible on the cursor monitor and **focused**.
- **Conflict:** a helper process held `Ctrl+Shift+Space` via `RegisterHotKey`; Copyfie logged the conflict and **automatically opened the palette** with the blocking overlay (“Shortcut unavailable”, conflicting chord, recorder, Save).
- **Rebind:** recorded `Ctrl+Shift+F9` in the overlay (recorder value verified via UIA), clicked **Use this shortcut** → log `shortcut registered`, overlay gone, `settings.json` contains `"hotkey": "CommandOrControl+Shift+F9"`. The new chord opened/focused the palette; the old one was still held by the helper.
- **Persistence + default restore:** reset `settings.json` to the default, restarted hidden → `global shortcut registered` and the default chord opened/focused the palette.
- **Single instance:** second process exited with code 0, process count stayed 1, and the running palette was shown/focused.
- **Autostart:** no `HKCU\...\Run` entry by default (correct opt-in state).
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (14 suites), `npm run check`, `npm test` (**29**), `npm run build` (**25.3 KB gz**), `cargo deny` → all clean.

### Decisions

- **Persist only the shortcut for now.** M1-10 extends the same `Settings` struct; unknown fields are preserved so nothing written later is lost.
- **Blocking rebind, no silent alternative.** If the default is taken, the user must choose; Escape does not dismiss the setup.
- **A modifier is required** in the recorder (Ctrl/⌘ or Alt), so plain typing can never become a global shortcut.
- **Autostart is opt-in** (plan example shows `true`, but an app that self-registers before the user asks is hostile). The tray check item reflects reality.
- **Pause is intentionally in-memory** until M1-10/M2-09 adds `capture_paused` to settings.
- **Debug-build console windows** make `Process.MainWindowHandle` unreliable in automation; focus was verified against the “Copyfie” Tauri window found through UI Automation. Release builds have no console.

### Measurements

- Frontend bundle: **25.3 KB gzipped** (JS 24.0 + CSS 2.2; 50 KB budget).
- Hotkey → palette shown and focused in under ~2 s including the test’s fixed sleep; no dedicated open-time metrics yet (M1-11).
- No RSS measurement yet (M1-12).

### Notes / follow-ups

- Tray menu clicks and the autostart toggle were **not automated**; add them to the M1-13 manual matrix.
- Stale autostart path repair (risk #30) is deferred; revisit with M2-09/M2-21.
- `SendKeys` does not accept `{SPACE}`; automation must send a literal space (`"^+ "`).
- The conflict helper and all test logs were removed; `settings.json` is back to the default shortcut.

### Next up

- **M1-10** — minimal settings + theme: full settings window/tab, hotkey recorder in settings, history length, launch at login, pause, theme tokens, About.

---

## 2026-11-12 — Session 11 (Milestone 1, M1-10)

**Scope:** M1-10 — minimal settings window and theme.

### Done

- **Settings model:** `Settings` now carries `theme` (`system|light|dark`), `accent`, `retention_days` (`0` = Forever), and `capture_paused`, alongside the hotkey. `sanitized()` validates theme, `#RRGGBB` accent, and retention (0 or 1–365); unknown fields are still preserved. Six unit tests.
- **Commands:** `settings_get`, `settings_save` (transactional hotkey registration first, then persist, autostart, capture config, window themes, and a `settings:changed` event), plus the existing hotkey commands.
- **Lazy settings window:** created on demand from `open_settings` (async command), destroyed on close, recreated later. 560×640, native decorations, theme-matched titlebar. Palette hides only after the window exists.
- **Multi-page frontend:** `settings.html` + `src/settings/` entry, Vite `rollupOptions.input`, a separate `settings.css` (scrollable/selectable, unlike the palette).
- **Settings UI:** Appearance (theme segmented control, accent picker), History (retention, pause), Shortcut (recorder + apply), Startup (launch at login), About (version/privacy/attribution). Every change saves immediately; failures revert to persisted values via `settings_get`.
- **Theme:** `theme.ts` applies cached theme/accent before mount (no flash), Rust reconciles, and `settings:changed` updates the palette live. `system` removes `data-theme` so `prefers-color-scheme` applies.
- **Integration:** `Cmd/Ctrl+,` opens settings; the tray has a **Settings…** item; startup applies persisted retention/pause to the capture config.

### Two real bugs found and fixed

1. **WebView2 `HRESULT 0x8007139F` (ERROR_INVALID_STATE) on the second webview.** WebView2 shares one environment per data directory, and its **browser arguments must match** across webviews. The palette sets throttling flags in `tauri.conf.json`; the runtime-created settings window had none. Applying the same `additionalBrowserArgs` to the settings builder fixed creation.
2. **Synchronous `open_settings` deadlocked** at `WebviewWindowBuilder::build()` (the command blocks the thread that must service window creation). Made the command `async`; the tray path spawns a worker thread.

### Verification

- `Ctrl+,` opens the settings window and focuses it; UIA exposes Appearance/History/Shortcut/Startup/About.
- **Theme:** clicked **Light** → `settings.json` `"theme": "light"`, both windows repainted light (screenshots), persisted across a full restart (palette light before any interaction).
- **Retention:** set 7 days → `settings.json` `"retention_days": 7`; a new capture stored after the change has `expires_at − captured_at = 7.0` days. Older rows keep 30 days; the pinned row is `never`.
- **Pause:** enabling it saved `capture_paused: true`; copying a unique string while paused left the row count at 3. Disabling and copying added the row (4).
- **Lifetime:** closing the settings window destroyed it; `Ctrl+,` recreated it successfully.
- **Reset:** returned to defaults (system theme, 30 days, unpaused, default shortcut) after testing.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (14 suites), `npm run check`, `npm test` (**33**), `npm run build`, `cargo deny` → all clean. Bundle: **31.3 KB gz** (two pages, 50 KB budget).

### Decisions

- **Autostart truth is the OS plugin**, not `settings.json`; the view reports `is_enabled()` and `settings_save` toggles it. Avoids drift.
- **Retention affects new captures only**; existing rows keep the expiry computed when captured. Pinned rows stay `NULL`.
- **Theme cache is local (localStorage) for paint, Rust is authoritative.** The cache is updated on every save and on every `settings:changed`.
- **Settings changes are immediate** (no Save button); failed saves revert the form.
- **Accent is validated server-side** to `#RRGGBB`; invalid values fall back to the default.
- **Tray check items are not yet synchronized** when pause/autostart change in Settings; they update when toggled from the tray. M2-09 should unify this.

### Measurements

- Frontend: palette 6.2 KB gz, settings 2.7 KB gz, shared chunk 19.7 KB gz, CSS included → **31.3 KB total** vs 50 KB budget.
- No RSS measurement yet (M1-12).

### Notes / follow-ups

- The settings hotkey recorder is wired to the same save path as the overlay; the overlay rebind was verified end-to-end, the settings recorder only by code/unit tests.
- `prefers-color-scheme` live OS switching was not exercised; static system/light/dark were.
- Settings window is fixed-size; revisit if translations or accessibility scaling need more room.
- Test history now includes four rows (one pinned, one with 7-day retention); leave or clear via Settings when desired.

### Next up

- **M1-11** — perf harness, 10k fixture, baseline report.

---

## 2026-11-12 — Session 12 (Milestone 1, M1-11)

**Scope:** M1-11 — perf harness, 10k fixture, baseline report.

### Done

- **Harness (`crates/copyfie-core/examples/perf.rs`):** deterministic 10k text fixture and measurements for insert, dedupe/recopy, FTS (common/rare/no-match/kind-filtered), recent listing, retention cleanup, DB size, and p50/p95/max/total. Prints JSON and writes `--out`.
- **`bench` profile:** opt-level 3 with LTO off, so measurements are optimized but iteration stays fast.
- **Scripts:** `scripts/perf/seed_10k.ps1` (seeds the app DB; refuses while Copyfie runs) and `scripts/perf/rss.ps1` (process-tree RSS sampler for M1-12).
- **Local filter benchmark:** the 10k vitest test now runs 60 queries and reports p50/p95 (`5.36 / 7.70 ms`) instead of a single assertion.
- **Baseline:** `docs/perf/0.1.0-windows-x86_64.json` and the write-up in `docs/PERF.md`.

### Baseline (10k items, Windows 11, Ryzen 5 5600H)

| Operation | p50 | p95 |
|---|---:|---:|
| Insert | 0.46 ms | 0.90 ms |
| Dedupe (recopy) | 0.38 ms | 1.74 ms |
| FTS common (100 of 10k) | 41.0 ms | 46.8 ms |
| FTS rare | 0.29 ms | 0.49 ms |
| FTS no-match | 0.18 ms | 0.21 ms |
| List recent 100 | 0.29 ms | 0.33 ms |
| Local filter 10k | 5.36 ms | 7.70 ms |

DB: 8.45 MB for 10,004 items (~840 B/item). Retention cleanup: 4.8 ms.

### Observations / follow-ups

- FTS ranked search is **close to the 50 ms p95 gate** when a query matches all 10k rows. The instant local layer (5.4 ms p50) is what keeps typing responsive; FTS remains the authoritative pass. If this regresses, consider a candidate cap before `bm25` or a recency pre-filter.
- Dedupe total time is inflated because every recopy updates `source_app_name`, firing the FTS update trigger. Optimization: skip source updates when unchanged. Not a correctness issue.
- One dedupe outlier (555 ms) from a WAL checkpoint; p50/p95 stable.
- Measurements use the `bench` profile; the release binary may differ slightly. RSS uses the release build (M1-12).

### Next up

- **M1-12** — measure real RSS per OS (core + process tree, 10k items, 10 min idle) and record in `docs/perf/`.

---

## 2026-11-12 — Session 13 (Milestone 1, M1-12 — RAM gate blocked)

**Scope:** M1-12 RSS measurement, captured on the Windows release build. **Stopped at the RAM gate per the milestone rules.**

### Done

- **Capture latency:** added `scripts/perf/capture_latency.py` (Win32 `CF_UNICODETEXT`, 5 ms SQLite polling). Release build, 10,024 items, 20 iterations: **20.1 ms p50 / 28.0 ms p95 / 28.0 ms max** — well inside the 500 ms budget.
- **RSS harness:** `scripts/perf/rss.ps1` samples the Copyfie process tree (core + WebView2 children) every 5 s. Release build, 10,004 items, palette hidden/pre-warmed, **10 minutes idle, 116 samples**.
- **Release build:** `target/release/copyfie.exe` 4,865,536 bytes, `tauri build --no-bundle` succeeded.
- **Recorded:** `docs/perf/rss-0.1.0-windows-x86_64.json`, `docs/perf/capture-latency-0.1.0-windows-x86_64.json`, `docs/PERF.md`, `docs/ACCEPTANCE.md`.
- **M1-10 follow-up:** Clear history in Settings (unpinned by default, optional pinned) and a `hotkey_to_show_us` log line.

### RSS result (gate)

| Component | Working set |
|---|---:|
| `copyfie.exe` core | **27.3 MB** (24.3 in the control run) |
| WebView2 browser | 116-121 MB |
| GPU process | 70-79 MB |
| Renderer | 71-77 MB |
| Network service | 37-40 MB |
| Storage service | 19-21 MB |
| Crashpad | 12.7 MB |
| **Total** | **360-370 MB** (min 360.0, avg 362.1, max 370.1) |
| Total private | ~168-180 MB |

- **Core passes** the <40 MB target.
- **Total fails** the 60-100 MB expectation and the hard 120 MB gate.
- **Fresh-profile control:** renamed the WebView2 user-data folder and re-ran 60 s → 354-363 MB. Not profile bloat.
- Working-set sums double-count shared WebView2 DLL pages, so the private total (~170 MB) is the fairer number - still above 120 MB.

### Decision required (per PLAN rules, not silently changed)

1. **Raise the total budget** (e.g., core <40 MB, total <400 MB) and keep pre-warm. Preserves the <100 ms open path; accepts a heavy idle footprint.
2. **Drop pre-warm** (create on hotkey, destroy on close): idle ≈ core (~30 MB), but cold open 250-700 ms.
3. **Hybrid:** pre-warm while active, destroy after N idle minutes. Idle meets the gate; first open after idle is cold.
4. **Native palette UI** (GPUI/egui) with Tauri only for settings: best memory, large rewrite.
5. **WebView2 tuning** (`--disable-gpu`, renderer limits): test before deciding; unlikely alone to reach 120 MB.

The 60-100 MB plan estimate was informed by native/GPUI apps (Ropy ~57 MB); a single hidden WebView2 window on this machine (Edge WebView2 v157) baselines near 330 MB working set regardless of our app's cache or profile.

### Notes

- The release binary predates the Clear history UI and `hotkey_to_show_us` line (both were added after the build started); rebuild after the gate decision.
- `docs/ACCEPTANCE.md` records every other M1 criterion with evidence and the remaining cross-platform gaps.
- M1-13 (acceptance pass) is **blocked** until the RAM decision is recorded.

### Next up

- **Await the RAM gate decision.** Then either implement the chosen option and re-measure, or record the raised budget and finish M1-13.

---

## 2026-11-12 — Session 14 (Milestone 1, hybrid RAM decision + M1-13)

**Scope:** Implement the user's decision: hybrid palette lifecycle, WebView2 tuning, cold-path optimization, per-OS gates, re-measurement, and the M1-13 acceptance pass.

### Decision implemented

**Hybrid (Auto, default):** the palette is pre-warmed while the user is active; after **10 idle minutes** the webview is destroyed and only the core process stays resident. The next hotkey creates it cold, then it is warm again. Settings expose **Keep palette ready: Auto / Always / Never** and **Forget after: 5 / 10 / 30 minutes**.

**Native palette (option 4) deferred** and recorded as a possible later rewrite if the warm WebView2 footprint becomes a competitive problem. It is not in M1 or v1.0 scope.

### Implementation

- **`PaletteManager`** (`src-tauri/src/palette.rs`): one place owns warm/cold show, pre-warm, idle destruction, settings application, and paint metrics. The palette window is now built at runtime (removed from `tauri.conf.json`) so it can be destroyed and recreated with identical options.
- **No-exit core:** destroying the last window triggered Tauri's default exit; the run loop now handles `RunEvent::ExitRequested` and prevents exit unless a quit code is set (tray **Quit**). This was the critical hybrid bug; capture and the hotkey survive with no windows.
- **Cold path:** a worker thread builds the hidden window with an initialization script containing the top 50 summaries, theme, accent, and `openId`. The page paints from the bootstrap without waiting for IPC, then reconciles. Warm/cold paints are reported via `palette_painted(openId)` and logged as `palette_open_ms=<n> cold=<bool>`.
- **Settings:** `palette_ready` and `palette_idle_minutes` in `settings.json`; sanitized; applied without restart (settings save pre-warms or destroys as needed). UI section added to the settings window. Also added **Clear history** (pinned kept by default, optional including pinned).
- **Idle monitor:** 10 s tick; Auto destroys after the timeout, Always never, Never destroys on hide. The core keeps capturing throughout.
- **Test hook:** a second launch with `--palette-destroy` destroys the hidden webview in the running instance (single-instance callback) for cold-open measurement.
- **Harness:** `scripts/perf/open_latency.py` drives warm/cold opens, handles log rotation, and reports p50/p95. `rss.ps1` samples working set; private bytes recorded separately.

### WebView2 tuning (time-boxed, kept)

| Config | Warm total | Cold p95 | Verdict |
|---|---:|---:|---|
| Baseline throttling flags | 365.6-375.0 MB | 520 ms | rejected |
| + `--disable-gpu --disable-breakpad` | 346.0-353.9 MB | 384-409 ms | better, cold marginal |
| + startup flags (no background networking/component update/default apps/sync/phishing detection/first run, metrics only) | **337.6-345.5 MB** | **390 ms** | **kept** |

Visuals verified in light and dark (palette rows/avatars/gradients, settings controls, native window shadow). All webviews share the args; WebView2 requires matching environment options.

### Final gates (Windows, release, 10,004 items)

| Gate | Target | Measured |
|---|---|---|
| Core working set | < 40 MB | **30.1 MB** |
| Total with webview destroyed | < 60 MB | **30.1 MB** (7.4 MB private) |
| Total with webview warm | budget from measurement | **342.8 MB avg / 345.5 MB max** (106.7 MB private) → budget **< 400 MB** |
| Cold open p95 | < 400 ms | **390 ms** (20 runs; p50 371) |
| Warm open p95 | < 100 ms | **37 ms** (10 runs; p50 31) |

Working-set sums double-count shared WebView2 pages; private bytes are reported too. After opening Settings, the warm total was 368.3/368.8 MB because the settings renderer lingers in the shared WebView2 process; the 400 MB budget covers it.

### M1-13 acceptance

- `docs/ACCEPTANCE.md` updated: all Windows criteria pass; macOS/Linux runtime remain platform measurements.
- New tasks **M1-14 (macOS)** and **M1-15 (Linux X11)** record RAM/open gates separately; Windows numbers do not apply to them.
- `PLAN.md`, `task.md`, `docs/PERF.md`, `docs/ARCHITECTURE.md`, `docs/PLATFORMS.md`, `docs/PRIVACY.md`, `docs/TESTING.md`, and `README.md` updated with the hybrid lifecycle, gates, bootstrap, and test hooks.
- Checks: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (14 suites), `npm run check`, `npm test` (33), `npm run build`, `npm run size:check` (**32.3 KB gz**), `cargo deny` → all clean.
- Release build succeeds (`tauri build --no-bundle`); no installers in M1 (M2-21).

### Notes / follow-ups

- Automation lesson: synthetic keys are dropped while an **elevated** window is foreground (UIPI). This initially made the hotkey look broken; activate a normal window before input tests.
- The log plugin rotates during long runs; the open-latency script now reads all rotated files.
- The settings renderer can linger after close; that is reflected in the warm budget.
- Native palette rewrite is a possible future optimization only.
- **Stop here.** Do not start v1.0 tasks until explicitly instructed. Remaining M1 measurement tasks (M1-14/M1-15) require macOS/Linux.

### Measurements

- Warm 342.8 MB avg / 345.5 MB max (106.7 MB private); destroyed 30.1 MB (7.4 MB private); core 30.1 MB.
- Cold open 371/390 ms p50/p95; warm 31/37 ms.
- Capture 20.1/28.0 ms; local filter 5.36/7.70 ms; FTS 41.0/46.8 ms; insert 0.46/0.90 ms.
- Frontend bundle 32.3 KB gz (two pages, 50 KB budget).
