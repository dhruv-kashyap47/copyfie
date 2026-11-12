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
      <p class="title">Could not load history</p>
      <p class="detail">{errorMessage ?? "The store did not respond."}</p>
      <button type="button" class="glass-control pressable" onclick={onRetry}>Try again</button>
    </div>
  {:else if status === "loading"}
    {#each Array(6) as _, index (index)}
      <div class="skeleton skeleton-pulse" style={`width:${68 + ((index * 17) % 26)}%`}></div>
    {/each}
  {:else if items.length === 0}
    <div class="state">
      {#if query.trim()}
        <p class="title">No matches</p>
        <p class="detail">Nothing in your history matches “{query.trim()}”.</p>
      {:else}
        <p class="title">No clipboard history yet</p>
        <p class="detail">Copy some text and it will appear here.</p>
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
    scrollbar-width: thin;
    scrollbar-color: var(--control-border) transparent;
  }

  .list::-webkit-scrollbar {
    width: 10px;
  }

  .list::-webkit-scrollbar-thumb {
    background: var(--control-border);
    border: 3px solid transparent;
    border-radius: 999px;
    background-clip: content-box;
  }

  .state {
    margin: auto;
    padding: 24px;
    text-align: center;
  }

  .title {
    margin: 0;
    font-size: var(--font-size-md);
    font-weight: 600;
  }

  .detail {
    margin: 6px 0 0;
    font-size: var(--font-size-sm);
    color: var(--fg-muted);
    line-height: 1.5;
  }

  .state button {
    margin-top: 12px;
    padding: 6px 12px;
    font-size: var(--font-size-sm);
    cursor: pointer;
  }

  .skeleton {
    height: 44px;
    margin: 4px 2px;
    border: 1px solid var(--control-border);
    border-radius: var(--radius-row);
    background: var(--control-bg);
  }
</style>
