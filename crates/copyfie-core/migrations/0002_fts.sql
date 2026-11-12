-- 0002_fts.sql — external-content FTS5 index.
--
-- Mirrors items.text, items.preview, and items.source_app_name. External
-- content avoids duplicating payloads; triggers keep it consistent for
-- inserts, deletes, and the columns we actually change on recopy.

CREATE VIRTUAL TABLE items_fts USING fts5(
  text,
  preview,
  source_app_name,
  content='items',
  content_rowid='id',
  tokenize="unicode61 remove_diacritics 2",
  prefix='2 3'
);

CREATE TRIGGER items_fts_ai AFTER INSERT ON items BEGIN
  INSERT INTO items_fts(rowid, text, preview, source_app_name)
  VALUES (new.id, new.text, new.preview, COALESCE(new.source_app_name, ''));
END;

CREATE TRIGGER items_fts_ad AFTER DELETE ON items BEGIN
  INSERT INTO items_fts(items_fts, rowid, text, preview, source_app_name)
  VALUES ('delete', old.id, old.text, old.preview, COALESCE(old.source_app_name, ''));
END;

CREATE TRIGGER items_fts_au AFTER UPDATE OF text, preview, source_app_name ON items BEGIN
  INSERT INTO items_fts(items_fts, rowid, text, preview, source_app_name)
  VALUES ('delete', old.id, old.text, old.preview, COALESCE(old.source_app_name, ''));
  INSERT INTO items_fts(rowid, text, preview, source_app_name)
  VALUES (new.id, new.text, new.preview, COALESCE(new.source_app_name, ''));
END;

INSERT INTO meta(key, value) VALUES ('fts_built', '1');
