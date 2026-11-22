// Mirrors src-tauri/src/model.rs (camelCase JSON).

export type BindKind = "cc" | "cc14" | "pb" | "note";
export interface Binding {
  kind: BindKind;
  channel: number; // 1-16, 0 = any
  number: number;
}

export type Trigger = "value" | "press" | "release" | "longPress" | "doublePress" | "increase" | "decrease";
export type Axis = "x" | "y";
export type MediaKey = "playPause" | "next" | "previous" | "stop" | "volumeUp" | "volumeDown" | "mute";

export type Action =
  | { type: "systemVolume" }
  | { type: "systemMute" }
  | { type: "micVolume" }
  | { type: "micMute" }
  | { type: "appVolume"; app: string }
  | { type: "outputDeviceNext" }
  | { type: "brightness" }
  | { type: "media"; key: MediaKey }
  | { type: "spotifyPlayPause" }
  | { type: "spotifyNext" }
  | { type: "spotifyPrevious" }
  | { type: "spotifyLike" }
  | { type: "spotifyShuffle" }
  | { type: "spotifyRepeat" }
  | { type: "spotifyVolume" }
  | { type: "spotifySeekBy"; seconds: number }
  | { type: "spotifySeekPosition" }
  | { type: "shortcut"; keys: string }
  | { type: "typeText"; text: string }
  | { type: "launch"; target: string }
  | { type: "openUrl"; url: string }
  | { type: "command"; command: string }
  | { type: "obsScene"; name: string }
  | { type: "obsToggleMute"; input: string }
  | { type: "obsInputVolume"; input: string }
  | { type: "obsToggleRecord" }
  | { type: "obsToggleStream" }
  | { type: "layerNext" }
  | { type: "layerSet"; layer: number }
  | { type: "profileNext" };

export type ActionType = Action["type"];

export interface ActionSpec {
  id: string;
  layer: number | null;
  trigger: Trigger;
  axis: Axis;
  action: Action;
}

export type ControlKind = "knob" | "encoder" | "slider" | "sliderH" | "joystick" | "button" | "pad" | "label";

export interface Control {
  id: string;
  kind: ControlKind;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
  color: string;
  binding: Binding | null;
  bindingY: Binding | null;
  invert: boolean;
  invertY: boolean;
  min: number;
  max: number;
  curve: number;
  actions: ActionSpec[];
}

export interface Profile {
  id: string;
  name: string;
  apps: string[];
  layers: string[];
  controls: Control[];
}

export interface AppConfig {
  profiles: Profile[];
  activeProfile: string;
  autoSwitch: boolean;
  midiInputs: string[];
  spotifyClientId: string;
  obsAuto: boolean;
  obsUrl: string;
  obsPassword: string;
  startHidden: boolean;
}

export interface MidiMsg {
  port: string;
  channel: number;
  kind: "cc" | "noteon" | "noteoff" | "pb";
  number: number;
  value: number;
}

export interface Status {
  layer: number;
  frontmost: string;
  spotifyConnected: boolean;
  spotifyRedirect: string;
}

export interface Preset {
  id: string;
  name: string;
  description: string;
  matchInputs: string[];
  profile: Profile;
}

export interface LocalObs {
  found: boolean;
  enabled: boolean;
  port: number;
  auth: boolean;
}
