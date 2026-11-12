// Typed IPC wrappers. Every command name and payload lives here so the
// components never call `invoke` directly.

import { invoke } from "@tauri-apps/api/core";
import type { ItemKind, ItemSummary, StoreStats } from "./types";

export interface ListOptions {
  limit?: number;
  offset?: number;
  kind?: ItemKind | null;
  pinnedOnly?: boolean;
}

export function listHistory(options: ListOptions = {}): Promise<ItemSummary[]> {
  return invoke<ItemSummary[]>("history_list", {
    limit: options.limit ?? 100,
    offset: options.offset ?? 0,
    kind: options.kind ?? null,
    pinnedOnly: options.pinnedOnly ?? false,
  });
}

export function searchHistory(
  query: string,
  kind: ItemKind | null = null,
  limit = 100,
): Promise<ItemSummary[]> {
  return invoke<ItemSummary[]>("history_search", { query, kind, limit });
}

export function pinItem(id: number, pinned: boolean): Promise<boolean> {
  return invoke<boolean>("history_pin", { id, pinned });
}

export function deleteItem(id: number): Promise<boolean> {
  return invoke<boolean>("history_delete", { id });
}

export function clearHistory(keepPinned: boolean): Promise<number> {
  return invoke<number>("history_clear", { keepPinned });
}

export function copyItem(id: number): Promise<void> {
  return invoke<void>("copy_item", { id });
}

export interface PasteReport {
  status: "pasted" | "copied";
  reason: string | null;
}

export function pasteItem(id: number): Promise<PasteReport> {
  return invoke<PasteReport>("paste_item", { id });
}

export function hidePalette(): Promise<void> {
  return invoke<void>("hide_palette");
}

export function storeStats(): Promise<StoreStats> {
  return invoke<StoreStats>("store_stats");
}
