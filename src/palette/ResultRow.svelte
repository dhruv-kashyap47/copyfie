<script lang="ts">
  import type { ItemSummary } from "../lib/types";
  import { relativeTime, thumbUrl } from "../lib/format";
  import { icons } from "../lib/icons";

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
  const preview = $derived(
    item.preview.trim() || (item.kind === "image" ? "Image" : item.kind === "files" ? "Files" : "(empty)"),
  );
  const kindIcon = $derived(item.kind === "image" ? icons.image : icons.files);
  const hasThumb = $derived(item.kind === "image" && Boolean(item.thumbName));
  const showType = $derived(item.kind === "image" || item.kind === "files");
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
  {#if hasThumb && item.thumbName}
    <img class="thumb" src={thumbUrl(item.thumbName)} alt="" />
  {:else if item.kind !== "text"}
    <span class="kind">
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        {@html kindIcon}
      </svg>
    </span>
  {/if}

  <div class="body">
    <p class="preview">{preview}</p>
    <div class="meta">
      {#if item.sourceAppName}
        <span class="source">{item.sourceAppName}</span>
        <span class="dot">·</span>
      {/if}
      <span class="time">{time}</span>
      {#if showType}
        <span class="dot">·</span>
        <span>{item.kind === "image" ? "Image" : "Files"}</span>
      {/if}
      {#if item.isPinned}
        <svg
          class="pin"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          role="img"
          aria-label="Pinned"
        >
          {@html icons.pin}
        </svg>
      {/if}
    </div>
  </div>

  {#if shortcut !== null}
    <span class="shortcut">{shortcut}</span>
  {/if}
</div>

<style>
  .row {
    display: flex;
    flex: none;
    align-items: center;
    gap: 10px;
    height: var(--row-height);
    padding: 0 10px;
    border-radius: var(--radius-row);
    cursor: default;
    position: relative;
  }

  .row.selected {
    background: var(--selection-bg);
    color: var(--selection-fg);
  }

  .row:hover:not(.selected) {
    background: var(--row-hover);
  }

  .kind {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    flex: none;
    color: var(--text-2);
  }

  .kind svg {
    width: 16px;
    height: 16px;
  }

  .thumb {
    width: 34px;
    height: 28px;
    flex: none;
    border-radius: 8px;
    object-fit: cover;
    background: var(--surface-2);
  }

  .body {
    flex: 1;
    min-width: 0;
  }

  .preview {
    margin: 0;
    overflow: hidden;
    color: var(--text);
    font-size: var(--font-size-title);
    font-weight: 500;
    line-height: 1.3;
    letter-spacing: -0.01em;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-top: 1px;
    overflow: hidden;
    color: var(--text-2);
    font-size: var(--font-size-meta);
    font-weight: 400;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .source {
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dot {
    opacity: 0.6;
  }

  .pin {
    width: 11px;
    height: 11px;
    flex: none;
    color: var(--accent);
  }

  .shortcut {
    flex: none;
    min-width: 10px;
    color: var(--text-2);
    font-size: var(--font-size-meta);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .row.selected .preview,
  .row.selected .meta,
  .row.selected .source,
  .row.selected .dot,
  .row.selected .shortcut,
  .row.selected .kind,
  .row.selected .pin {
    color: var(--selection-fg);
  }

  .row.selected .thumb {
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.25);
  }
</style>
