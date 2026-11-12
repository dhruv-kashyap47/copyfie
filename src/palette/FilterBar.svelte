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

<div class="filters glass-track" role="tablist" aria-label="Filter by content type">
  {#each FILTERS as chip (chip)}
    <button
      type="button"
      role="tab"
      aria-selected={filter === chip}
      class:active={filter === chip}
      class="pressable"
      onclick={() => onSelect(chip)}
    >
      {filterLabel(chip)}
    </button>
  {/each}
</div>

<style>
  .filters {
    display: flex;
    gap: 2px;
    margin: 0 14px 8px;
    padding: 3px;
  }

  button {
    flex: 1;
    padding: 5px 8px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--fg-muted);
    font-size: var(--font-size-xs);
    font-weight: 600;
    cursor: pointer;
  }

  button:hover {
    background: var(--control-hover);
    color: var(--fg);
  }

  button.active {
    background: var(--row-selected);
    color: var(--fg);
    box-shadow: inset 0 0 0 1px var(--row-selected-border);
  }
</style>
