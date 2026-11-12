import { describe, expect, it } from "vitest";
import {
  formatBytes,
  isMacPlatform,
  modifierLabel,
  relativeTime,
  sourceColor,
  sourceInitial,
} from "./format";

describe("relativeTime", () => {
  const now = 1_767_225_600_000;

  it("uses compact units", () => {
    expect(relativeTime(now, now)).toBe("now");
    expect(relativeTime(now - 30_000, now)).toBe("now");
    expect(relativeTime(now - 5 * 60_000, now)).toBe("5m");
    expect(relativeTime(now - 3 * 3_600_000, now)).toBe("3h");
    expect(relativeTime(now - 2 * 86_400_000, now)).toBe("2d");
  });

  it("falls back to a short date after a week", () => {
    const label = relativeTime(now - 30 * 86_400_000, now);
    expect(label).not.toMatch(/d$/);
    expect(label.length).toBeGreaterThan(3);
  });

  it("never reports negative elapsed time", () => {
    expect(relativeTime(now + 10_000, now)).toBe("now");
  });
});

describe("formatBytes", () => {
  it("formats bytes, KB, MB and GB", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2.0 KB");
    expect(formatBytes(20 * 1024)).toBe("20 KB");
    expect(formatBytes(3 * 1024 * 1024)).toBe("3.0 MB");
    expect(formatBytes(2 * 1024 * 1024 * 1024)).toBe("2.0 GB");
  });
});

describe("source identity", () => {
  it("derives a stable color and initial", () => {
    expect(sourceColor("Notepad")).toBe(sourceColor("Notepad"));
    expect(sourceColor("Notepad")).toMatch(/^hsl\(/);
    expect(sourceColor(null)).toBe(sourceColor("Unknown"));
    expect(sourceInitial("notepad")).toBe("N");
    expect(sourceInitial("  chrome")).toBe("C");
    expect(sourceInitial(null)).toBe("?");
    expect(sourceInitial("   ")).toBe("?");
  });
});

describe("platform modifier", () => {
  it("detects Apple platforms", () => {
    expect(isMacPlatform("MacIntel")).toBe(true);
    expect(isMacPlatform("Win32")).toBe(false);
    expect(isMacPlatform("Linux x86_64")).toBe(false);
  });

  it("returns a label", () => {
    expect(["⌘", "Ctrl"]).toContain(modifierLabel());
  });
});
