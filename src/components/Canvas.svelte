<script lang="ts">
  // The picture of the controller. Edit mode: drag, resize, select, drop new
  // controls from the palette. Live mode: everything follows the hardware.
  import { onMount } from "svelte";
  import ControlView from "./ControlView.svelte";
  import Icon from "./Icon.svelte";
  import { app, profile, save, uid } from "../lib/state.svelte";
  import { CONTROL_KINDS } from "../lib/catalog";
  import type { Control, ControlKind } from "../lib/types";

  const GRID = 10;
  const snap = (v: number) => Math.round(v / GRID) * GRID;

  let now = $state(performance.now());
  let area: HTMLDivElement;

  onMount(() => {
    const t = setInterval(() => (now = performance.now()), 80);
    const key = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement)?.tagName;
      if (!app.editing || !app.selected || tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
      if (e.key === "Backspace" || e.key === "Delete") {
        remove(app.selected);
        e.preventDefault();
      }
      const c = profile()?.controls.find((c) => c.id === app.selected);
      const d: Record<string, [number, number]> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
      if (c && d[e.key]) {
        const step = e.shiftKey ? GRID * 5 : GRID;
        c.x = Math.max(0, c.x + d[e.key][0] * step);
        c.y = Math.max(0, c.y + d[e.key][1] * step);
        save();
        e.preventDefault();
      }
    };
    window.addEventListener("keydown", key);
    return () => {
      clearInterval(t);
      window.removeEventListener("keydown", key);
    };
  });

  function remove(id: string) {
    const p = profile();
    if (!p) return;
    p.controls = p.controls.filter((c) => c.id !== id);
    app.selected = null;
    save();
  }

  export function addControl(kind: ControlKind, x?: number, y?: number) {
    const p = profile();
    if (!p) return;
    const k = CONTROL_KINDS.find((k) => k.kind === kind)!;
    const n = p.controls.filter((c) => c.kind === kind).length + 1;
    const c: Control = {
      id: uid(),
      kind,
      label: kind === "label" ? "Text" : `${k.label} ${n}`,
      x: snap(x ?? 40 + area.scrollLeft),
      y: snap(y ?? 40 + area.scrollTop),
      w: k.w,
      h: k.h,
      color: "#2f6fde",
      binding: null,
      bindingY: null,
      invert: false,
      invertY: false,
      min: 0,
      max: 1,
      curve: 1,
      actions: [],
    };
    p.controls.push(c);
    app.selected = c.id;
    save();
  }

  // ---- dragging and resizing
  let drag: { id: string; mode: "move" | "size"; sx: number; sy: number; ox: number; oy: number; ow: number; oh: number } | null = null;

  function down(e: PointerEvent, c: Control, mode: "move" | "size") {
    if (e.button === 1) return; // middle button: let the canvas pan
    e.stopPropagation();
    app.selected = c.id;
    if (!app.editing) return;
    drag = { id: c.id, mode, sx: e.clientX, sy: e.clientY, ox: c.x, oy: c.y, ow: c.w, oh: c.h };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!drag) return;
    const c = profile()?.controls.find((c) => c.id === drag!.id);
    if (!c) return;
    const dx = e.clientX - drag.sx;
    const dy = e.clientY - drag.sy;
    if (drag.mode === "move") {
      c.x = Math.max(0, snap(drag.ox + dx));
      c.y = Math.max(0, snap(drag.oy + dy));
    } else {
      c.w = Math.max(30, snap(drag.ow + dx));
      c.h = Math.max(20, snap(drag.oh + dy));
      if (c.kind === "knob" || c.kind === "encoder" || c.kind === "joystick" || c.kind === "pad") c.h = c.w;
    }
  }
  function up() {
    if (drag) save();
    drag = null;
  }

  // ---- panning: drag the empty canvas (or use the middle mouse button anywhere)
  let pan: { sx: number; sy: number; l: number; t: number; moved: boolean } | null = $state(null);
  function panDown(e: PointerEvent) {
    if (e.button !== 0 && e.button !== 1) return;
    pan = { sx: e.clientX, sy: e.clientY, l: area.scrollLeft, t: area.scrollTop, moved: false };
    area.setPointerCapture(e.pointerId);
  }
  function panMove(e: PointerEvent) {
    if (!pan) return;
    const dx = e.clientX - pan.sx;
    const dy = e.clientY - pan.sy;
    if (Math.abs(dx) + Math.abs(dy) > 3) pan.moved = true;
    area.scrollLeft = pan.l - dx;
    area.scrollTop = pan.t - dy;
  }
  function panUp() {
    if (pan && !pan.moved) app.selected = null; // a plain click on the background deselects
    pan = null;
  }

  function drop(e: DragEvent) {
    e.preventDefault();
    const kind = e.dataTransfer?.getData("text/midimaster-kind") as ControlKind;
    if (!kind) return;
    const r = area.getBoundingClientRect();
    const k = CONTROL_KINDS.find((k) => k.kind === kind)!;
    addControl(kind, e.clientX - r.left + area.scrollLeft - k.w / 2, e.clientY - r.top + area.scrollTop - k.h / 2);
  }

  const size = $derived.by(() => {
    const cs = profile()?.controls ?? [];
    return {
      w: Math.max(...cs.map((c) => c.x + c.w), 0) + 900,
      h: Math.max(...cs.map((c) => c.y + c.h), 0) + 700,
    };
  });
</script>

<div
  class="area"
  class:editing={app.editing}
  bind:this={area}
  role="application"
  ondragover={(e) => e.preventDefault()}
  ondrop={drop}
  class:panning={pan?.moved}
  onpointerdown={panDown}
  onpointermove={panMove}
  onpointerup={panUp}
  onpointercancel={panUp}
>
  <div class="board" style="width:{size.w}px; height:{size.h}px">
    {#each profile()?.controls ?? [] as c (c.id)}
      {@const v = app.values[c.id]}
      {@const rest = c.kind === "joystick" ? 0.5 : 0}
      <div
        class="item"
        class:selected={app.selected === c.id}
        class:learning={app.learning?.id === c.id}
        style="left:{c.x}px; top:{c.y}px; width:{c.w}px; height:{c.h}px"
        role="button"
        tabindex="-1"
        onpointerdown={(e) => down(e, c, "move")}
        onpointermove={move}
        onpointerup={up}
      >
        <ControlView control={c} x={v?.x ?? rest} y={v?.y ?? rest} active={!!v && now - v.t < 260} />
        {#if app.editing && app.selected === c.id}
          <div
            class="handle"
            role="separator"
            onpointerdown={(e) => down(e, c, "size")}
            onpointermove={move}
            onpointerup={up}
          ></div>
        {/if}
      </div>
    {/each}
  </div>
  {#if (profile()?.controls.length ?? 0) === 0}
    <div class="empty">
      <div>
        <p>Drag controls from the left onto this area to build your controller,</p>
        <button class="btn tonal has-icon" onclick={() => (app.presetDialog = "replace")}><Icon name="tune" size={18} />or start from a device preset</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .area {
    position: relative;
    flex: 1;
    overflow: auto;
    margin: 0 0 16px;
    border-radius: 16px;
    background: var(--md-surface-container-lowest);
  }
  .area.editing {
    background-image: radial-gradient(var(--md-outline-variant) 1px, transparent 1px);
    background-size: 10px 10px;
  }
  .board { position: relative; }
  .area { cursor: grab; }
  .area.panning, .area.panning .item { cursor: grabbing; }
  .item { position: absolute; touch-action: none; }
  .editing .item { cursor: grab; }
  .item.selected { outline: 2px solid var(--md-primary); outline-offset: 6px; border-radius: 12px; }
  .item.learning { outline: 2px dashed var(--md-primary); outline-offset: 6px; animation: pulse 0.9s infinite; border-radius: 12px; }
  @keyframes pulse { 50% { outline-color: transparent; } }
  .handle {
    position: absolute;
    right: -12px;
    bottom: -12px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--md-primary);
    border: 3px solid var(--md-surface-container-lowest);
    cursor: nwse-resize;
  }
  .empty {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: var(--md-on-surface-variant);
    text-align: center;
    pointer-events: none;
  }
  .empty button { pointer-events: auto; }
</style>
