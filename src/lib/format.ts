// Display formatting. No date library: Unix ms in, compact strings out.

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** Compact relative time: "now", "5m", "3h", "2d", then a short date. */
export function relativeTime(capturedAt: number, now = Date.now()): string {
  const elapsed = Math.max(0, now - capturedAt);

  if (elapsed < MINUTE) {
    return "now";
  }

  if (elapsed < HOUR) {
    return `${Math.floor(elapsed / MINUTE)}m`;
  }

  if (elapsed < DAY) {
    return `${Math.floor(elapsed / HOUR)}h`;
  }

  if (elapsed < 7 * DAY) {
    return `${Math.floor(elapsed / DAY)}d`;
  }

  return new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(capturedAt);
}

/** Byte sizes for image/file rows and settings. */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
  }
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

const SOURCE_HUES = [214, 196, 174, 32, 158, 348];

/** Stable colored initial background for a source app name. */
export function sourceColor(name: string | null): string {
  const value = name?.trim() || "Unknown";
  let hash = 0;
  for (let index = 0; index < value.length; index += 1) {
    hash = (hash * 31 + value.charCodeAt(index)) >>> 0;
  }
  return `hsl(${SOURCE_HUES[hash % SOURCE_HUES.length]} 58% 60%)`;
}

export function sourceInitial(name: string | null): string {
  const value = name?.trim();
  return value ? value.charAt(0).toUpperCase() : "?";
}

/** True on Apple platforms, where the command key is the modifier. */
export function isMacPlatform(platform: string): boolean {
  return /Mac|iPhone|iPad/i.test(platform);
}

/** "⌘" on macOS, "Ctrl" elsewhere. */
export function modifierLabel(): string {
  if (typeof navigator === "undefined") {
    return "Ctrl";
  }
  return isMacPlatform(navigator.platform ?? "") ? "⌘" : "Ctrl";
}
