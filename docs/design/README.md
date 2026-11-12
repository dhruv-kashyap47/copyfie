# Copyfie — Liquid Glass screenshots

Captured from the real Windows release build (`target/release/copyfie.exe`) with
the palette window at 680 × 480 and the settings window at 560 × 640.

- `palette-light-bright.png` / `palette-light-dark.png`
- `palette-dark-bright.png` / `palette-dark-dark.png`
- `settings-light.png` / `settings-dark.png`
- `wallpapers/bright.png`, `wallpapers/dark.png` — deterministic wallpaper plates

## Method

`scripts/design/capture.ps1` launches the real app and captures the window plus
a small margin. The wallpaper is shown by a borderless full-screen window behind
the palette (`wallpaper_window.ps1`), so the screenshots are reproducible without
changing the developer's desktop wallpaper. The script restores `settings.json`
when it finishes.

```powershell
powershell -File scripts/design/capture.ps1 -Theme dark -Wallpaper bright `
  -Window palette -Out docs/design/palette-dark-bright.png
```

The palette is transparent and frameless; the screenshot therefore includes the
wallpaper behind its rounded corners. Settings is a solid native window.

Accents: **champagne** (`#D8B36A`) in dark, **cobalt** (`#1E5BD7`) in light.
Typeface: **Geist Sans** (SIL OFL 1.1), self-hosted; the tiny keycap glyphs use the
platform monospace stack.

The screenshots are visual evidence, not pixel-diff fixtures; GPU/font
rasterization can vary slightly across machines.
