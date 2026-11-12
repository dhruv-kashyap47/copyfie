# Copyfie — Architecture

## Workspace

| Crate / directory | Responsibility | Hard rules |
|---|---|---|
| `crates/copyfie-core` | Models, canonicalization, BLAKE3, SQLite store, FTS5, retention, settings model | No Tauri, no OS APIs, no clipboard reads |
| `platform/common` | Clipboard/focus traits, shared `clipboard-rs` and `enigo` adapters | No OS-specific code |
| `platform/windows` | Win32 clipboard markers, foreground process, focus restore | — |
| `platform/macos` | `NSPasteboard` markers, `NSWorkspace` foreground app, activation | — |
| `platform/linux` | X11 `_NET_ACTIVE_WINDOW`, `WM_CLASS`, KDE hint | Wayland capture is deferred |
| `src-tauri` | Tauri shell: commands, windows, hotkey, tray, capture runner, paste orchestrator, settings | Thin; no SQL or clipboard parsing |
| `src/` | Svelte palette + settings windows | No state library, no router |

## Data flow

### Capture

```
OS notification
  → ClipboardReader::probe (formats only)
  → PrivacyGate (paused / concealed / blocklist / has text)
  → ClipboardReader::read_text
  → canonicalize + BLAKE3
  → echo suppression (one-shot token)
  → StoreActor upsert (single writer, dedupe index)
  → coalesced history:changed (at most one per 80 ms)
  → optional delete_expired on the cleanup cadence
```

Payload data is **never read** before the gate. A concealed item, a paused
capture, or a blocked app returns a skip reason and nothing touches SQLite.

### Palette

```
hotkey / tray / second instance
  → show_palette: capture paste target, position on cursor monitor, show, focus
  → emit palette:open { openId }
  → frontend loads 100 summaries, filters locally per keystroke
  → 50 ms debounce → history_search (FTS5, bm25, pinned first)
  → history:changed refreshes the list while open
```

### Paste

```
Enter
  → paste_item(id)
  → hide palette, wait until hidden
  → restore captured target (HWND / pid / X11 window)
  → poll until the target is active (700 ms cap)
  → arm suppression, write clipboard, settle 25 ms
  → Ctrl+V / Cmd+V (enigo)
  → touch_used(id)
```

If focus is not confirmed, nothing is sent: the item stays on the clipboard and
the palette returns with a `copied` report.

## Invariants

1. **The store has one writer.** `StoreHandle` owns a thread with the only
   `Connection`; commands and capture send closures and await replies.
2. **Dedupe is the unique index `(kind, content_hash)`.** `is_new` comes from
   which statement matched, not from timestamps.
3. **Capture never depends on entitlement.** Trial/locked state affects
   presentation only (M2-15).
4. **Echo writes are suppressed.** Every write from Copyfie arms a one-shot
   content token before touching the clipboard.
5. **Content never appears in logs.** Logs carry ids, counts, and skip reasons.
6. **The palette never waits on SQLite to open.** It renders cached summaries
   and refreshes after.
7. **No OS APIs in core; no Tauri in core or platform crates.**

## IPC surface (M1)

Commands:

| Command | Purpose |
|---|---|
| `app_info` | Name, app/core version, bundle id |
| `history_list` / `history_search` | Recent summaries / FTS results (100 cap) |
| `history_pin` / `history_delete` / `history_clear` | Mutations |
| `copy_item` / `paste_item` | Copy-only and full paste orchestration |
| `store_stats` | Counts for diagnostics |
| `hotkey_status` / `set_hotkey` | Shortcut state and rebind |
| `settings_get` / `settings_save` | Settings form |
| `palette_show` / `hide_palette` / `open_settings` | Window control |

Events:

| Event | Payload |
|---|---|
| `history:changed` | `{ revision }` (coalesced) |
| `palette:open` | `{ openId }` |
| `settings:changed` | `{ theme, accent }` |

## Storage layout

```
<app_data>/com.copyfie.app/       (Windows: %APPDATA%)
  clipboard.db (+ -wal, -shm)     SQLite, schema v3, FTS5
  settings.json (+ .bak, .tmp)    theme, accent, retention, pause, hotkey
  thumbs/                         content-addressed thumbnails (M2)
  logs/
```

## Known debt

- The Rust summary cache (10k previews) is not implemented yet; the palette
  loads the 100 most recent and relies on FTS for the rest.
- Dedupe updates `source_app_name` unconditionally, firing the FTS trigger on
  every recopy. Correct, but avoidable work.
- Settings tray check items are not synchronized when pause/autostart change in
  Settings.
- Wayland capability notice and copy-only UX are M2-12; capture is currently
  not started on Wayland.
