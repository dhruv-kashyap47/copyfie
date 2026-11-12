// Theme application and cache.
//
// The palette and settings window apply the cached theme before mounting
// (avoiding a flash), then reconcile with Rust. `system` removes `data-theme`
// so the `prefers-color-scheme` media query in tokens.css applies.

export type Theme = "system" | "light" | "dark";

export const DEFAULT_ACCENT = "#9D86FF";

const THEME_KEY = "copyfie.theme";
const ACCENT_KEY = "copyfie.accent";

export function normalizeTheme(value: string): Theme {
  return value === "light" || value === "dark" ? value : "system";
}

export function normalizeAccent(value: string): string {
  const trimmed = (value ?? "").trim();
  const hex = trimmed.startsWith("#") ? trimmed.slice(1) : trimmed;
  return /^[0-9a-fA-F]{6}$/.test(hex) ? `#${hex.toUpperCase()}` : DEFAULT_ACCENT;
}

export function applyTheme(theme: string, accent: string): void {
  if (typeof document === "undefined") {
    return;
  }

  const root = document.documentElement;
  const normalized = normalizeTheme(theme);

  if (normalized === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.dataset.theme = normalized;
  }

  root.style.setProperty("--accent", normalizeAccent(accent));
}

export function cacheTheme(theme: string, accent: string): void {
  try {
    localStorage.setItem(THEME_KEY, normalizeTheme(theme));
    localStorage.setItem(ACCENT_KEY, normalizeAccent(accent));
  } catch {
    // Cache failures are harmless; Rust remains the source of truth.
  }
}

export function cachedTheme(): { theme: Theme; accent: string } {
  try {
    return {
      theme: normalizeTheme(localStorage.getItem(THEME_KEY) ?? "system"),
      accent: normalizeAccent(localStorage.getItem(ACCENT_KEY) ?? DEFAULT_ACCENT),
    };
  } catch {
    return { theme: "system", accent: DEFAULT_ACCENT };
  }
}
