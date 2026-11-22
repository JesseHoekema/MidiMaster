<script lang="ts">
  import { onMount } from "svelte";
  import { app, profile, init } from "./lib/state.svelte";
  import type { ControlKind } from "./lib/types";
  import type { Tab } from "./lib/state.svelte";
  import Canvas from "./components/Canvas.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Inspector from "./components/Inspector.svelte";
  import Monitor from "./components/Monitor.svelte";
  import ControllerPanel from "./components/ControllerPanel.svelte";
  import Settings from "./components/Settings.svelte";

  let ready = $state(false);
  let error = $state("");
  let canvas: Canvas | undefined = $state();

  onMount(() => {
    init()
      .then(() => (ready = true))
      .catch((e) => (error = String(e)));
  });

  const tabs: [Tab, string][] = [
    ["builder", "Controller"],
    ["monitor", "MIDI monitor"],
    ["controller", "Device settings"],
    ["settings", "Settings"],
  ];
  const unmapped = $derived((profile()?.controls ?? []).filter((c) => c.kind !== "label" && !c.binding).length);
</script>

<div class="shell">
  <header>
    <div class="brand"><img src="/icon.png" alt="" /> MidiMaster</div>
    <nav>
      {#each tabs as [t, l]}
        <button class:on={app.tab === t} onclick={() => (app.tab = t)}>{l}</button>
      {/each}
    </nav>
    <span class="spacer"></span>
    {#if app.tab === "builder"}
      {#if unmapped}<span class="warn">{unmapped} not mapped</span>{/if}
      {#if (profile()?.layers.length ?? 0) > 1}<span class="pill">Layer: {profile()?.layers[app.layer]}</span>{/if}
      <div class="mode">
        <button class:on={app.editing} onclick={() => (app.editing = true)}>Edit</button>
        <button class:on={!app.editing} onclick={() => { app.editing = false; app.learning = null; }}>Live</button>
      </div>
    {/if}
  </header>

  <main>
    {#if error}
      <div class="fatal">Couldn't start: {error}</div>
    {:else if !ready}
      <div class="fatal">Loading…</div>
    {:else if app.tab === "builder"}
      <Sidebar onadd={(k: ControlKind) => canvas?.addControl(k)} />
      <Canvas bind:this={canvas} />
      <Inspector />
    {:else if app.tab === "monitor"}
      <Monitor />
    {:else if app.tab === "controller"}
      <ControllerPanel />
    {:else}
      <Settings />
    {/if}
  </main>

  <div class="toasts">
    {#each app.toasts as t (t.id)}
      <div class="toast {t.kind}">{t.text}</div>
    {/each}
  </div>
</div>

<style>
  .shell { display: flex; flex-direction: column; height: 100%; }
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 8px 14px;
    border-bottom: 1px solid var(--line);
    background: var(--panel);
  }
  .brand { display: flex; align-items: center; gap: 8px; font-weight: 700; letter-spacing: -0.01em; }
  .brand img { width: 22px; height: 22px; }
  nav { display: flex; gap: 2px; margin-left: 10px; }
  nav button, .mode button {
    border: 0;
    background: none;
    padding: 6px 12px;
    border-radius: 7px;
    cursor: pointer;
    color: var(--muted);
  }
  nav button:hover { color: var(--ink); }
  nav button.on { background: var(--accent-soft); color: var(--accent); font-weight: 600; }
  .spacer { flex: 1; }
  .mode { display: flex; border: 1px solid var(--line); border-radius: 8px; padding: 2px; }
  .mode button.on { background: var(--accent); color: var(--accent-ink); font-weight: 600; }
  .warn { color: var(--danger); font-size: 12px; font-weight: 600; }
  .pill { font-size: 12px; background: var(--accent-soft); color: var(--accent); padding: 3px 9px; border-radius: 99px; font-weight: 600; }
  main { flex: 1; display: flex; min-height: 0; }
  .fatal { margin: auto; color: var(--muted); }
  .toasts { position: fixed; bottom: 16px; left: 50%; transform: translateX(-50%); display: grid; gap: 8px; z-index: 10; pointer-events: none; }
  .toast {
    background: var(--ink);
    color: var(--bg);
    padding: 9px 16px;
    border-radius: 9px;
    box-shadow: 0 6px 24px #0004;
    max-width: 560px;
    animation: rise 0.18s ease-out;
  }
  .toast.error { background: var(--danger); color: #fff; }
  @keyframes rise { from { transform: translateY(8px); opacity: 0; } }
</style>
