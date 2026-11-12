# Copyfie — Task Tracker

> Read this file and `progress.md` at the start of every session.
> Update both after every completed task, before committing.
> Status values: `todo` | `in-progress` | `blocked` | `done`
> Format: `- [ ] <ID> <task> (<effort>) — deps: <IDs> — status: <status>`

Legend:

| Prefix | Milestone |
|---|---|
| `M1` | MVP — text capture, hotkey palette, search, paste |
| `M2` | v1.0 — images, files, pins, licensing, trial, installers |
| `MX` | v1.x — deferred platform and packaging work (after v1.0) |
| `M3` | v1.1 — snippets, OCR, custom themes, icons |
| `M4` | v1.2 — optional encrypted folder sync |

Effort: **S** ≈ 1–3 days · **M** ≈ 1–1.5 weeks · **L** ≈ 2–3 weeks.

## M1 — MVP

- [x] M1-01 Repo/workspace, Tauri hello, CI skeleton, `cargo-deny`, NOTICE/THIRD_PARTY (S) — deps: none — status: done
- [x] M1-02 `copyfie-core`: models, BLAKE3, SQLite schema/migrations, dedupe, basic retention, unit tests (M) — deps: M1-01 — status: done
- [ ] M1-03 Store actor + custom URI protocol for thumbnails (stub) (M) — deps: M1-02 — status: todo
- [ ] M1-04 Capture pipeline: privacy gate, text capture, source app, suppression, coalesced `history:changed` (L) — deps: M1-02, M1-03 — status: todo
- [ ] M1-05 Platform text capture: Windows, macOS, Linux X11 (L) — deps: M1-04 — status: todo
- [ ] M1-06 Palette UI: Svelte shell, list, keyboard, filters, empty/error states (L) — deps: M1-02, M1-03 — status: todo
- [ ] M1-07 Pre-warm + show/focus + multi-monitor positioning (M) — deps: M1-06 — status: todo
- [ ] M1-08 Paste orchestrator: hide, restore focus, write, suppress echo, synthesize paste (L) — deps: M1-05, M1-07 — status: todo
- [ ] M1-09 Hotkey + first-run conflict detection + rebind + tray + single instance + autostart (M) — deps: M1-07 — status: todo
- [ ] M1-10 Minimal settings + theme (M) — deps: M1-06 — status: todo
- [ ] M1-11 Perf harness, 10k fixture, baseline report (M) — deps: M1-02 — status: todo
- [ ] M1-12 Measure real RSS per OS (core + total process tree, 10k items, 10 min idle), record in `docs/perf/` (M) — deps: M1-11 — status: todo
- [ ] M1-13 MVP acceptance pass + docs (M) — deps: M1-01..M1-12 — status: todo

## M2 — v1.0

- [ ] M2-00 Naming clearance: `copyfie.app` availability/ownership, trademark search, GitHub/winget/Homebrew/package handles; lock `com.copyfie.app` and copyright line (S) — deps: none — status: todo
- [ ] M2-01 Release gate: proprietary LICENSE, EULA, terms, privacy policy, refund policy (S) — deps: M2-00 — status: todo
- [ ] M2-02 Marketing site: landing, pricing, product description, `/buy`, `/welcome`, `/restore`, legal pages (M) — deps: M2-00 — status: todo
- [ ] M2-03 Paddle seller onboarding as Indian business/individual; submit site + pricing + policies + description (M) — deps: M2-01, M2-02 — status: todo
- [ ] M2-04 Image capture: hash, downscale, PNG, thumbnail, atomic files (M) — deps: M1-04 — status: todo
- [ ] M2-05 File-path capture: Windows CF_HDROP, macOS file URLs, Linux URI list (M) — deps: M2-04 — status: todo
- [ ] M2-06 Pin/unpin + pin-aware expiry + retention setting + image disk cap + blob GC (M) — deps: M1-02, M2-04 — status: todo
- [ ] M2-07 Fuzzy search + type filters + source-app name/initial display (M) — deps: M1-06 — status: todo
- [ ] M2-08 Blocklist capture gate + settings UI + “block current app” (M) — deps: M1-04 — status: todo
- [ ] M2-09 Full settings: hotkey recorder, history length, image cap, launch at login, theme, pause capture (M) — deps: M1-10 — status: todo
- [ ] M2-10 macOS Accessibility onboarding + copy-only fallback (M) — deps: M1-08 — status: todo
- [ ] M2-11 Windows elevated-target detection + copy-only fallback (S) — deps: M1-08 — status: todo
- [ ] M2-12 Wayland capability notice + copy-only fallback (no full Wayland support) (S) — deps: M1-05 — status: todo
- [ ] M2-13 Trial module (14 days): signed record, redundant anchors, clock-rollback detection, entitlement state, day 10/13/14 reminders (M) — deps: M1-02 — status: todo
- [ ] M2-14 License module: Ed25519 verification, key format, persistent key, 3-device activation, grace period (M) — deps: M2-13 — status: todo
- [ ] M2-15 Limited-mode UI: 25 text items, real “N more items waiting” count, locked settings except Clear history, footer line (M) — deps: M2-13, M2-14 — status: todo
- [ ] M2-16 Paddle checkout + customer journey: hosted checkout, `/welcome`, `copyfie://activate`, emailed key, manual paste (M) — deps: M2-02, M2-03, M2-14 — status: todo
- [ ] M2-17 License backend: webhook signature/replay/idempotency, Ed25519 issuance, email, restore, revocation, reconciliation (L) — deps: M2-16 — status: todo
- [ ] M2-18 Download page + published checksums: `SHA256SUMS`, `.sig`, key fingerprint, manual steps, uninstall commands (M) — deps: M2-02, M2-21 — status: todo
- [ ] M2-19 Payouts (India): wire vs Payoneer, schedule, $100 minimum, fee/FX model, FIRC/accountant (S) — deps: M2-03 — status: todo
- [ ] M2-20 Pricing validation: competitor scan, confirm launch and list price (S) — deps: M2-02 — status: todo
- [ ] M2-21 Packaging: NSIS, DMG, deb/rpm, `.desktop`, icons, EULA/notices in installer (L) — deps: M1-13 — status: todo
- [ ] M2-22 One-line installers: `install.ps1`, `install.sh`, `uninstall.sh`, signature checks, per-user install, autostart, printed uninstall (L) — deps: M2-21 — status: todo
- [ ] M2-23 Uninstall flow: `--uninstall` / `--purge`, remove app/autostart/desktop/icons, history kept by default (M) — deps: M2-21 — status: todo
- [ ] M2-24 Install channels: winget manifest, `copyfie/tap` Homebrew cask (M) — deps: M2-18, M2-21 — status: todo
- [ ] M2-25 Install CI: clean Windows/macOS/Ubuntu/Fedora runners, tampered artifact, no-admin, shellcheck + PSScriptAnalyzer (M) — deps: M2-22 — status: todo
- [ ] M2-26 Manual “Check for updates” link + documented update path (S) — deps: M2-21 — status: todo
- [ ] M2-27 Perf/soak validation, size gate, privacy docs (M) — deps: M1-12 — status: todo
- [ ] M2-28 Pre-launch verification: live test purchase/refund, webhook replay, restore, payout test, accountant GST/export review (M) — deps: M2-17, M2-19, M2-25 — status: todo
- [ ] M2-29 Attribution/THIRD_PARTY audit (S) — deps: none — status: todo

## MX — v1.x deferred (after v1.0)

- [ ] MX-01 Auto-updater: signed `latest.json`, update banner, failed-update recovery (M) — deps: M2-21 — status: todo
- [ ] MX-02 Full Wayland support: data-control, GlobalShortcuts portal, Hyprland binding, activation token, virtual-keyboard paste (L) — deps: M2-12 — status: todo
- [ ] MX-03 MSI installer (S) — deps: M2-21 — status: todo
- [ ] MX-04 AppImage (S) — deps: M2-21 — status: todo
- [ ] MX-05 Flatpak (M) — deps: M2-21 — status: todo
- [ ] MX-06 Universal macOS DMG (S) — deps: M2-21 — status: todo
- [ ] MX-07 AUR `copyfie-bin`, signed apt/rpm repository, Homebrew core cask submission (M) — deps: M2-24 — status: todo

## M3 — v1.1

- [ ] M3-01 Snippets CRUD, variables, search integration, pin-to-top (M) — deps: M1-02 — status: todo
- [ ] M3-02 OCR: Windows OCR API, macOS Vision, optional Linux Tesseract; OCR text index (L) — deps: M2-04 — status: todo
- [ ] M3-03 Custom themes: JSON token packs, preview, contrast validation, import/export (S) — deps: M1-10 — status: todo
- [ ] M3-04 Source-app icons (replaces colored initial) (S) — deps: M2-07 — status: todo
- [ ] M3-05 Snippet/theme backup and restore (S) — deps: M3-01, M3-03 — status: todo

## M4 — v1.2

- [ ] M4-01 Device pairing (QR/manual sync key), X25519 identity, per-device keys (M) — deps: M2-14 — status: todo
- [ ] M4-02 Content encryption (XChaCha20-Poly1305), metadata envelope (M) — deps: M4-01 — status: todo
- [ ] M4-03 Folder transport: per-device append-only oplog + manifest in the chosen folder (L) — deps: M4-02 — status: todo
- [ ] M4-04 Conflict resolution: HLC + LWW, tombstones, pinned precedence; conflict copies never silently overwrite (L) — deps: M4-03 — status: todo
- [ ] M4-05 Image blob sync + thumbnail regeneration (M) — deps: M4-03 — status: todo
- [ ] M4-06 Sync status UI, pause, conflict log, key rotation/revocation, “folder not available” handling (M) — deps: M4-04 — status: todo
- [ ] M4-07 Security review + threat model (provider access, rollback, conflict oracles) (M) — deps: M4-04, M4-05, M4-06 — status: todo

---

## Rules

- Work one task at a time, in ID order within the active milestone.
- Verify the task works before marking it `done`.
- After each task: update this file and `progress.md`, commit with the task ID, push to `origin/main`.
- Keep the build green: format, lint, tests before each commit.
- Record measurements (RSS, installer size, palette-open p95) in `progress.md` when they exist. If total RSS exceeds **120 MB**, stop and report.
- If blocked, mark `blocked`, log it, and ask.
- Stay inside the current milestone. Deferred work stays deferred.
- Never commit secrets.
