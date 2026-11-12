<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  import { isMacPlatform } from "../lib/format";
  import { chordFromEvent, formatAccelerator } from "../lib/hotkey";
  import {
    clearHistory,
    settingsGet,
    settingsSave,
    type SettingsInput,
    type SettingsView,
  } from "../lib/ipc";
  import { applyTheme, cacheTheme, normalizeTheme, type Theme } from "../lib/theme";

  const mac = isMacPlatform(typeof navigator !== "undefined" ? (navigator.platform ?? "") : "");

  let view = $state<SettingsView | null>(null);
  let theme = $state<Theme>("system");
  let accent = $state("#5B9DFF");
  let retention = $state<number>(30);
  let paused = $state(false);
  let launchAtLogin = $state(false);
  let paletteReady = $state("auto");
  let paletteIdleMinutes = $state(10);
  let hotkey = $state("");
  let recording = $state(false);
  let pendingHotkey = $state<string | null>(null);
  let clearPinned = $state(false);
  let clearing = $state(false);
  let recorder = $state<HTMLDivElement | null>(null);
  let saving = $state(false);
  let status = $state<string | null>(null);
  let error = $state<string | null>(null);
  let loadError = $state<string | null>(null);
  let saveRequest = 0;
  let statusTimer: ReturnType<typeof setTimeout> | undefined;

  function describe(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }

  function applyView(loaded: SettingsView) {
    view = loaded;
    theme = normalizeTheme(loaded.theme);
    accent = loaded.accent;
    retention = loaded.retentionDays;
    paused = loaded.capturePaused;
    launchAtLogin = loaded.launchAtLogin;
    hotkey = loaded.hotkey;
    paletteReady = loaded.paletteReady;
    paletteIdleMinutes = loaded.paletteIdleMinutes;
  }

  async function reload() {
    try {
      const loaded = await settingsGet();
      applyView(loaded);
    } catch (cause) {
      loadError = describe(cause);
    }
  }

  async function saveNow() {
    if (!view) {
      return;
    }

    const request = ++saveRequest;
    saving = true;
    error = null;

    const input: SettingsInput = {
      theme,
      accent,
      retentionDays: Number(retention),
      capturePaused: paused,
      launchAtLogin,
      hotkey,
      paletteReady,
      paletteIdleMinutes: Number(paletteIdleMinutes),
    };

    try {
      const updated = await settingsSave(input);
      if (request !== saveRequest) {
        return;
      }
      applyView(updated);
      applyTheme(updated.theme, updated.accent);
      cacheTheme(updated.theme, updated.accent);
      status = "Saved";
      if (statusTimer) {
        clearTimeout(statusTimer);
      }
      statusTimer = setTimeout(() => (status = null), 1600);
    } catch (cause) {
      if (request !== saveRequest) {
        return;
      }
      error = describe(cause);
      // Revert optimistic controls to the persisted values.
      await reload();
    } finally {
      if (request === saveRequest) {
        saving = false;
      }
    }
  }

  function persist() {
    void saveNow();
  }

  function setTheme(next: Theme) {
    theme = next;
    persist();
  }

  function startRecording() {
    recording = true;
    pendingHotkey = null;
    error = null;
    requestAnimationFrame(() => recorder?.focus());
  }

  function onRecorderKeydown(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      recording = false;
      pendingHotkey = null;
      return;
    }

    const next = chordFromEvent(event);
    if (next) {
      pendingHotkey = next;
    }
  }

  async function applyHotkey() {
    if (!pendingHotkey) {
      return;
    }
    hotkey = pendingHotkey;
    await saveNow();
    if (!error) {
      recording = false;
      pendingHotkey = null;
    }
  }

  async function clearAll() {
    const message = clearPinned
      ? "Delete ALL history, including pinned items?"
      : "Delete all unpinned history? Pinned items are kept.";
    if (!window.confirm(message)) {
      return;
    }

    clearing = true;
    error = null;
    try {
      const removed = await clearHistory(!clearPinned);
      status = `Cleared ${removed} item${removed === 1 ? "" : "s"}`;
      if (statusTimer) {
        clearTimeout(statusTimer);
      }
      statusTimer = setTimeout(() => (status = null), 2200);
    } catch (cause) {
      error = describe(cause);
    } finally {
      clearing = false;
    }
  }

  onMount(() => {
    void reload();

    let disposed = false;
    let unlisten: (() => void) | undefined;

    listen<{ theme: string; accent: string }>("settings:changed", (event) => {
      applyTheme(event.payload.theme, event.payload.accent);
      cacheTheme(event.payload.theme, event.payload.accent);
    }).then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlisten = stop;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
      if (statusTimer) {
        clearTimeout(statusTimer);
      }
    };
  });
</script>

<header>
  <div class="header-copy">
    <h1>Copyfie Settings</h1>
    <p>Local history, appearance and startup.</p>
  </div>
  <span class="state">{saving ? "Saving…" : (status ?? "")}</span>
</header>

<main>
  {#if loadError}
    <div class="empty">
      <p class="title">Could not load settings</p>
      <p class="detail">{loadError}</p>
      <button type="button" onclick={() => void reload()}>Try again</button>
    </div>
  {:else if !view}
    <p class="loading">Loading settings…</p>
  {:else}
    <section>
      <h2>Appearance</h2>
      <div class="row">
        <span class="label">Theme</span>
        <div class="segmented" role="group" aria-label="Theme">
          {#each ["system", "light", "dark"] as option (option)}
            <button
              type="button"
              class:active={theme === option}
              aria-pressed={theme === option}
              onclick={() => setTheme(option as Theme)}
            >
              {option === "system" ? "System" : option === "light" ? "Light" : "Dark"}
            </button>
          {/each}
        </div>
      </div>
    </section>

    <section>
      <h2>Palette</h2>
      <div class="row">
        <label for="palette-ready">Keep palette ready</label>
        <select id="palette-ready" bind:value={paletteReady} onchange={persist}>
          <option value="auto">Auto (recommended)</option>
          <option value="always">Always</option>
          <option value="never">Never</option>
        </select>
      </div>
      {#if paletteReady === "auto"}
        <div class="row">
          <label for="palette-idle">Forget after</label>
          <select id="palette-idle" bind:value={paletteIdleMinutes} onchange={persist}>
            <option value={5}>5 minutes</option>
            <option value={10}>10 minutes</option>
            <option value={30}>30 minutes</option>
          </select>
        </div>
      {/if}
      <p class="hint">
        Auto keeps the palette warm while you work and releases its memory after
        the idle period. Always keeps it pre-warmed; Never creates it on every
        hotkey (lowest idle memory, slower open).
      </p>
    </section>

    <section>
      <h2>History</h2>
      <div class="row">
        <label for="retention">Keep history</label>
        <select id="retention" bind:value={retention} onchange={persist}>
          <option value={7}>7 days</option>
          <option value={30}>30 days</option>
          <option value={90}>90 days</option>
          <option value={365}>365 days</option>
          <option value={0}>Forever</option>
        </select>
      </div>
      <div class="row">
        <label for="paused">Pause capture</label>
        <input id="paused" type="checkbox" bind:checked={paused} onchange={persist} />
      </div>
      <div class="row">
        <span class="label">Clear history</span>
        <div class="clear-actions">
          <label class="inline">
            <input type="checkbox" bind:checked={clearPinned} /> include pinned
          </label>
          <button type="button" class="danger" disabled={clearing} onclick={() => void clearAll()}>
            Clear…
          </button>
        </div>
      </div>
      <p class="hint">Pinned items are never deleted by retention.</p>
    </section>

    <section>
      <h2>Shortcut</h2>
      <div class="row">
        <span class="label">Open palette</span>
        <div class="shortcut">
          <kbd>{formatAccelerator(hotkey, mac)}</kbd>
          <button type="button" onclick={startRecording}>Change</button>
        </div>
      </div>

      {#if recording}
        <div
          class="recorder"
          bind:this={recorder}
          role="textbox"
          tabindex="0"
          aria-label="Record a shortcut"
          onkeydown={onRecorderKeydown}
        >
          {#if pendingHotkey}
            {formatAccelerator(pendingHotkey, mac)}
          {:else}
            <span class="placeholder">Press a shortcut…</span>
          {/if}
        </div>
        <div class="recorder-actions">
          <button type="button" onclick={() => void applyHotkey()} disabled={!pendingHotkey}>
            Apply shortcut
          </button>
          <button type="button" class="ghost" onclick={() => (recording = false)}>Cancel</button>
        </div>
        <p class="hint">Include {mac ? "⌘" : "Ctrl"} or {mac ? "⌥" : "Alt"}.</p>
      {/if}
    </section>

    <section>
      <h2>Startup</h2>
      <div class="row">
        <label for="login">Launch at login</label>
        <input id="login" type="checkbox" bind:checked={launchAtLogin} onchange={persist} />
      </div>
    </section>

    <section class="about">
      <h2>About</h2>
      <p>Copyfie {view.appVersion}</p>
      <p>Clipboard history stays on this device. No account, no telemetry.</p>
      <p>Portions adapted from Ortu and Ropy (MIT). See THIRD_PARTY.md.</p>
    </section>
  {/if}
</main>

{#if error}
  <footer class="error" role="alert">{error}</footer>
{/if}

<style>
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 20px 20px 14px;
  }

  .header-copy {
    min-width: 0;
  }

  h1 {
    margin: 0;
    font-size: var(--font-size-xl);
    letter-spacing: -0.01em;
  }

  .header-copy p {
    margin: 4px 0 0;
    font-size: var(--font-size-xs);
    color: var(--fg-muted);
  }

  .state {
    flex: none;
    padding-top: 2px;
    font-size: var(--font-size-xs);
    color: var(--fg-muted);
  }

  main {
    padding: 0 20px 24px;
  }

  section {
    margin-bottom: 10px;
    padding: 14px 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius-well);
    background: var(--bg-elevated);
  }

  section:last-child {
    margin-bottom: 0;
  }

  h2 {
    margin: 0 0 10px;
    font-size: var(--font-size-sm);
    font-weight: 700;
    letter-spacing: 0.01em;
    color: var(--fg);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 34px;
  }

  label,
  .label {
    color: var(--fg);
    font-size: var(--font-size-sm);
  }

  select {
    min-width: 130px;
    padding: 5px 8px;
    border: var(--control-border-width) solid var(--control-border);
    border-radius: var(--radius-sm);
    background: var(--control-bg);
    color: var(--fg);
  }

  select:focus-visible {
    border-color: var(--focus);
    outline: none;
  }

  input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
    cursor: pointer;
  }

  .segmented {
    display: flex;
    gap: 2px;
    padding: 2px;
    border: var(--control-border-width) solid var(--control-border);
    border-radius: var(--radius-sm);
    background: var(--track-bg);
  }

  .segmented button {
    padding: 5px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--fg-muted);
    font-size: var(--font-size-xs);
    cursor: pointer;
  }

  .segmented button:hover {
    background: var(--control-hover);
    color: var(--fg);
  }

  .segmented button.active {
    background: var(--row-selected);
    color: var(--fg);
    box-shadow: inset 0 0 0 1px var(--row-selected-border);
  }

  .shortcut {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  kbd {
    padding: 2px 6px;
    border: 1px solid var(--control-border);
    border-radius: 5px;
    background: var(--control-bg);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .shortcut button,
  .recorder-actions button,
  .empty button {
    padding: 5px 10px;
    border: 1px solid var(--control-border);
    border-radius: var(--radius-sm);
    background: var(--control-bg);
    color: var(--fg);
    font-size: var(--font-size-xs);
    cursor: pointer;
    transition: transform var(--dur-press) var(--ease);
  }

  .shortcut button:hover,
  .recorder-actions button:hover,
  .empty button:hover {
    background: var(--control-hover);
  }

  .shortcut button:active,
  .recorder-actions button:active,
  .empty button:active {
    transform: scale(0.98);
  }

  .recorder {
    display: grid;
    place-items: center;
    min-height: 46px;
    margin-top: 8px;
    border: 1px dashed var(--control-border);
    border-radius: var(--radius-control);
    background: var(--field-bg);
    font-family: var(--font-mono);
    font-size: 12.5px;
    cursor: text;
  }

  .recorder:focus-visible {
    border-color: var(--accent);
    outline: none;
  }

  .placeholder {
    color: var(--fg-muted);
    font-family: var(--font-ui);
  }

  .recorder-actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .recorder-actions .ghost {
    background: transparent;
  }

  .hint {
    margin: 8px 0 0;
    font-size: var(--font-size-xs);
    line-height: 1.5;
    color: var(--fg-muted);
  }

  .clear-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .inline {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: var(--font-size-xs);
    color: var(--fg-muted);
  }

  .clear-actions .danger {
    padding: 5px 10px;
    border: 1px solid var(--control-border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--danger);
    font-size: var(--font-size-xs);
    cursor: pointer;
  }

  .clear-actions .danger:hover:not(:disabled) {
    border-color: var(--danger);
  }

  .clear-actions .danger:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .about p {
    margin: 0 0 6px;
    font-size: var(--font-size-xs);
    line-height: 1.5;
    color: var(--fg-muted);
  }

  .loading,
  .empty {
    padding: 24px 0;
    color: var(--fg-muted);
  }

  .empty .title {
    margin: 0 0 6px;
    color: var(--fg);
    font-weight: 600;
  }

  .detail {
    margin: 0 0 12px;
    font-size: var(--font-size-sm);
  }

  footer.error {
    padding: 8px 20px;
    border-top: 1px solid var(--border);
    color: var(--danger);
    font-size: var(--font-size-xs);
  }
</style>