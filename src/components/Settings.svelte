<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { app, refreshStatus, save, toast } from "../lib/state.svelte";

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
      <h2>Settings</h2>

      <div class="card">
        <h3>MIDI inputs</h3>
        {#each inputs as name}
          <label class="check">
            <input type="checkbox" checked={!c.midiInputs.length || c.midiInputs.includes(name)} onchange={(e) => toggleInput(name, e.currentTarget.checked)} />
            {name}
          </label>
        {:else}
          <p class="hint">No MIDI devices found. Plug in your controller.</p>
        {/each}
        <button class="btn small" onclick={async () => (inputs = await invoke("list_midi_inputs"))}>Refresh</button>
        <p class="hint" style="margin-top:8px">New devices are picked up automatically within a couple of seconds.</p>
      </div>

      <div class="card">
        <h3>General</h3>
        <label class="check"><input type="checkbox" checked={autostart} onchange={(e) => setAutostart(e.currentTarget.checked)} /> Start MidiMaster when I log in</label>
        <label class="check"><input type="checkbox" bind:checked={c.startHidden} onchange={save} /> Start in the background (only the tray icon)</label>
        <label class="check"><input type="checkbox" bind:checked={c.autoSwitch} onchange={save} /> Switch profiles automatically by the app in front</label>
        <p class="hint">Closing the window keeps MidiMaster running in the {isMac ? "menu bar" : "system tray"}. Quit from there.</p>
        {#if isMac}
          <p class="hint">Media keys, shortcuts and typing need <b>Accessibility</b> permission: System Settings → Privacy &amp; Security → Accessibility → turn on MidiMaster.</p>
        {/if}
      </div>

      <div class="card">
        <h3>Spotify</h3>
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
            <button class="btn small" onclick={async () => { await invoke("spotify_disconnect"); refreshStatus(); }}>Disconnect</button>
          {:else}
            <button class="btn primary" onclick={spotify} disabled={!c.spotifyClientId || busy === "spotify"}>
              {busy === "spotify" ? "Waiting for the browser…" : "Connect Spotify"}
            </button>
          {/if}
        </div>
      </div>

      <div class="card">
        <h3>OBS Studio</h3>
        <p class="hint">In OBS: Tools → WebSocket Server Settings → Enable. Copy the password from “Show Connect Info”.</p>
        <div class="row">
          <label class="field" style="flex:2"><span>Address</span><input bind:value={c.obsUrl} oninput={save} /></label>
          <label class="field" style="flex:1"><span>Password</span><input type="password" bind:value={c.obsPassword} oninput={save} /></label>
        </div>
        <button class="btn" onclick={obsTest} disabled={busy === "obs"}>{busy === "obs" ? "Connecting…" : "Test connection"}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .panel { flex: 1; overflow-y: auto; background: var(--bg); }
  .inner { max-width: 680px; margin: 0 auto; padding: 24px 20px 60px; }
  h2 { margin: 0 0 4px; font-size: 20px; }
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 16px; margin-top: 14px; user-select: text; }
  .steps { padding-left: 18px; }
  .steps li { margin-bottom: 4px; }
  code { background: var(--accent-soft); padding: 1px 5px; border-radius: 4px; font-size: 12px; }
  .ok { color: var(--ok); font-weight: 600; }
</style>
