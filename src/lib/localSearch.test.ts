import { describe, expect, it } from "vitest";
import { filterItems, matchesFilter, scoreMatch } from "./localSearch";
import type { ItemSummary } from "./types";

function item(
  id: number,
  preview: string,
  options: Partial<ItemSummary> = {},
): ItemSummary {
  return {
    id,
    kind: "text",
    preview,
    sourceAppName: null,
    isPinned: false,
    capturedAt: 1_767_225_600_000 - id * 1000,
    thumbName: null,
    byteSize: preview.length,
    width: null,
    height: null,
    ...options,
  };
}

describe("scoreMatch", () => {
  it("ranks prefix, word start, substring, then subsequence", () => {
    expect(scoreMatch("hello world", "hello")).toBe(0);
    expect(scoreMatch("say hello", "hello")).toBe(1);
    expect(scoreMatch("othello", "hello")).toBe(2);
    expect(scoreMatch("h-e-l-l-o", "hello")).toBe(3);
    expect(scoreMatch("nothing", "zzz")).toBeNull();
  });
});

describe("filterItems", () => {
  const items = [
    item(1, "alpha beta", { isPinned: true }),
    item(2, "beta gamma"),
    item(3, "alphabet soup"),
    item(4, "image data", { kind: "image" }),
    item(5, "C:\\notes.txt", { kind: "files" }),
  ];

  it("filters by kind and pins", () => {
    expect(matchesFilter(items[3], "image")).toBe(true);
    expect(matchesFilter(items[3], "text")).toBe(false);
    expect(matchesFilter(items[0], "pinned")).toBe(true);
    expect(matchesFilter(items[1], "pinned")).toBe(false);
    expect(filterItems(items, "", "image").map((entry) => entry.id)).toEqual([4]);
    expect(filterItems(items, "", "files").map((entry) => entry.id)).toEqual([5]);
    expect(filterItems(items, "", "pinned").map((entry) => entry.id)).toEqual([1]);
  });

  it("keeps pinned items first, then match quality, then recency", () => {
    expect(filterItems(items, "beta", "all").map((entry) => entry.id)).toEqual([1, 2]);
    expect(filterItems(items, "alpha", "all").map((entry) => entry.id)).toEqual([1, 3]);
    expect(filterItems(items, "", "text").map((entry) => entry.id)).toEqual([1, 2, 3]);
  });

  it("matches the source app name too", () => {
    const chrome = item(10, "unrelated", { sourceAppName: "Google Chrome" });
    const list = [chrome, item(11, "chrome notes")];
    expect(filterItems(list, "chrome", "all").map((entry) => entry.id)).toEqual([11, 10]);
  });

  it("returns an empty list when nothing matches", () => {
    expect(filterItems(items, "zzz", "all")).toEqual([]);
  });

  it("caps the visible rows", () => {
    const many = Array.from({ length: 40 }, (_, index) => item(index + 1, `row ${index}`));
    expect(filterItems(many, "", "all", 25)).toHaveLength(25);
  });

  it("stays fast on 10k items", () => {
    const many = Array.from({ length: 10_000 }, (_, index) =>
      item(index + 1, `entry number ${index} with some searchable text`),
    );

    const started = performance.now();
    const result = filterItems(many, "number 9999", "all");
    const elapsed = performance.now() - started;

    expect(result.length).toBeGreaterThan(0);
    expect(elapsed).toBeLessThan(100);
  });
});
