<script lang="ts">
  // M3 dialog shown when a newer MidiMaster is available.
  import { upd, installUpdate } from "../lib/updater.svelte";
  import Icon from "./Icon.svelte";

  const busy = $derived(upd.phase === "downloading" || upd.phase === "installing");
  const pct = $derived(upd.total ? Math.min(100, (upd.done / upd.total) * 100) : 0);
  const mb = (n: number) => (n / 1048576).toFixed(1);
</script>

<div class="scrim" role="presentation" onclick={() => { if (!busy) upd.show = false; }}></div>
<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="upd-title">
  <Icon name="refresh" />
  <h2 id="upd-title" class="headline">Update available</h2>
  <p class="supporting">
    MidiMaster <b>{upd.latest}</b> is ready to install. You have {upd.current || "an older version"}.
  </p>

  {#if upd.notes && !busy}
    <div class="notes">{upd.notes}</div>
  {/if}

  {#if busy}
    <div class="progress" class:indeterminate={!upd.total || upd.phase === "installing"}>
      <div class="bar" style="width:{upd.total ? pct : 30}%"></div>
    </div>
    <p class="body-small status">
      {#if upd.phase === "installing"}
        Installing… MidiMaster restarts by itself.
      {:else if upd.total}
        Downloading… {mb(upd.done)} of {mb(upd.total)} MB
      {:else}
        Downloading…
      {/if}
    </p>
  {:else if upd.phase === "error"}
    <p class="error"><Icon name="warning" size={18} /> The update failed: {upd.message}</p>
  {/if}

  <div class="actions">
    {#if !busy}
      <button class="btn text" onclick={() => (upd.show = false)}>Later</button>
      <button class="btn filled" onclick={installUpdate}>{upd.phase === "error" ? "Try again" : "Update"}</button>
    {/if}
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: #0006; z-index: 40; animation: fade 0.2s; }
  .dialog {
    position: fixed;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: min(440px, calc(100vw - 48px));
    padding: 24px;
    border-radius: 28px;
    background: var(--md-surface-container-high);
    color: var(--md-on-surface);
    z-index: 41;
    display: grid;
    justify-items: center;
    animation: pop 0.25s cubic-bezier(0.2, 0, 0, 1);
  }
  .dialog > :global(.icon) { color: var(--md-secondary); }
  .headline { font: 400 24px/32px Roboto, sans-serif; margin: 16px 0; }
  .supporting { margin: 0 0 16px; color: var(--md-on-surface-variant); text-align: center; }
  .supporting b { color: var(--md-on-surface); font-weight: 500; }
  .notes {
    width: 100%;
    max-height: 180px;
    overflow-y: auto;
    white-space: pre-wrap;
    padding: 12px 16px;
    border-radius: 12px;
    background: var(--md-surface-container-lowest);
    font: 400 13px/20px Roboto, sans-serif;
    user-select: text;
  }
  .progress { width: 100%; height: 4px; border-radius: 2px; background: var(--md-secondary-container); overflow: hidden; margin-top: 8px; }
  .bar { height: 100%; background: var(--md-primary); border-radius: 2px; transition: width 0.2s; }
  .indeterminate .bar { animation: slide 1.2s infinite cubic-bezier(0.4, 0, 0.2, 1); }
  .status { margin: 12px 0 0; }
  .error { display: flex; gap: 8px; align-items: flex-start; color: var(--md-error); margin: 8px 0 0; }
  .actions { display: flex; gap: 8px; justify-content: flex-end; width: 100%; margin-top: 24px; min-height: 40px; }
  @keyframes slide { from { transform: translateX(-100%); } to { transform: translateX(340%); } }
  @keyframes fade { from { opacity: 0; } }
  @keyframes pop { from { opacity: 0; transform: translate(-50%, -48%) scale(0.96); } }
</style>
