//! Deterministic 10k fixture and baseline benchmarks for `copyfie-core`.
//!
//! Usage:
//!   cargo run -p copyfie-core --example perf --profile bench -- \
//!     --db <path> --items 10000 --queries 200 --out <report.json>
//!
//! It seeds unique text items, then measures insert, dedupe (recopy), FTS
//! queries (common, rare, no-match, kind-filtered), a recent listing, and the
//! retention cleanup. The report is printed as JSON and optionally written to
//! `--out`.
//!
//! Point `--db` at a real app database to build the 10k fixture used by the
//! M1-12 RSS measurement. The app must be closed while seeding.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use copyfie_core::model::ListQuery;
use copyfie_core::{Database, ItemKind, NewItem, Retention, SourceApp};
use serde_json::json;

const DAY_MS: i64 = 86_400_000;
const BASE_TIME: i64 = 1_767_225_600_000;

const WORDS: &[&str] = &[
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "theta", "kappa", "lambda", "sigma",
];

fn main() {
    let mut db_path: Option<PathBuf> = None;
    let mut items = 10_000_usize;
    let mut queries = 200_usize;
    let mut out: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db" => db_path = args.next().map(PathBuf::from),
            "--items" => {
                items = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(items)
            }
            "--queries" => {
                queries = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(queries)
            }
            "--out" => out = args.next().map(PathBuf::from),
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    let path = db_path.unwrap_or_else(|| std::env::temp_dir().join("copyfie-perf.db"));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create db directory");
    }

    let db = Database::open(&path).expect("open database");
    eprintln!("database: {}", path.display());

    let fixture: Vec<NewItem> = (0..items).map(make_item).collect();

    let insert = time_many(items, |index| {
        db.upsert(&fixture[index]).expect("insert");
    });

    let dedupe = time_many(items, |index| {
        db.upsert(&fixture[index]).expect("dedupe");
    });

    let common = time_many(queries, |_| {
        let results = db.search("item", None, 100).expect("search");
        assert_eq!(results.len(), 100, "common query should fill the limit");
    });

    let rare = time_many(queries, |_| {
        let results = db.search("9999", None, 100).expect("search");
        assert_eq!(results.len(), 1, "rare query should match exactly one row");
    });

    let no_match = time_many(queries, |_| {
        let results = db.search("zzzzno-match", None, 100).expect("search");
        assert!(results.is_empty());
    });

    let kind_filtered = time_many(queries, |_| {
        let results = db
            .search("needle", Some(ItemKind::Text), 100)
            .expect("search");
        assert_eq!(results.len(), 100);
    });

    let list = time_many(queries, |_| {
        let results = db
            .list(&ListQuery {
                limit: 100,
                ..ListQuery::default()
            })
            .expect("list");
        assert_eq!(results.len(), 100);
    });

    let cleanup_start = Instant::now();
    let removed = db
        .delete_expired(BASE_TIME + DAY_MS)
        .expect("retention cleanup");
    let cleanup_elapsed = cleanup_start.elapsed();
    let cleanup = Stats::from_samples(cleanup_elapsed, vec![cleanup_elapsed]);

    let file_bytes = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
    let total_items = db.count(None).expect("count");

    let report = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "items": total_items,
        "queries": queries,
        "db_bytes": file_bytes,
        "insert": insert.json(),
        "dedupe": dedupe.json(),
        "fts_common": common.json(),
        "fts_rare": rare.json(),
        "fts_no_match": no_match.json(),
        "fts_kind_text": kind_filtered.json(),
        "list_recent_100": list.json(),
        "retention_cleanup": {
            "removed": removed,
            "total_ms": round_ms(cleanup.total),
        },
        "notes": "text-only history; 30-day retention; bundled SQLite + FTS5",
    });

    let encoded = serde_json::to_string_pretty(&report).expect("serialize report");
    println!("{encoded}");

    if let Some(out) = out {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).expect("create report directory");
        }
        std::fs::write(&out, &encoded).expect("write report");
        eprintln!("report: {}", out.display());
    }
}

fn make_item(index: usize) -> NewItem {
    let source = SourceApp {
        id: Some(format!("C:\\Apps\\App{}.exe", index % 12)),
        name: Some(format!("App {}", index % 12)),
        window_title: Some(format!("Window {index}")),
    };

    let text = format!(
        "Item {index} {} lorem ipsum dolor sit amet keyword{} needle{}",
        WORDS[index % WORDS.len()],
        index % 50,
        index % 500
    );

    NewItem::text(&text, source, BASE_TIME + index as i64, Retention::Days(30))
}

fn time_many<F: FnMut(usize)>(count: usize, mut operation: F) -> Stats {
    let mut samples = Vec::with_capacity(count);
    let started = Instant::now();
    for index in 0..count {
        let sample = Instant::now();
        operation(index);
        samples.push(sample.elapsed());
    }
    Stats::from_samples(started.elapsed(), samples)
}

#[derive(Debug)]
struct Stats {
    total: Duration,
    p50: Duration,
    p95: Duration,
    max: Duration,
}

impl Stats {
    fn from_samples(total: Duration, mut samples: Vec<Duration>) -> Self {
        if samples.is_empty() {
            return Self {
                total,
                p50: Duration::ZERO,
                p95: Duration::ZERO,
                max: Duration::ZERO,
            };
        }
        samples.sort_unstable();
        let p50 = samples[samples.len() / 2];
        let p95 = samples[(samples.len() * 95 / 100).min(samples.len() - 1)];
        let max = *samples.last().expect("non-empty");
        Self {
            total,
            p50,
            p95,
            max,
        }
    }

    fn json(&self) -> serde_json::Value {
        json!({
            "p50_ms": round_ms(self.p50),
            "p95_ms": round_ms(self.p95),
            "max_ms": round_ms(self.max),
            "total_ms": round_ms(self.total),
        })
    }
}

fn round_ms(duration: Duration) -> f64 {
    (duration.as_secs_f64() * 1000.0 * 1000.0).round() / 1000.0
}
