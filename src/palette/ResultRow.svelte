<script lang="ts">
  import type { ItemSummary } from "../lib/types";
  import { formatBytes, relativeTime, sourceColor, sourceInitial } from "../lib/format";

  let {
    item,
    index,
    selected,
    shortcut,
    onActivate,
    onHover,
  }: {
    item: ItemSummary;
    index: number;
    selected: boolean;
    shortcut: number | null;
    onActivate: (index: number) => void;
    onHover: (index: number) => void;
  } = $props();

  const time = $derived(relativeTime(item.capturedAt));
  const preview = $derived(item.preview.trim() || (item.kind === "image" ? "Image" : item.kind === "files" ? "Files" : "(empty)"));
  const detail = $derived(
    item.kind === "image" && item.width !== null && item.height !== null
      ? `${item.width}×${item.height} · ${formatBytes(item.byteSize)}`
      : item.kind === "files"
        ? "Files"
        : "Text",
  );
</script>

<div
  id={`row-${item.id}`}
  class="row"
  class:selected
  role="option"
  aria-selected={selected}
  tabindex="-1"
  onmouseenter={() => onHover(index)}
  onclick={() => onActivate(index)}
  onkeydown={(event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onActivate(index);
    }
  }}
>
  <span class="kind" data-kind={item.kind}>
    {#if item.kind === "image"}
      <svg viewBox="0 0 20 20" aria-hidden="true">
        <rect x="3" y="4" width="14" height="12" rx="2" fill="none" stroke="currentColor" stroke-width="1.5" />
        <circle cx="7.5" cy="8.5" r="1.3" fill="currentColor" />
        <path d="m4 15 4.5-4 3 2.5L15 9l2 2.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
      </svg>
    {:else if item.kind === "files"}
      <svg viewBox="0 0 20 20" aria-hidden="true">
        <path d="M3 6.5A1.5 1.5 0 0 1 4.5 5h3l1.5 2h6.5A1.5 1.5 0 0 1 17 8.5v6A1.5 1.5 0 0 1 15.5 16h-11A1.5 1.5 0 0 1 3 14.5Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
      </svg>
    {:else}
      <svg viewBox="0 0 20 20" aria-hidden="true">
        <path d="M5 5h10M5 9h10M5 13h7" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      </svg>
    {/if}
  </span>

  <div class="body">
    <p class="preview">{preview}</p>
    <div class="meta">
      {#if item.sourceAppName}
        <span class="avatar" style={`background:${sourceColor(item.sourceAppName)}`}>{sourceInitial(item.sourceAppName)}</span>
        <span class="source">{item.sourceAppName}</span>
        <span class="dot">·</span>
      {/if}
      <span>{time}</span>
      <span class="dot">·</span>
      <span>{detail}</span>
      {#if item.isPinned}
        <svg class="pin" viewBox="0 0 20 20" aria-label="Pinned">
          <path d="M12.8 3.2a1 1 0 0 1 1.4 1.4l-1.1 1.1 2 2-4.2 4.2-2.8.8.8-2.8 4.2-4.2-2-2Z" fill="currentColor" />
          <path d="m8.6 11.4-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      {/if}
    </div>
  </div>

  {#if shortcut !== null}
    <kbd class="shortcut">{shortcut}</kbd>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: var(--row-height);
    padding: 6px 10px;
    border-radius: 8px;
    cursor: default;
  }

  .row.selected {
    background: var(--row-selected);
  }

  .row:hover:not(.selected) {
    background: var(--row-hover);
  }

  .kind {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    flex: none;
    border-radius: 7px;
    background: var(--bg);
    color: var(--fg-muted);
  }

  .kind[data-kind="image"] {
    color: #8fb8ff;
  }

  .kind[data-kind="files"] {
    color: #ffcf8f;
  }

  .kind svg {
    width: 17px;
    height: 17px;
  }

  .body {
    flex: 1;
    min-width: 0;
  }

  .preview {
    margin: 0;
    font-size: 13px;
    line-height: 1.4;
    color: var(--fg);
    white-space: pre-line;
    overflow: hidden;
    display: -webkit-box;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    word-break: break-word;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-top: 2px;
    font-size: 10.5px;
    color: var(--fg-muted);
  }

  .avatar {
    display: inline-grid;
    place-items: center;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    color: #121019;
    font-size: 8px;
    font-weight: 700;
  }

  .source {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dot {
    opacity: 0.6;
  }

  .pin {
    width: 12px;
    height: 12px;
    color: var(--accent);
  }

  .shortcut {
    flex: none;
    display: grid;
    place-items: center;
    min-width: 18px;
    height: 18px;
    padding: 0 3px;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-size: 10px;
  }
</style>
