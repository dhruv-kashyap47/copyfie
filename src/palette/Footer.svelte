<script lang="ts">
  let {
    count,
    toast,
    modifier,
  }: {
    count: number;
    toast: string | null;
    modifier: string;
  } = $props();
</script>

<footer>
  {#if toast}
    <span class="toast" role="status">{toast}</span>
  {:else}
    <span class="hints">
      <kbd>↑</kbd><kbd>↓</kbd> navigate
      <span class="sep">·</span>
      <kbd>Enter</kbd> copy
      <span class="sep">·</span>
      <kbd>{modifier}+P</kbd> pin
      <span class="sep">·</span>
      <kbd>Tab</kbd> filter
      <span class="sep">·</span>
      <kbd>Esc</kbd> close
    </span>
  {/if}
  <span class="count">{count} item{count === 1 ? "" : "s"}</span>
</footer>

<style>
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 7px 14px;
    border-top: 1px solid var(--border);
    font-size: 10.5px;
    color: var(--fg-muted);
  }

  .hints {
    display: flex;
    align-items: center;
    gap: 5px;
    overflow: hidden;
    white-space: nowrap;
  }

  .sep {
    opacity: 0.5;
  }

  kbd {
    padding: 1px 4px;
    border: 1px solid var(--border);
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: var(--fg);
  }

  .toast {
    color: var(--accent);
    font-weight: 600;
  }

  .count {
    flex: none;
  }
</style>
