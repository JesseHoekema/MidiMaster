<script lang="ts">
  // Right panel: settings of the selected control: MIDI mapping (with Learn),
  // value shaping, and the actions it triggers.
  import { invoke } from "@tauri-apps/api/core";
  import { app, describeBinding, profile, save, selectedControl, toast, uid } from "../lib/state.svelte";
  import { ACTIONS, TRIGGERS, actionInfo, describeAction, newAction } from "../lib/catalog";
  import type { ActionSpec, ActionType, Axis, Binding, BindKind } from "../lib/types";
  import { fill } from "../lib/range";
  import Icon from "./Icon.svelte";

  const c = $derived(selectedControl());
  const p = $derived(profile());
  const continuousKind = $derived(c && ["knob", "encoder", "slider", "sliderH", "joystick"].includes(c.kind));
  const groups = [...new Set(ACTIONS.map((a) => a.group))];
  const axes = $derived<Axis[]>(c?.kind === "joystick" ? ["x", "y"] : ["x"]);

  function learn(axis: Axis) {
    if (!c) return;
    app.learning = app.learning?.id === c.id && app.learning.axis === axis ? null : { id: c.id, axis };
  }
  function setBinding(axis: Axis, b: Binding | null) {
    if (!c) return;
    if (axis === "y") c.bindingY = b;
    else c.binding = b;
    save();
  }
  function editBinding(axis: Axis, field: keyof Binding, value: string) {
    const cur = (axis === "y" ? c?.bindingY : c?.binding) ?? { kind: "cc" as BindKind, channel: 1, number: 0 };
    const b = { ...cur, [field]: field === "kind" ? value : Math.max(0, Math.min(field === "channel" ? 16 : 127, Number(value) || 0)) };
    setBinding(axis, b as Binding);
  }

  // OBS scene/source names for the suggestions in OBS actions
  let obsLists = $state<{ scenes: string[]; inputs: string[] } | null>(null);
  let obsError = $state("");
  async function loadObs() {
    try {
      obsLists = await invoke("obs_lists");
      obsError = "";
    } catch (e) {
      obsError = String(e);
    }
  }
  const usesObs = $derived(!!c?.actions.some((s) => s.action.type.startsWith("obs")));
  $effect(() => {
    if (usesObs && !obsLists && !obsError) loadObs();
  });

  function addAction(t: string) {
    if (!c || !t) return;
    const info = actionInfo(t as ActionType);
    const spec: ActionSpec = {
      id: uid(),
      layer: null,
      trigger: info.continuous ? "value" : c.kind === "button" || c.kind === "pad" ? "press" : "increase",
      axis: "x",
      action: newAction(info.type),
    };
    c.actions.push(spec);
    save();
  }
  function setActionType(s: ActionSpec, t: ActionType) {
    const info = actionInfo(t);
    s.action = newAction(t);
    if (info.continuous) s.trigger = "value";
    else if (s.trigger === "value") s.trigger = c?.kind === "button" || c?.kind === "pad" ? "press" : "increase";
    save();
  }
  async function test(s: ActionSpec) {
    const v = app.values[c!.id];
    try {
      const msg = await invoke<string | null>("test_action", { action: $state.snapshot(s.action), value: (s.axis === "y" ? v?.y : v?.x) ?? 0.5 });
      toast("info", msg ?? "Done");
    } catch (e) {
      toast("error", String(e));
    }
  }
</script>

<aside class="inspector">
  {#if !c}
    <div class="none">
      <Icon name="touch_app" size={40} />
      <p class="title-medium">Select a control</p>
      <p>Then map it to your hardware and give it actions.</p>
      <p class="hint">Tip: in the Monitor tab you can see every MIDI message, and which ones aren't mapped yet.</p>
    </div>
  {:else}
    <h3>Control</h3>
    <label class="field"><span>Name</span><input bind:value={c.label} oninput={save} /></label>
    <div class="row">
      <label class="field" style="flex:1"><span>Colour</span><input type="color" bind:value={c.color} oninput={save} class="color" /></label>
      <div class="field" style="flex:2">
        <span class="sub">Value now</span>
        <div class="value">
          {#if c.kind === "joystick"}
            X {Math.round((app.values[c.id]?.x ?? 0) * 100)}% · Y {Math.round((app.values[c.id]?.y ?? 0) * 100)}%
          {:else if c.kind !== "label"}
            {Math.round((app.values[c.id]?.x ?? 0) * 100)}%
          {/if}
        </div>
      </div>
    </div>

    {#if c.kind !== "label"}
      <h3>MIDI</h3>
      {#each axes as axis}
        {@const b = axis === "y" ? c.bindingY : c.binding}
        <div class="binding" class:on={!!b}>
          <div class="row">
            <b style="flex:1">{c.kind === "joystick" ? (axis === "x" ? "Left / right: " : "Up / down: ") : ""}{describeBinding(b)}</b>
            <button class="btn small has-icon {app.learning?.id === c.id && app.learning.axis === axis ? 'filled' : 'tonal'}" onclick={() => learn(axis)}>
              <Icon name="sensors" size={18} />{app.learning?.id === c.id && app.learning.axis === axis ? "Move it now…" : "Learn"}
            </button>
            {#if b}<button class="icon-btn small" onclick={() => setBinding(axis, null)} title="Remove mapping"><Icon name="link_off" size={20} /></button>{/if}
          </div>
          <div class="row manual">
            <select value={b?.kind ?? ""} onchange={(e) => editBinding(axis, "kind", e.currentTarget.value)}>
              <option value="" disabled>Type…</option>
              <option value="cc">CC</option>
              <option value="cc14">14-bit CC</option>
              <option value="pb">Pitch bend</option>
              <option value="note">Note</option>
            </select>
            <input type="number" min="0" max="16" title="Channel (0 = any)" value={b?.channel ?? 1} onchange={(e) => editBinding(axis, "channel", e.currentTarget.value)} />
            {#if b?.kind !== "pb"}
              <input type="number" min="0" max="127" title="Number" value={b?.number ?? 0} onchange={(e) => editBinding(axis, "number", e.currentTarget.value)} />
            {/if}
          </div>
          <label class="check left small"><input type="checkbox" checked={axis === "y" ? c.invertY : c.invert} onchange={(e) => { if (axis === "y") c.invertY = e.currentTarget.checked; else c.invert = e.currentTarget.checked; save(); }} /> Reverse direction</label>
        </div>
      {/each}

      {#if continuousKind}
        <h3>Value</h3>
        <div class="row">
          <label class="field"><span>From {Math.round(c.min * 100)}%</span><input type="range" min="0" max="1" step="0.01" bind:value={c.min} oninput={save} use:fill={c.min} /></label>
          <label class="field"><span>To {Math.round(c.max * 100)}%</span><input type="range" min="0" max="1" step="0.01" bind:value={c.max} oninput={save} use:fill={c.max} /></label>
        </div>
        <label class="field">
          <span>Response: {c.curve < 0.95 ? "fast at the start" : c.curve > 1.05 ? "fine at the start (good for volume)" : "straight"}</span>
          <input type="range" min="0.3" max="3" step="0.05" bind:value={c.curve} oninput={save} use:fill={c.curve} />
        </label>
      {/if}

      <h3>Actions</h3>
      {#each c.actions as s (s.id)}
        {@const info = actionInfo(s.action.type)}
        <div class="action">
          <div class="row top">
            <select value={s.action.type} onchange={(e) => setActionType(s, e.currentTarget.value as ActionType)}>
              {#each groups as g}
                <optgroup label={g}>
                  {#each ACTIONS.filter((a) => a.group === g) as a}<option value={a.type}>{a.label}</option>{/each}
                </optgroup>
              {/each}
            </select>
            <button class="icon-btn small" onclick={() => test(s)} title="Test: run it now"><Icon name="bolt" size={20} /></button>
            <button class="icon-btn small" onclick={() => { c.actions = c.actions.filter((x) => x !== s); save(); }} title="Remove action"><Icon name="delete" size={20} /></button>
          </div>
          {#each info.params ?? [] as prm}
            {@const rec = s.action as Record<string, unknown>}
            <label class="field">
              <span>{prm.label}</span>
              {#if prm.kind === "select" && s.action.type === "layerSet"}
                <select value={rec[prm.key]} onchange={(e) => { rec[prm.key] = Number(e.currentTarget.value); save(); }}>
                  {#each p?.layers ?? [] as l, i}<option value={i}>{l}</option>{/each}
                </select>
              {:else if prm.kind === "select"}
                <select value={rec[prm.key]} onchange={(e) => { rec[prm.key] = e.currentTarget.value; save(); }}>
                  {#each prm.options ?? [] as [v, l]}<option value={v}>{l}</option>{/each}
                </select>
              {:else if prm.kind === "number"}
                <input type="number" value={rec[prm.key]} oninput={(e) => { rec[prm.key] = Number(e.currentTarget.value); save(); }} />
              {:else if s.action.type.startsWith("obs")}
                <div class="row">
                  <input
                    value={rec[prm.key]}
                    placeholder={obsLists ? "Pick or type a name" : prm.placeholder}
                    list={prm.key === "name" ? "obs-scenes" : "obs-inputs"}
                    oninput={(e) => { rec[prm.key] = e.currentTarget.value; save(); }}
                  />
                  <button class="icon-btn small" onclick={loadObs} title="Get the names from OBS again"><Icon name="refresh" size={20} /></button>
                </div>
              {:else}
                <input value={rec[prm.key]} placeholder={prm.placeholder} oninput={(e) => { rec[prm.key] = e.currentTarget.value; save(); }} />
              {/if}
            </label>
          {/each}
          {#if info.hint}<p class="hint">{info.hint}</p>{/if}
          {#if s.action.type.startsWith("obs") && obsError}<p class="hint">Couldn't get names from OBS: {obsError}</p>{/if}
          <div class="row">
            <label class="field"><span>When</span>
              {#if info.continuous}
                <select disabled><option>Follows the value</option></select>
              {:else}
                <select bind:value={s.trigger} onchange={save}>
                  {#each TRIGGERS as [t, l]}<option value={t}>{l}</option>{/each}
                </select>
              {/if}
            </label>
            {#if (p?.layers.length ?? 1) > 1}
              <label class="field"><span>Layer</span>
                <select value={s.layer ?? -1} onchange={(e) => { const v = Number(e.currentTarget.value); s.layer = v < 0 ? null : v; save(); }}>
                  <option value={-1}>All layers</option>
                  {#each p?.layers ?? [] as l, i}<option value={i}>{l}</option>{/each}
                </select>
              </label>
            {/if}
            {#if c.kind === "joystick"}
              <label class="field"><span>Axis</span>
                <select bind:value={s.axis} onchange={save}>
                  <option value="x">Left / right</option>
                  <option value="y">Up / down</option>
                </select>
              </label>
            {/if}
          </div>
        </div>
      {/each}
      <select value="" onchange={(e) => { addAction(e.currentTarget.value); e.currentTarget.value = ""; }} class="add">
        <option value="">Add action…</option>
        {#each groups as g}
          <optgroup label={g}>
            {#each ACTIONS.filter((a) => a.group === g) as a}<option value={a.type}>{a.label}</option>{/each}
          </optgroup>
        {/each}
      </select>
      {#if c.actions.length}
        <p class="hint" style="margin-top:8px">{c.actions.map((s) => describeAction(s.action)).join(" · ")}</p>
      {/if}
    {/if}
  {/if}
  <datalist id="obs-scenes">{#each obsLists?.scenes ?? [] as n}<option value={n}></option>{/each}</datalist>
  <datalist id="obs-inputs">{#each obsLists?.inputs ?? [] as n}<option value={n}></option>{/each}</datalist>
</aside>

<style>
  .inspector {
    width: 340px;
    flex: none;
    padding: 4px 24px 24px 16px;
    overflow-y: auto;
    user-select: text;
  }
  .none { color: var(--md-on-surface-variant); padding-top: 48px; text-align: center; display: grid; justify-items: center; gap: 4px; }
  .none .title-medium { color: var(--md-on-surface); margin-top: 8px; }
  .none p { margin: 0; }
  .sub { font: 400 12px/16px Roboto, sans-serif; letter-spacing: 0.4px; color: var(--md-on-surface-variant); padding-left: 4px; }
  .value { font: 500 22px/28px Roboto, sans-serif; font-variant-numeric: tabular-nums; padding: 4px; color: var(--md-primary); }
  .binding { border-radius: 12px; padding: 12px; margin-bottom: 12px; background: var(--md-surface-container-high); }
  .binding.on { background: var(--md-secondary-container); color: var(--md-on-secondary-container); }
  .binding b { font: 500 14px/20px Roboto, sans-serif; }
  .manual { margin: 12px 0 8px; }
  .manual select { flex: 2; }
  .manual input { flex: 1; min-width: 0; }
  .check.small { font-size: 13px; }
  .action { border-radius: 12px; padding: 12px 12px 0; margin-bottom: 12px; background: var(--md-surface-container-high); }
  .action .top { margin-bottom: 12px; }
  .action .top select { flex: 1; font-weight: 500; }
  .row > .field { flex: 1; min-width: 0; }
  .add { border-style: dashed; color: var(--md-primary); font-weight: 500; }
</style>
