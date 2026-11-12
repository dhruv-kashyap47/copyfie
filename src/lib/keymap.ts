// Keyboard state machine for the palette.
//
// Pure functions over a minimal event shape so the whole keymap is unit-tested
// without a DOM. `Ctrl+P` / `Ctrl+N` navigation from the plan was dropped
// because `Cmd/Ctrl+P` is pin/unpin; arrows, PageUp/Down, Home/End remain.

export interface KeyLike {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  isComposing?: boolean;
}

export interface KeyContext {
  query: string;
  isComposing?: boolean;
}

export type KeyAction =
  | { type: "none" }
  | { type: "move"; delta: number }
  | { type: "edge"; edge: "start" | "end" }
  | { type: "select" }
  | { type: "plain" }
  | { type: "quick"; index: number }
  | { type: "close" }
  | { type: "filter"; direction: 1 | -1 }
  | { type: "pin" }
  | { type: "delete" }
  | { type: "settings" }
  | { type: "block" };

const DIGIT = /^[1-9]$/;

export function mapKey(event: KeyLike, context: KeyContext): KeyAction {
  if (context.isComposing || event.isComposing) {
    return { type: "none" };
  }

  const modifier = event.ctrlKey || event.metaKey;
  const key = event.key;

  if (modifier && key === ",") {
    return { type: "settings" };
  }
  if (modifier && (key === "p" || key === "P")) {
    return { type: "pin" };
  }
  if (modifier && key === "Backspace") {
    return { type: "delete" };
  }
  if (modifier && (key === "b" || key === "B")) {
    return { type: "block" };
  }
  if (modifier && DIGIT.test(key)) {
    return { type: "quick", index: Number(key) - 1 };
  }

  switch (key) {
    case "ArrowDown":
      return { type: "move", delta: 1 };
    case "ArrowUp":
      return { type: "move", delta: -1 };
    case "PageDown":
      return { type: "move", delta: 8 };
    case "PageUp":
      return { type: "move", delta: -8 };
    case "Home":
      return { type: "edge", edge: "start" };
    case "End":
      return { type: "edge", edge: "end" };
    case "Enter":
      return event.shiftKey ? { type: "plain" } : { type: "select" };
    case "Escape":
      return { type: "close" };
    case "Tab":
      return { type: "filter", direction: event.shiftKey ? -1 : 1 };
    default:
      break;
  }

  // Bare digits quick-paste only while the query is empty, so typing "2026"
  // in the search box is never intercepted.
  if (!modifier && !event.altKey && DIGIT.test(key) && context.query.length === 0) {
    return { type: "quick", index: Number(key) - 1 };
  }

  return { type: "none" };
}

/** Arrow movement wraps; PageUp/PageDown clamp to the list bounds. */
export function moveSelection(current: number, count: number, delta: number): number {
  if (count <= 0) {
    return 0;
  }
  if (delta === 1) {
    return (current + 1) % count;
  }
  if (delta === -1) {
    return (current - 1 + count) % count;
  }
  return Math.min(count - 1, Math.max(0, current + delta));
}
