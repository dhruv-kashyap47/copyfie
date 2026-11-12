# Copyfie — quiet macOS design spec (v2)

Status: implemented (MR-05…MR-08). Svelte + plain CSS only. No new runtime dependencies,
no `backdrop-filter`, no gradients, no palette logo, one system-blue accent.

## 1. Window edge fix

**Root cause found:** the thin square outline was not the CSS shell. It was Windows DWM
drawing a 1 px border around the rectangular 680 × 480 transparent window, plus the native
window shadow (`shadow(true)`). The old shell also had an inset highlight and an inner border,
which read as a second ring inside the CSS edge.

**Fix, in order:**

1. `transparent(true)`, `decorations(false)`, **`shadow(false)`**.
2. WebView/page backgrounds transparent (`html`, `body`, `#app`), no full-window paint.
3. 24 px transparent margin inside the window; the CSS shell is 632 × 432.
4. `DwmSetWindowAttribute(DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE)` removes the DWM border.
5. `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND)` stops DWM
   rounding the window rectangle; the CSS shell owns the only radius.
6. Removed the redundant transparent `background_color` call; kept `no_redirection_bitmap`
   to avoid the white flash.
7. One CSS hairline edge, no inset highlight, no inner border. One shadow.

Verified over white, black, and a busy photo wallpaper: no rectangle, no clipped corner,
single rounded shell. The Settings window remains a normal solid native window.

## 2. Tokens

Flat colour, one accent, one hairline, one shadow. All values in `src/styles/tokens.css`.

| Token | Light | Dark |
|---|---|---|
| `--surface` | `#FFFFFF` | `#1C1C1E` |
| `--surface-2` | `#F2F2F7` | `#2C2C2E` |
| `--text` | `#1D1D1F` | `#F5F5F7` |
| `--text-2` | `#6E6E73` (AA-tuned) | `rgba(235,235,245,.62)` |
| `--separator` | `rgba(60,60,67,.12)` | `rgba(255,255,255,.08)` |
| `--edge` | `rgba(0,0,0,.10)` | `rgba(255,255,255,.12)` |
| `--accent` | `#007AFF` | `#0A84FF` |
| `--selection-bg` / `--selection-fg` | `#0060DF` / `#FFFFFF` | `#0A63E6` / `#FFFFFF` |
| `--focus-ring` | `rgba(0,122,255,.40)` | `rgba(10,132,255,.40)` |
| `--row-hover` | `rgba(60,60,67,.06)` | `rgba(255,255,255,.06)` |
| `--shadow-shell` | `0 20px 48px -20px rgba(0,0,0,.20), 0 4px 12px -6px rgba(0,0,0,.10)` | `0 20px 48px -20px rgba(0,0,0,.58), 0 4px 12px -6px rgba(0,0,0,.30)` |
| Radii | shell `14`, well/search `10`, row `8`, keycap `5` | same |

Selection was darkened from the pure system blue until white text passes AA:
light `#0060DF` = **5.62:1**, dark `#0A63E6` = **5.33:1**. Text is not reduced or faded.
`--text-2` is `#6E6E73` in light rather than 60 % alpha, to keep body/meta text above 4.5:1.

Switches, selected segments, scrollbar and toasts use the same token set; danger red is only
used for destructive actions. No gradients, glows, or multi-layer borders.

The shadow is slightly tighter than the first draft (`48px -20px` instead of `60px`) to keep
software-rendered (`--disable-gpu`) paint cheap; it still reads as the same soft lift.

## 3. Typography

Stack: `-apple-system, BlinkMacSystemFont, "SF Pro Text", "Inter", "Segoe UI Variable Text",
system-ui, sans-serif`. macOS gets the system SF stack; Windows/Linux get the bundled
**Inter** Latin-subset variable WOFF2 (33 KB, SIL OFL 1.1). No Geist, no monospace font files.

| Role | Size / weight |
|---|---|
| Search | 15 / 400 |
| Row title | 13 / 500, single line, ellipsis, `-0.01em` |
| Meta | 11 / 400, `--text-2`, tabular numbers |
| Filter labels | 12 / 500 |
| Footer hints | 11 / 400 |
| Settings section labels | 12 / 500, `--text-2` |
| Settings rows | 13 / 400 |

Keycaps are small sans-serif chips (radius 5), not monospace. macOS shows `↵`, `⌘P`, `⎋`;
Windows/Linux show `Enter`, `Ctrl+P`, `Esc`.

## 4. Palette layout

- **No logo.** Header is one full-width search field: 40 px high, `surface-2`, no border,
  16 px Lucide search icon, placeholder “Search”, focus = 2 px accent-at-40 % ring.
- **Filters:** one lightweight segmented track (`surface-2`, radius 8, no outer border).
  Selected pill = `surface` + one soft 1 px shadow. All / Text / Images / Files / Pinned.
- **Rows:** 44 px, no dividers, 2 px gap. Text rows have no icon; image/file rows use a
  16 px Lucide stroke icon or a 34 × 28 rounded-8 thumbnail. Title is one line; meta is
  `App · 5h` (type only for images/files). Pin is a small accent pin.
  Index hints `1–9` are plain `--text-2` digits, no boxes.
- **Selected row:** solid `--selection-bg` with white title/meta/icon/digit. No glass, no blur,
  no left bar. Hover is a 6 % neutral fill.
- **Footer:** one line, hints left, count right, `--text-2`, hairline top only.
- **Scrollbar:** thin overlay, thumb only on list hover.
- **Empty state:** one centered line + one-line hint, both `--text-2`.

## 5. Settings

Solid `surface`, single column, max-width 480. Inset grouped list:

- Section label 12/500 `--text-2`.
- Group = `surface-2`, radius 10, hairline separators between rows, 44 px rows.
- iOS-style toggles (44 × 26, accent when on, knob-only transform).
- Segmented controls for theme and palette readiness; plain selects for longer choices;
  plain recorder for the hotkey; red text button only for Clear history.
- Header: settings wordmark at 28 px height (2x asset 56 px) plus version in `--text-2`.
  High contrast replaces the raster wordmark with a text “Copyfie” fallback.
- Conflict card, toasts and dialogs use the same solid surface, one hairline, radius 12.

## 6. Logo mapping

Source folder `logo/` copied into `assets/logo/` (never edited):

| Source | Size | Role | Shipped/derived |
|---|---|---|---|
| `app-icon.png` | 1254 × 1254 | App icon (exe/installer/taskbar), tray on Windows/Linux | `assets/logo/app-icon.png`; `tauri icon` → `src-tauri/icons/` |
| `main-logo-1.png` | 2170 × 725 | Settings wordmark | `copyfie-wordmark-light.png` 182 × 56, 3.1 KB; `copyfie-wordmark-dark.png` 182 × 56, 3.9 KB; solid surface-colour backgrounds, text fallback in high contrast |
| `main-logo-1.svg` | 3.8 MB base64 raster wrapper | Source only | not shipped |
| `app-icon.png` | 1254 × 1254 | macOS tray template (derived) | `tray-template.png` 44 × 44, 433 B, `icon_as_template` |

The supplied set has **no dedicated tray asset**. Windows/Linux use the generated 32 px app
icon (the “C” remains legible); macOS uses a derived monochrome silhouette and must be
reviewed on a real menu bar. See `assets/logo/README.md`.

## 7. Accessibility

- Body/meta text ≥ 4.5:1; selected white-on-blue **5.62:1 light / 5.33:1 dark**; UI edges and
  focus rings remain distinct.
- `prefers-contrast: more` → pure black/white surfaces, 2 px edges, no shadows, text brand
  fallback. Verified in Chromium.
- `prefers-reduced-motion: reduce` → no toast/skeleton/press motion. Verified.
- `prefers-reduced-transparency: reduce` → surfaces are already opaque; the query removes
  shadows where the engine supports it. (Chromium’s emulation reports the feature but did not
  apply the query in the test harness; actual OS behavior needs a macOS/Windows check.)
- Keyboard behavior, listbox/option roles, `aria-activedescendant`, and focus handling are
  unchanged. The selected row is visible without any effect.

## 8. Performance (Windows, release, 10,004 items)

| Metric | v1 (before) | v2 | Change |
|---|---:|---:|---:|
| Warm-open p95 | 47 ms (second pass 36) | **37 ms** (24 samples, max 50) | better |
| Cold-open p95 | 394 ms (second pass 395) | **383 ms** (20 samples, max 383) | better; **above 380 warning** |
| Warm RSS | 345.8 / 353.9 MB avg/max | **317.7 / 322.4 MB** | −28 MB avg |
| Idle webview-destroyed RSS | 30.1 MB | **25.9 MB** | −4.2 MB |
| Idle CPU hidden / visible | 0.08 / 0.64 % one core | **0.03 / 0.31 %** | lower |
| JS+CSS gzipped | 35.5 KB | **42.6 KB** | +7.1 KB (settings wordmark PNGs inlined); still < 50 KB |

Cold open was borderline: early 10-run passes produced single outliers (415–449 ms). Reducing
the cold bootstrap from 50 to 20 summaries and removing the redundant transparent
`background_color` call brought p50 to **352.5 ms** and a stable 20-run p95 to **383 ms**.
The gate (<400 ms) passes; the 380 ms watch line is flagged.

## 9. Unverified / caveats

- macOS and Linux not run here. DWM border removal is Windows-only; macOS/Linux still need a
  real transparency, native titlebar, tray template and font-smoothing pass.
- Windows 10 (DWM attributes unsupported) and mixed-DPI/multi-monitor were not exercised.
- The busy wallpaper is an Unsplash photo used for visual QA only.
- The macOS tray template is derived from the app icon; it is not an original supplied asset.
- The selected fill is intentionally darker than the UI accent to protect text contrast.
