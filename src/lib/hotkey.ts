// Shortcut recording helpers. Pure functions so the recorder is unit-testable.

export interface ChordEvent {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta", "AltGraph", "CapsLock", "OS"]);

const KEY_NAMES: Record<string, string> = {
  " ": "Space",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Escape: "Escape",
  Enter: "Enter",
  Tab: "Tab",
  Backspace: "Backspace",
  Delete: "Delete",
  Insert: "Insert",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
};

export function isModifierKey(key: string): boolean {
  return MODIFIER_KEYS.has(key);
}

function normalizeKey(key: string): string | null {
  if (KEY_NAMES[key]) {
    return KEY_NAMES[key];
  }
  if (/^[a-zA-Z]$/.test(key)) {
    return key.toUpperCase();
  }
  if (/^[0-9]$/.test(key)) {
    return key;
  }
  if (/^F([1-9]|1[0-2])$/.test(key)) {
    return key;
  }
  return null;
}

/**
 * Convert a keydown event into a Tauri accelerator, or `null` when it is not a
 * usable shortcut (modifier only, no modifier, unsupported key).
 *
 * A shortcut must include Ctrl/Cmd or Alt, so plain typing can never become a
 * global shortcut.
 */
export function chordFromEvent(event: ChordEvent): string | null {
  if (isModifierKey(event.key)) {
    return null;
  }

  if (!event.ctrlKey && !event.metaKey && !event.altKey) {
    return null;
  }

  const key = normalizeKey(event.key);
  if (!key) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) {
    parts.push("CommandOrControl");
  }
  if (event.altKey) {
    parts.push("Alt");
  }
  if (event.shiftKey) {
    parts.push("Shift");
  }
  parts.push(key);
  return parts.join("+");
}

/** Human-readable accelerator for the current platform. */
export function formatAccelerator(accelerator: string, mac: boolean): string {
  return accelerator
    .replace(/^CommandOrControl/, mac ? "⌘" : "Ctrl")
    .replace(/\bAlt\b/g, mac ? "⌥" : "Alt")
    .replace(/\bShift\b/g, mac ? "⇧" : "Shift")
    .replace(/\+/g, mac ? "" : "+");
}
