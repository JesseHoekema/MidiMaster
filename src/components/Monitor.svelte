<script lang="ts">
  // Every incoming MIDI message, newest first, with the control it moves.
  import { app } from "../lib/state.svelte";

  let onlyUnmapped = $state(false);
  const rows = $derived(onlyUnmapped ? app.log.filter((l) => !l.mapped) : app.log);
  const kindName = { cc: "CC", noteon: "Note on", noteoff: "Note off", pb: "Pitch bend" };
</script>

<div class="monitor">
  <div class="bar">
    <b>MIDI monitor</b>
    <span class="hint" style="margin:0">Move anything on your controller. Messages no control listens to are marked.</span>
    <span style="flex:1"></span>
    <label class="check" style="margin:0"><input type="checkbox" bind:checked={onlyUnmapped} /> Only unmapped</label>
    <button class="btn small" onclick={() => (app.log = [])}>Clear</button>
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
        <span>{l.mapped ?? "not mapped"}</span>
      </div>
    {:else}
      <div class="empty">Nothing yet. Turn a knob on your controller.</div>
    {/each}
  </div>
</div>

<style>
  .monitor { flex: 1; display: flex; flex-direction: column; min-width: 0; background: var(--panel); }
  .bar { display: flex; gap: 12px; align-items: center; padding: 12px 16px; border-bottom: 1px solid var(--line); }
  .table { overflow-y: auto; flex: 1; font-variant-numeric: tabular-nums; user-select: text; }
  .tr { display: grid; grid-template-columns: 90px 1.4fr 40px 90px 50px 70px 1fr; gap: 10px; padding: 5px 16px; border-bottom: 1px solid var(--line); }
  .tr.head { position: sticky; top: 0; background: var(--panel); font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
  .tr.unmapped span:last-child { color: var(--danger); font-weight: 600; }
  .port { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .empty { padding: 40px; text-align: center; color: var(--muted); }
</style>
