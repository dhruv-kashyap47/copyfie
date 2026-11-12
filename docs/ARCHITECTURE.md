# Copyfie - Architecture

## Workspace

| Crate / directory | Responsibility | Hard rules |
|---|---|---|
| `crates/copyfie-core` | Models, canonicalization, BLAKE3, SQLite store, FTS5, retention, settings model | No Tauri, no OS APIs, no clipboard reads |
| `platform/common` | Clipboard/focus traits, shared `clipboard-rs` and `enigo` adapters | No OS-specific code |
| `platform/windows` | Win32 clipboard markers, foreground process, focus restore | - |
| `platform/macos` | `NSPasteboard` markers, `NSWorkspace` foreground app, activation | - |
| `platform/linux` | X11 `_NET_ACTIVE_WINDOW`, `WM_CLASS`, KDE hint | Wayland capture is deferred |
| `src-tauri` | Tauri shell: commands, palette lifecycle, windows, hotkey, tray, capture runner, paste orchestrator, settings | Thin; no SQL or clipboard parsing |
| `src/` | Svelte palette + settings windows | No state library, no router |

## Data flow

### Capture

```
OS notification
  -> ClipboardReader::probe (formats only)
  -> PrivacyGate (paused / concealed / blocklist / has text)
  -> ClipboardReader::read_text
  -> canonicalize + BLAKE3
  -> echo suppression (one-shot token)
  -> StoreActor upsert (single writer, dedupe index)
  -> coalesced history:changed (at most one per 80 ms)
  -> optional delete_expired on the cleanup cadence
```

Payload data is **never read** before the gate. Capture runs on its own thread
and is independent of whether the palette webview exists.

### Palette lifecycle (hybrid)

```
hotkey / tray / second instance
  -> PaletteManager::show
       warm: existing hidden webview
             capture paste target, position, show, focus, emit palette:open
       cold: worker thread builds the window with a bootstrap snapshot
             (top 50 summaries + theme + accent + openId)
             position, show, focus, emit palette:open
  -> frontend paints (bootstrap or palette:open), filters locally per keystroke
  -> 50 ms debounce -> history_search (FTS5, bm25, pinned first)
  -> hide (Esc / paste / settings)
       Auto: start the 10-minute idle countdown
       Never: destroy the webview immediately
  -> idle monitor (10 s tick) destroys the webview after the timeout; the core
     keeps capturing and the hotkey cold-creates the next palette
```

`palette_ready` settings:

- `auto` (default): pre-warmed while active, destroyed after
  `palette_idle_minutes` (5/10/30).
- `always`: never destroy.
- `never`: no pre-warm; create on hotkey, destroy on hide.

Because destroying the last window would normally exit Tauri, the run loop
handles `RunEvent::ExitRequested` and prevents exit unless a quit code is set
(tray **Quit**). Window creation must run off the main thread after the event
loop starts; cold opens and pre-warm-after-settings use worker threads.

The frontend reports its first paint through `palette_painted(openId)`; Rust
logs `palette_open_ms=<n> cold=<bool>`. Cold starts use the
`WebviewWindowBuilder` initialization script, so no IPC is on the first paint
path.

All webviews share `BROWSER_ARGS` (throttling prevention, `--disable-gpu`,
`--disable-breakpad`, and startup flags). WebView2 keys its environment by data
directory and requires matching options; a mismatch fails the second webview
with `ERROR_INVALID_STATE (0x8007139F)`.

### Paste

```
Enter
  -> paste_item(id)
  -> hide palette, wait until hidden (then after_hide)
  -> restore captured target (HWND / pid / X11 window)
  -> poll until the target is active (700 ms cap)
  -> arm suppression, write clipboard, settle 25 ms
  -> Ctrl+V / Cmd+V (enigo)
  -> touch_used(id)
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
6. **The palette never waits on SQLite to open.** Warm opens render cached
   summaries; cold opens receive a bootstrap snapshot before page scripts.
7. **No OS APIs in core; no Tauri in core or platform crates.**
8. **The core outlives the webview.** Destroying the palette never stops
   capture or the hotkey, and never exits the process.
9. **All webviews share one WebView2 environment configuration.**

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
| `palette_show` / `hide_palette` / `palette_painted` / `open_settings` | Window control + paint metrics |

Events:

| Event | Payload |
|---|---|
| `history:changed` | `{ revision }` (coalesced) |
| `palette:open` | `{ openId, cold }` |
| `settings:changed` | `{ theme, accent }` |

## Storage layout

```
<app_data>/com.copyfie.app/       (Windows: %APPDATA%)
  clipboard.db (+ -wal, -shm)     SQLite, schema v3, FTS5
  settings.json (+ .bak, .tmp)    theme, accent, retention, pause, hotkey,
                                  palette_ready, palette_idle_minutes
  thumbs/                         content-addressed thumbnails (M2)
  logs/
```

## Known debt

- The Rust summary cache (10k previews) is not implemented; the palette loads
  100 recent summaries and relies on FTS for the rest. Cold starts bootstrap 50.
- Dedupe updates `source_app_name` unconditionally, firing the FTS trigger on
  every recopy. Correct, but avoidable work.
- Settings tray check items are not synchronized when pause/autostart change in
  Settings.
- Wayland capability notice and copy-only UX are M2-12; capture is not started
  on Wayland.
- A native palette (GPUI/egui) with Tauri only for settings remains a possible
  later optimization if the warm WebView2 footprint becomes a competitive issue.
