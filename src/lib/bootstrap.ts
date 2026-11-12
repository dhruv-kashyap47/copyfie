// Bootstrap injected by Rust when the palette webview is created.
//
// The cold path passes the top summaries, theme, and the open id inside the
// HTML initialization script, so the first paint does not wait for IPC.

import type { ItemSummary } from "./types";

export interface PaletteBootstrap {
  openId: number;
  cold: boolean;
  theme: string;
  accent: string;
  items: ItemSummary[];
}

export function readBootstrap(): PaletteBootstrap | null {
  const value = (
    window as unknown as { __COPYFIE_BOOTSTRAP__?: PaletteBootstrap }
  ).__COPYFIE_BOOTSTRAP__;

  if (!value || !Array.isArray(value.items)) {
    return null;
  }

  return value;
}
