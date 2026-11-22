<p align="center">
  <img src="app-icon.svg" alt="MidiMaster logo" width="128" height="128">
</p>

<h1 align="center">MidiMaster</h1>

<p align="center">
  Build a picture of your MIDI controller, map every knob, fader, button and joystick,
  and use it to control your computer. Runs on <b>macOS</b> and <b>Windows</b>.
</p>

- **Device presets:** pick *MidiMaster Deck* and every control is placed where it sits on the real Deck, already mapped. Or start empty.
- **Builder:** drag knobs, encoders, faders (vertical and horizontal), joysticks, buttons, pads and text onto a canvas; move, resize, name and colour them.
- **MIDI Learn:** select a control, click *Learn*, move the real thing. Supports CC, 14-bit CC, pitch bend and notes.
- **Live view:** every control follows the hardware and lights up when it moves. Unmapped controls are outlined; the *MIDI monitor* lists every message and which ones nothing listens to.
- **Actions:** system volume, mute, mic volume and mute, per-app volume (Windows), next output device (Mac), screen brightness, media keys, Spotify (play/pause, next, previous, like, shuffle, repeat, volume, jump, scrub), keyboard shortcuts, typing text, opening apps, files and websites, running commands, OBS (scenes, mute, source volume, record, stream).
- **Triggers:** follow the value, press, release, long press, double press, turn up, turn down.
- **Layers:** one button switches what all controls do.
- **Profiles:** switch automatically by the app in front.
- **MidiMaster Deck settings:** the Deck's own settings (output mode, resolution, acceleration, directions, joystick), stored on the Deck.
- Drag the empty canvas (or use the middle mouse button) to move around.
- Runs in the menu bar / system tray with a one-colour icon; optional start at login.

## Download

Every push to `main` builds installers on GitHub (Actions → latest run → *Artifacts*):
`MidiMaster-windows` (`.msi` and `.exe`) and `MidiMaster-macos` (`.dmg`, Intel + Apple Silicon).
Pushing a tag like `v0.1.0` also attaches them to a GitHub release.

The apps aren't code-signed, so the first launch needs one extra step:

- **macOS:** right-click MidiMaster → *Open* → *Open*.
- **Windows:** “Windows protected your PC” → *More info* → *Run anyway*.

## Permissions (macOS)

Media keys, keyboard shortcuts and typing need **Accessibility** permission:
System Settings → Privacy & Security → Accessibility → turn on MidiMaster.

## Spotify

Media keys work for any player without logging in. The Spotify connection adds the extra actions:

1. Create an app at <https://developer.spotify.com/dashboard>.
2. Add the redirect URI `http://127.0.0.1:43821/callback` and tick *Web API*.
3. Paste the Client ID in MidiMaster → Settings → Spotify, then *Connect*.

Playback control through the Web API needs Spotify Premium.

## OBS

MidiMaster connects to OBS automatically: it reads the port and password from OBS's own settings,
the way other control apps do. You only have to switch the server on once:
OBS → Tools → WebSocket Server Settings → *Enable WebSocket server*.
In OBS actions you can then pick your scenes and sources from a list.

## Limits

- Per-app volume is Windows-only (macOS has no API for it without an extra audio driver).
- Switching the output device is Mac-only for now.
- External monitor brightness works on Windows when the monitor supports DDC/CI; on Mac only built-in and Apple displays.

## Development

Needs Node 20+ and Rust.

```bash
npm install
npx tauri dev
```

```bash
cd src-tauri && cargo test --lib
```

Code: `src-tauri/src` (Rust: MIDI, mapping engine, actions, Spotify, OBS, serial) and `src` (Svelte interface).
Settings live in `config.json` in the app's config folder.
