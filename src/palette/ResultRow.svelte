<script lang="ts">
  import type { ItemSummary } from "../lib/types";
  import { formatBytes, relativeTime, sourceColor, sourceInitial } from "../lib/format";
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
  const preview = $derived(item.preview.trim() || (item.kind === "image" ? "Image" : item.kind === "files" ? "Files" : "(empty)"));
  const detail = $derived(
    item.kind === "image" && item.width !== null && item.height !== null
      ? `${item.width}×${item.height} · ${formatBytes(item.byteSize)}`
      : item.kind === "files"
        ? "Files"
        : "Text",
  );
  const kindIcon = $derived(
    item.kind === "image" ? icons.image : item.kind === "files" ? icons.files : icons.text,
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
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      {@html kindIcon}
    </svg>
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
        <svg
          class="pin"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
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
    <kbd class="shortcut">{shortcut}</kbd>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: var(--row-height);
    padding: 7px 10px;
    border-radius: var(--radius-row);
    box-shadow: inset 0 0 0 1px transparent;
    cursor: default;
    position: relative;
  }

  .row.selected {
    background: var(--row-selected);
    box-shadow: inset 0 0 0 1px var(--row-selected-border);
  }

  .row.selected::before {
    content: "";
    position: absolute;
    left: 0;
    top: 9px;
    bottom: 9px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--selected-bar);
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
    border: 1px solid var(--control-border);
    border-radius: var(--radius-sm);
    background: var(--control-bg);
    color: var(--fg-subtle);
  }

  .kind[data-kind="text"] {
    color: var(--kind-text);
  }

  .kind[data-kind="image"] {
    color: var(--kind-image);
  }

  .kind[data-kind="files"] {
    color: var(--kind-files);
  }

  .kind svg {
    width: 16px;
    height: 16px;
  }

  .body {
    flex: 1;
    min-width: 0;
  }

  .preview {
    margin: 0;
    font-size: var(--font-size-md);
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
    font-size: var(--font-size-xs);
    color: var(--fg-muted);
  }

  .avatar {
    display: inline-grid;
    place-items: center;
    width: 14px;
    height: 14px;
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
    min-width: 20px;
    height: 20px;
    padding: 0 4px;
    border: 1px solid var(--control-border);
    border-radius: 5px;
    background: var(--control-bg);
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-size: 10px;
  }
</style>
