<script lang="ts">
  // Every incoming MIDI message, newest first, with the control it moves.
  import { app } from "../lib/state.svelte";

  let onlyUnmapped = $state(false);
  const rows = $derived(onlyUnmapped ? app.log.filter((l) => !l.mapped) : app.log);
  const kindName = { cc: "CC", noteon: "Note on", noteoff: "Note off", pb: "Pitch bend" };
</script>

<div class="monitor">
  <div class="bar">
    <span class="hint" style="margin:0">Move anything on your controller. Messages that no control listens to are marked.</span>
    <span style="flex:1"></span>
    <label class="check" style="min-height:0">Only unmapped <input type="checkbox" class="switch" bind:checked={onlyUnmapped} /></label>
    <button class="btn text small" onclick={() => (app.log = [])}>Clear</button>
  </div>
  <div class="table">
    <div class="tr head"><span>Time</span><span>Device</span><span>Ch</span><span>Type</span><span>No.</span><span>Value</span><span>Control</span></div>
    {#each rows as l, i (l.t + "-" + i)}
      <div class="tr" class:unmapped={!l.mapped}>
        <span>{new Date(l.t).toLocaleTimeString()}</span>
        <span class="port">{l.port}</span>
        <span>{l.channel}</span>
        <span>{kindName[l.kind]}</span>
        <span>{l.kind === "pb" ? "–" : l.number}</span>
        <span>{l.value}</span>
        <span>{#if l.mapped}{l.mapped}{:else}<span class="chip error small">Not mapped</span>{/if}</span>
      </div>
    {:else}
      <div class="empty">Nothing yet. Turn a knob on your controller.</div>
    {/each}
  </div>
</div>

<style>
  .monitor { flex: 1; display: flex; flex-direction: column; min-width: 0; padding: 0 24px 16px; }
  .bar { display: flex; gap: 16px; align-items: center; padding: 0 4px 12px; }
  .table { overflow-y: auto; flex: 1; font-variant-numeric: tabular-nums; user-select: text; background: var(--md-surface-container-lowest); border-radius: 16px; }
  .tr { display: grid; grid-template-columns: 100px 1.4fr 40px 100px 50px 70px 1fr; gap: 12px; align-items: center; min-height: 44px; padding: 0 20px; border-bottom: 1px solid var(--md-outline-variant); }
  .tr.head { position: sticky; top: 0; min-height: 48px; background: var(--md-surface-container-lowest); font: 500 12px/16px Roboto, sans-serif; letter-spacing: 0.5px; color: var(--md-on-surface-variant); z-index: 1; }
  .chip.small { height: 24px; padding: 0 8px; font-size: 12px; }
  .port { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .empty { padding: 48px; text-align: center; color: var(--md-on-surface-variant); }
</style>
