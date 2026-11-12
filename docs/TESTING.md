# Copyfie — Testing

## Automated

| Layer | Command | Coverage |
|---|---|---|
| Rust | `cargo test --workspace` | Core hash/model/store/migrations/FTS/retention, capture gate/pipeline/suppression/events, actor, thumbnails, settings, platform helpers |
| Rust lint | `cargo clippy --workspace --all-targets -- -D warnings` | Deny warnings |
| Format | `cargo fmt --all --check` | — |
| Frontend | `npm test` | Keymap, local search (10k perf), formatting, theme, shortcut recording |
| Types | `npm run check` | svelte-check + TypeScript |
| Build | `npm run build` + `npm run size:check` | 50 KB gzipped JS+CSS budget |
| Licences | `cargo deny --config scripts/license/cargo-deny.toml check` | Allowlist, bans, advisories, sources |
| Perf | `cargo run -p copyfie-core --example perf --profile bench -- …` | Insert/dedupe/FTS/list/cleanup at 10k (see `docs/PERF.md`) |
| RSS | `scripts/perf/rss.ps1` | Process tree; warm and destroyed samples (M1-12) |
| Open latency | `scripts/perf/open_latency.py` | Warm/cold hotkey→paint p50/p95 from the app log (M1-12) |

CI runs the lint/test/build matrix on `ubuntu-22.04`, `windows-latest`, and
`macos-14`, plus `cargo-deny` and `npm audit`.

## Manual matrix (Milestone 1)

| # | Check | Windows | macOS | Linux X11 |
|---|---|---|---|---|
| 1 | Copy text in browser/editor/terminal → appears | ☑ | ☐ | ☐ |
| 2 | Hotkey opens the palette and focuses it | ☑ | ☐ | ☐ |
| 3 | Enter pastes into the app focused before opening | ☑ | ☐ | ☐ |
| 4 | Copying an item back does not duplicate or bump it | ☑ | ☐ | ☐ |
| 5 | Password-manager copy is skipped | ☑ (Office marker simulated) | ☐ | ☐ |
| 6 | Arrow keys, PageUp/Down, Home/End, Tab filters, Esc | ☑ | ☐ | ☐ |
| 7 | Search filters instantly; FTS results replace local ones | ☑ | ☐ | ☐ |
| 8 | Pin/unpin, delete, quick digits 1–9 | ☑ pin/quick; delete store-tested | ☐ | ☐ |
| 9 | Shortcut conflict → blocking recorder → rebind | ☑ | ☐ | ☐ |
| 10 | Single instance reveals the existing palette | ☑ | ☐ | ☐ |
| 11 | Tray: show, settings, pause, start at login, quit | ☐ (menu not automated) | ☐ | ☐ |
| 12 | Theme light/dark/system and accent apply live | ☑ | ☐ | ☐ |
| 13 | Retention setting affects new items; pins never expire | ☑ | ☐ | ☐ |
| 14 | Pause stops capture; resume restarts it | ☑ | ☐ | ☐ |
| 15 | Settings window is destroyed on close and recreated | ☑ | ☐ | ☐ |
| 16 | Hidden pre-warm: palette already populated when shown | ☑ | ☐ | ☐ |
| 17 | 10k items: open/search/paste remain responsive | ☑ | ☐ | ☐ |
| 18 | Hybrid: palette warm while active; webview destroyed after the idle timeout; core keeps capturing | ☑ | ☐ | ☐ |
| 19 | Palette settings: Auto/Always/Never and 5/10/30 minutes apply without restart | ☑ | ☐ | ☐ |
| 20 | Cold open after destroy p95 <400 ms; warm p95 <100 ms | ☑ (390/37 ms) | ☐ | ☐ |
| 21 | Webview destroyed: total RSS <60 MB, core <40 MB | ☑ (30.1/30.1 MB) | ☐ | ☐ |
| 22 | Warm RSS within the recorded budget (<400 MB Windows) | ☑ (345.5 MB max) | ☐ | ☐ |
| 23 | Escape hides; hotkey reopens warm or cold | ☑ | ☐ | ☐ |
| 24 | No network traffic while idle | ☑ (none implemented) | ☑ | ☑ |

## How to run a manual pass

1. `npm install`, then `npm run tauri build -- --debug --no-bundle`.
2. Run `target/debug/copyfie.exe` (hidden) and use the hotkey; or add `--show`
   for a visible palette at launch.
3. Keep `%APPDATA%\com.copyfie.app\logs\` open for outcomes (no content).
4. Seed the 10k fixture with `scripts/perf/seed_10k.ps1` after quitting.
5. For cold-open testing, a second launch with `--palette-destroy` destroys the
   hidden webview in the running instance (test hook used by
   `scripts/perf/open_latency.py`; a non-elevated window must be foreground).
6. Measure memory with `scripts/perf/rss.ps1` in both `Always`/prewarmed and
   destroyed states; measure opens with `scripts/perf/open_latency.py`.

## Known test gaps

- macOS and Linux X11 runtime paths are compiled but not executed in this
  environment; CI compiles them, and **M1-14/M1-15** record their RAM/open
  measurements. The matrix above is the release gate.
- Delete via `Ctrl+Backspace` is covered by store/keymap tests but not by UI
  automation (synthetic focus was unreliable).
- Tray menu interaction and autostart toggling are code-verified only.
- Synthetic keys are dropped while an elevated window is foreground (UIPI);
  activate a normal window before running input automation.
