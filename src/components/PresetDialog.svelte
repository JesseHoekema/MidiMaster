<script lang="ts">
  // M3 dialog: pick a device preset, either as a new profile or to replace the
  // current profile's layout.
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { app, profile, save, toast, uid } from "../lib/state.svelte";
  import type { Preset } from "../lib/types";
  import Icon from "./Icon.svelte";

  let { mode, onclose }: { mode: "new" | "replace"; onclose: () => void } = $props();
  let presets = $state<Preset[]>([]);
  let inputs = $state<string[]>([]);
  let chosen = $state("");

  onMount(async () => {
    [presets, inputs] = await Promise.all([invoke<Preset[]>("get_presets"), invoke<string[]>("list_midi_inputs")]);
    chosen = presets.find(connected)?.id ?? presets[0]?.id ?? "";
  });

  const connected = (p: Preset) => p.matchInputs.some((m) => inputs.some((i) => i.toLowerCase().includes(m)));

  function apply() {
    const pre = presets.find((p) => p.id === chosen);
    const cfg = app.config;
    if (!pre || !cfg) return;
    if (mode === "new") {
      const np = { ...$state.snapshot(pre.profile), id: uid() };
      cfg.profiles.push(np);
      cfg.activeProfile = np.id;
      app.layer = 0;
    } else {
      const p = profile();
      if (!p) return;
      p.controls = $state.snapshot(pre.profile.controls);
      p.layers = $state.snapshot(pre.profile.layers);
    }
    app.selected = null;
    app.values = {};
    save();
    toast("info", mode === "new" ? `Profile “${pre.profile.name}” added` : `Layout replaced with ${pre.name}`);
    onclose();
  }
</script>

<div class="scrim" role="presentation" onclick={onclose}></div>
<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="preset-title">
  <Icon name="tune" />
  <h2 id="preset-title" class="headline">{mode === "new" ? "New profile" : "Load a device preset"}</h2>
  <p class="supporting">
    {mode === "new"
      ? "Pick your device. A preset places every control where it sits on the real thing, already mapped."
      : "This replaces the controls of the current profile. Their actions are removed too."}
  </p>
  <div class="list">
    {#each presets as p (p.id)}
      <label class="option" class:sel={chosen === p.id}>
        <input type="radio" name="preset" value={p.id} bind:group={chosen} />
        <span class="text">
          <span class="row">
            <b>{p.name}</b>
            {#if connected(p)}<span class="chip selected tiny"><Icon name="usb" size={14} /> Connected</span>{/if}
          </span>
          <span class="body-small">{p.description}</span>
        </span>
      </label>
    {/each}
  </div>
  <div class="actions">
    <button class="btn text" onclick={onclose}>Cancel</button>
    <button class="btn text" onclick={apply} disabled={!chosen}>{mode === "new" ? "Create" : "Replace layout"}</button>
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: #0006; z-index: 30; animation: fade 0.2s; }
  .dialog {
    position: fixed;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: min(460px, calc(100vw - 48px));
    padding: 24px;
    border-radius: 28px;
    background: var(--md-surface-container-high);
    color: var(--md-on-surface);
    z-index: 31;
    display: grid;
    justify-items: center;
    animation: pop 0.25s cubic-bezier(0.2, 0, 0, 1);
  }
  .dialog > :global(.icon) { color: var(--md-secondary); }
  .headline { font: 400 24px/32px Roboto, sans-serif; margin: 16px 0; }
  .supporting { margin: 0 0 16px; color: var(--md-on-surface-variant); text-align: center; }
  .list { display: grid; gap: 8px; width: 100%; }
  .option {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    padding: 14px 16px;
    border-radius: 16px;
    background: var(--md-surface-container-lowest);
    cursor: pointer;
  }
  .option.sel { background: var(--md-secondary-container); color: var(--md-on-secondary-container); }
  .option input { margin-top: 2px; accent-color: var(--md-primary); width: 18px; height: 18px; flex: none; }
  .text { display: grid; gap: 4px; }
  .text b { font: 500 16px/24px Roboto, sans-serif; }
  .tiny { height: 24px; padding: 0 8px 0 6px; font-size: 12px; gap: 4px; }
  .actions { display: flex; gap: 8px; justify-content: flex-end; width: 100%; margin-top: 24px; }
  @keyframes fade { from { opacity: 0; } }
  @keyframes pop { from { opacity: 0; transform: translate(-50%, -48%) scale(0.96); } }
</style>
