// Instant local filtering over the loaded summaries.
//
// This runs on every keystroke and must stay under a few milliseconds for the
// 10k-item cache target. It is intentionally allocation-light: lowercase the
// query once, use the already-lowercase haystack built per item, and avoid
// regexes in the hot loop.

import type { Filter, ItemSummary } from "./types";

export interface SearchableItem {
  item: ItemSummary;
  haystack: string;
}

export function matchesFilter(item: ItemSummary, filter: Filter): boolean {
  switch (filter) {
    case "all":
      return true;
    case "text":
      return item.kind === "text";
    case "image":
      return item.kind === "image";
    case "files":
      return item.kind === "files";
    case "pinned":
      return item.isPinned;
  }
}

const WORD_SEPARATORS = new Set(" \t\n\r([{\"'/\\-_.:;,");

function isWordStart(haystack: string, index: number): boolean {
  return index === 0 || WORD_SEPARATORS.has(haystack.charAt(index - 1));
}

/**
 * Lower score is better. Returns `null` when the item does not match.
 * 0 prefix, 1 word start, 2 substring, 3 subsequence.
 */
export function scoreMatch(haystack: string, query: string): number | null {
  const index = haystack.indexOf(query);
  if (index === 0) {
    return 0;
  }
  if (index > 0) {
    return isWordStart(haystack, index) ? 1 : 2;
  }

  // Subsequence: every query character appears in order.
  let position = 0;
  for (const character of query) {
    position = haystack.indexOf(character, position);
    if (position === -1) {
      return null;
    }
    position += 1;
  }
  return 3;
}

/**
 * Filter, rank, and cap the visible rows.
 *
 * Input order is the backend order (pinned first, then recency), so a stable
 * index tiebreaker preserves recency within equal match quality.
 */
export function filterItems(
  items: readonly ItemSummary[],
  query: string,
  filter: Filter,
  limit = 50,
): ItemSummary[] {
  const normalized = query.trim().toLowerCase();

  const candidates: { item: ItemSummary; score: number; index: number }[] = [];
  for (let index = 0; index < items.length; index += 1) {
    const item = items[index];
    if (!matchesFilter(item, filter)) {
      continue;
    }

    if (normalized.length === 0) {
      candidates.push({ item, score: 0, index });
      continue;
    }

    const haystack = `${item.preview}\n${item.sourceAppName ?? ""}`.toLowerCase();
    const score = scoreMatch(haystack, normalized);
    if (score !== null) {
      candidates.push({ item, score, index });
    }
  }

  candidates.sort((left, right) => {
    if (left.item.isPinned !== right.item.isPinned) {
      return left.item.isPinned ? -1 : 1;
    }
    if (left.score !== right.score) {
      return left.score - right.score;
    }
    return left.index - right.index;
  });

  return candidates.slice(0, limit).map((candidate) => candidate.item);
}
