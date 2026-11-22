<script lang="ts">
  // Right panel: settings of the selected control: MIDI mapping (with Learn),
  // value shaping, and the actions it triggers.
  import { invoke } from "@tauri-apps/api/core";
  import { app, describeBinding, profile, save, selectedControl, toast, uid } from "../lib/state.svelte";
  import { ACTIONS, TRIGGERS, actionInfo, describeAction, newAction } from "../lib/catalog";
  import type { ActionSpec, ActionType, Axis, Binding, BindKind } from "../lib/types";

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
      <p><b>Select a control</b> to map it and give it actions.</p>
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
            <button class="btn small" class:active={app.learning?.id === c.id && app.learning.axis === axis} onclick={() => learn(axis)}>
              {app.learning?.id === c.id && app.learning.axis === axis ? "Move it now…" : "Learn"}
            </button>
            {#if b}<button class="btn small" onclick={() => setBinding(axis, null)} title="Remove mapping">✕</button>{/if}
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
          <label class="check small"><input type="checkbox" checked={axis === "y" ? c.invertY : c.invert} onchange={(e) => { if (axis === "y") c.invertY = e.currentTarget.checked; else c.invert = e.currentTarget.checked; save(); }} /> Reverse direction</label>
        </div>
      {/each}

      {#if continuousKind}
        <h3>Value</h3>
        <div class="row">
          <label class="field"><span>From {Math.round(c.min * 100)}%</span><input type="range" min="0" max="1" step="0.01" bind:value={c.min} oninput={save} /></label>
          <label class="field"><span>To {Math.round(c.max * 100)}%</span><input type="range" min="0" max="1" step="0.01" bind:value={c.max} oninput={save} /></label>
        </div>
        <label class="field">
          <span>Response: {c.curve < 0.95 ? "fast at the start" : c.curve > 1.05 ? "fine at the start (good for volume)" : "straight"}</span>
          <input type="range" min="0.3" max="3" step="0.05" bind:value={c.curve} oninput={save} />
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
            <button class="btn small" onclick={() => test(s)} title="Run it now">Test</button>
            <button class="btn small danger" onclick={() => { c.actions = c.actions.filter((x) => x !== s); save(); }} title="Remove">✕</button>
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
              {:else}
                <input value={rec[prm.key]} placeholder={prm.placeholder} oninput={(e) => { rec[prm.key] = e.currentTarget.value; save(); }} />
              {/if}
            </label>
          {/each}
          {#if info.hint}<p class="hint">{info.hint}</p>{/if}
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
        <option value="">+ Add action…</option>
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
</aside>

<style>
  .inspector {
    width: 330px;
    flex: none;
    border-left: 1px solid var(--line);
    background: var(--panel);
    padding: 16px;
    overflow-y: auto;
    user-select: text;
  }
  .none { color: var(--muted); padding-top: 30px; text-align: center; }
  .sub { font-size: 12px; color: var(--muted); font-weight: 500; }
  .value { font-variant-numeric: tabular-nums; padding: 6px 0; }
  .color { padding: 2px; height: 32px; }
  .binding { border: 1px solid var(--line); border-radius: var(--radius); padding: 10px; margin-bottom: 10px; background: var(--panel-2); }
  .binding.on { border-color: var(--accent); }
  .manual { margin: 8px 0; }
  .manual select { flex: 2; }
  .manual input { flex: 1; }
  .check.small { font-size: 12px; margin: 0; }
  .action { border: 1px solid var(--line); border-radius: var(--radius); padding: 10px 10px 2px; margin-bottom: 10px; background: var(--panel-2); }
  .action .top { margin-bottom: 10px; }
  .action .top select { flex: 1; font-weight: 600; }
  .row > .field { flex: 1; }
  .add { color: var(--accent); font-weight: 600; }
</style>
