# Copyfie — Performance Baseline (Milestone 1)

> Measurements are a snapshot, not a target contract. Re-run on the same
> machine when touching the store, FTS, or the local filter.

## Environment

| | |
|---|---|
| Date | 2026-11-12 |
| Build | `bench` profile (opt-level 3, no fat LTO) for the core; Vite production build for the UI |
| OS | Windows 11 Home Single Language, 10.0.26300 |
| CPU | AMD Ryzen 5 5600H |
| RAM | 15.3 GB |
| Data | 10,000 unique text items, 30-day retention, 12 synthetic source apps |
| SQLite | Bundled rusqlite 0.42 + FTS5, WAL, 8 MB page cache |

## Fixture

`cargo run -p copyfie-core --example perf --profile bench -- --items 10000 --queries 200`

Each item is `"Item {i} {word} lorem ipsum dolor sit amet keyword{i%50} needle{i%500}"`
with a unique timestamp and a rotating source app. Timestamps are deterministic
(base `1_767_225_600_000`), so runs are comparable.

## Results

| Operation | p50 | p95 | Max | Total (10k/200) |
|---|---:|---:|---:|---:|
| Insert (new) | **0.46 ms** | 0.90 ms | 30.6 ms | 5.27 s |
| Dedupe (recopy) | **0.38 ms** | 1.74 ms | 554.9 ms* | 11.47 s |
| FTS — common term (100 of 10k) | **41.0 ms** | 46.8 ms | 64.2 ms | 8.36 s |
| FTS — rare term | **0.29 ms** | 0.49 ms | 1.15 ms | 0.07 s |
| FTS — no match | **0.18 ms** | 0.21 ms | 0.33 ms | 0.04 s |
| FTS — kind filter (Text) | **43.9 ms** | 47.7 ms | 55.9 ms | 8.86 s |
| List recent 100 | **0.29 ms** | 0.33 ms | 1.33 ms | 0.06 s |
| Retention cleanup (0 removed) | — | — | — | **4.8 ms** |
| Local filter, 10k summaries (V8) | **5.36 ms** | 7.70 ms | — | 60 runs |

\* One outlier from a WAL checkpoint during the recopy loop; p50/p95 are stable.

Storage: **8,450,048 bytes (~8.1 MiB)** for 10,004 items, including the FTS
index. That is ~840 bytes per text item.

## Observations

- **Insert and dedupe are sub-millisecond at p50**, so capture is not the
  bottleneck. The dedupe loop is slower in total because every recopy updates
  `source_app_name`, which fires the FTS update trigger and reindexes the row.
  A follow-up can skip source updates when the values are unchanged and avoid
  the trigger; correctness is unaffected.
- **FTS ranked search is near the 50 ms p95 gate** when a query matches all
  10k rows (`bm25` must score every match). Common single-word searches hit
  46.8 ms p95 here. Rare/no-match queries are sub-millisecond.
- **The instant layer is the UX guarantee:** the palette filters the loaded
  summaries synchronously at **5.4 ms p50 / 7.7 ms p95 for 10k items** (V8,
  desktop). FTS is the authoritative pass and only replaces local results when
  it finds something.
- **Recent listing is constant-time** (~0.3 ms) thanks to the
  `(is_pinned, captured_at)` index.
- **Retention cleanup** over 10k rows is one indexed delete; ~4.8 ms with
  nothing to remove. It runs at startup and every five minutes, not per write.
- The FTS limit is 100 rows; the palette renders 50.

## End-to-end capture latency (Windows, release)

`scripts/perf/capture_latency.py` sets `CF_UNICODETEXT` through Win32 and polls
the SQLite database every 5 ms until the row is visible. Over 20 iterations on
the release build with 10,024 items:

| p50 | p95 | Max |
|---:|---:|---:|
| **20.1 ms** | **28.0 ms** | 28.0 ms |

This covers OS notification, the privacy probe, the payload read, canonicalize +
BLAKE3, the store write, and the SQLite commit visibility. The acceptance budget
is 500 ms p95.

## How to run

Seed a disposable database and print the report:

```sh
cargo run -p copyfie-core --example perf --profile bench -- \
  --items 10000 --queries 200 --out docs/perf/<version>-<os>-<arch>.json
```

Seed the app database for RSS measurements (quit Copyfie first):

```powershell
.\scripts\perf\seed_10k.ps1
```

Local filter benchmark:

```sh
npx vitest run src/lib/localSearch.test.ts --reporter verbose
```

## Memory and open latency (M1-12, final)

**Hybrid lifecycle:** the palette is pre-warmed while active and the webview is
destroyed after 10 idle minutes (`Auto`, default). `Always` never destroys;
`Never` creates on hotkey and destroys on close. Capture continues with no
webview.

Release build, 10,004 items, 60-second steady-state samples at 5 s. (A
10-minute pre-hybrid run showed <1% variance across 116 samples; the earlier
figures are kept below as the baseline.)

### Final RSS (tuned build)

| State | Working set | Private committed |
|---|---:|---:|
| Palette warm (fresh, idle) | **342.8 MB avg / 345.5 MB max** | **106.7 MB** |
| Palette warm (after opening Settings) | 368.3 MB avg / 368.8 MB max | 112.0 MB |
| Palette webview destroyed (idle) | **30.1 MB** | **7.4 MB** |
| Core process alone | 30.1-30.2 MB | 6.7 MB |

Working-set sums double-count shared WebView2 pages; private bytes are the
fairer comparison. The settings window's renderer can remain in the shared
WebView2 process after it closes, which explains the higher post-Settings warm
number.

### WebView2 tuning (time-boxed)

| Configuration | Warm total (60 s) | Cold open p95 | Result |
|---|---:|---:|---|
| Baseline throttling flags only | 365.6-375.0 MB | 520 ms | rejected (cold too slow, heavier) |
| + `--disable-gpu --disable-breakpad` | 346.0-353.9 MB | 384-409 ms | better; cold marginal |
| + startup flags (`--disable-background-networking`, `--disable-component-update`, `--disable-default-apps`, `--disable-sync`, `--disable-client-side-phishing-detection`, `--metrics-recording-only`, `--no-first-run`) | **337.6-345.5 MB** | **390 ms** | **kept** - visuals verified in light/dark settings and palette |

All webviews share these args; WebView2 requires the environment options to
match or the second webview fails with `ERROR_INVALID_STATE (0x8007139F)`.
Visual check: palette rows, avatars, gradients, settings controls, and the
native window shadow all rendered correctly under software rendering.

### Open latency

| Path | p50 | p95 | Max | Samples |
|---|---:|---:|---:|---:|
| Warm (existing webview) | **31.0 ms** | **37.0 ms** | 37.0 ms | 10 |
| Cold (webview destroyed, bootstrap) | **371.0 ms** | **390.0 ms** | 390.0 ms | 20 |

The cold path injects the top 50 summaries, theme, and accent into the page
before startup JS, so the first paint does not wait for IPC. The remaining cost
is WebView2/browser startup.

### Gates (Windows, recorded)

| Gate | Target | Measured |
|---|---|---|
| Core working set | < 40 MB | **30.1 MB** (pass) |
| Total, webview destroyed | < 60 MB | **30.1 MB** (pass) |
| Total, webview warm | < 400 MB (budget set from measurement) | **345.5 MB max** (pass) |
| Cold open p95 | < 400 ms | **390 ms** (pass) |
| Warm open p95 | < 100 ms | **37 ms** (pass) |

macOS and Linux must be measured on their own before v1.0; Windows numbers do
not carry over. A native (non-WebView2) palette remains a possible later
optimization if footprint becomes a competitive issue.

### Earlier baseline (pre-hybrid, pre-flags)

For context: pre-warmed WebView2 with only throttling flags was **360-370 MB**
over 10 minutes (116 samples), ~168-180 MB private, and cold opens were
**520 ms p95**. The fresh-profile control (354-363 MB) ruled out profile bloat.
This is what triggered the hybrid decision.

