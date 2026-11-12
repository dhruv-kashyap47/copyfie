<script lang="ts">
  import { isMacPlatform } from "../lib/format";

  let {
    count,
    toast,
    modifier,
  }: {
    count: number;
    toast: string | null;
    modifier: string;
  } = $props();

  const mac = isMacPlatform(typeof navigator !== "undefined" ? (navigator.platform ?? "") : "");
  const pasteKey = mac ? "↵" : "Enter";
  const closeKey = mac ? "⎋" : "Esc";
</script>

<footer>
  {#if toast}
    <span class="toast glass-toast toast-in" role="status">{toast}</span>
  {:else}
    <span class="hints">
      <kbd>{pasteKey}</kbd> paste
      <span class="sep">·</span>
      <kbd>{modifier}+P</kbd> pin
      <span class="sep">·</span>
      <kbd>Tab</kbd> filter
      <span class="sep">·</span>
      <kbd>{closeKey}</kbd>
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
    padding: 8px 12px;
    border-top: 1px solid var(--separator);
    color: var(--text-2);
    font-size: var(--font-size-footer);
    font-weight: 400;
  }

  .hints {
    display: flex;
    align-items: center;
    gap: 4px;
    overflow: hidden;
    white-space: nowrap;
  }

  .sep {
    opacity: 0.6;
  }

  kbd {
    padding: 1px 5px;
    border-radius: var(--radius-key);
    background: var(--surface-2);
    color: var(--text-2);
    font-family: var(--font-ui);
    font-size: 10px;
    line-height: 1.4;
  }

  .toast {
    padding: 3px 8px;
    color: var(--toast-text);
    font-size: var(--font-size-meta);
    font-weight: 500;
  }

  .count {
    flex: none;
    font-variant-numeric: tabular-nums;
  }
</style>
