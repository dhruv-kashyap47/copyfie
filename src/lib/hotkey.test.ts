import { describe, expect, it } from "vitest";
import { chordFromEvent, formatAccelerator, isModifierKey } from "./hotkey";

function event(key: string, overrides: Partial<Parameters<typeof chordFromEvent>[0]> = {}) {
  return {
    key,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    ...overrides,
  };
}

describe("chordFromEvent", () => {
  it("builds CommandOrControl shortcuts for letters, digits and F-keys", () => {
    expect(chordFromEvent(event("v", { ctrlKey: true }))).toBe("CommandOrControl+V");
    expect(chordFromEvent(event("9", { metaKey: true }))).toBe("CommandOrControl+9");
    expect(chordFromEvent(event("F9", { ctrlKey: true, shiftKey: true }))).toBe(
      "CommandOrControl+Shift+F9",
    );
  });

  it("normalizes named keys", () => {
    expect(chordFromEvent(event(" ", { ctrlKey: true, shiftKey: true }))).toBe(
      "CommandOrControl+Shift+Space",
    );
    expect(chordFromEvent(event("ArrowUp", { altKey: true }))).toBe("Alt+Up");
  });

  it("requires a modifier so plain typing is never captured", () => {
    expect(chordFromEvent(event("a"))).toBeNull();
    expect(chordFromEvent(event(" "))).toBeNull();
  });

  it("ignores modifier-only presses", () => {
    expect(chordFromEvent(event("Control", { ctrlKey: true }))).toBeNull();
    expect(chordFromEvent(event("Shift", { shiftKey: true }))).toBeNull();
    expect(isModifierKey("Meta")).toBe(true);
  });

  it("rejects unsupported keys", () => {
    expect(chordFromEvent(event("F13", { ctrlKey: true }))).toBeNull();
    expect(chordFromEvent(event("Dead", { ctrlKey: true }))).toBeNull();
  });
});

describe("formatAccelerator", () => {
  it("formats for Windows/Linux", () => {
    expect(formatAccelerator("CommandOrControl+Shift+Space", false)).toBe("Ctrl+Shift+Space");
  });

  it("formats for macOS", () => {
    expect(formatAccelerator("CommandOrControl+Shift+Space", true)).toBe("⌘⇧Space");
  });
});
