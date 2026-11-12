<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  import {
    deleteItem,
    hidePalette,
    hotkeyStatus as fetchHotkeyStatus,
    listHistory,
    openSettings,
    palettePainted,
    pasteItem,
    pinItem,
    searchHistory,
    settingsGet,
    type HotkeyStatus,
  } from "../lib/ipc";
  import { readBootstrap } from "../lib/bootstrap";
  import { filterItems } from "../lib/localSearch";
  import { mapKey, moveSelection } from "../lib/keymap";
  import { modifierLabel } from "../lib/format";
  import { applyTheme, cacheTheme } from "../lib/theme";
  import { FILTERS, kindForFilter, type Filter, type ItemSummary } from "../lib/types";

  import FilterBar from "./FilterBar.svelte";
  import Footer from "./Footer.svelte";
  import HotkeySetup from "./HotkeySetup.svelte";
  import ResultList from "./ResultList.svelte";
  import SearchBar from "./SearchBar.svelte";

  const RECENT_LIMIT = 100;
  const SEARCH_DEBOUNCE_MS = 50;
  const TOAST_MS = 1600;

  let query = $state("");
  let filter = $state<Filter>("all");
  const boot = readBootstrap();
  let recent = $state<ItemSummary[]>(boot?.items ?? []);
  let fts = $state<ItemSummary[] | null>(null);
  let selected = $state(0);
  let status = $state<"loading" | "idle" | "error">(boot ? "idle" : "loading");
  let errorMessage = $state<string | null>(null);
  let toast = $state<string | null>(null);
  let composing = $state(false);
  let hotkey = $state<HotkeyStatus | null>(null);
  let hotkeyBlocked = $state(false);

  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  let searchRequest = 0;
  let recentRequest = 0;

  const visible = $derived(fts ?? filterItems(recent, query, filter));
  const showShortcuts = $derived(query.trim().length === 0);
  const activeDescendant = $derived(
    visible[selected] ? `row-${visible[selected].id}` : undefined,
  );
  const modifier = modifierLabel();

  function describe(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }

  function schedulePainted(openId: number) {
    if (openId <= 0) {
      return;
    }
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        void palettePainted(openId).catch(() => {
          // Metrics are best-effort; never affect the palette.
        });
      });
    });
  }

  function showToast(message: string) {
    toast = message;
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
    toastTimer = setTimeout(() => {
      toast = null;
    }, TOAST_MS);
  }

  async function refresh() {
    const request = ++recentRequest;
    try {
      const list = await listHistory({ limit: RECENT_LIMIT });
      if (request !== recentRequest) {
        return;
      }
      recent = list;
      status = "idle";
      errorMessage = null;
    } catch (cause) {
      if (request !== recentRequest) {
        return;
      }
      status = "error";
      errorMessage = describe(cause);
    }
  }

  async function refreshHotkey() {
    try {
      const status = await fetchHotkeyStatus();
      hotkey = status;
      // Only a finished, failed registration is a conflict. Before the startup
      // attempt, "not registered" is expected.
      hotkeyBlocked = status.initialized && !status.registered;
    } catch {
      // Hotkey status is advisory here; never block the palette on it.
    }
  }

  async function refreshTheme() {
    try {
      const settings = await settingsGet();
      applyTheme(settings.theme, settings.accent);
      cacheTheme(settings.theme, settings.accent);
    } catch {
      // The cached theme is already painted; this is a reconciliation pass.
    }
  }

  async function runSearch() {
    const term = query.trim();
    if (term.length === 0) {
      return;
    }

    const request = ++searchRequest;
    try {
      const found = await searchHistory(term, kindForFilter(filter), 100);
      if (request !== searchRequest) {
        return;
      }
      fts = found.length > 0 ? found : null;
      selected = 0;
    } catch {
      // Search failures are non-fatal: keep the instant local results.
    }
  }

  // Instant local results, then an authoritative FTS pass after a short idle.
  $effect(() => {
    const term = query;
    const activeFilter = filter;
    const request = ++searchRequest;

    fts = null;
    selected = 0;

    if (term.trim().length === 0) {
      return;
    }

    const timer = setTimeout(() => {
      void (async () => {
        try {
          const found = await searchHistory(term, kindForFilter(activeFilter), 100);
          if (request !== searchRequest) {
            return;
          }
          fts = found.length > 0 ? found : null;
          selected = 0;
        } catch {
          // Keep local matches.
        }
      })();
    }, SEARCH_DEBOUNCE_MS);

    return () => clearTimeout(timer);
  });

  // Clamp selection whenever the visible set changes.
  $effect(() => {
    const count = visible.length;
    if (selected >= count) {
      selected = Math.max(0, count - 1);
    }
  });

  onMount(() => {
    if (boot) {
      applyTheme(boot.theme, boot.accent);
      cacheTheme(boot.theme, boot.accent);
    }

    void refresh();
    void refreshHotkey();
    void refreshTheme();

    if (boot && boot.openId > 0) {
      // Cold start: the palette:open event was emitted before this webview
      // finished loading, so paint is reported from the bootstrap path.
      schedulePainted(boot.openId);
      requestAnimationFrame(() => {
        document.getElementById("search-input")?.focus();
      });
    }

    const focusInput = () => {
      document.getElementById("search-input")?.focus();
    };
    focusInput();
    requestAnimationFrame(focusInput);

    let disposed = false;
    let unlisten: (() => void) | undefined;
    let unlistenOpen: (() => void) | undefined;
    let unlistenTheme: (() => void) | undefined;
    let unlistenWindowFocus: (() => void) | undefined;

    listen("history:changed", () => {
      void refresh();
      if (query.trim().length > 0) {
        void runSearch();
      }
    }).then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlisten = stop;
      }
    });

    listen<{ openId: number; cold: boolean }>("palette:open", (event) => {
      query = "";
      filter = "all";
      composing = false;
      selected = 0;
      toast = null;
      void refresh();
      void refreshHotkey();
      schedulePainted(event.payload.openId);
      requestAnimationFrame(() => {
        document.getElementById("search-input")?.focus();
      });
    }).then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlistenOpen = stop;
      }
    });

    listen<{ theme: string; accent: string }>("settings:changed", (event) => {
      applyTheme(event.payload.theme, event.payload.accent);
      cacheTheme(event.payload.theme, event.payload.accent);
    }).then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlistenTheme = stop;
      }
    });

    const onWindowFocus = () => {
      document.getElementById("search-input")?.focus();
    };
    window.addEventListener("focus", onWindowFocus);
    unlistenWindowFocus = () => window.removeEventListener("focus", onWindowFocus);

    window.addEventListener("keydown", onKeydown, true);

    return () => {
      disposed = true;
      unlisten?.();
      unlistenOpen?.();
      unlistenTheme?.();
      unlistenWindowFocus?.();
      window.removeEventListener("keydown", onKeydown, true);
      if (toastTimer) {
        clearTimeout(toastTimer);
      }
    };
  });

  async function activate(index: number) {
    const item = visible[index];
    if (!item) {
      return;
    }

    if (item.kind !== "text") {
      showToast("Images and files arrive in v1.0");
      return;
    }

    try {
      const report = await pasteItem(item.id);
      if (report.status === "copied") {
        showToast(report.reason ? `Copied — ${report.reason}` : "Copied to clipboard");
      }
      void refresh();
    } catch (cause) {
      showToast(describe(cause));
    }
  }

  async function togglePin(index: number) {
    const item = visible[index];
    if (!item) {
      return;
    }

    try {
      await pinItem(item.id, !item.isPinned);
      await refresh();
      if (query.trim().length > 0) {
        await runSearch();
      }
    } catch (cause) {
      showToast(describe(cause));
    }
  }

  async function removeAt(index: number) {
    const item = visible[index];
    if (!item) {
      return;
    }

    if (item.isPinned && !window.confirm("Delete this pinned item?")) {
      return;
    }

    try {
      await deleteItem(item.id);
      await refresh();
      if (query.trim().length > 0) {
        await runSearch();
      }
    } catch (cause) {
      showToast(describe(cause));
    }
  }

  function setFilter(next: Filter) {
    filter = next;
    selected = 0;
  }

  function cycleFilter(direction: 1 | -1) {
    const index = FILTERS.indexOf(filter);
    const next = FILTERS[(index + direction + FILTERS.length) % FILTERS.length];
    setFilter(next);
  }

  function onKeydown(event: KeyboardEvent) {
    if (hotkeyBlocked) {
      // The setup overlay owns the keyboard; its recorder stops propagation.
      return;
    }

    const action = mapKey(event, { query, isComposing: composing });
    if (action.type === "none") {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    switch (action.type) {
      case "move":
        selected = moveSelection(selected, visible.length, action.delta);
        break;
      case "edge":
        selected = action.edge === "start" ? 0 : Math.max(0, visible.length - 1);
        break;
      case "select":
        void activate(selected);
        break;
      case "plain":
        // Plain-text paste becomes distinct in M1-08; copying is identical for
        // text-only history today.
        void activate(selected);
        break;
      case "quick":
        void activate(action.index);
        break;
      case "filter":
        cycleFilter(action.direction);
        break;
      case "pin":
        void togglePin(selected);
        break;
      case "delete":
        void removeAt(selected);
        break;
      case "close":
        void hidePalette();
        break;
      case "settings":
        openSettings().catch((cause) => showToast(`settings failed: ${describe(cause)}`));
        break;
      case "block":
        showToast("App blocking arrives in M2-08");
        break;
    }
  }
</script>

<main class="shell glass-shell">
  <div class="well glass-well">
    <div class="top">
      <SearchBar
        bind:query
        {activeDescendant}
        onCompositionChange={(value) => (composing = value)}
      />
    </div>

    <FilterBar {filter} onSelect={setFilter} />

    <ResultList
      items={visible}
      {selected}
      {status}
      {errorMessage}
      {query}
      {showShortcuts}
      onActivate={(index) => void activate(index)}
      onHover={(index) => (selected = index)}
      onRetry={() => void refresh()}
    />

    <Footer count={visible.length} {toast} {modifier} />
  </div>

  {#if hotkeyBlocked && hotkey}
    <HotkeySetup
      configured={hotkey.configured}
      onSaved={(accelerator) => {
        hotkey = { configured: accelerator, registered: true, initialized: true };
        hotkeyBlocked = false;
        requestAnimationFrame(() => document.getElementById("search-input")?.focus());
      }}
    />
  {/if}
</main>

<style>
  .shell {
    display: flex;
    height: 100%;
    padding: 4px;
    position: relative;
  }

  .well {
    display: flex;
    flex: 1;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    overflow: hidden;
  }

  .top {
    display: flex;
    align-items: center;
    padding: 10px 10px 6px;
  }
</style>
