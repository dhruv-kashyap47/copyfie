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
    margin: 0 10px 6px;
    padding: 2px;
  }

  button {
    flex: 1;
    height: 26px;
    padding: 0 8px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-2);
    font-size: var(--font-size-filter);
    font-weight: 500;
    cursor: pointer;
  }

  button:hover {
    color: var(--text);
  }

  button.active {
    background: var(--seg-selected);
    color: var(--seg-selected-text);
    box-shadow: var(--shadow-seg);
  }
</style>
