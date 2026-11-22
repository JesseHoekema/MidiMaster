<script lang="ts">
  import { onMount } from "svelte";
  import { app, profile, init } from "./lib/state.svelte";
  import type { Tab } from "./lib/state.svelte";
  import type { ControlKind } from "./lib/types";
  import Canvas from "./components/Canvas.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Inspector from "./components/Inspector.svelte";
  import Monitor from "./components/Monitor.svelte";
  import ControllerPanel from "./components/ControllerPanel.svelte";
  import Settings from "./components/Settings.svelte";
  import Icon from "./components/Icon.svelte";
  import PresetDialog from "./components/PresetDialog.svelte";

  let ready = $state(false);
  let error = $state("");
  let canvas: Canvas | undefined = $state();

  onMount(() => {
    init()
      .then(() => (ready = true))
      .catch((e) => (error = String(e)));
  });

  const tabs: [Tab, string, string][] = [
    ["builder", "Controller", "tune"],
    ["monitor", "Monitor", "graphic_eq"],
    ["controller", "Deck", "settings_input_component"],
    ["settings", "Settings", "settings"],
  ];
  const titles: Record<Tab, string> = {
    builder: "Controller",
    monitor: "MIDI monitor",
    controller: "MidiMaster Deck",
    settings: "Settings",
  };
  const unmapped = $derived((profile()?.controls ?? []).filter((c) => c.kind !== "label" && !c.binding).length);
</script>

<div class="shell">
  <nav class="rail">
    <img src="/icon.png" alt="MidiMaster" class="logo" />
    {#each tabs as [t, l, icon]}
      <button class="dest" class:on={app.tab === t} onclick={() => (app.tab = t)}>
        <span class="indicator"><Icon name={icon} /></span>
        <span class="lbl">{l}</span>
      </button>
    {/each}
  </nav>

  <div class="content">
    <header class="topbar">
      <h1 class="title-large">{titles[app.tab]}</h1>
      {#if app.tab === "builder" && ready}
        <span class="chip">{profile()?.name}</span>
        {#if (profile()?.layers.length ?? 0) > 1}
          <span class="chip selected has-icon"><Icon name="layers" size={18} /> {profile()?.layers[app.layer]}</span>
        {/if}
        {#if unmapped}
          <span class="chip error has-icon"><Icon name="link_off" size={18} /> {unmapped} not mapped</span>
        {/if}
        <span class="spacer"></span>
        <div class="seg">
          <button class:sel={app.editing} onclick={() => (app.editing = true)}>
            <Icon name={app.editing ? "check" : "edit"} size={18} /> Edit
          </button>
          <button class:sel={!app.editing} onclick={() => { app.editing = false; app.learning = null; }}>
            <Icon name={!app.editing ? "check" : "play_arrow"} size={18} /> Live
          </button>
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
  </div>

  {#if app.presetDialog}
    <PresetDialog mode={app.presetDialog} onclose={() => (app.presetDialog = null)} />
  {/if}

  <div class="snackbars">
    {#each app.toasts as t (t.id)}
      <div class="snackbar" class:error={t.kind === "error"}>
        {#if t.kind === "error"}<Icon name="warning" size={20} />{/if}
        <span>{t.text}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .shell { display: flex; height: 100%; }
  .rail {
    width: 88px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 16px 0;
    background: var(--md-surface);
  }
  .logo { width: 40px; height: 40px; margin-bottom: 20px; }
  .dest {
    display: grid;
    justify-items: center;
    gap: 4px;
    width: 80px;
    border: 0;
    background: none;
    cursor: pointer;
    color: var(--md-on-surface-variant);
    padding: 0;
  }
  .indicator {
    position: relative;
    overflow: hidden;
    display: grid;
    place-items: center;
    width: 56px;
    height: 32px;
    border-radius: 16px;
    transition: background 0.2s;
  }
  .indicator::after { content: ""; position: absolute; inset: 0; background: currentColor; opacity: 0; transition: opacity 0.15s; }
  .dest:hover .indicator::after { opacity: 0.08; }
  .dest.on .indicator { background: var(--md-secondary-container); color: var(--md-on-secondary-container); }
  .lbl { font: 500 12px/16px Roboto, sans-serif; letter-spacing: 0.5px; }
  .dest.on .lbl { color: var(--md-on-surface); font-weight: 700; }

  .content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--md-surface-container-low);
    border-radius: 24px 0 0 0;
    overflow: hidden;
  }
  .topbar { display: flex; align-items: center; gap: 12px; height: 64px; padding: 0 16px 0 24px; flex: none; }
  .spacer { flex: 1; }
  main { flex: 1; display: flex; min-height: 0; }
  .fatal { margin: auto; color: var(--md-on-surface-variant); }

  .snackbars { position: fixed; left: 104px; bottom: 24px; display: grid; gap: 8px; z-index: 20; pointer-events: none; }
  .snackbar {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 288px;
    max-width: 560px;
    padding: 14px 16px;
    border-radius: 4px;
    background: var(--md-inverse-surface);
    color: var(--md-inverse-on-surface);
    box-shadow: 0 3px 5px -1px #0003, 0 6px 10px #00000024, 0 1px 18px #0000001f;
    animation: rise 0.25s cubic-bezier(0.2, 0, 0, 1);
  }
  .snackbar.error :global(.icon) { color: var(--md-error); }
  @media (prefers-color-scheme: dark) {
    .snackbar.error :global(.icon) { color: #ba1a1a; }
  }
  @keyframes rise { from { transform: translateY(12px); opacity: 0; } }
</style>
