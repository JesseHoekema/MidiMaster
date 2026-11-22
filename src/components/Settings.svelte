<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { app, refreshStatus, save, toast } from "../lib/state.svelte";
  import Icon from "./Icon.svelte";

  let inputs = $state<string[]>([]);
  let autostart = $state(false);
  let busy = $state("");
  const isMac = navigator.userAgent.includes("Mac");

  onMount(async () => {
    inputs = await invoke("list_midi_inputs");
    autostart = await isEnabled().catch(() => false);
  });

  function toggleInput(name: string, on: boolean) {
    const c = app.config!;
    // empty list = all inputs; switching one off makes the list explicit
    let list = c.midiInputs.length ? [...c.midiInputs] : [...inputs];
    list = on ? [...new Set([...list, name])] : list.filter((n) => n !== name);
    c.midiInputs = list.length === inputs.length ? [] : list;
    save();
  }
  async function setAutostart(on: boolean) {
    try {
      if (on) await enable();
      else await disable();
      autostart = on;
    } catch (e) {
      toast("error", String(e));
    }
  }
  async function spotify() {
    busy = "spotify";
    save();
    await new Promise((r) => setTimeout(r, 400)); // let the client ID save first
    try {
      await invoke("spotify_connect");
      toast("info", "Spotify connected");
    } catch (e) {
      toast("error", String(e));
    }
    busy = "";
    refreshStatus();
  }
  async function obsTest() {
    busy = "obs";
    save();
    await new Promise((r) => setTimeout(r, 400));
    try {
      toast("info", await invoke<string>("obs_test"));
    } catch (e) {
      toast("error", String(e));
    }
    busy = "";
  }
</script>

{#if app.config}
  {@const c = app.config}
  <div class="panel">
    <div class="inner">

      <div class="card">
        <h3><Icon name="settings_input_component" size={20} /> MIDI inputs</h3>
        {#each inputs as name}
          <label class="check">
            {name}
            <input type="checkbox" class="switch" checked={!c.midiInputs.length || c.midiInputs.includes(name)} onchange={(e) => toggleInput(name, e.currentTarget.checked)} />
          </label>
        {:else}
          <p class="hint">No MIDI devices found. Plug in your controller.</p>
        {/each}
        <button class="btn text small has-icon" onclick={async () => (inputs = await invoke("list_midi_inputs"))}><Icon name="refresh" size={18} />Refresh</button>
        <p class="hint" style="margin-top:8px">New devices are picked up automatically within a couple of seconds.</p>
      </div>

      <div class="card">
        <h3><Icon name="settings" size={20} /> General</h3>
        <label class="check">Start MidiMaster when I log in<input type="checkbox" class="switch" checked={autostart} onchange={(e) => setAutostart(e.currentTarget.checked)} /></label>
        <label class="check">Start in the background (only the tray icon)<input type="checkbox" class="switch" bind:checked={c.startHidden} onchange={save} /></label>
        <label class="check">Switch profiles automatically by the app in front<input type="checkbox" class="switch" bind:checked={c.autoSwitch} onchange={save} /></label>
        <div class="divider"></div>
        <p class="hint">Closing the window keeps MidiMaster running in the {isMac ? "menu bar" : "system tray"}. Quit from there.</p>
        {#if isMac}
          <p class="hint">Media keys, shortcuts and typing need <b>Accessibility</b> permission: System Settings → Privacy &amp; Security → Accessibility → turn on MidiMaster.</p>
        {/if}
      </div>

      <div class="card">
        <h3><Icon name="music_note" size={20} /> Spotify</h3>
        <p class="hint">
          Media keys work without this. The Spotify connection adds like, shuffle, repeat, Spotify volume and seeking.
          Skipping, volume and seeking need Spotify Premium.
        </p>
        <ol class="hint steps">
          <li>Go to <b>developer.spotify.com/dashboard</b> and create an app (any name).</li>
          <li>Add this Redirect URI: <code>{app.status?.spotifyRedirect}</code> and tick “Web API”.</li>
          <li>Copy the app's <b>Client ID</b> here and click Connect.</li>
        </ol>
        <label class="field"><span>Client ID</span><input bind:value={c.spotifyClientId} oninput={save} placeholder="32 letters and numbers" /></label>
        <div class="row">
          {#if app.status?.spotifyConnected}
            <span class="ok">● Connected</span>
            <button class="btn text small" onclick={async () => { await invoke("spotify_disconnect"); refreshStatus(); }}>Disconnect</button>
          {:else}
            <button class="btn filled" onclick={spotify} disabled={!c.spotifyClientId || busy === "spotify"}>
              {busy === "spotify" ? "Waiting for the browser…" : "Connect Spotify"}
            </button>
          {/if}
        </div>
      </div>

      <div class="card">
        <h3><Icon name="videocam" size={20} /> OBS Studio</h3>
        <p class="hint">In OBS: Tools → WebSocket Server Settings → Enable. Copy the password from “Show Connect Info”.</p>
        <div class="row">
          <label class="field" style="flex:2"><span>Address</span><input bind:value={c.obsUrl} oninput={save} /></label>
          <label class="field" style="flex:1"><span>Password</span><input type="password" bind:value={c.obsPassword} oninput={save} /></label>
        </div>
        <button class="btn tonal" onclick={obsTest} disabled={busy === "obs"}>{busy === "obs" ? "Connecting…" : "Test connection"}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .panel { flex: 1; overflow-y: auto; }
  .inner { max-width: 720px; padding: 0 24px 48px; }
  .card { background: var(--md-surface-container-lowest); border-radius: 16px; padding: 20px; margin-bottom: 16px; user-select: text; }
  .card h3 { display: flex; align-items: center; gap: 8px; margin-top: 0; }
  .steps { padding-left: 18px; }
  .steps li { margin-bottom: 4px; }
  .ok { color: var(--md-success); font-weight: 500; }
</style>
