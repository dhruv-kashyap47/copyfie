<script lang="ts">
  import { FILTERS, filterLabel, type Filter } from "../lib/types";

  let {
    filter,
    onSelect,
  }: {
    filter: Filter;
    onSelect: (filter: Filter) => void;
  } = $props();
</script>

<div class="filters" role="tablist" aria-label="Filter by content type">
  {#each FILTERS as chip (chip)}
    <button
      type="button"
      role="tab"
      aria-selected={filter === chip}
      class:active={filter === chip}
      onclick={() => onSelect(chip)}
    >
      {filterLabel(chip)}
    </button>
  {/each}
</div>

<style>
  .filters {
    display: flex;
    gap: 6px;
    padding: 0 16px 10px;
  }

  button {
    padding: 3px 10px;
    border: 1px solid transparent;
    border-radius: 999px;
    background: transparent;
    color: var(--fg-muted);
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
  }

  button:hover {
    background: var(--row-hover);
    color: var(--fg);
  }

  button.active {
    background: var(--row-selected);
    border-color: var(--border);
    color: var(--fg);
  }
</style>
