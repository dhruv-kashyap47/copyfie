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
      <button type="button" onclick={onRetry}>Try again</button>
    </div>
  {:else if status === "loading"}
    {#each Array(6) as _, index (index)}
      <div class="skeleton" style={`width:${68 + ((index * 17) % 26)}%`}></div>
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
    padding: 4px 8px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .state {
    margin: auto;
    padding: 24px;
    text-align: center;
  }

  .title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }

  .detail {
    margin: 6px 0 0;
    font-size: 12px;
    color: var(--fg-muted);
    line-height: 1.5;
  }

  .state button {
    margin-top: 12px;
    padding: 6px 12px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--bg-elevated);
    color: var(--fg);
    font-size: 12px;
    cursor: pointer;
  }

  .state button:hover {
    background: var(--row-hover);
  }

  .skeleton {
    height: 40px;
    margin: 4px 10px;
    border-radius: 8px;
    background: linear-gradient(90deg, var(--bg-elevated), var(--row-hover), var(--bg-elevated));
    background-size: 200% 100%;
    animation: shimmer 1.2s ease-in-out infinite;
  }

  @keyframes shimmer {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }
</style>
