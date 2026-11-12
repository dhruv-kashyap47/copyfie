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

## RSS and idle CPU

Milestone 1 measures the real process tree (Copyfie + WebView2 children) with
10k items, the palette pre-warmed, and 10 minutes idle. Results are recorded in
`docs/perf/` and summarized in the M1 progress entry. The gate is:

- core process < 40 MB,
- total process tree 60–100 MB expected,
- **stop and resolve if total > 120 MB**.
