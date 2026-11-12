<script lang="ts">
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

<div class="search">
  <svg class="icon" viewBox="0 0 20 20" aria-hidden="true">
    <circle cx="8.5" cy="8.5" r="5.5" fill="none" stroke="currentColor" stroke-width="1.6" />
    <path d="M12.8 12.8 16.5 16.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
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
    <button type="button" class="clear" aria-label="Clear search" onclick={clear}>×</button>
  {/if}
</div>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
  }

  .icon {
    width: 18px;
    height: 18px;
    flex: none;
    color: var(--fg-muted);
  }

  input {
    flex: 1;
    min-width: 0;
    padding: 0;
    background: transparent;
    border: none;
    outline: none;
    color: var(--fg);
    font-size: 15px;
    user-select: text;
    cursor: text;
  }

  input::placeholder {
    color: var(--fg-muted);
  }

  .clear {
    flex: none;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: transparent;
    color: var(--fg-muted);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
  }

  .clear:hover {
    color: var(--fg);
    background: var(--row-hover);
  }
</style>
