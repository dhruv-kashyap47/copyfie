// Shared UI types. Field names match the Rust `ItemSummary` serde shape.

export type ItemKind = "text" | "image" | "files";

export type Filter = "all" | "text" | "image" | "files" | "pinned";

export interface ItemSummary {
  id: number;
  kind: ItemKind;
  preview: string;
  sourceAppName: string | null;
  isPinned: boolean;
  capturedAt: number;
  thumbName: string | null;
  byteSize: number;
  width: number | null;
  height: number | null;
}

export interface StoreStats {
  total: number;
  text: number;
  images: number;
  files: number;
}

export const FILTERS: readonly Filter[] = ["all", "text", "image", "files", "pinned"];

export function kindForFilter(filter: Filter): ItemKind | null {
  switch (filter) {
    case "text":
      return "text";
    case "image":
      return "image";
    case "files":
      return "files";
    case "all":
    case "pinned":
      return null;
  }
}

export function filterLabel(filter: Filter): string {
  switch (filter) {
    case "all":
      return "All";
    case "text":
      return "Text";
    case "image":
      return "Images";
    case "files":
      return "Files";
    case "pinned":
      return "Pinned";
  }
}
