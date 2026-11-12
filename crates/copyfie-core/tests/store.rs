//! Store behaviour: dedupe, listing, pins, retention, and meta rows.

use copyfie_core::hash;
use copyfie_core::model::{ItemKind, ListQuery, NewItem, SourceApp};
use copyfie_core::retention::{Retention, DAY_MS};
use copyfie_core::store::Database;

const T0: i64 = 1_767_225_600_000;

fn text(content: &str, at: i64, days: u32) -> NewItem {
    NewItem::text(content, SourceApp::default(), at, Retention::Days(days))
}

fn source(name: &str) -> SourceApp {
    SourceApp {
        id: Some(format!("C:\\Apps\\{name}.exe")),
        name: Some(name.to_string()),
        window_title: Some(format!("{name} window")),
    }
}

fn open() -> Database {
    Database::open_in_memory().expect("open in-memory store")
}

#[test]
fn new_content_inserts_one_row() {
    let db = open();
    let inserted = db.upsert(&text("hello", T0, 30)).expect("upsert");

    assert!(inserted.is_new);
    assert_eq!(db.count(None).expect("count"), 1);

    let row = db.get(inserted.id).expect("get").expect("row");
    assert_eq!(row.text, "hello");
    assert_eq!(row.preview, "hello");
    assert_eq!(row.first_seen_at, T0);
    assert_eq!(row.captured_at, T0);
    assert_eq!(row.expires_at, Some(T0 + 30 * DAY_MS));
    assert!(!row.is_pinned);
    assert_eq!(row.use_count, 0);
}

#[test]
fn recopy_dedupes_and_moves_to_top() {
    let db = open();

    let first = db.upsert(&text("same", T0, 30)).expect("first");
    let second = db.upsert(&text("same", T0 + 5_000, 30)).expect("second");

    assert!(first.is_new);
    assert!(!second.is_new);
    assert_eq!(first.id, second.id);
    assert_eq!(db.count(None).expect("count"), 1);

    let row = db.get(first.id).expect("get").expect("row");
    assert_eq!(row.captured_at, T0 + 5_000);
    assert_eq!(row.first_seen_at, T0);
    assert_eq!(row.expires_at, Some(T0 + 5_000 + 30 * DAY_MS));
}

#[test]
fn recopy_refreshes_source_and_resets_use_count() {
    let db = open();

    let item = NewItem::text("payload", source("Notepad"), T0, Retention::Days(30));
    let first = db.upsert(&item).expect("first");
    db.touch_used(first.id, T0 + 1).expect("touch");

    let recopy = NewItem::text("payload", source("Terminal"), T0 + 2, Retention::Days(30));
    let second = db.upsert(&recopy).expect("recopy");

    let row = db.get(second.id).expect("get").expect("row");
    assert_eq!(row.source_app_name.as_deref(), Some("Terminal"));
    assert_eq!(row.use_count, 0);

    let summary = &db.list(&ListQuery::default()).expect("list")[0];
    assert_eq!(summary.source_app_name.as_deref(), Some("Terminal"));
}

#[test]
fn canonicalization_dedupes_crlf_and_trailing_nul() {
    let db = open();

    let first = db
        .upsert(&text("line one\r\nline two\0", T0, 30))
        .expect("first");
    let second = db
        .upsert(&text("line one\nline two", T0 + 1, 30))
        .expect("second");

    assert_eq!(first.id, second.id);
    assert!(!second.is_new);

    let row = db.get(first.id).expect("get").expect("row");
    assert_eq!(row.text, "line one\nline two");
    assert_eq!(row.content_hash, hash::hash_text("line one\nline two"));
}

#[test]
fn full_text_is_kept_but_preview_is_capped() {
    let db = open();
    let long: String = "x".repeat(5_000);
    let inserted = db.upsert(&text(&long, T0, 30)).expect("upsert");

    let row = db.get(inserted.id).expect("get").expect("row");
    assert_eq!(row.text.len(), 5_000);
    assert_eq!(row.preview.chars().count(), 400);
    assert_eq!(row.byte_size, 5_000);
}

#[test]
fn list_orders_pins_first_then_recency() {
    let db = open();
    let oldest = db.upsert(&text("oldest", T0, 30)).expect("a");
    let newest = db.upsert(&text("newest", T0 + 2, 30)).expect("b");
    let middle = db.upsert(&text("middle", T0 + 1, 30)).expect("c");

    db.set_pinned(oldest.id, true, T0 + 3, Retention::Days(30))
        .expect("pin");

    let listed = db.list(&ListQuery::default()).expect("list");
    let ids: Vec<i64> = listed.iter().map(|item| item.id).collect();
    assert_eq!(ids, vec![oldest.id, newest.id, middle.id]);
    assert!(listed[0].is_pinned);
}

#[test]
fn list_filters_kind_and_pins_and_pages() {
    let db = open();
    let text_id = db.upsert(&text("text item", T0, 30)).expect("text").id;
    let files = NewItem::files(
        &["C:\\a.txt"],
        SourceApp::default(),
        T0 + 1,
        Retention::Days(30),
    );
    let files_id = db.upsert(&files).expect("files").id;

    let files_only = db
        .list(&ListQuery {
            kind: Some(ItemKind::Files),
            ..ListQuery::default()
        })
        .expect("filter");
    assert_eq!(files_only.len(), 1);
    assert_eq!(files_only[0].id, files_id);

    let paged = db
        .list(&ListQuery {
            limit: 1,
            offset: 1,
            ..ListQuery::default()
        })
        .expect("page");
    assert_eq!(paged.len(), 1);
    assert_eq!(paged[0].id, text_id);

    db.set_pinned(files_id, true, T0 + 2, Retention::Days(30))
        .expect("pin");
    let pinned = db
        .list(&ListQuery {
            pinned_only: true,
            ..ListQuery::default()
        })
        .expect("pinned");
    assert_eq!(pinned.len(), 1);
    assert_eq!(pinned[0].id, files_id);
}

#[test]
fn retention_deletes_unpinned_at_the_boundary() {
    let db = open();
    db.upsert(&text("expires", T0, 7)).expect("insert");

    let boundary = T0 + 7 * DAY_MS;
    assert_eq!(db.delete_expired(boundary - 1).expect("early"), 0);
    assert_eq!(db.delete_expired(boundary).expect("boundary"), 1);
    assert_eq!(db.count(None).expect("count"), 0);
}

#[test]
fn forever_retention_never_expires() {
    let db = open();
    db.upsert(&NewItem::text(
        "keep",
        SourceApp::default(),
        T0,
        Retention::Forever,
    ))
    .expect("insert");

    assert_eq!(
        db.delete_expired(T0 + 100 * 365 * DAY_MS).expect("clean"),
        0
    );
}

#[test]
fn pins_survive_cleanup_and_unpin_expires_immediately() {
    let db = open();
    let inserted = db.upsert(&text("pinned", T0, 7)).expect("insert");

    db.set_pinned(inserted.id, true, T0 + 1, Retention::Days(7))
        .expect("pin");
    let pinned = db.get(inserted.id).expect("get").expect("row");
    assert!(pinned.is_pinned);
    assert_eq!(pinned.expires_at, None);

    assert_eq!(
        db.delete_expired(T0 + 10 * DAY_MS).expect("clean"),
        0,
        "pinned rows are never deleted by age"
    );

    // Unpinning after the original expiry deletes the row immediately.
    db.set_pinned(inserted.id, false, T0 + 10 * DAY_MS, Retention::Days(7))
        .expect("unpin");
    assert_eq!(db.count(None).expect("count"), 0);
}

#[test]
fn unpin_before_expiry_recomputes_the_deadline() {
    let db = open();
    let inserted = db.upsert(&text("later", T0, 7)).expect("insert");

    db.set_pinned(inserted.id, true, T0 + 1, Retention::Days(7))
        .expect("pin");
    db.set_pinned(inserted.id, false, T0 + 2, Retention::Days(7))
        .expect("unpin");

    let row = db.get(inserted.id).expect("get").expect("row");
    assert!(!row.is_pinned);
    assert_eq!(row.expires_at, Some(T0 + 7 * DAY_MS));
}

#[test]
fn clear_keeps_pins_unless_asked_otherwise() {
    let db = open();
    let pinned = db.upsert(&text("pinned", T0, 30)).expect("a").id;
    db.upsert(&text("plain", T0 + 1, 30)).expect("b");
    db.set_pinned(pinned, true, T0 + 2, Retention::Days(30))
        .expect("pin");

    assert_eq!(db.clear(true).expect("clear"), 1);
    assert_eq!(db.count(None).expect("count"), 1);

    assert_eq!(db.clear(false).expect("clear all"), 1);
    assert_eq!(db.count(None).expect("count"), 0);
}

#[test]
fn touch_used_increments_and_stamps() {
    let db = open();
    let id = db.upsert(&text("used", T0, 30)).expect("insert").id;

    assert!(db.touch_used(id, T0 + 9).expect("touch"));
    assert!(!db.touch_used(id + 100, T0 + 9).expect("missing"));

    let row = db.get(id).expect("get").expect("row");
    assert_eq!(row.use_count, 1);
    assert_eq!(row.last_used_at, Some(T0 + 9));
}

#[test]
fn delete_is_idempotent_and_missing_ids_return_none() {
    let db = open();
    let id = db.upsert(&text("gone", T0, 30)).expect("insert").id;

    assert!(db.delete(id).expect("delete"));
    assert!(!db.delete(id).expect("second delete"));
    assert!(db.get(id).expect("get").is_none());
    assert!(!db
        .set_pinned(id, true, T0, Retention::Days(30))
        .expect("pin"));
}

#[test]
fn meta_round_trips() {
    let db = open();
    assert_eq!(db.get_meta("trial_started_at").expect("get"), None);

    db.set_meta("trial_started_at", "1767225600000")
        .expect("set");
    db.set_meta("trial_started_at", "1767225600001")
        .expect("update");

    assert_eq!(
        db.get_meta("trial_started_at").expect("get").as_deref(),
        Some("1767225600001")
    );
}

#[test]
fn counts_are_per_kind() {
    let db = open();
    db.upsert(&text("a", T0, 30)).expect("a");
    db.upsert(&text("b", T0 + 1, 30)).expect("b");
    db.upsert(&NewItem::files(
        &["C:\\x.txt"],
        SourceApp::default(),
        T0 + 2,
        Retention::Days(30),
    ))
    .expect("files");

    assert_eq!(db.count(None).expect("all"), 3);
    assert_eq!(db.count(Some(ItemKind::Text)).expect("text"), 2);
    assert_eq!(db.count(Some(ItemKind::Files)).expect("files"), 1);
    assert_eq!(db.count(Some(ItemKind::Image)).expect("images"), 0);
}

#[test]
fn file_items_dedupe_regardless_of_order() {
    let db = open();
    let a = NewItem::files(
        &["C:\\a.txt", "C:\\b.txt"],
        SourceApp::default(),
        T0,
        Retention::Days(30),
    );
    let b = NewItem::files(
        &["C:\\b.txt\\", "C:\\a.txt"],
        SourceApp::default(),
        T0 + 1,
        Retention::Days(30),
    );

    let first = db.upsert(&a).expect("first");
    let second = db.upsert(&b).expect("second");
    assert_eq!(first.id, second.id);
    assert!(!second.is_new);
}
