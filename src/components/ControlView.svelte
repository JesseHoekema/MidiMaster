<script lang="ts">
  // Draws one control (knob, fader, joystick, ...) at its current value.
  import type { Control } from "../lib/types";

  let { control, x = 0, y = 0, active = false }: { control: Control; x?: number; y?: number; active?: boolean } = $props();

  const c = $derived(control);
  const mapped = $derived(c.kind === "label" || !!c.binding);
  // knob arc: 270° sweep starting bottom-left
  const R = 40;
  function arc(v: number) {
    const a0 = (135 * Math.PI) / 180;
    const a1 = a0 + (270 * Math.PI) / 180 * Math.max(0, Math.min(1, v));
    const p = (a: number) => `${50 + R * Math.cos(a)} ${50 + R * Math.sin(a)}`;
    return `M ${p(a0)} A ${R} ${R} 0 ${a1 - a0 > Math.PI ? 1 : 0} 1 ${p(a1)}`;
  }
  const angle = $derived(135 + 270 * x);
</script>

<div class="ctl {c.kind}" class:unmapped={!mapped} class:active style="--c:{c.color}">
  {#if c.kind === "knob" || c.kind === "encoder"}
    <svg viewBox="0 0 100 100" class="fill">
      <path d={arc(1)} class="track" />
      {#if x > 0.002}<path d={arc(x)} class="value" />{/if}
      <circle cx="50" cy="50" r="29" class="cap" class:ridged={c.kind === "encoder"} />
      <line x1="50" y1="50" x2={50 + 22 * Math.cos((angle * Math.PI) / 180)} y2={50 + 22 * Math.sin((angle * Math.PI) / 180)} class="pointer" />
    </svg>
  {:else if c.kind === "slider" || c.kind === "sliderH"}
    {@const vert = c.kind === "slider"}
    <div class="slot" class:h={!vert}>
      <div class="fillbar" style={vert ? `height:${x * 100}%` : `width:${x * 100}%`}></div>
      <div class="cap" style={vert ? `bottom:calc(${x * 100}% - 11px)` : `left:calc(${x * 100}% - 11px)`}></div>
    </div>
  {:else if c.kind === "joystick"}
    <div class="joy">
      <div class="cross"></div>
      <div class="dot" style="left:{x * 100}%; top:{(1 - y) * 100}%"></div>
    </div>
  {:else if c.kind === "button" || c.kind === "pad"}
    <div class="press" class:on={x >= 0.5}></div>
  {/if}
  {#if c.kind !== "label"}
    <div class="name">{c.label}</div>
  {:else}
    <div class="text">{c.label || "Text"}</div>
  {/if}
</div>

<style>
  .ctl {
    position: relative;
    width: 100%;
    height: 100%;
    border-radius: 10px;
    transition: box-shadow 0.25s;
  }
  .ctl.active { box-shadow: 0 0 0 2px var(--c), 0 0 18px -2px var(--c); transition: box-shadow 0s; }
  .ctl.unmapped { outline: 1.5px dashed var(--muted); outline-offset: 2px; opacity: 0.75; }
  .name {
    position: absolute;
    left: 50%;
    bottom: -18px;
    transform: translateX(-50%);
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
    pointer-events: none;
  }
  .slider .name { bottom: -28px; }
  .sliderH .name { bottom: -20px; }
  .text { font-weight: 600; font-size: 15px; height: 100%; display: flex; align-items: center; }
  svg.fill { width: 100%; height: 100%; display: block; }
  .track { stroke: var(--device-2); stroke-width: 7; fill: none; stroke-linecap: round; }
  .value { stroke: var(--c); stroke-width: 7; fill: none; stroke-linecap: round; }
  circle.cap { fill: var(--device); stroke: var(--device-2); stroke-width: 2; }
  circle.cap.ridged { stroke-dasharray: 2.5 2.5; stroke-width: 4; }
  .pointer { stroke: var(--device-ink); stroke-width: 4; stroke-linecap: round; }

  .slot { position: absolute; inset: 0; margin: auto; background: var(--device); border-radius: 6px; width: 12px; }
  .slot.h { width: auto; height: 12px; }
  .fillbar { position: absolute; left: 0; bottom: 0; width: 100%; background: var(--c); border-radius: 6px; opacity: 0.85; }
  .slot.h .fillbar { height: 100%; }
  .slot .cap {
    position: absolute;
    left: 50%;
    width: 46px;
    height: 22px;
    margin-left: -23px;
    border-radius: 5px;
    background: linear-gradient(#45474e, #2a2b30);
    box-shadow: 0 2px 6px #0006, inset 0 10px 0 -9px #fff3;
  }
  .slot .cap::after { content: ""; position: absolute; left: 0; right: 0; top: 10px; height: 2px; background: #f2f2f2; }
  .slot.h .cap { left: auto; top: 50%; width: 22px; height: 46px; margin: -23px 0 0 0; }
  .slot.h .cap::after { top: 0; bottom: 0; left: 10px; width: 2px; height: auto; }

  .joy { position: absolute; inset: 0; border-radius: 50%; background: radial-gradient(circle, var(--device-2), var(--device)); }
  .cross { position: absolute; inset: 15%; border-radius: 50%; border: 1px dashed #ffffff22; }
  .dot {
    position: absolute;
    width: 30%;
    height: 30%;
    border-radius: 50%;
    transform: translate(-50%, -50%);
    background: radial-gradient(circle at 35% 35%, #5a5c64, #1e1f23);
    box-shadow: 0 0 0 3px var(--c);
  }
  .press {
    position: absolute;
    inset: 0;
    border-radius: 8px;
    background: var(--device);
    border: 2px solid var(--device-2);
    transition: background 0.12s;
  }
  .pad .press { border-radius: 10px; }
  .press.on { background: var(--c); border-color: var(--c); box-shadow: 0 0 16px -2px var(--c); }
</style>
