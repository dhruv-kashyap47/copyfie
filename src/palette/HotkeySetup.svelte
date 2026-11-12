<script lang="ts">
  import { onMount } from "svelte";

  import { isMacPlatform } from "../lib/format";
  import { chordFromEvent, formatAccelerator, isModifierKey } from "../lib/hotkey";
  import { setHotkey } from "../lib/ipc";

  let {
    configured,
    onSaved,
  }: {
    configured: string;
    onSaved: (accelerator: string) => void;
  } = $props();

  const mac = isMacPlatform(typeof navigator !== "undefined" ? (navigator.platform ?? "") : "");

  let chord = $state<string | null>(null);
  let error = $state<string | null>(null);
  let saving = $state(false);
  let recorder = $state<HTMLDivElement | null>(null);

  onMount(() => {
    recorder?.focus();
  });

  function onKeydown(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    // Blocking setup step: Escape is intentionally ignored.
    if (event.key === "Escape") {
      return;
    }

    const next = chordFromEvent(event);
    if (next) {
      chord = next;
      error = null;
      return;
    }

    if (!event.ctrlKey && !event.metaKey && !event.altKey && !isModifierKey(event.key)) {
      error = mac ? "Add ⌘ or ⌥ to the shortcut." : "Add Ctrl or Alt to the shortcut.";
    }
  }

  async function save() {
    if (!chord || saving) {
      return;
    }

    saving = true;
    error = null;

    try {
      const status = await setHotkey(chord);
      onSaved(status.configured);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      saving = false;
    }
  }
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-labelledby="hotkey-title">
  <div class="card">
    <h2 id="hotkey-title">Shortcut unavailable</h2>
    <p>
      <kbd>{formatAccelerator(configured, mac)}</kbd> is already used by another app.
      Record a different shortcut to keep Copyfie working.
    </p>

    <div
      class="recorder"
      bind:this={recorder}
      role="textbox"
      tabindex="0"
      aria-label="Record a global shortcut"
      onkeydown={onKeydown}
    >
      {#if chord}
        {formatAccelerator(chord, mac)}
      {:else}
        <span class="placeholder">Press a shortcut…</span>
      {/if}
    </div>

    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}

    <button type="button" disabled={!chord || saving} onclick={save}>
      {saving ? "Saving…" : "Use this shortcut"}
    </button>

    <p class="hint">
      Include {mac ? "⌘" : "Ctrl"} or {mac ? "⌥" : "Alt"}, for example
      {mac ? "⌘⇧F9" : "Ctrl+Shift+F9"}.
    </p>
  </div>
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: var(--scrim);
  }

  .card {
    width: min(430px, calc(100% - 40px));
    padding: 20px;
    border: 1px solid var(--dialog-border);
    border-radius: var(--radius-dialog);
    background: var(--dialog-bg);
    box-shadow: var(--shadow-dialog);
  }

  h2 {
    margin: 0 0 8px;
    color: var(--text);
    font-size: 15px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  p {
    margin: 0 0 14px;
    color: var(--text-2);
    font-size: var(--font-size-body);
    line-height: 1.5;
  }

  kbd {
    padding: 1px 5px;
    border-radius: var(--radius-key);
    background: var(--surface-2);
    color: var(--text);
    font-family: var(--font-ui);
    font-size: 11px;
  }

  .recorder {
    display: grid;
    place-items: center;
    min-height: 46px;
    margin-bottom: 12px;
    border: 1px dashed var(--separator);
    border-radius: var(--radius-well);
    background: var(--surface-2);
    color: var(--text);
    font-family: var(--font-ui);
    font-size: 13px;
    cursor: text;
  }

  .recorder:focus-visible {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 2px var(--focus-ring);
  }

  .placeholder {
    color: var(--text-2);
  }

  .error {
    margin: 0 0 10px;
    color: var(--danger);
  }

  button {
    width: 100%;
    height: 36px;
    border: none;
    border-radius: var(--radius-row);
    background: var(--selection-bg);
    color: var(--selection-fg);
    font-size: var(--font-size-body);
    font-weight: 600;
    cursor: pointer;
    transition: transform var(--dur-press) var(--ease);
  }

  button:active:not(:disabled) {
    transform: scale(0.98);
  }

  button:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .hint {
    margin: 10px 0 0;
    color: var(--text-2);
    font-size: 11px;
  }
</style>
