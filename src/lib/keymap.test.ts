import { describe, expect, it } from "vitest";
import { mapKey, moveSelection, type KeyLike } from "./keymap";

function key(value: string, overrides: Partial<KeyLike> = {}): KeyLike {
  return {
    key: value,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    altKey: false,
    ...overrides,
  };
}

const emptyQuery = { query: "" };
const typedQuery = { query: "hello" };

describe("mapKey", () => {
  it("ignores plain typing so the search input receives it", () => {
    expect(mapKey(key("a"), emptyQuery)).toEqual({ type: "none" });
    expect(mapKey(key(" "), typedQuery)).toEqual({ type: "none" });
    expect(mapKey(key("Backspace"), typedQuery)).toEqual({ type: "none" });
  });

  it("navigates with arrows and wraps at the ends", () => {
    expect(mapKey(key("ArrowDown"), emptyQuery)).toEqual({ type: "move", delta: 1 });
    expect(mapKey(key("ArrowUp"), emptyQuery)).toEqual({ type: "move", delta: -1 });
    expect(mapKey(key("PageDown"), emptyQuery)).toEqual({ type: "move", delta: 8 });
    expect(mapKey(key("PageUp"), emptyQuery)).toEqual({ type: "move", delta: -8 });

    expect(moveSelection(2, 3, 1)).toBe(0);
    expect(moveSelection(0, 3, -1)).toBe(2);
    expect(moveSelection(1, 10, 8)).toBe(9);
    expect(moveSelection(8, 10, 8)).toBe(9);
    expect(moveSelection(3, 10, -8)).toBe(0);
    expect(moveSelection(0, 0, 1)).toBe(0);
  });

  it("supports home, end, enter, shift+enter, escape and tab", () => {
    expect(mapKey(key("Home"), emptyQuery)).toEqual({ type: "edge", edge: "start" });
    expect(mapKey(key("End"), emptyQuery)).toEqual({ type: "edge", edge: "end" });
    expect(mapKey(key("Enter"), typedQuery)).toEqual({ type: "select" });
    expect(mapKey(key("Enter", { shiftKey: true }), typedQuery)).toEqual({ type: "plain" });
    expect(mapKey(key("Escape"), typedQuery)).toEqual({ type: "close" });
    expect(mapKey(key("Tab"), typedQuery)).toEqual({ type: "filter", direction: 1 });
    expect(mapKey(key("Tab", { shiftKey: true }), typedQuery)).toEqual({
      type: "filter",
      direction: -1,
    });
  });

  it("quick-pastes with bare digits only when the query is empty", () => {
    expect(mapKey(key("1"), emptyQuery)).toEqual({ type: "quick", index: 0 });
    expect(mapKey(key("9"), emptyQuery)).toEqual({ type: "quick", index: 8 });
    expect(mapKey(key("0"), emptyQuery)).toEqual({ type: "none" });
    expect(mapKey(key("1"), typedQuery)).toEqual({ type: "none" });
  });

  it("quick-pastes with the modifier regardless of the query", () => {
    expect(mapKey(key("3", { ctrlKey: true }), typedQuery)).toEqual({ type: "quick", index: 2 });
    expect(mapKey(key("3", { metaKey: true }), typedQuery)).toEqual({ type: "quick", index: 2 });
  });

  it("maps pin, delete, block and settings shortcuts", () => {
    expect(mapKey(key("p", { ctrlKey: true }), typedQuery)).toEqual({ type: "pin" });
    expect(mapKey(key("P", { metaKey: true }), typedQuery)).toEqual({ type: "pin" });
    expect(mapKey(key("Backspace", { ctrlKey: true }), typedQuery)).toEqual({ type: "delete" });
    expect(mapKey(key("b", { ctrlKey: true }), typedQuery)).toEqual({ type: "block" });
    expect(mapKey(key(",", { ctrlKey: true }), typedQuery)).toEqual({ type: "settings" });
  });

  it("never intercepts keys during IME composition", () => {
    expect(mapKey(key("Enter", { isComposing: true }), typedQuery)).toEqual({ type: "none" });
    expect(mapKey(key("ArrowDown"), { query: "", isComposing: true })).toEqual({ type: "none" });
    expect(mapKey(key("1"), { query: "", isComposing: true })).toEqual({ type: "none" });
  });

  it("does not use Ctrl+N/Ctrl+P for navigation, because Ctrl+P is pin", () => {
    expect(mapKey(key("n", { ctrlKey: true }), typedQuery)).toEqual({ type: "none" });
    expect(mapKey(key("p", { ctrlKey: true }), typedQuery)).toEqual({ type: "pin" });
  });
});
