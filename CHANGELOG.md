# Changelog

What's new in each version of MidiMaster. When you push a version tag, the section for that
version goes into the GitHub release and into the update popup in the app.

Write a new section at the top before each release, with the same version number as in
`src-tauri/tauri.conf.json`. Use the headings **New**, **Improved** and **Fixed** (leave out the ones you don't need).

## [0.2.0] - 2026-11-23

### New
- **Automatic updates.** MidiMaster checks for a new version when it starts and every few hours.
  When there is one, a popup shows what's new; click **Update** and it downloads, installs and restarts by itself.
- **Check for updates** button and the current version in Settings → About and updates.
- Updates are signed: MidiMaster only installs updates made by this project.

### Improved
- Downloads are named after the platform: `macos_silicon`, `macos_intel` and `windows`,
  so it's clear which file is yours.

### Fixed
- The Windows installers are now always included in the release (they were missing from the first 0.1.0 upload).

## [0.1.0] - 2026-11-22

The first release.

### New
- **Controller builder:** drag knobs, encoders, faders (vertical and horizontal), joysticks, buttons,
  pads and text onto a canvas; move, resize, name and colour them. Drag the empty canvas to move around.
- **Device presets:** pick *MidiMaster Deck* and every control is placed where it sits on the real Deck,
  already mapped. Or start empty. Shows when your Deck is connected.
- **MIDI Learn:** select a control, click Learn, move the real thing. Supports CC, 14-bit CC,
  pitch bend and notes, and binds the moment you move it.
- **Live view and MIDI monitor:** controls follow the hardware and light up when they move;
  the monitor lists every message and marks the ones that aren't mapped.
- **Actions:** system volume and mute, microphone volume and mute, per-app volume (Windows),
  next output device (Mac), screen brightness, media keys, Spotify (play/pause, next, previous, like,
  shuffle, repeat, volume, jump, scrub), keyboard shortcuts, typing text, opening apps, files and
  websites, running commands, and OBS (scenes, mute, source volume, record, stream).
- **Triggers:** follow the value, press, release, long press, double press, turn up, turn down.
- **Layers** (one button switches what every control does) and **profiles** that switch by
  themselves depending on the app in front.
- **OBS connects automatically** by reading its own WebSocket settings; scene and source names
  are suggested in OBS actions.
- **MidiMaster Deck settings:** output mode (7-bit, 14-bit, pitch bend), resolution, acceleration,
  directions and joystick, stored on the Deck itself.
- Material Design 3 look with light and dark mode.
- Runs in the menu bar / system tray with a one-colour icon; optional start at login.
- Installers for Mac Apple Silicon, Mac Intel and Windows.
