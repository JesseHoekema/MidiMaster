// Global app state (Svelte 5 runes) plus the link to the Rust backend.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppConfig, Axis, Binding, Control, MidiMsg, Profile, Status } from "./types";

export type Tab = "builder" | "monitor" | "controller" | "settings";
export interface LogEntry extends MidiMsg {
  t: number;
  mapped: string | null; // label of the control it moves, if any
}

export const app = $state({
  config: null as AppConfig | null,
  values: {} as Record<string, { x: number; y: number; t: number }>,
  log: [] as LogEntry[],
  selected: null as string | null,
  editing: true,
  layer: 0,
  status: null as Status | null,
  learning: null as { id: string; axis: Axis } | null,
  toasts: [] as { id: number; kind: string; text: string }[],
  tab: "builder" as Tab,
});

export function profile(): Profile | undefined {
  const c = app.config;
  if (!c) return undefined;
  return c.profiles.find((p) => p.id === c.activeProfile) ?? c.profiles[0];
}

export function selectedControl(): Control | undefined {
  return profile()?.controls.find((c) => c.id === app.selected);
}

export const uid = () => Math.random().toString(36).slice(2, 10);

// ---------------------------------------------------------------- saving
let saveTimer: ReturnType<typeof setTimeout> | undefined;
export function save() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    if (app.config) invoke("save_config", { config: $state.snapshot(app.config) });
  }, 300);
}

export function toast(kind: string, text: string) {
  const id = Date.now() + Math.random();
  app.toasts.push({ id, kind, text });
  setTimeout(() => (app.toasts = app.toasts.filter((t) => t.id !== id)), kind === "error" ? 6000 : 3000);
}

// ---------------------------------------------------------------- MIDI matching
export function matches(b: Binding | null, m: MidiMsg): boolean {
  if (!b || (b.channel !== 0 && b.channel !== m.channel)) return false;
  switch (b.kind) {
    case "cc":
      return m.kind === "cc" && m.number === b.number;
    case "cc14":
      return m.kind === "cc" && (m.number === b.number || m.number === b.number + 32);
    case "pb":
      return m.kind === "pb";
    case "note":
      return (m.kind === "noteon" || m.kind === "noteoff") && m.number === b.number;
  }
}

export function describeBinding(b: Binding | null): string {
  if (!b) return "Not mapped";
  const ch = b.channel === 0 ? "any ch" : `ch ${b.channel}`;
  switch (b.kind) {
    case "cc":
      return `CC ${b.number} · ${ch}`;
    case "cc14":
      return `14-bit CC ${b.number}+${b.number + 32} · ${ch}`;
    case "pb":
      return `Pitch bend · ${ch}`;
    case "note":
      return `Note ${b.number} · ${ch}`;
  }
}

// ---------------------------------------------------------------- MIDI learn
let pendingMsb: { m: MidiMsg; timer: ReturnType<typeof setTimeout> } | null = null;

function bind(b: Binding) {
  const l = app.learning;
  const c = profile()?.controls.find((c) => c.id === l?.id);
  if (!l || !c) return;
  if (l.axis === "y") c.bindingY = b;
  else c.binding = b;
  app.learning = null;
  save();
  toast("info", `${c.label || "Control"} → ${describeBinding(b)}`);
}

function learn(m: MidiMsg) {
  if (m.kind === "noteoff") return;
  if (m.kind === "pb") return bind({ kind: "pb", channel: m.channel, number: 0 });
  if (m.kind === "noteon") return bind({ kind: "note", channel: m.channel, number: m.number });
  // CC: a CC below 32 followed right away by CC+32 is a 14-bit pair
  if (pendingMsb && m.number === pendingMsb.m.number + 32 && m.channel === pendingMsb.m.channel) {
    clearTimeout(pendingMsb.timer);
    const n = pendingMsb.m.number;
    pendingMsb = null;
    return bind({ kind: "cc14", channel: m.channel, number: n });
  }
  if (pendingMsb && m.number === pendingMsb.m.number && m.channel === pendingMsb.m.channel) {
    clearTimeout(pendingMsb.timer);
    pendingMsb = null;
    return bind({ kind: "cc", channel: m.channel, number: m.number });
  }
  if (m.number < 32) {
    if (pendingMsb) clearTimeout(pendingMsb.timer);
    const msb = m;
    pendingMsb = {
      m,
      timer: setTimeout(() => {
        pendingMsb = null;
        bind({ kind: "cc", channel: msb.channel, number: msb.number });
      }, 60),
    };
    return;
  }
  bind({ kind: "cc", channel: m.channel, number: m.number });
}

// ---------------------------------------------------------------- start
export async function refreshStatus() {
  app.status = await invoke<Status>("get_status");
  app.layer = app.status.layer;
}

async function reloadConfig() {
  app.config = await invoke<AppConfig>("get_config");
  app.values = {};
  for (const v of await invoke<{ id: string; x: number; y: number }[]>("control_snapshot")) {
    app.values[v.id] = { x: v.x, y: v.y, t: 0 };
  }
}

export async function init() {
  await reloadConfig();
  await refreshStatus();

  await listen<MidiMsg>("midi", (e) => {
    const m = e.payload;
    const c = profile()?.controls.find((c) => matches(c.binding, m) || matches(c.bindingY, m));
    app.log.unshift({ ...m, t: Date.now(), mapped: c ? c.label || c.kind : null });
    if (app.log.length > 300) app.log.length = 300;
    if (app.learning) learn(m);
  });
  await listen<{ id: string; x: number; y: number }>("control", (e) => {
    app.values[e.payload.id] = { x: e.payload.x, y: e.payload.y, t: performance.now() };
  });
  await listen<number>("layer", (e) => (app.layer = e.payload));
  await listen<string>("profile", async () => {
    await reloadConfig();
    app.selected = null;
  });
  await listen<{ kind: string; text: string }>("toast", (e) => toast(e.payload.kind, e.payload.text));
  await listen<string>("frontmost", (e) => {
    if (app.status) app.status.frontmost = e.payload;
  });
}
