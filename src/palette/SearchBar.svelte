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
    stroke-width="1.5"
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
    placeholder="Search"
    autocomplete="off"
    autocapitalize="off"
    spellcheck="false"
    oncompositionstart={() => onCompositionChange?.(true)}
    oncompositionend={() => onCompositionChange?.(false)}
  />

  {#if query.length > 0}
    <button type="button" class="clear" aria-label="Clear search" onclick={clear}>
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
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
    height: 40px;
    padding: 0 10px;
  }

  .search-field:focus-within {
    box-shadow: 0 0 0 2px var(--focus-ring);
  }

  .icon {
    width: 16px;
    height: 16px;
    flex: none;
    color: var(--text-2);
  }

  input {
    flex: 1;
    min-width: 0;
    padding: 0;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text);
    font-size: var(--font-size-search);
    font-weight: 400;
    caret-color: var(--accent);
    user-select: text;
    cursor: text;
  }

  input::placeholder {
    color: var(--text-2);
  }

  .clear {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    flex: none;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
  }

  .clear:hover {
    color: var(--text);
  }

  .clear svg {
    width: 14px;
    height: 14px;
  }
</style>
