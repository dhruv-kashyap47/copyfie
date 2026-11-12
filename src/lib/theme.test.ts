import { describe, expect, it } from "vitest";
import { DEFAULT_ACCENT, normalizeAccent, normalizeTheme } from "./theme";

describe("normalizeTheme", () => {
  it("accepts the three supported themes", () => {
    expect(normalizeTheme("system")).toBe("system");
    expect(normalizeTheme("light")).toBe("light");
    expect(normalizeTheme("dark")).toBe("dark");
  });

  it("falls back to system for anything else", () => {
    expect(normalizeTheme("neon")).toBe("system");
    expect(normalizeTheme("")).toBe("system");
    expect(normalizeTheme("DARK")).toBe("system");
  });
});

describe("normalizeAccent", () => {
  it("normalizes hex colors", () => {
    expect(normalizeAccent("#1e5bd7")).toBe("#1E5BD7");
    expect(normalizeAccent("1E5BD7")).toBe("#1E5BD7");
    expect(normalizeAccent("  #aabbcc  ")).toBe("#AABBCC");
  });

  it("falls back to the default accent", () => {
    expect(normalizeAccent("not-a-color")).toBe(DEFAULT_ACCENT);
    expect(normalizeAccent("#12345")).toBe(DEFAULT_ACCENT);
    expect(normalizeAccent("")).toBe(DEFAULT_ACCENT);
  });
});
