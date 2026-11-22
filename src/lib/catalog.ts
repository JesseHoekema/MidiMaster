// Everything the user can add: control types and actions.
import type { Action, ActionType, ControlKind, Trigger } from "./types";

export const CONTROL_KINDS: { kind: ControlKind; label: string; w: number; h: number }[] = [
  { kind: "knob", label: "Knob", w: 80, h: 80 },
  { kind: "encoder", label: "Encoder", w: 80, h: 80 },
  { kind: "slider", label: "Fader", w: 50, h: 240 },
  { kind: "sliderH", label: "Horizontal fader", w: 240, h: 50 },
  { kind: "joystick", label: "Joystick", w: 140, h: 140 },
  { kind: "button", label: "Button", w: 70, h: 40 },
  { kind: "pad", label: "Pad", w: 80, h: 80 },
  { kind: "label", label: "Text", w: 140, h: 30 },
];

export type Param = { key: string; label: string; kind: "text" | "number" | "select"; placeholder?: string; options?: [string, string][] };

export interface ActionInfo {
  type: ActionType;
  label: string;
  group: string;
  continuous: boolean;
  params?: Param[];
  hint?: string;
}

export const MEDIA_KEYS: [string, string][] = [
  ["playPause", "Play / pause"],
  ["next", "Next track"],
  ["previous", "Previous track"],
  ["stop", "Stop (Windows)"],
  ["volumeUp", "Volume up"],
  ["volumeDown", "Volume down"],
  ["mute", "Mute"],
];

export const ACTIONS: ActionInfo[] = [
  { type: "systemVolume", label: "System volume", group: "Audio", continuous: true },
  { type: "systemMute", label: "Mute / unmute", group: "Audio", continuous: false },
  { type: "micVolume", label: "Microphone volume", group: "Audio", continuous: true },
  { type: "micMute", label: "Microphone mute", group: "Audio", continuous: false },
  { type: "appVolume", label: "App volume (Windows)", group: "Audio", continuous: true,
    params: [{ key: "app", label: "App", kind: "text", placeholder: "spotify, discord, chrome…" }],
    hint: "The program's name, like spotify or Discord.exe. Not possible on macOS." },
  { type: "outputDeviceNext", label: "Next output device (Mac)", group: "Audio", continuous: false },
  { type: "brightness", label: "Screen brightness", group: "Display", continuous: true,
    hint: "Mac: built-in and Apple displays. Windows: laptop screen and monitors with DDC/CI." },
  { type: "media", label: "Media key", group: "Media (any player)", continuous: false,
    params: [{ key: "key", label: "Key", kind: "select", options: MEDIA_KEYS }],
    hint: "Works with Spotify, Apple Music, YouTube and more. On Mac it needs Accessibility permission." },
  { type: "spotifyPlayPause", label: "Play / pause", group: "Spotify", continuous: false },
  { type: "spotifyNext", label: "Next song", group: "Spotify", continuous: false },
  { type: "spotifyPrevious", label: "Previous song", group: "Spotify", continuous: false },
  { type: "spotifyLike", label: "Like / unlike song", group: "Spotify", continuous: false },
  { type: "spotifyShuffle", label: "Shuffle on / off", group: "Spotify", continuous: false },
  { type: "spotifyRepeat", label: "Repeat mode", group: "Spotify", continuous: false },
  { type: "spotifyVolume", label: "Spotify volume", group: "Spotify", continuous: true },
  { type: "spotifySeekBy", label: "Jump forward / back", group: "Spotify", continuous: false,
    params: [{ key: "seconds", label: "Seconds (negative = back)", kind: "number" }] },
  { type: "spotifySeekPosition", label: "Scrub through song", group: "Spotify", continuous: true },
  { type: "shortcut", label: "Keyboard shortcut", group: "Keyboard & apps", continuous: false,
    params: [{ key: "keys", label: "Keys", kind: "text", placeholder: "cmd+shift+4, ctrl+c, f5…" }],
    hint: "Join keys with +. Names: cmd/win, ctrl, alt/option, shift, space, enter, tab, esc, up, down, left, right, f1–f12." },
  { type: "typeText", label: "Type text", group: "Keyboard & apps", continuous: false,
    params: [{ key: "text", label: "Text", kind: "text" }] },
  { type: "launch", label: "Open app or file", group: "Keyboard & apps", continuous: false,
    params: [{ key: "target", label: "App name or path", kind: "text", placeholder: "Spotify, Calculator, C:\\…\\app.exe" }] },
  { type: "openUrl", label: "Open website", group: "Keyboard & apps", continuous: false,
    params: [{ key: "url", label: "URL", kind: "text", placeholder: "https://…" }] },
  { type: "command", label: "Run command", group: "Keyboard & apps", continuous: false,
    params: [{ key: "command", label: "Command", kind: "text", placeholder: "say hello" }],
    hint: "Runs in sh (Mac) or cmd (Windows)." },
  { type: "obsScene", label: "Switch scene", group: "OBS", continuous: false,
    params: [{ key: "name", label: "Scene name", kind: "text" }] },
  { type: "obsToggleMute", label: "Mute / unmute source", group: "OBS", continuous: false,
    params: [{ key: "input", label: "Source name", kind: "text", placeholder: "Mic/Aux" }] },
  { type: "obsInputVolume", label: "Source volume", group: "OBS", continuous: true,
    params: [{ key: "input", label: "Source name", kind: "text", placeholder: "Desktop Audio" }] },
  { type: "obsToggleRecord", label: "Start / stop recording", group: "OBS", continuous: false },
  { type: "obsToggleStream", label: "Start / stop stream", group: "OBS", continuous: false },
  { type: "layerNext", label: "Next layer", group: "MidiMaster", continuous: false },
  { type: "layerSet", label: "Go to layer", group: "MidiMaster", continuous: false,
    params: [{ key: "layer", label: "Layer", kind: "select" }] },
  { type: "profileNext", label: "Next profile", group: "MidiMaster", continuous: false },
];

export const actionInfo = (t: ActionType) => ACTIONS.find((a) => a.type === t)!;

export function newAction(t: ActionType): Action {
  const a: Record<string, unknown> = { type: t };
  for (const p of actionInfo(t).params ?? []) {
    a[p.key] = p.kind === "number" ? (t === "spotifySeekBy" ? 10 : 0) : p.kind === "select" ? (t === "media" ? "playPause" : 0) : "";
  }
  return a as Action;
}

export const TRIGGERS: [Trigger, string][] = [
  ["press", "Press"],
  ["release", "Release"],
  ["longPress", "Long press"],
  ["doublePress", "Double press"],
  ["increase", "Turn up / right"],
  ["decrease", "Turn down / left"],
];

export function describeAction(a: Action): string {
  const info = actionInfo(a.type);
  const p = info.params?.[0];
  let extra = "";
  if (p) {
    const v = (a as Record<string, unknown>)[p.key];
    if (a.type === "media") extra = MEDIA_KEYS.find((k) => k[0] === v)?.[1] ?? "";
    else if (a.type !== "layerSet" && v !== "" && v !== undefined) extra = String(v);
  }
  return `${info.group === "Spotify" ? "Spotify: " : info.group === "OBS" ? "OBS: " : ""}${info.label}${extra ? ` — ${extra}` : ""}`;
}
