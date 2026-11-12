# Copyfie — Liquid Glass Design Spec

Status: **proposal for approval** (MR-01). No UI code is changed until this is signed off.

Scope: palette, settings, tray, toasts, and the shared component/token layer. Stack stays
Tauri 2 + Svelte 5 + TypeScript + plain CSS. No UI framework, no Tailwind.

## 1. What we took from the references

We read the Liquid Glass generator source, the LogRocket and CSS-Tricks articles, and the
Apple developer/HIG articles (via their JSON documentation endpoints). The Figma community
file only returned its title (`iOS 26 Liquid Glass`); it is not readable as text, so nothing
is claimed about its contents. The generator's exported CSS/SVG structure is reproduced in
§5.3 and credited in code.

The principles we are adopting:

| Principle | What it means for Copyfie |
|---|---|
| **Glass is a functional layer, not a background.** | The floating palette shell, its controls, and (future) floating chrome may be glass. History text, previews, settings forms, and banners never are. |
| **Never glass on glass.** | Exactly one glass surface per window. Rows, chips, fields, the footer and toasts use tinted *solid* layers on top of the glass/well, not another `backdrop-filter`. |
| **Regular, not clear.** | Copyfie is text-heavy, so it uses the regular material. Clear (highly translucent, 35 % dimming over bright media) is explicitly out of scope: it would hurt legibility and adds no value over a desktop. |
| **Tint is semantic and sparing.** | Accent tint marks selection, the active segment, and the trial state. It is never a decorative wash over text. |
| **Three material layers.** | Highlight (top specular edge), shadow (outer + inner depth), illumination (a very low-alpha tint/gradient). These are CSS layers, not three blurs. |
| **Concentric shapes.** | Outer 24 px radius; every nested surface subtracts its inset (`r_inner = max(8, r_outer − padding)`). Rows 10 px, controls 12 px, pills 999 px. |
| **Group before you decorate.** | Filter chips share one segmented track; footer hints share one band; toolbar-like actions are grouped. No row of independent floating pills. |
| **Morph only for meaning.** | No ornamental shape-shifting. Selection and open/close use short opacity/transform transitions. Paste never waits for an animation. |
| **Legibility beats the effect.** | All text sits on a near-opaque `--glass-well` (88–92 %). The glass rim is decoration around it. AA is verified over bright and dark wallpapers. |
| **Accessibility is a variant, not an afterthought.** | Reduced transparency, increased contrast, reduced motion and a user toggle produce a solid high-contrast mode. |
| **Performance is part of the design.** | One large glass surface, no nested filters, no animated blur, and a lite mode that can drop to a solid surface. |

## 2. Surface map

```
┌──────────────────────────────────────────────────────────┐
│  glass shell (Tier 1 CSS / Tier 2 native, one surface)   │
│  ┌────────────────────────────────────────────────────┐  │
│  │ solid glass-well: search + filter track            │  │
│  ├────────────────────────────────────────────────────┤  │
│  │ solid glass-well: result list                      │  │
│  │   selected row = accent tint + border + left bar   │  │
│  ├────────────────────────────────────────────────────┤  │
│  │ solid glass-well: footer / trial line              │  │
│  └────────────────────────────────────────────────────┘  │
│  toast = solid chip, sits above footer, no blur          │
└──────────────────────────────────────────────────────────┘
```

| Surface | Material | Why |
|---|---|---|
| Palette window shell | **Glass** (Tier 1/2) | It is the floating control layer over the desktop. |
| Search field | Solid inset control | Text input must be crisp; it is inside the glass, not glass itself. |
| Result rows | Solid `--glass-well` | Dense text; blur behind text is forbidden. |
| Selected row | Accent-tinted solid + 1 px border + 3 px left bar | State must survive a blurred or disabled glass effect. |
| Filter chips / segmented control | One solid track, tinted selected pill | Grouped control; no per-chip blur. |
| Preview pane (M2+) | Solid matte | Content layer. Image controls are solid overlays. |
| Footer / trial banner | Solid strip inside the well | One line of chrome; trial line is a tinted solid strip. |
| Settings window | **Solid** document body; native titlebar | It is a long form, not a floating control layer. |
| Tray menu | **Native OS menu** | Cannot be styled safely; native is correct. |
| Toasts | Solid high-contrast chip | Legibility; it can overlap glass, so it must not be glass. |
| Hotkey-conflict overlay | Scrim + solid card | Avoids glass-on-glass; blocks the palette until resolved. |

## 3. Design tokens

Values live in `src/styles/tokens.css`; glass primitives live only in `src/styles/glass.css`.
Components may set layout/size, never raw blur, tint, or shadow values.

### 3.1 Shared scales

| Token | Value |
|---|---|
| `--font-ui` | `ui-sans-serif, system-ui, "Segoe UI Variable Text", "Segoe UI", Roboto, "Noto Sans", sans-serif` |
| `--font-mono` | `ui-monospace, "Cascadia Mono", Consolas, "SFMono-Regular", monospace` |
| Type | `0.6875 / 0.75 / 0.8125 / 0.9375 / 1.0625 / 1.25 rem` (11 / 12 / 13 / 15 / 17 / 20 px at 16 px root; meta, small, body, search, section, title); line-height 1.35–1.5 |
| Spacing | `4 / 8 / 12 / 16 / 20 / 24 / 32 px` |
| Radii | window `24`, panel `16`, control `12`, row `10`, small `8`, pill `999` |
| Row height | `52px` (up from 48; Apple's list guidance asks for more breathing room) |
| Motion | press `90ms`, hover/selection `140ms`, open `160ms`, close `100ms`; `--ease: cubic-bezier(.2,.8,.2,1)` |
| Blur | `--glass-blur: 22px`; lite `10px`; native `0` (native material supplies the blur) |
| Saturation | `1.5`; lite `1.2` |
| Focus | 2 px accent ring, 2 px offset, plus the selected-row border/bar |

### 3.2 Light and dark

| Token | Dark | Light |
|---|---|---|
| `--surface` (solid fallback) | `#0f0d14` | `#f7f5fb` |
| `--surface-2` | `#17131f` | `#ffffff` |
| `--glass-fill` | `linear-gradient(180deg, rgba(38,33,48,.66), rgba(16,13,22,.74))` | `linear-gradient(180deg, rgba(255,255,255,.70), rgba(249,247,255,.80))` |
| `--glass-well` | `rgba(14,12,19,.90)` | `rgba(255,255,255,.92)` |
| `--glass-tint` | `rgba(157,134,255,.08)` | `rgba(91,61,240,.05)` |
| `--glass-border` | `rgba(255,255,255,.16)` | `rgba(28,22,40,.12)` |
| `--glass-highlight` | `rgba(255,255,255,.42)` | `rgba(255,255,255,.90)` |
| `--glass-inner` | `rgba(0,0,0,.36)` | `rgba(28,22,40,.08)` |
| `--glass-shadow` | `0 28px 64px -24px rgba(0,0,0,.68), 0 2px 10px rgba(0,0,0,.38)` | `0 28px 64px -24px rgba(38,28,72,.26), 0 2px 10px rgba(38,28,72,.12)` |
| `--fg` | `#f6f3ff` | `#1a1526` |
| `--fg-muted` | `#c9c2d8` | `#514868` |
| `--fg-subtle` | `#a79fb8` | `#6d6485` |
| `--stroke` | `rgba(255,255,255,.14)` | `rgba(28,22,40,.12)` |
| `--row-hover` | `rgba(255,255,255,.06)` | `rgba(28,22,40,.05)` |
| `--row-selected` | `rgba(157,134,255,.22)` | `rgba(91,61,240,.12)` |
| `--row-selected-border` | `rgba(190,175,255,.72)` | `rgba(91,61,240,.55)` |
| `--control-bg` | `rgba(255,255,255,.07)` | `rgba(255,255,255,.66)` |
| `--control-border` | `rgba(255,255,255,.14)` | `rgba(28,22,40,.14)` |
| `--danger` | `#ff8b95` | `#b32633` |
| `--focus` | `#c8b8ff` | `#4a2ee0` |
| `--scrim` | `rgba(8,6,12,.72)` | `rgba(250,249,255,.75)` |

Muted text is intentionally stronger than today's `#9d95b3`; the effective-well contrast
target is **≥ 4.5:1 for all body/meta text** and **≥ 3:1 for borders, the selected bar and
icons**. `--fg-subtle` is only for non-essential decoration.

The accent stays user-configurable (`#9D86FF` default). We do **not** read the OS accent:
the cross-platform APIs are inconsistent and a wrong accent is worse than a chosen one. This
is recorded as a deliberate non-feature.

## 4. Composition rules

- The shell has **one** `backdrop-filter` at most. `html`, `body` and `#app` are transparent.
- The well is an opaque-enough layer **inside** the shell. It has no filter.
- Text is never placed directly on the low-opacity rim, a gradient, or a blurred row.
- Shadow is layered: one soft ambient (`0 2px 10px`) and one wide lift (`0 28px 64px`), plus
  an inset top highlight. No coloured glow.
- The specular highlight is a 1 px top inset plus a 1 px light border; it is visible in both
  themes. A 2 % SVG-noise data URI may add material grain; it costs one paint, not a filter.
- The palette stays 680 × 480 and is not resizable, so its glass area is bounded.
- The settings window stays native-decorated and solid; no CSS rounding of the OS window.
- The tray is untouched native code. A future "Reduce transparency" tray check item is allowed,
  but not required for v1.
- **Scroll edge:** if the footer ever overlays the list, it gets a short top scrim so rows fade
  under it. We do not stack a second glass header/footer on the shell.

### 4.1 Interaction behavior

Apple's `interactive` material responds to touch/pointer with a small light and shape response.
In Copyfie that becomes, using only compositor-friendly properties:

| State | Response |
|---|---|
| Hover (row, button, chip) | background tint lifts by one step, `140 ms`; no movement |
| Press | `transform: scale(.98)` + slightly brighter specular edge, `90 ms`; releases in `90 ms` |
| Selection | tint + border + left bar, `140 ms`; no layout shift, no blur change |
| Keyboard focus | 2 px `--focus` ring; identical information to the pointer state |
| Disabled | 45 % opacity, no press response, cursor default |

The material never "ripples", never animates blur, and never scales the whole palette on a
keystroke. `prefers-reduced-motion` removes the press scale and all transitions.

## 5. Rendering tiers (build in this order)

### 5.1 Tier 1 — CSS, works everywhere

Tauri window: `transparent(true)`, `decorations(false)`, `shadow(true)`, transparent page.
The shell uses:

```css
background: var(--glass-fill);
border: 1px solid var(--glass-border);
border-radius: var(--radius-window);
box-shadow:
  inset 0 1px 0 var(--glass-highlight),
  inset 0 -1px 0 var(--glass-inner),
  var(--glass-shadow);
backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-sat)) brightness(1.04);
```

**Important technical finding:** CSS `backdrop-filter` blurs only same-document content; it
cannot sample the desktop behind a transparent window (Tauri #15512/#2827; the same limitation
exists in Electron). So Tier 1 is a *faux-glass* surface: the translucency, gradient, noise,
specular edge and shadow sell the material, but the desktop stays sharp underneath. This is
acceptable as the everywhere baseline and is why Tier 2 exists. We still apply
`backdrop-filter` because it correctly frosts in-window content (for example the list under a
floating footer) and costs nothing when the backdrop is empty.

Fallback: `@supports not (backdrop-filter: blur(1px))` → solid `--surface`.

### 5.2 Tier 2 — native material (Windows, macOS)

Verified against the installed Tauri **2.12.5** source:

- `WebviewWindowBuilder::effects(WindowEffectsConfig)` (creation) and
  `WebviewWindow::set_effects(...)` (runtime) exist. Effects **require a transparent window**.
- `WindowEffect` includes `Mica`, `MicaDark`, `MicaLight`, `Acrylic`, `Blur`,
  `LiquidGlassRegular`, `LiquidGlassClear` (macOS 26+), and older macOS vibrancy materials
  (`HudWindow`, `Popover`, `Sidebar`, …). `WindowEffectsConfig` has `state`, `radius`, `color`,
  and `interactive` (macOS 27+).
- **Linux: unsupported.** Linux stays Tier 1/fallback.
- `macOSPrivateApi` is a no-op in Tauri 2.12.1+; no extra flag is needed for transparency.

Plan:

| OS | Material | Notes |
|---|---|---|
| Windows 11 22H2+ | **Acrylic** first; Mica as the subtle alternative | Acrylic blurs the desktop (floating palette). Mica only samples wallpaper; keep as a setting/fallback. Tauri docs warn Acrylic/Blur can be slow while *resizing/dragging* — the palette is fixed-size, so acceptable. |
| Windows 10 | Acrylic where supported; otherwise Tier 1 | Same call, with a runtime failure fallback. |
| macOS 26+ | `LiquidGlassRegular` with radius 24, state Active | The closest to the reference. `interactive` only if it costs nothing. |
| macOS 14–15 | `HudWindow`/`Popover` vibrancy | Regular material for a floating panel. |
| Linux | none | Tier 1 only; compositor transparency may itself fail → solid fallback. |

When a native material is active, set `data-material="native"`: the CSS blur token goes to
`0`, the fill becomes more transparent, and CSS keeps only tint, border, highlight, noise and
shadow. This avoids double-blur and double-cost. Apply the effect at window creation and
re-apply it after a hybrid cold open; measure that open path.

**Lifecycle checks:** transparent + frameless + native shadow + rounded corners + focus;
warm show/hide; cold re-create; settings window creation; pre-warm with the effect already
applied. If native shadow + transparency conflicts on Windows, drop `shadow(true)` and use the
CSS outer shadow (the window is transparent, so the CSS shadow is visible). No per-open cost is
allowed.

### 5.3 Tier 3 — refraction, WebView2 only

Source technique: the Liquid Glass generator's SVG (`feTurbulence` → `feGaussianBlur` →
`feDisplacementMap`, R/G channels), adapted with our own parameters and values.

```svg
<filter id="copyfie-refraction" x="-10%" y="-10%" width="120%" height="120%">
  <feTurbulence type="fractalNoise" baseFrequency="0.008" numOctaves="2" seed="7" result="noise"/>
  <feGaussianBlur in="noise" stdDeviation="1.5" result="soft"/>
  <feDisplacementMap in="SourceGraphic" in2="soft" scale="44"
                     xChannelSelector="R" yChannelSelector="G"/>
</filter>
```

Apply it to a **floating footer rail over the result list**, not to the desktop shell: the list
is same-document content, so the refraction is actually visible. The desktop shell keeps Tier 2
native material (or a filter-free Tier 1 fill). **Tier 3 does not nest inside a CSS-filtered
shell:** it is only enabled when the shell has no `backdrop-filter`, so the window still has at
most one CSS filter surface. A tiny chromatic-aberration pass (±2 scale, 6 % screen blend) is
optional and off by default; it is the first thing to drop if it costs frames.

Runtime gate, all required:

1. Windows + WebView2 (never macOS/Linux);
2. `CSS.supports("backdrop-filter", "url(#copyfie-refraction)")` (and the `-webkit-` form);
3. effects not set to Lite, transparency not reduced, contrast not increased, motion not reduced;
4. a 30-frame rAF scroll probe shows p95 frame time ≤ 20 ms; otherwise auto-disable for the
   session and log the fallback.

Because `CSS.supports` can accept a filter the compositor then ignores, the visual contract is
"no worse than the normal footer": the filter is an enhancement, and the footer remains legible
if it renders as a plain blur. It is **off by default** in v1; Settings exposes
"Experimental refraction (Windows)" only after the perf run proves it is free. No filter
animation, ever.

### 5.4 Fallback

Solid, high-contrast surface: `--surface` background, 1 px `--stroke`, no transparency, no
blur, no gradient. Triggered by reduced transparency (On/Auto+OS), `prefers-contrast: more`,
no backdrop support, Lite mode, or a failed native effect. Text and focus colours stay the
same tokens; only the material changes.

## 6. Accessibility contract (non-negotiable)

- **Contrast:** body/meta text ≥ 4.5:1, UI/borders/focus ≥ 3:1, over a white and a black
  wallpaper in both themes. The well at 88–92 % is the guarantee; text is never on the rim.
- **Reduced transparency:** `prefers-reduced-transparency: reduce` → solid. Settings adds
  **Reduce transparency: Auto / On / Off** (`auto` follows the OS; `on` forces solid; `off`
  keeps glass unless `prefers-contrast: more` is set). Persisted in `settings.json`, applied
  live to both windows, cached for pre-paint.
- **Increased contrast:** `prefers-contrast: more` → solid, 2 px borders, pure fg/bg, no
  accent-only state. The selected row still has its left bar and border.
- **Reduced motion:** `prefers-reduced-motion: reduce` → no open/close transform, no shimmer,
  no morph, no refraction. State changes are instant. Paste/search still never wait.
- **Selection/focus:** the selected row is identifiable without blur: 3 px accent left bar +
  1.5 px `--row-selected-border` inset + tinted fill. Keyboard focus stays in the search input;
  `aria-activedescendant` and the existing keymap are unchanged.
- **Screen readers:** keep combobox/listbox/option roles, `role="status"` toasts, labelled
  icon buttons (`aria-label`), and `aria-hidden` on decorative SVGs. The conflict overlay is a
  focus trap; Escape remains ignored while blocked.
- **Type/scaling:** type tokens use `rem`; the window is fixed-size, so 125 % OS scaling must
  be checked for clipping. No text is clipped to satisfy a radius.
- **Trial banner:** one line, solid, never a modal, never steals focus; it replaces the footer
  hint rather than stacking another bar.

## 7. Performance guardrails

Baseline to beat (Windows release, 10 004 items, M1-13): core **30.1 MB**, webview destroyed
**30.1 MB**, warm **342.8 MB avg / 345.5 MB max**, cold open **390 ms p95**, warm open
**37 ms p95**.

Rules:

- One `backdrop-filter` surface per window; no nested chains. Settings has none.
- No blur animation. Only `transform`/`opacity` transitions; selection uses background/border.
- The shell is fixed at 680 × 480; the list uses `contain: layout paint`.
- Tier 3 is gated and auto-disables on frame-time regression.
- **Lite glass mode:** Tier 1, blur 10 px, no noise, no Tier 3, simpler shadow. Auto when
  `navigator.deviceMemory ≤ 4` or `hardwareConcurrency ≤ 4`, or when the frame probe fails.
  Settings: **Visual effects: Auto / Full / Lite**.
- `--disable-gpu` is currently kept for the RAM/open gates. CSS blur may therefore rasterize
  on the CPU; this is the biggest visual-perf risk. Measure idle CPU and typing CPU with 10k
  items. If regressed, prefer native Tier 2 on Windows and set CSS blur to 0; if still
  regressed, default to Lite.
- **Before/after** measurement, same harness as M1-13: core RSS; total with the webview
  destroyed (idle); total with the webview warm; cold-open p95; warm-open p95; CPU/GPU while
  idle and while typing in search with 10k items. Run the "before" pass on the current UI
  first, then the redesigned UI. Record both in `docs/PERF.md` and `progress.md`.
- Gates: warm p95 **< 100 ms**, cold p95 **< 400 ms**, warm total **< 400 MB**, and no
  material increase in idle CPU/GPU. If a gate fails, simplify (fewer glass surfaces, lower
  blur, drop Tier 3) and report the trade-off.

## 8. Components and file layout

```
src/styles/tokens.css        # all values, light/dark, data-material/transparency/effects
src/styles/glass.css         # the only place with backdrop-filter / glass shadows
src/lib/glass/GlassPanel.svelte
src/lib/glass/GlassButton.svelte
src/lib/glass/GlassSegmented.svelte
src/lib/glass/GlassInput.svelte
src/lib/glass/GlassToast.svelte
src/lib/icons.ts             # Lucide paths, attributed; no SF Symbols/assets
```

- `GlassPanel`: `variant="shell|well|control"`, `tone="neutral|accent|danger"`, `padding`,
  `radius`, `as`. No component-local filter values.
- `GlassButton`: `variant="ghost|solid|accent"`, `size="sm|md"`, pressed/disabled, icon slot.
- `GlassSegmented`: one track, options, value, `onchange`, `ariaLabel`; filter mode uses
  `role="tablist"`/`tab`, theme mode uses `role="group"`/`aria-pressed`.
- `GlassInput`: inset solid field, leading icon, clear button, all aria props passthrough.
- `GlassToast`: solid chip, `role="status"`, no focus steal.
- Settings additions: `reduce_transparency` (`auto|on|off`), `visual_effects`
  (`auto|full|lite`), and an optional `refraction` flag. They are sanitized in Rust, cached for
  pre-paint, emitted with `settings:changed`, and applied to both windows without a restart.
- A CI script (`scripts/design/check_glass.mjs`) fails if `backdrop-filter` appears outside
  `glass.css`, if more than one occurrence exists per built page, or if raw `blur(` appears in
  component CSS.

## 9. Assets and legal

- Icons: **Lucide** (ISC). Copy only the paths used (search, x, pin, text, image, folder,
  settings, check, alert) into `src/lib/icons.ts` with the upstream notice; add Lucide to
  `THIRD_PARTY.md`/`NOTICE`.
- Fonts: system stack only. No SF Pro, no SF Symbols, no Apple artwork, no copied Figma assets.
- CSS/SVG technique credit: a code comment in `glass.css` cites
  `https://www.liquid-glass.pro/generator.html` for the turbulence/displacement structure.
- The app mark stays the existing "C" tile; app icons are unchanged by this redesign.

## 10. Acceptance checklist

- [ ] Tokens + `glass.css` + Glass* components exist; no one-off blur values.
- [ ] Palette and settings updated; tray untouched; toasts solid.
- [ ] Light/dark screenshots over bright/dark wallpapers in `docs/design/`.
- [ ] AA contrast check passes for both themes and both wallpaper extremes.
- [ ] Reduced transparency (Auto/On/Off), increased contrast, reduced motion verified.
- [ ] Selected row is obvious with glass disabled; keyboard behaviour unchanged and tests pass.
- [ ] One glass surface per window; no nested `backdrop-filter`.
- [ ] Tier 1 verified everywhere; Tier 2 Windows verified (macOS/Linux compiled-only if not
      available); Tier 3 gated and default-off until measured.
- [ ] Warm/cold open, RSS, idle/typing CPU recorded; gates still pass.
- [ ] Lite mode works and is documented.
- [ ] Attribution updated; no Apple assets or fonts.

## 11. Open risks

1. **Desktop blur:** Tier 1 cannot frost the desktop; Tier 2 is required. If Windows Acrylic
   conflicts with the transparent/shadow path, the palette will look like translucent frosted
   glass but the desktop will remain sharp. This is a known limitation, not a bug.
2. **`--disable-gpu` + CSS blur:** software rasterization could raise CPU. Native material or
   Lite mode is the mitigation.
3. **macOS/Linux untested here:** macOS vibrancy/Liquid Glass and Linux compositor behaviour
   must be treated as compile-verified, not runtime-verified, until measured on those OSes.
4. **WebView2 `backdrop-filter: url()`** support is inconsistent; Tier 3 is an enhancement and
   never a dependency.
5. **Transparent-window shadow/corners** on Windows may need `no_redirection_bitmap` or a
   CSS-only shadow. Verify before claiming DWM rounding.
