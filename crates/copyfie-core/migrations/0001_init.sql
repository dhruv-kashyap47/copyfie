-- 0001_init.sql — core tables.
--
-- Forward-only. Never edit after release.

CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE items (
  id                  INTEGER PRIMARY KEY,
  kind                INTEGER NOT NULL,          -- 0 text, 1 image, 2 files
  content_hash        BLOB NOT NULL,             -- BLAKE3-256, 32 bytes
  text                TEXT NOT NULL DEFAULT '',  -- text payload or JSON path array
  preview             TEXT NOT NULL,             -- first ~400 chars, display/search
  blob_name           TEXT,                      -- "<hex>.png" for images
  thumb_name          TEXT,                      -- "<hex>.png"
  byte_size           INTEGER NOT NULL DEFAULT 0,
  width               INTEGER,
  height              INTEGER,
  source_app_id       TEXT,                      -- exe path / bundle id / WM_CLASS
  source_app_name     TEXT,
  source_window_title TEXT,
  is_pinned           INTEGER NOT NULL DEFAULT 0,
  captured_at         INTEGER NOT NULL,          -- unix ms
  first_seen_at       INTEGER NOT NULL,
  last_used_at        INTEGER,
  use_count           INTEGER NOT NULL DEFAULT 0,
  expires_at          INTEGER,                   -- NULL = never
  ocr_text            TEXT,                      -- OCR feature (v1.1)
  CHECK (kind IN (0, 1, 2)),
  CHECK (is_pinned IN (0, 1))
);
