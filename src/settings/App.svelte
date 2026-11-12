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
  import wordmarkLight from "../../assets/logo/copyfie-wordmark-light.png";
  import wordmarkDark from "../../assets/logo/copyfie-wordmark-dark.png";

  const mac = isMacPlatform(typeof navigator !== "undefined" ? (navigator.platform ?? "") : "");

  let view = $state<SettingsView | null>(null);
  let theme = $state<Theme>("system");
  let accent = $state("#007AFF");
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
  <div class="brand">
    <img class="brand-light" src={wordmarkLight} alt="Copyfie" />
    <img class="brand-dark" src={wordmarkDark} alt="Copyfie" />
    <span class="brand-text">Copyfie</span>
    <span class="version">{view ? `v${view.appVersion}` : ""}</span>
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
      <div class="group">
        <div class="row">
          <span>Theme</span>
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
      </div>
    </section>

    <section>
      <h2>Palette</h2>
      <div class="group">
        <div class="row">
          <span>Keep palette ready</span>
          <div class="segmented" role="group" aria-label="Keep palette ready">
            {#each [["auto", "Auto"], ["always", "Always"], ["never", "Never"]] as [value, label] (value)}
              <button
                type="button"
                class:active={paletteReady === value}
                aria-pressed={paletteReady === value}
                onclick={() => {
                  paletteReady = value;
                  persist();
                }}
              >
                {label}
              </button>
            {/each}
          </div>
        </div>
        {#if paletteReady === "auto"}
          <div class="row">
            <label for="palette-idle">Forget after</label>
            <select id="palette-idle" class="select" bind:value={paletteIdleMinutes} onchange={persist}>
              <option value={5}>5 minutes</option>
              <option value={10}>10 minutes</option>
              <option value={30}>30 minutes</option>
            </select>
          </div>
        {/if}
      </div>
      <p class="hint">
        Auto keeps the palette warm while you work and releases its memory after
        the idle period. Always keeps it pre-warmed; Never creates it on every
        hotkey.
      </p>
    </section>

    <section>
      <h2>History</h2>
      <div class="group">
        <div class="row">
          <label for="retention">Keep history</label>
          <select id="retention" class="select" bind:value={retention} onchange={persist}>
            <option value={7}>7 days</option>
            <option value={30}>30 days</option>
            <option value={90}>90 days</option>
            <option value={365}>365 days</option>
            <option value={0}>Forever</option>
          </select>
        </div>
        <label class="row" for="paused">
          <span>Pause capture</span>
          <span class="switch">
            <input id="paused" type="checkbox" bind:checked={paused} onchange={persist} />
            <span class="knob"></span>
          </span>
        </label>
        <div class="row">
          <span>Clear history</span>
          <button type="button" class="danger" disabled={clearing} onclick={() => void clearAll()}>
            {clearing ? "Clearing…" : "Clear…"}
          </button>
        </div>
        <label class="row" for="clear-pinned">
          <span>Include pinned items</span>
          <span class="switch">
            <input id="clear-pinned" type="checkbox" bind:checked={clearPinned} />
            <span class="knob"></span>
          </span>
        </label>
      </div>
      <p class="hint">Pinned items are never deleted by retention.</p>
    </section>

    <section>
      <h2>Shortcut</h2>
      <div class="group">
        <div class="row">
          <span>Open palette</span>
          <div class="shortcut">
            <kbd>{formatAccelerator(hotkey, mac)}</kbd>
            <button type="button" onclick={startRecording}>Change</button>
          </div>
        </div>

        {#if recording}
          <div class="row recorder-row">
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
              <button type="button" class="primary" onclick={() => void applyHotkey()} disabled={!pendingHotkey}>
                Apply shortcut
              </button>
              <button type="button" onclick={() => (recording = false)}>Cancel</button>
            </div>
            <p class="inline-hint">Include {mac ? "⌘" : "Ctrl"} or {mac ? "⌥" : "Alt"}.</p>
          </div>
        {/if}
      </div>
    </section>

    <section>
      <h2>Startup</h2>
      <div class="group">
        <label class="row" for="login">
          <span>Launch at login</span>
          <span class="switch">
            <input id="login" type="checkbox" bind:checked={launchAtLogin} onchange={persist} />
            <span class="knob"></span>
          </span>
        </label>
      </div>
    </section>

    <section>
      <h2>About</h2>
      <div class="group about">
        <p>Clipboard history stays on this device. No account, no telemetry.</p>
        <p>Portions adapted from Ortu and Ropy (MIT). See THIRD_PARTY.md.</p>
      </div>
    </section>
  {/if}
</main>

{#if error}
  <footer class="error" role="alert">{error}</footer>
{/if}

<style>
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    max-width: 480px;
    margin: 0 auto;
    padding: 24px 20px 16px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .brand img {
    display: block;
    height: 28px;
    width: auto;
  }

  .brand img.brand-light {
    display: none;
  }

  :root[data-theme="light"] .brand img.brand-light {
    display: block;
  }

  :root[data-theme="light"] .brand img.brand-dark {
    display: none;
  }

  @media (prefers-color-scheme: light) {
    :root:not([data-theme]) .brand img.brand-light {
      display: block;
    }

    :root:not([data-theme]) .brand img.brand-dark {
      display: none;
    }
  }

  .brand-text {
    display: none;
    color: var(--text);
    font-size: 18px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  @media (prefers-contrast: more) {
    .brand img {
      display: none !important;
    }

    .brand-text {
      display: inline;
    }
  }

  .version {
    color: var(--text-2);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }

  .state {
    flex: none;
    color: var(--text-2);
    font-size: 11px;
  }

  main {
    max-width: 480px;
    margin: 0 auto;
    padding: 0 20px 24px;
  }

  section {
    margin-bottom: 18px;
  }

  section:last-child {
    margin-bottom: 0;
  }

  h2 {
    margin: 0 0 6px;
    padding: 0 4px;
    color: var(--text-2);
    font-size: var(--font-size-section);
    font-weight: 500;
    letter-spacing: 0;
  }

  .group {
    overflow: hidden;
    border-radius: var(--radius-group);
    background: var(--surface-2);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 44px;
    padding: 0 14px;
    border-top: 1px solid var(--separator);
    color: var(--text);
    font-size: var(--font-size-body);
    font-weight: 400;
    text-decoration: none;
  }

  .row:first-child {
    border-top: none;
  }

  .hint,
  .inline-hint {
    margin: 6px 4px 0;
    color: var(--text-2);
    font-size: 11px;
    font-weight: 400;
    line-height: 1.45;
  }

  .segmented {
    display: flex;
    gap: 2px;
    padding: 2px;
    border-radius: 7px;
    background: var(--seg-bg);
  }

  .segmented button {
    height: 26px;
    padding: 0 9px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--text-2);
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
  }

  .segmented button.active {
    background: var(--seg-selected);
    color: var(--seg-selected-text);
    box-shadow: var(--shadow-seg);
  }

  .select {
    height: 28px;
    max-width: 180px;
    padding: 0 8px;
    border: none;
    border-radius: 7px;
    background: var(--surface);
    color: var(--text);
    font-size: 12px;
    font-weight: 400;
    cursor: pointer;
  }

  .switch {
    position: relative;
    width: 44px;
    height: 26px;
    flex: none;
  }

  .switch input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    opacity: 0;
    cursor: pointer;
    z-index: 1;
  }

  .knob {
    position: absolute;
    inset: 0;
    border-radius: var(--radius-switch);
    background: var(--switch-off);
    pointer-events: none;
  }

  .knob::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--switch-knob);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
    transition: transform var(--dur-state) var(--ease);
  }

  .switch input:checked + .knob {
    background: var(--accent);
  }

  .switch input:checked + .knob::after {
    transform: translateX(18px);
  }

  .switch input:focus-visible + .knob {
    box-shadow: 0 0 0 2px var(--focus-ring);
  }

  .shortcut {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  kbd {
    padding: 1px 6px;
    border-radius: var(--radius-key);
    background: var(--surface);
    color: var(--text);
    font-family: var(--font-ui);
    font-size: 11px;
  }

  .shortcut button,
  .recorder-actions button,
  .empty button {
    height: 26px;
    padding: 0 10px;
    border: none;
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
    transition: transform var(--dur-press) var(--ease);
  }

  .shortcut button:active,
  .recorder-actions button:active,
  .empty button:active {
    transform: scale(0.98);
  }

  .danger {
    height: 26px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--danger);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
  }

  .danger:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .recorder-row {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    padding: 10px 14px;
  }

  .recorder {
    display: grid;
    place-items: center;
    min-height: 38px;
    border: 1px dashed var(--separator);
    border-radius: var(--radius-row);
    background: var(--surface);
    color: var(--text);
    font-size: 12px;
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

  .recorder-actions {
    display: flex;
    gap: 8px;
  }

  .recorder-actions .primary {
    background: var(--selection-bg);
    color: var(--selection-fg);
  }

  .recorder-actions .primary:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .inline-hint {
    margin: 0;
  }

  .about {
    padding: 14px;
  }

  .about p {
    margin: 0 0 8px;
    color: var(--text-2);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .about p:last-child {
    margin-bottom: 0;
  }

  .loading,
  .empty {
    padding: 24px 4px;
    color: var(--text-2);
  }

  .empty .title {
    margin: 0 0 4px;
    color: var(--text);
    font-size: var(--font-size-title);
    font-weight: 500;
  }

  .detail {
    margin: 0 0 12px;
    font-size: var(--font-size-meta);
  }

  footer.error {
    padding: 8px 20px;
    border-top: 1px solid var(--separator);
    color: var(--danger);
    font-size: 11px;
  }
</style>