<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type AppInfo = {
    name: string;
    version: string;
    coreVersion: string;
    identifier: string;
  };

  let info = $state<AppInfo | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      info = await invoke<AppInfo>("app_info");
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  });
</script>

<main class="shell">
  <header class="brand" aria-label="Copyfie">
    <span class="brand-plain">Copy</span><span class="brand-accent">fie</span>
  </header>

  <p class="tagline">Clipboard history, instantly.</p>

  <section class="card" aria-live="polite">
    {#if info}
      <p class="state state--ok">Tauri bridge connected</p>
      <dl class="meta">
        <div><dt>App</dt><dd>{info.name} {info.version}</dd></div>
        <div><dt>Core</dt><dd>{info.coreVersion}</dd></div>
        <div><dt>Bundle</dt><dd>{info.identifier}</dd></div>
      </dl>
      <p class="note">Milestone 1 scaffold — capture, palette, and paste land next.</p>
    {:else if error}
      <p class="state state--error">Bridge unavailable</p>
      <p class="note">{error}</p>
    {:else}
      <p class="state">Connecting to the Tauri bridge…</p>
    {/if}
  </section>
</main>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 28px;
    height: 100%;
  }

  .brand {
    font-size: 30px;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
  }

  .brand-plain {
    color: var(--fg);
  }

  .brand-accent {
    background: linear-gradient(135deg, #b9a6ff 0%, #7c5cff 55%, #5b3df0 100%);
    background-clip: text;
    -webkit-background-clip: text;
    color: transparent;
  }

  .tagline {
    margin: 0;
    color: var(--fg-muted);
    font-size: 13px;
  }

  .card {
    margin-top: 8px;
    padding: 16px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--bg-elevated);
  }

  .state {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--fg-muted);
  }

  .state--ok {
    color: var(--accent);
  }

  .state--error {
    color: var(--danger);
  }

  .meta {
    display: grid;
    gap: 6px;
    margin: 14px 0 0;
    font-size: 12px;
  }

  .meta div {
    display: flex;
    gap: 8px;
  }

  .meta dt {
    width: 56px;
    color: var(--fg-muted);
  }

  .meta dd {
    margin: 0;
    font-family: var(--font-mono);
  }

  .note {
    margin: 14px 0 0;
    font-size: 12px;
    line-height: 1.5;
    color: var(--fg-muted);
  }
</style>
