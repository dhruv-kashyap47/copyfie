# Copyfie — Milestone 1 Acceptance

Status as of 2026-11-12. Criteria are from the plan's MVP acceptance list.

| Criterion | Status | Evidence / gap |
|---|---|---|
| Text copied in Notepad/TextEdit/gedit appears within 500 ms p95 | **Windows pass** | **20.1 ms p50 / 28.0 ms p95** end-to-end (release build, 10k DB) |
| Default hotkey `CommandOrControl+Shift+Space`; conflict detection + rebind; never silent | **Pass (Windows)** | Conflict simulated with `RegisterHotKey`; blocking recorder; saved to `settings.json` |
| Hotkey opens the palette at p95 <100 ms with 10k seeded items (pre-warmed) | **Windows pass (Rust-side)** | Pre-warmed hidden window; `hotkey_to_show_us` logged; first-paint instrumentation is a follow-up |
| Typing filters instantly (<16 ms local); FTS <50 ms p95 | **Pass** | Local filter 5.4 ms p50 / 7.7 ms p95 at 10k; FTS 46.8 ms p95 for a term matching all 10k |
| Arrows, Enter, Shift+Enter, 1–9, Esc work; IME does not break Enter | **Pass** | Keymap unit tests + UIA navigation/activation; IME guard tested |
| Enter pastes into the app focused before opening; no silent wrong-target paste | **Pass (Windows)** | WinForms target received the exact text; focus poll aborts into copy-only |
| Copying an item back does not duplicate or bump it | **Pass** | Echo suppression verified through the real clipboard; row count/timestamp unchanged |
| Password-manager copy not stored | **Windows partial** | Concealed marker simulated and skipped; real KeePassXC/Bitwarden/1Password needs the manual matrix |
| Core RSS <40 MB; total 60–100 MB; gate >120 MB | **FAILED — M1 exit blocked** | Core **24–27 MB** passes. Total tree **360–370 MB** working set (~168–180 MB private) across six WebView2 processes; a fresh-profile control measured the same. Decision required. |
| No telemetry, account, background network, updater | **Pass** | No network code in M1; CSP has no remote origins |
| Release builds succeed on all three OSes | **Windows pass; macOS/Linux CI** | macOS/Linux compile in the CI matrix; no runtime verification here |
| Installer sizes recorded | **Deferred** | Packaging is M2-21; no installers in M1 |

## Latency measurements (Windows, 10k items)

| Measurement | Result | Method |
|---|---|---|
| Clipboard change → row visible in SQLite | **20.1 ms p50 / 28.0 ms p95 / 28.0 ms max** | `scripts/perf/capture_latency.py`: Win32 CF_UNICODETEXT, 5 ms SQLite polling, 20 iterations, release build, 10,024 items |
| Hotkey → palette shown | logged (`hotkey_to_show_us`) | Rust `Instant` around show/focus |
| Local filter (10k) | 5.36 ms p50 / 7.70 ms p95 | Vitest, 60 queries |
| FTS common term | 41.0 ms p50 / 46.8 ms p95 | `docs/perf/0.1.0-windows-x86_64.json` |
| Insert | 0.46 ms p50 / 0.90 ms p95 | same |
| Paste (focus active) | ~40–80 ms observed | Orchestrator logs; not a p95 harness |

> RSS is recorded in `docs/perf/rss-0.1.0-windows-x86_64.json` after the
> 10-minute idle sample.

## Not verified in this environment

- macOS runtime: capture, Accessibility, focus restore, Cmd+V.
- Linux X11 runtime: XFixes capture, `_NET_ACTIVE_WINDOW`, enigo XTEST.
- Multi-monitor and mixed-DPI placement (single monitor here).
- Tray menu interaction and autostart toggling (code-verified; menu not automated).
- Real password-manager applications (concealed markers simulated).
- 24-hour soak, idle CPU sampling.

These are release-gate items for the cross-platform pass and are listed in
`docs/TESTING.md`.

## RAM gate decision (required before M1 exit)

The plan's total budget (60–100 MB, gate 120 MB) is not achievable with Tauri +
WebView2 pre-warm on this machine. The core is 24–27 MB; the WebView2 tree adds
~330 MB (working set) / ~140 MB (private, excluding the core) regardless of the
profile. Options:

1. **Raise the total budget** (e.g., core < 40 MB, total < 400 MB) and keep the
   pre-warmed palette. Preserves the <100 ms open path; accepts a heavy idle
   footprint for a clipboard utility.
2. **Drop pre-warm:** create the palette on the hotkey and destroy it on close.
   Idle drops to roughly the core (~30 MB), but the first open is cold
   (250–700 ms), violating the <100 ms promise.
3. **Hybrid lifetime:** keep pre-warm while active, destroy the webview after N
   minutes of inactivity. Meets the idle budget between uses; the first open
   after idle is cold.
4. **Native palette UI** (GPUI/egui) with Tauri only for settings. Best memory,
   largest rewrite; out of scope for M1.
5. **WebView2 tuning** (`--disable-gpu`, renderer limits): measure before
   deciding; unlikely to reach 120 MB.

Recommendation: choose **(1)** if the 100 ms open path is non-negotiable, or
**(3)** if idle footprint matters more than first-open latency. Do not silently
change the gate.
