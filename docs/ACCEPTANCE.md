# Copyfie — Milestone 1 Acceptance

Status as of 2026-11-12. Criteria are from the plan's MVP acceptance list.

| Criterion | Status | Evidence / gap |
|---|---|---|
| Text copied in Notepad/TextEdit/gedit appears within 500 ms p95 | **Windows pass** | **20.1 ms p50 / 28.0 ms p95** end-to-end (release build, 10k DB) |
| Default hotkey `CommandOrControl+Shift+Space`; conflict detection + rebind; never silent | **Pass (Windows)** | Conflict simulated with `RegisterHotKey`; blocking recorder; saved to `settings.json` |
| Hotkey opens the palette at p95 <100 ms warm / <400 ms cold with 10k seeded items | **Windows pass** | Warm **37 ms p95** (10 runs), cold **390 ms p95** (20 runs, hybrid bootstrap) |
| Typing filters instantly (<16 ms local); FTS <50 ms p95 | **Pass** | Local filter 5.4 ms p50 / 7.7 ms p95 at 10k; FTS 46.8 ms p95 for a term matching all 10k |
| Arrows, Enter, Shift+Enter, 1–9, Esc work; IME does not break Enter | **Pass** | Keymap unit tests + UIA navigation/activation; IME guard tested |
| Enter pastes into the app focused before opening; no silent wrong-target paste | **Pass (Windows)** | WinForms target received the exact text; focus poll aborts into copy-only |
| Copying an item back does not duplicate or bump it | **Pass** | Echo suppression verified through the real clipboard; row count/timestamp unchanged |
| Password-manager copy not stored | **Windows partial** | Concealed marker simulated and skipped; real KeePassXC/Bitwarden/1Password needs the manual matrix |
| RAM: core <40 MB; total destroyed <60 MB; warm total within its measured budget; cold p95 <400 ms; warm p95 <100 ms | **Windows pass** | Core **30.1 MB**; destroyed **30.1 MB** (7.4 MB private); warm **345.5 MB max** (106.7 MB private) → budget **<400 MB**; cold **390 ms p95**; warm **37 ms p95**. Hybrid lifecycle + WebView2 flags. |
| No telemetry, account, background network, updater | **Pass** | No network code in M1; CSP has no remote origins |
| Release builds succeed on all three OSes | **Windows pass; macOS/Linux CI** | macOS/Linux compile in the CI matrix; no runtime verification here |
| Installer sizes recorded | **Deferred** | Packaging is M2-21; no installers in M1 |

## Latency measurements (Windows, 10k items)

| Measurement | Result | Method |
|---|---|---|
| Clipboard change -> row visible in SQLite | **20.1 ms p50 / 28.0 ms p95 / 28.0 ms max** | `scripts/perf/capture_latency.py`: Win32 CF_UNICODETEXT, 5 ms SQLite polling, 20 iterations, release build, 10,024 items |
| Hotkey -> palette shown, warm | **31.0 ms p50 / 37.0 ms p95 / 37.0 ms max** | `scripts/perf/open_latency.py`, 10 runs, log metrics |
| Hotkey -> palette shown, cold (webview destroyed) | **371.0 ms p50 / 390.0 ms p95 / 390.0 ms max** | 20 runs, `--palette-destroy` test hook + bootstrap first paint |
| Local filter (10k) | 5.36 ms p50 / 7.70 ms p95 | Vitest, 60 queries |
| FTS common term | 41.0 ms p50 / 46.8 ms p95 | `docs/perf/0.1.0-windows-x86_64.json` |
| Insert | 0.46 ms p50 / 0.90 ms p95 | same |
| Paste (focus active) | ~40–80 ms observed | Orchestrator logs; not a p95 harness |

> RSS: `docs/perf/rss-final-warm.json`, `docs/perf/rss-final-idle.json`; opens:
> `docs/perf/open-latency-tuned.json`; tuning baseline:
> `docs/perf/rss-hybrid-*.json`, `docs/perf/open-latency-baseline.json`.

## Not verified in this environment

- macOS runtime: capture, Accessibility, focus restore, Cmd+V.
- Linux X11 runtime: XFixes capture, `_NET_ACTIVE_WINDOW`, enigo XTEST.
- Multi-monitor and mixed-DPI placement (single monitor here).
- Tray menu interaction and autostart toggling (code-verified; menu not automated).
- Real password-manager applications (concealed markers simulated).
- 24-hour soak, idle CPU sampling.

These are release-gate items for the cross-platform pass and are listed in
`docs/TESTING.md`.

## RAM gate decision (resolved)

**Chosen: hybrid lifecycle + per-OS measured gates.** The original 60-100 MB
total / 120 MB gate is not achievable with Tauri + WebView2 on Windows; the
core is ~30 MB but a warm WebView2 tree is ~343 MB. After a 10-minute idle
period the webview is destroyed and the app returns to **~30 MB**. The cold
path was optimized (bootstrap + flags) to **390 ms p95**.

- `Auto` (default): pre-warmed while active, destroyed after 10 idle minutes.
- `Always`: never destroy.
- `Never`: create on hotkey, destroy on close.
- Settings expose the mode and the idle timeout (5/10/30 minutes).
- Warm budget is recorded from the measurement: **<400 MB** on Windows.
- WebView2 flags kept after a time-boxed test: `--disable-gpu`,
  `--disable-breakpad`, and startup flags; visuals verified.

**Native palette rewrite** (GPUI/egui, Tauri only for settings) remains a
possible later optimization if memory footprint becomes a competitive issue. It
is out of scope for Milestone 1.

**macOS and Linux** must be measured independently (M1-14/M1-15). Do not apply
the Windows budget to them; record their own warm/destroyed numbers and open
p95.
