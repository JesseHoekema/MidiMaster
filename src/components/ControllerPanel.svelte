<script lang="ts">
  // Settings stored on the MidiMaster Deck itself (replaces the old web page).
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { toast } from "../lib/state.svelte";
  import { fill } from "../lib/range";
  import Icon from "./Icon.svelte";

  interface Cfg { mode: number; clicks: number; accel: number; encRev: number; panInv: number; tiltInv: number; sweep: number; dead: number }
  interface Mon { enc: number[]; pan: number; tilt: number }

  let ports = $state<{ name: string; product: string }[]>([]);
  let port = $state("");
  let connected = $state(false);
  let cfg = $state<Cfg | null>(null);
  let mon = $state<Mon | null>(null);
  let note = $state("");
  const unlisten: UnlistenFn[] = [];

  const MODES = [
    ["7-bit CC (0–127)", "Encoder 1 on CC 16, encoders 2–5 on CC 23–26. 128 steps."],
    ["14-bit CC (0–16383)", "Coarse on CC 16 / 23–26, fine on CC 48 / 55–58. Learn them in MidiMaster as 14-bit CC."],
    ["Pitch Bend (0–16383)", "Encoders 1–5 on MIDI channel 1–5."],
  ];

  async function scan() {
    ports = await invoke("controller_ports");
    if (!port && ports.length) port = ports[0].name;
  }
  async function connect() {
    try {
      await invoke("controller_open", { name: port });
      connected = true;
      await send("get");
      await send("mon 1");
    } catch (e) {
      toast("error", String(e));
    }
  }
  async function disconnect() {
    await invoke("controller_close");
    connected = false;
    cfg = null;
    mon = null;
  }
  const send = (line: string) => invoke("controller_send", { line }).catch((e) => toast("error", String(e)));

  let timers: Record<string, ReturnType<typeof setTimeout>> = {};
  function set(key: keyof Cfg, value: number) {
    if (cfg) cfg[key] = value;
    note = "Saving…";
    clearTimeout(timers[key]);
    timers[key] = setTimeout(() => send(`set ${key} ${value}`), 60);
  }

  onMount(async () => {
    unlisten.push(
      await listen<string>("controller-line", (e) => {
        try {
          const m = JSON.parse(e.payload);
          if (m.cfg) cfg = m.cfg;
          if (m.mon) mon = m.mon;
          if (m.saved) note = "Saved on the Deck.";
          if (m.synced) note = "All values sent.";
          if (m.error) toast("error", `Deck: ${m.error}`);
        } catch {}
      }),
      await listen("controller-closed", () => {
        connected = false;
        toast("error", "MidiMaster Deck disconnected");
      }),
    );
    await scan();
  });
  onDestroy(() => {
    unlisten.forEach((u) => u());
    if (connected) invoke("controller_close");
  });
</script>

<div class="panel">
  <div class="inner">
    <p class="hint">These settings live on your MidiMaster Deck itself and are saved there automatically.</p>

    <div class="card row">
      <select bind:value={port} disabled={connected} style="flex:1">
        {#each ports as p}<option value={p.name}>{p.product} — {p.name}</option>{:else}<option value="">No MidiMaster Deck found. Plug it in and refresh.</option>{/each}
      </select>
      <button class="icon-btn" onclick={scan} disabled={connected} title="Look for the Deck again"><Icon name="refresh" /></button>
      {#if connected}
        <button class="btn" onclick={disconnect}>Disconnect</button>
      {:else}
        <button class="btn filled has-icon" onclick={connect} disabled={!port}><Icon name="usb" size={18} />Connect</button>
      {/if}
    </div>

    {#if cfg}
      <div class="card">
        <h3>Knobs (encoders)</h3>
        <span class="label">Output</span>
        <div class="seg">
          {#each MODES as [l], i}<button class:sel={cfg.mode === i} onclick={() => set("mode", i)}>{#if cfg.mode === i}<Icon name="check" size={18} />{/if}{l}</button>{/each}
        </div>
        <p class="hint">{MODES[cfg.mode][1]}</p>
        <label class="field">
          <span>Clicks for full range: {cfg.clicks} (≈ {(cfg.clicks / 20).toFixed(1)} turns)</span>
          <input type="range" min="16" max="2000" value={cfg.clicks} oninput={(e) => set("clicks", +e.currentTarget.value)} use:fill={cfg.clicks} />
        </label>
        <label class="field">
          <span>Acceleration when turning fast: {cfg.accel}× {cfg.accel === 1 ? "(off)" : ""}</span>
          <input type="range" min="1" max="16" value={cfg.accel} oninput={(e) => set("accel", +e.currentTarget.value)} use:fill={cfg.accel} />
        </label>
        <label class="check">Reverse direction<input type="checkbox" class="switch" checked={!!cfg.encRev} onchange={(e) => set("encRev", e.currentTarget.checked ? 1 : 0)} /></label>
      </div>

      <div class="card">
        <h3>Joystick</h3>
        <label class="check">Reverse left / right<input type="checkbox" class="switch" checked={!!cfg.panInv} onchange={(e) => set("panInv", e.currentTarget.checked ? 1 : 0)} /></label>
        <label class="check">Reverse up / down<input type="checkbox" class="switch" checked={!!cfg.tiltInv} onchange={(e) => set("tiltInv", e.currentTarget.checked ? 1 : 0)} /></label>
        <label class="field">
          <span>Speed: {cfg.sweep.toFixed(1)} s from 0 to 127 at full push</span>
          <input type="range" min="0.5" max="20" step="0.5" value={cfg.sweep} oninput={(e) => set("sweep", +e.currentTarget.value)} use:fill={cfg.sweep} />
        </label>
        <label class="field">
          <span>Dead zone: {cfg.dead}</span>
          <input type="range" min="0" max="200" value={cfg.dead} oninput={(e) => set("dead", +e.currentTarget.value)} use:fill={cfg.dead} />
        </label>
      </div>

      <div class="card row wrap">
        <button class="btn tonal" onclick={() => send("sync")}>Send all values</button>
        <button class="btn" onclick={() => send("zero")}>Set all knobs to 0</button>
        <button class="btn text" onclick={() => { if (confirm("Reset the MidiMaster Deck settings to defaults?")) send("defaults"); }}>Defaults</button>
        <span class="hint" style="margin:0">{note}</span>
      </div>

      {#if mon}
        <div class="card">
          <h3>Live values on the Deck</h3>
          {#each mon.enc as v, i}
            {@const max = cfg.mode === 0 ? 127 : 16383}
            {@const val = cfg.mode === 0 ? v >> 7 : v}
            <div class="meter"><span>Knob {i + 1}</span><div class="track"><div style="width:{(val / max) * 100}%"></div></div><span class="num">{val}</span></div>
          {/each}
          <div class="meter"><span>Pan</span><div class="track"><div style="width:{(Math.max(0, mon.pan) / 127) * 100}%"></div></div><span class="num">{mon.pan}</span></div>
          <div class="meter"><span>Tilt</span><div class="track"><div style="width:{(Math.max(0, mon.tilt) / 127) * 100}%"></div></div><span class="num">{mon.tilt}</span></div>
        </div>
      {/if}
    {:else if connected}
      <p class="hint">Waiting for the Deck… (it needs the MidiMaster Deck firmware)</p>
    {/if}
  </div>
</div>

<style>
  .panel { flex: 1; overflow-y: auto; }
  .inner { max-width: 720px; padding: 0 24px 48px; }
  .card { background: var(--md-surface-container-lowest); border-radius: 16px; padding: 20px; margin-top: 16px; }
  .card h3 { margin-top: 0; }
  .wrap { flex-wrap: wrap; }
  .label { font: 400 12px/16px Roboto, sans-serif; letter-spacing: 0.4px; color: var(--md-on-surface-variant); display: block; margin: 0 0 8px 4px; }
  .seg { margin-bottom: 8px; flex-wrap: wrap; }
  .meter { display: grid; grid-template-columns: 64px 1fr 64px; gap: 12px; align-items: center; margin-bottom: 10px; font: 500 12px/16px Roboto, sans-serif; }
  .track { height: 8px; background: var(--md-secondary-container); border-radius: 4px; overflow: hidden; }
  .track div { height: 100%; background: var(--md-primary); border-radius: 4px; }
  .num { text-align: right; color: var(--md-on-surface-variant); font-variant-numeric: tabular-nums; }
</style>
