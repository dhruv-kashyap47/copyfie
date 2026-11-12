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
