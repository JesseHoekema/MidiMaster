//! Data model shared with the frontend (serialised as camelCase JSON).

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BindKind {
    /// 7-bit control change, 0-127
    Cc,
    /// 14-bit control change: MSB on `number`, LSB on `number + 32`
    Cc14,
    /// Pitch bend, 0-16383
    Pb,
    /// Note on/off (velocity 0 or note off = released)
    Note,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub kind: BindKind,
    /// MIDI channel 1-16, 0 = any channel
    pub channel: u8,
    pub number: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    /// Continuous: the control's value (0-1) drives the action
    #[default]
    Value,
    Press,
    Release,
    LongPress,
    DoublePress,
    /// Fires once for every step the value goes up
    Increase,
    /// Fires once for every step the value goes down
    Decrease,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    #[default]
    X,
    Y,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Action {
    // --- audio
    SystemVolume,
    SystemMute,
    MicVolume,
    MicMute,
    AppVolume { app: String },
    OutputDeviceNext,
    // --- display
    Brightness,
    // --- media keys (work with any player)
    Media { key: MediaKey },
    // --- Spotify Web API
    SpotifyPlayPause,
    SpotifyNext,
    SpotifyPrevious,
    SpotifyLike,
    SpotifyShuffle,
    SpotifyRepeat,
    SpotifyVolume,
    SpotifySeekBy { seconds: i32 },
    SpotifySeekPosition,
    // --- keyboard and apps
    Shortcut { keys: String },
    TypeText { text: String },
    Launch { target: String },
    OpenUrl { url: String },
    Command { command: String },
    // --- OBS (obs-websocket 5)
    ObsScene { name: String },
    ObsToggleMute { input: String },
    ObsInputVolume { input: String },
    ObsToggleRecord,
    ObsToggleStream,
    // --- MidiMaster itself
    LayerNext,
    LayerSet { layer: u8 },
    ProfileNext,
}

impl Action {
    /// Continuous actions take a 0-1 value; the rest fire once.
    pub fn is_continuous(&self) -> bool {
        matches!(
            self,
            Action::SystemVolume
                | Action::MicVolume
                | Action::AppVolume { .. }
                | Action::Brightness
                | Action::SpotifyVolume
                | Action::SpotifySeekPosition
                | Action::ObsInputVolume { .. }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MediaKey {
    PlayPause,
    Next,
    Previous,
    Stop,
    VolumeUp,
    VolumeDown,
    Mute,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionSpec {
    pub id: String,
    /// None = active in every layer
    #[serde(default)]
    pub layer: Option<u8>,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub axis: Axis,
    pub action: Action,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Control {
    pub id: String,
    /// knob | slider | sliderH | joystick | button | encoder | pad | label
    pub kind: String,
    #[serde(default)]
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub binding: Option<Binding>,
    /// Second axis of a joystick
    #[serde(default)]
    pub binding_y: Option<Binding>,
    #[serde(default)]
    pub invert: bool,
    #[serde(default)]
    pub invert_y: bool,
    /// Output range: the control's 0-1 is mapped onto min..max before actions see it
    #[serde(default)]
    pub min: f64,
    #[serde(default = "one")]
    pub max: f64,
    /// 1 = linear, <1 = fast start, >1 = slow start (good for volume)
    #[serde(default = "one")]
    pub curve: f64,
    #[serde(default)]
    pub actions: Vec<ActionSpec>,
}

fn default_color() -> String {
    "#2f6fde".into()
}
fn one() -> f64 {
    1.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    /// Switch to this profile automatically when one of these apps is in front
    #[serde(default)]
    pub apps: Vec<String>,
    #[serde(default = "default_layers")]
    pub layers: Vec<String>,
    #[serde(default)]
    pub controls: Vec<Control>,
}

fn default_layers() -> Vec<String> {
    vec!["Main".into()]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub profiles: Vec<Profile>,
    pub active_profile: String,
    #[serde(default = "yes")]
    pub auto_switch: bool,
    /// MIDI input names to listen to; empty = all
    #[serde(default)]
    pub midi_inputs: Vec<String>,
    #[serde(default)]
    pub spotify_client_id: String,
    /// Read the OBS WebSocket port and password from OBS's own settings file
    #[serde(default = "yes")]
    pub obs_auto: bool,
    #[serde(default = "default_obs_url")]
    pub obs_url: String,
    #[serde(default)]
    pub obs_password: String,
    #[serde(default)]
    pub start_hidden: bool,
}

fn yes() -> bool {
    true
}
fn default_obs_url() -> String {
    "ws://127.0.0.1:4455".into()
}

impl AppConfig {
    pub fn active(&self) -> Option<&Profile> {
        self.profiles
            .iter()
            .find(|p| p.id == self.active_profile)
            .or(self.profiles.first())
    }
}

/// A ready-made layout for a known device.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    /// MIDI input names (lower case, substring match) that belong to this device
    pub match_inputs: Vec<String>,
    pub profile: Profile,
}

fn control(id: &str, kind: &str, label: &str, x: f64, y: f64, w: f64, h: f64, b: Option<Binding>, by: Option<Binding>) -> Control {
    Control {
        id: id.into(),
        kind: kind.into(),
        label: label.into(),
        x,
        y,
        w,
        h,
        color: default_color(),
        binding: b,
        binding_y: by,
        invert: false,
        invert_y: false,
        min: 0.0,
        max: 1.0,
        curve: 1.0,
        actions: vec![],
    }
}

/// The MidiMaster Deck (RP2040-Zero PCB): 5 encoders with push buttons, 5 faders,
/// joystick with button. Horizontal positions follow the PCB (4 px per mm):
/// columns 26 mm apart, joystick 28.5 mm left of the first column, near the front.
pub fn deck_profile() -> Profile {
    let cc = |n| Some(Binding { kind: BindKind::Cc, channel: 1, number: n });
    let enc_cc = [16u8, 23, 24, 25, 26];
    let btn_cc = [17u8, 27, 28, 29, 30];
    let mut controls = Vec::new();
    for i in 0..5 {
        let cx = 242.0 + 104.0 * i as f64; // column centre: (94 + 26 i - 43.45) mm * 4 + 40
        let n = i + 1;
        controls.push(control(&format!("enc{n}"), "encoder", &format!("Knob {n}"), cx - 36.0, 40.0, 72.0, 72.0, cc(enc_cc[i]), None));
        controls.push(control(&format!("btn{n}"), "button", &format!("Push {n}"), cx - 28.0, 136.0, 56.0, 28.0, cc(btn_cc[i]), None));
        controls.push(control(&format!("fader{n}"), "slider", &format!("Fader {n}"), cx - 25.0, 204.0, 50.0, 240.0, cc(7 + i as u8), None));
    }
    controls.push(control("joy", "joystick", "Joystick", 64.0, 348.0, 128.0, 128.0, cc(20), cc(21)));
    controls.push(control("joyBtn", "button", "Joy push", 96.0, 504.0, 64.0, 28.0, cc(22), None));
    Profile { id: "deck".into(), name: "MidiMaster Deck".into(), apps: vec![], layers: default_layers(), controls }
}

pub fn presets() -> Vec<Preset> {
    vec![
        Preset {
            id: "midimaster-deck".into(),
            name: "MidiMaster Deck".into(),
            description: "5 knobs with push, 5 faders and a joystick, laid out like the real Deck and already mapped.".into(),
            match_inputs: vec!["midimaster".into(), "rp2040".into()],
            profile: deck_profile(),
        },
        Preset {
            id: "empty".into(),
            name: "Empty".into(),
            description: "Start from nothing and drag in your own controls.".into(),
            match_inputs: vec![],
            profile: Profile { id: "empty".into(), name: "New profile".into(), apps: vec![], layers: default_layers(), controls: vec![] },
        },
    ]
}

pub fn default_config() -> AppConfig {
    let mut deck = deck_profile();
    deck.id = "default".into();
    AppConfig {
        profiles: vec![deck],
        active_profile: "default".into(),
        auto_switch: true,
        midi_inputs: vec![],
        spotify_client_id: String::new(),
        obs_auto: true,
        obs_url: default_obs_url(),
        obs_password: String::new(),
        start_hidden: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips_through_json() {
        let json = serde_json::to_string(&default_config()).unwrap();
        let back: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.profiles[0].controls.len(), 17);
        if std::env::var("PRINT_DEFAULT").is_ok() {
            println!("{json}");
        }
    }

    #[test]
    fn actions_use_tagged_camel_case() {
        let a: Action = serde_json::from_str(r#"{"type":"appVolume","app":"spotify"}"#).unwrap();
        assert_eq!(a, Action::AppVolume { app: "spotify".into() });
        let s = serde_json::to_string(&Action::SpotifySeekBy { seconds: -10 }).unwrap();
        assert_eq!(s, r#"{"type":"spotifySeekBy","seconds":-10}"#);
    }
}
