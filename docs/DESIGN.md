# Copyfie — Liquid Glass Design Spec (simplified)

Status: **approved direction** (MR-01). CSS-only refresh; no native materials, no SVG
refraction, no new settings, no component library.

## 1. Principles

Liquid Glass is a **functional layer**, not a background. Copyfie uses one glass-style shell
around one mostly solid content well. History text never sits on the translucent rim.

- **One shell, one well.** The shell is a gradient/alpha frame; every row, field, chip and
  footer lives on the well.
- **No `backdrop-filter`.** In a transparent Tauri window it cannot blur the desktop, and with
  `--disable-gpu` it would cost CPU for no visible gain. The material comes from gradient,
  alpha, a 1 px border, a bright top inset and layered shadows only.
- **No glass on glass.** The well, controls, selected row and toasts are solid/tinted layers.
- **Readability first.** Body text sits on a 92–96 % opaque well and meets 4.5:1. UI edges and
  the selected bar meet 3:1.
- **Group, don't scatter.** Filters share one track; hints share one footer; no floating pills.
- **Concentric radii.** Outer 24 px; every inset subtracts its padding (well 16, row 10,
  control 10, small 8).
- **Motion is optional and short.** Only `transform` and `opacity`, under 160 ms. Paste and
  search never wait for an animation.
- **Settings is a document.** Solid surface, native titlebar, no glass body. Tray stays native.
  Toasts stay solid.

## 2. Surface map

| Surface | Material |
|---|---|
| Palette shell | **Glass-style frame**: gradient + alpha, 1 px light border, bright top inset, two soft shadows |
| Content well (search, filters, list, footer) | **Solid** `--well-bg`, 1 px `--well-border` |
| Search field | Inset solid control |
| Result rows | Transparent on the well |
| Selected row | Accent tint + 1 px border + 3 px left bar (works with glass off) |
| Filter chips | One segmented track; selected pill gets the accent tint |
| Footer / future trial line | Solid band; trial line uses the accent tint |
| Toasts | Solid high-contrast chip |
| Hotkey-conflict overlay | Dim scrim + solid card |
| Settings window | Solid document; native titlebar |
| Tray menu | Native OS menu |

## 3. Tokens

All values live in `src/styles/tokens.css`; glass primitives live in `src/styles/glass.css`.
Components use classes, not raw blur/shadow values.

| Group | Tokens |
|---|---|
| Surfaces | `--shell-top`, `--shell-bottom`, `--shell-border`, `--shell-highlight`, `--shell-inner`, `--shell-shadow`, `--well-bg`, `--well-border`, `--well-shadow` |
| Controls | `--control-bg`, `--control-border`, `--control-hover`, `--field-bg`, `--track-bg` |
| Text | `--fg`, `--fg-muted`, `--fg-subtle`, `--accent`, `--danger`, `--focus` |
| Selection | `--row-hover`, `--row-selected`, `--row-selected-border`, `--selected-bar` |
| Shape | `--radius-shell: 24px`, `--radius-well: 16px`, `--radius-row: 10px`, `--radius-control: 10px`, `--radius-sm: 8px` |
| Spacing / type | `--space-1…6`, `--row-height: 52px`, rem type scale (11–20 px) |
| Motion | `--dur-press: 90ms`, `--dur-state: 120ms`, `--dur-in: 140ms`, `--ease: cubic-bezier(.2,.8,.2,1)` |

Light and dark values are defined with `:root[data-theme="light"]` and the
`prefers-color-scheme: light` fallback. The shell stays a light material in light mode and a
dark material in dark mode; the wallpaper never decides the theme.

**Curated accents per theme** (no accent picker):

- **Dark:** champagne `#D8B36A` on graphite — warm metallic light in the glass.
- **Light:** cobalt `#1E5BD7` on cool white — crisp, Apple-like blue.
- Focus ring, selected bar, pin icon and mark follow the theme accent. The stored `accent`
  field remains for settings-file compatibility but no longer overrides the theme.

Key values:

- Dark shell: `linear-gradient(160deg, rgba(43,52,66,.76), rgba(11,15,21,.88))`; well
  `rgba(10,13,18,.94)`.
- Light shell: `linear-gradient(160deg, rgba(255,255,255,.84), rgba(244,247,252,.92))`; well
  `rgba(255,255,255,.96)`.
- Border: `rgba(255,255,255,.22)` dark / `rgba(17,23,34,.14)` light.
- Top highlight: inset `0 1px 0` at `rgba(255,255,255,.52)` dark / `.95` light.
- Shadow: `0 24px 48px -24px` plus `0 4px 12px -6px`, low alpha.
- Selected row: theme-accent tint, `inset 0 0 0 1px` border, `inset 3px 0 0` bar.

## 4. CSS technique

```css
.glass-shell {
  background: linear-gradient(160deg, var(--shell-top), var(--shell-bottom));
  border: 1px solid var(--shell-border);
  border-radius: var(--radius-shell);
  box-shadow:
    inset 0 1px 0 var(--shell-highlight),
    inset 0 -1px 0 var(--shell-inner),
    var(--shell-shadow);
}
.glass-well {
  background: var(--well-bg);
  border: 1px solid var(--well-border);
  border-radius: var(--radius-well);
  box-shadow: var(--well-shadow);
}
```

The window is transparent and frameless, with an 8 px transparent margin inside the window so
the shell radius and CSS shadow are visible. Native window shadow stays enabled. No pseudo
elements are used for fake blur. The shell is only a frame: the well covers the content area.

## 5. Accessibility

CSS media queries only; there is no in-app visual-effects setting.

- `prefers-reduced-transparency: reduce` → shell and well become fully opaque, shadows reduce.
- `prefers-contrast: more` → solid surface, pure foreground/background, 2 px borders, no
  alpha, no gradients. Selected row keeps its border and left bar.
- `prefers-reduced-motion: reduce` → all animations/transitions removed.
- Contrast targets: body/meta text ≥ 4.5:1; UI borders, focus ring and selected bar ≥ 3:1.
- The selected row is identifiable without glass: tint + border + 3 px bar.
- Keyboard behaviour, ARIA roles and focus handling are unchanged.

## 6. Motion

- Press: `transform: scale(.98)`, 90 ms.
- State changes: instant colour/border changes, no transition (colour is not animated).
- No shell entrance animation: the palette appears immediately on cold and warm opens.
- Toast: opacity + 4 px translate, 140 ms.
- No shimmer; the loading skeleton uses an opacity pulse, disabled by reduced motion.

## 7. Icons and assets

- **Geist Sans** for the UI, self-hosted variable WOFF2 (SIL OFL 1.1). This is the premium
  typeface; system fonts are fallback only. Keycap hints use the platform monospace stack.
- Lucide paths only, copied into `src/lib/icons.ts` (search, x, text, image, files, pin).
  Lucide is ISC; attribution is added to `THIRD_PARTY.md`/`NOTICE`.
- No SF Pro, SF Symbols, Apple artwork or Figma assets.
- No new npm dependencies; the font files are copied assets, not a package.

## 8. Performance

Baseline (M1-13, Windows, 10k items): warm open **37 ms p95**, cold open **390 ms p95**, warm
RSS **342.8 MB avg / 345.5 MB max**, core **30.1 MB**.

After the refresh, re-measure warm/cold open p95, warm total RSS and idle CPU. Gates: warm
p95 **< 100 ms**, cold p95 **< 400 ms**, no meaningful RSS/CPU increase. No `backdrop-filter`,
one shell, one well, no filter animation. If a gate fails, simplify shadows/gradients first.

Measured after: warm **47 ms p95** (36 ms on a second pass), cold **394 ms p95** (395 ms on a
second pass), warm RSS **345.8 MB avg / 353.9 MB max**, idle CPU **0.08%** hidden / **0.64%**
visible (of one core). The first cold pass with font preloads was 405 ms; removing the preloads
and dropping the second font/shell fade restored the gate. Details in `docs/PERF.md`.

## 9. Out of scope (cut)

Native window materials (Acrylic/Mica/vibrancy), SVG displacement/refraction, frame-time
probes, lite-mode auto-detection, the "Visual effects", "Reduce transparency" and "Refraction"
settings, Rust/settings changes, CI glass checks, and a `Glass*` component library. The existing
Svelte components are restyled in place.

## 10. Acceptance

- [ ] `tokens.css` + `glass.css`; no `backdrop-filter`; no raw one-off blur values.
- [ ] Palette shell/well, search, filters, selected row, footer and toast updated.
- [ ] Settings solid; tray native; Lucide icons attributed; Geist Sans self-hosted.
- [ ] Reduced transparency/contrast/motion media queries verified.
- [ ] Light/dark screenshots over bright/dark wallpapers in `docs/design/`.
- [ ] Warm/cold open, RSS and idle CPU measured; gates pass.
- [ ] `npm run check`, `npm test`, `npm run build`, Rust checks still clean.
