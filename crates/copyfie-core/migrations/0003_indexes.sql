-- 0003_indexes.sql — query indexes.
--
-- items_kind_hash is the dedupe identity; the upsert cannot work without it.

CREATE UNIQUE INDEX items_kind_hash ON items(kind, content_hash);
CREATE INDEX items_captured ON items(captured_at DESC);
CREATE INDEX items_pinned_captured ON items(is_pinned DESC, captured_at DESC);
CREATE INDEX items_expiry ON items(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX items_source ON items(source_app_id);
