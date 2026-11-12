# Copyfie — screenshots

Current set: **v2 (quiet macOS)** in `docs/design/v2/`. The top-level
`palette-*.png` / `settings-*.png` files are the previous v1 set, kept as history.

## v2 matrix (14 images)

| File pattern | Contents |
|---|---|
| `palette-{light,dark}-{white,black,busy}.png` | Palette, both themes, three wallpapers |
| `settings-{light,dark}-{white,black,busy}.png` | Settings, both themes, three wallpapers |
| `palette-selected.png` | Dark palette with row 3 selected (solid blue fill) |
| `palette-empty.png` | Light palette, no-match empty state |

Wallpapers: `docs/design/wallpapers/white.png`, `black.png`, and `busy.jpg`.
The busy image is an Unsplash sample (photo ID `1500530855697-b586d89ba3ee`) used
only for visual QA; the white/black plates are generated.

## Method

`scripts/design/capture.ps1` launches the real Windows release build and captures
the window plus an 18 px margin. A borderless full-work-area plate is shown behind
the palette; it is topmost for palette shots and normal for Settings (a topmost
plate would cover the non-topmost Settings window). The developer's desktop
wallpaper is never changed and `settings.json` is restored in `finally`.

```powershell
powershell -File scripts/design/capture.ps1 -Theme dark -Wallpaper busy `
  -Window palette -Out docs/design/v2/palette-dark-busy.png
```

The palette window is transparent and 680 × 480 with a 24 px transparent margin;
the visible shell is 632 × 432. The screenshot therefore includes the wallpaper
around the rounded shell. Settings is a solid native window.

## Edge check

The v1 square outline came from Windows DWM's 1 px border plus the native window
shadow around the rectangular window, not from the CSS shell. v2 sets
`shadow(false)`, DWM border color `DWMWA_COLOR_NONE`, corner preference
`DWMWCP_DONOTROUND`, and keeps the page transparent. The screenshots over white,
black and the busy photo show only the single rounded shell.

## Visual system

- **One accent:** system blue `#007AFF` light / `#0A84FF` dark.
- **Selection:** `#0060DF` light / `#0A63E6` dark with white text (5.62:1 / 5.33:1).
- **Type:** system SF stack on macOS, self-hosted Inter Latin subset on Windows/Linux.
- **Surfaces:** flat; one hairline edge; one soft shadow. No palette logo.

Screenshots are visual evidence, not pixel-diff fixtures; GPU/font rasterization
can vary slightly across machines.
