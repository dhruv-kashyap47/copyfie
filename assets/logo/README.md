# Logo sources and mapping

Source folder: `C:\Users\91947\Downloads\copyfie\logo` (copied, never modified).

| Source file | Format / size | Role | Shipped as |
|---|---|---|---|
| `app-icon.png` | PNG, 1254 × 1254 | App icon (exe, installer, taskbar) | `assets/logo/app-icon.png`; generated into `src-tauri/icons/` with `tauri icon` |
| `app-icon.png` | PNG, 1254 × 1254 | Tray icon on Windows/Linux | generated `src-tauri/icons/32x32.png` |
| `main-logo-1.png` | PNG, 2170 × 725 | Settings-page wordmark | derived light/dark assets below |
| `main-logo-1.svg` | SVG, 3.8 MB, embedded base64 raster | Source only — not shipped | — |

## Derived assets

- `copyfie-wordmark-dark.png` — 182 × 56 (2x of 91 × 28), solid `#1C1C1E`
  background matching the dark surface, original white/purple wordmark. **3,873 bytes.**
- `copyfie-wordmark-light.png` — 182 × 56, solid `#FFFFFF` background matching the
  light surface, “Copy” recoloured to `#1D1D1F`, purple “fie” preserved. **3,083 bytes.**
- `tray-template.png` — 44 × 44 monochrome silhouette derived from the app icon. Used only
  on macOS with `icon_as_template`. **433 bytes.** This is derived, not an original supplied
  asset; it needs a real macOS menu-bar review.

The wordmark PNGs were keyed from the supplied black-background PNG (luminance → alpha),
composited onto the exact `surface` colour so the header has no visible rectangle, then
quantized to 96 colours. The source files are untouched. High contrast hides the raster
wordmark and shows a text “Copyfie” brand instead, so the logo background never breaks the
pure black/white fallback.

The 3.8 MB SVG is a wrapper around a base64 raster and is not used in the bundle.

## Notes

- The app icon is a dark square with a bright “C” and a sparkle. At 32 px it remains legible;
  at 16 px it is close to the limit. If the Windows tray looks too detailed on a specific DPI,
  provide a dedicated simplified tray asset rather than editing the supplied icon.
- No Apple assets, SF Symbols, or SF font files are used.
