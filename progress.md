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

- **M1-04** — capture pipeline: privacy gate, text capture, source app, suppression, coalesced `history:changed`.
