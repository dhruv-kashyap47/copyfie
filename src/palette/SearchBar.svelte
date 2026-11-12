<script lang="ts">
  import { icons } from "../lib/icons";

  let {
    query = $bindable(""),
    activeDescendant = undefined,
    onCompositionChange,
  }: {
    query?: string;
    activeDescendant?: string;
    onCompositionChange?: (composing: boolean) => void;
  } = $props();

  function focus() {
    document.getElementById("search-input")?.focus();
  }

  function clear() {
    query = "";
    focus();
  }
</script>

<div class="search-field glass-inset">
  <svg
    class="icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    {@html icons.search}
  </svg>

  <input
    id="search-input"
    bind:value={query}
    type="text"
    role="combobox"
    aria-expanded="true"
    aria-controls="results"
    aria-activedescendant={activeDescendant}
    aria-label="Search clipboard history"
    aria-autocomplete="list"
    placeholder="Search clipboard history"
    autocomplete="off"
    autocapitalize="off"
    spellcheck="false"
    oncompositionstart={() => onCompositionChange?.(true)}
    oncompositionend={() => onCompositionChange?.(false)}
  />

  {#if query.length > 0}
    <button type="button" class="clear glass-control pressable" aria-label="Clear search" onclick={clear}>
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        {@html icons.x}
      </svg>
    </button>
  {/if}
</div>

<style>
  .search-field {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
    padding: 7px 7px 7px 10px;
  }

  .search-field:focus-within {
    border-color: var(--focus);
    box-shadow: 0 0 0 1px var(--focus);
  }

  .icon {
    width: 16px;
    height: 16px;
    flex: none;
    color: var(--fg-subtle);
  }

  input {
    flex: 1;
    min-width: 0;
    padding: 0;
    background: transparent;
    border: none;
    outline: none;
    color: var(--fg);
    font-size: var(--font-size-lg);
    user-select: text;
    cursor: text;
  }

  input::placeholder {
    color: var(--fg-subtle);
  }

  .clear {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    flex: none;
    padding: 0;
    border-radius: 6px;
    color: var(--fg-muted);
    cursor: pointer;
  }

  .clear:hover {
    color: var(--fg);
  }

  .clear svg {
    width: 12px;
    height: 12px;
  }
</style>
