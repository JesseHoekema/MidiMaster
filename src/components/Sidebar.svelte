<script lang="ts">
  // Left panel: profile picker, layers, and the palette of controls to drag in.
  import { invoke } from "@tauri-apps/api/core";
  import { app, profile, save, uid } from "../lib/state.svelte";
  import { CONTROL_KINDS } from "../lib/catalog";
  import type { ControlKind, Profile } from "../lib/types";
  import ControlView from "./ControlView.svelte";
  import Icon from "./Icon.svelte";

  let { onadd }: { onadd: (k: ControlKind) => void } = $props();
  const p = $derived(profile());

  function pick(id: string) {
    if (!app.config) return;
    app.config.activeProfile = id;
    app.selected = null;
    app.values = {};
    app.layer = 0;
    save();
  }
  function addProfile(copy: boolean) {
    if (!app.config || !p) return;
    const np: Profile = copy
      ? { ...$state.snapshot(p), id: uid(), name: `${p.name} copy`, apps: [] }
      : { id: uid(), name: "New profile", apps: [], layers: ["Main"], controls: [] };
    app.config.profiles.push(np);
    pick(np.id);
  }
  function removeProfile() {
    if (!app.config || !p || app.config.profiles.length < 2) return;
    if (!confirm(`Delete the profile "${p.name}"?`)) return;
    app.config.profiles = app.config.profiles.filter((x) => x.id !== p.id);
    pick(app.config.profiles[0].id);
  }

  let appsText = $state("");
  $effect(() => {
    appsText = p?.apps.join(", ") ?? "";
  });
  function setApps() {
    if (!p) return;
    p.apps = appsText.split(",").map((s) => s.trim()).filter(Boolean);
    save();
  }
  function useFrontmost() {
    const f = app.status?.frontmost;
    if (!p || !f || p.apps.includes(f)) return;
    p.apps.push(f);
    appsText = p.apps.join(", ");
    save();
  }

  function setLayer(i: number) {
    invoke("set_layer", { layer: i });
    app.layer = i;
  }
</script>

<aside class="side">
  <h3>Profile</h3>
  <label class="field"><span>Active profile</span>
    <select value={app.config?.activeProfile} onchange={(e) => pick(e.currentTarget.value)}>
      {#each app.config?.profiles ?? [] as pr}<option value={pr.id}>{pr.name}</option>{/each}
    </select>
  </label>
  {#if p}
    <label class="field"><span>Name</span><input bind:value={p.name} oninput={save} /></label>
    <label class="field">
      <span>Switch to it when this app is in front</span>
      <input bind:value={appsText} onchange={setApps} placeholder="e.g. Spotify, OBS" />
    </label>
    {#if app.status?.frontmost && app.config?.autoSwitch}
      <button class="btn text small link" onclick={useFrontmost}><Icon name="add" size={18} />{app.status.frontmost}</button>
    {/if}
    <div class="row wrap">
      <button class="btn tonal small has-icon" onclick={() => addProfile(false)}><Icon name="add" size={18} />New</button>
      <button class="icon-btn small" onclick={() => addProfile(true)} title="Duplicate profile"><Icon name="content_copy" size={20} /></button>
      <button class="icon-btn small danger" onclick={removeProfile} disabled={(app.config?.profiles.length ?? 0) < 2} title="Delete profile"><Icon name="delete" size={20} /></button>
    </div>

    <h3>Layers</h3>
    <p class="hint">A button with “Next layer” switches what every control does.</p>
    {#each p.layers as l, i}
      <div class="row layer">
        <button class="dot" class:on={app.layer === i} onclick={() => setLayer(i)} title="Make active" aria-label="Make active"></button>
        <input value={l} oninput={(e) => { p.layers[i] = e.currentTarget.value; save(); }} />
        {#if p.layers.length > 1}
          <button class="icon-btn small" onclick={() => { p.layers.splice(i, 1); save(); }} title="Remove layer"><Icon name="close" size={18} /></button>
        {/if}
      </div>
    {/each}
    <button class="btn text small has-icon" onclick={() => { p.layers.push(`Layer ${p.layers.length + 1}`); save(); }}><Icon name="add" size={18} />Add layer</button>
  {/if}

  {#if app.editing}
    <h3>Add controls</h3>
    <p class="hint">Drag onto the canvas, or click.</p>
    <div class="palette">
      {#each CONTROL_KINDS as k}
        <button
          class="tile"
          draggable="true"
          ondragstart={(e) => e.dataTransfer?.setData("text/midimaster-kind", k.kind)}
          onclick={() => onadd(k.kind)}
        >
          <div class="mini" class:wide={k.kind === "sliderH"} class:tall={k.kind === "slider"}>
            <ControlView control={{ id: "", kind: k.kind, label: "", x: 0, y: 0, w: 0, h: 0, color: "#2f6fde", binding: { kind: "cc", channel: 1, number: 0 }, bindingY: null, invert: false, invertY: false, min: 0, max: 1, curve: 1, actions: [] }} x={0.6} y={0.6} />
          </div>
          <span>{k.label}</span>
        </button>
      {/each}
    </div>
  {/if}
</aside>

<style>
  .side {
    width: 256px;
    flex: none;
    padding: 4px 16px 24px 24px;
    overflow-y: auto;
  }
  .wrap { flex-wrap: wrap; }
  .layer { margin-bottom: 8px; }
  .dot { width: 20px; height: 20px; flex: none; border-radius: 50%; border: 2px solid var(--md-on-surface-variant); background: none; cursor: pointer; padding: 0; position: relative; }
  .dot.on { border-color: var(--md-primary); }
  .dot.on::after { content: ""; position: absolute; inset: 3px; border-radius: 50%; background: var(--md-primary); }
  .link { margin: -12px 0 12px -10px; }
  .palette { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .tile {
    position: relative;
    overflow: hidden;
    display: grid;
    justify-items: center;
    gap: 8px;
    padding: 14px 6px 10px;
    border: 0;
    border-radius: 12px;
    background: var(--md-surface-container-highest);
    color: var(--md-on-surface);
    cursor: grab;
    font: 500 12px/16px Roboto, sans-serif;
    letter-spacing: 0.5px;
  }
  .tile::after { content: ""; position: absolute; inset: 0; background: currentColor; opacity: 0; transition: opacity 0.15s; }
  .tile:hover::after { opacity: 0.08; }
  .tile:active { cursor: grabbing; }
  .mini { width: 44px; height: 44px; pointer-events: none; }
  .mini.wide { width: 64px; height: 26px; margin: 9px 0; }
  .mini.tall { width: 26px; height: 44px; }
  .mini :global(.name) { display: none; }
  .mini :global(.slot .cap) { width: 22px; margin-left: -11px; height: 10px; }
  .mini :global(.slot.h .cap) { width: 10px; height: 22px; margin-top: -11px; }
</style>
