<script lang="ts">
  import ResultRow from "./ResultRow.svelte";
  import type { ItemSummary } from "../lib/types";

  let {
    items,
    selected,
    status,
    errorMessage,
    query,
    showShortcuts,
    onActivate,
    onHover,
    onRetry,
  }: {
    items: ItemSummary[];
    selected: number;
    status: "loading" | "idle" | "error";
    errorMessage: string | null;
    query: string;
    showShortcuts: boolean;
    onActivate: (index: number) => void;
    onHover: (index: number) => void;
    onRetry: () => void;
  } = $props();

  $effect(() => {
    const current = items[selected];
    if (!current) {
      return;
    }
    document.getElementById(`row-${current.id}`)?.scrollIntoView({ block: "nearest" });
  });
</script>

<div id="results" class="list" role="listbox" aria-label="Clipboard history">
  {#if status === "error"}
    <div class="state">
      <p class="state-line">Could not load history</p>
      <p class="state-hint">{errorMessage ?? "The store did not respond."}</p>
      <button type="button" class="glass-control pressable" onclick={onRetry}>Try again</button>
    </div>
  {:else if status === "loading"}
    {#each Array(6) as _, index (index)}
      <div class="skeleton skeleton-pulse" style={`width:${68 + ((index * 17) % 26)}%`}></div>
    {/each}
  {:else if items.length === 0}
    <div class="state">
      {#if query.trim()}
        <p class="state-line">No matches for “{query.trim()}”.</p>
        <p class="state-hint">Try a different search.</p>
      {:else}
        <p class="state-line">No clipboard history yet.</p>
        <p class="state-hint">Copy something and it will appear here.</p>
      {/if}
    </div>
  {:else}
    {#each items as item, index (item.id)}
      <ResultRow
        {item}
        {index}
        selected={index === selected}
        shortcut={showShortcuts && index < 9 ? index + 1 : null}
        {onActivate}
        {onHover}
      />
    {/each}
  {/if}
</div>

<style>
  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 2px 8px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overscroll-behavior: contain;
    scrollbar-width: thin;
    scrollbar-color: transparent transparent;
  }

  .list:hover {
    scrollbar-color: var(--scroll-thumb) transparent;
  }

  .list::-webkit-scrollbar {
    width: 8px;
  }

  .list::-webkit-scrollbar-thumb {
    background: transparent;
    border: 2px solid transparent;
    border-radius: 999px;
    background-clip: content-box;
  }

  .list:hover::-webkit-scrollbar-thumb {
    background: var(--scroll-thumb);
    border: 2px solid transparent;
    background-clip: content-box;
  }

  .state {
    margin: auto;
    padding: 24px;
    text-align: center;
  }

  .state-line {
    margin: 0;
    color: var(--text-2);
    font-size: var(--font-size-title);
    font-weight: 500;
  }

  .state-hint {
    margin: 4px 0 0;
    color: var(--text-2);
    font-size: var(--font-size-meta);
    font-weight: 400;
  }

  .state button {
    margin-top: 12px;
    padding: 6px 10px;
    font-size: var(--font-size-meta);
    cursor: pointer;
  }

  .skeleton {
    flex: none;
    height: var(--row-height);
    margin: 0 2px;
    border-radius: var(--radius-row);
    background: var(--surface-2);
  }
</style>
