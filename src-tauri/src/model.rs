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

/// The starting profile: a picture of the RP2040 controller this app was made for.
pub fn default_config() -> AppConfig {
    let cc = |n| Some(Binding { kind: BindKind::Cc, channel: 1, number: n });
    let mut controls = Vec::new();
    let mk = |id: &str, kind: &str, label: &str, x, y, w, h, b, by| Control {
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
    };
    let enc_cc = [16u8, 23, 24, 25, 26];
    let btn_cc = [17u8, 27, 28, 29, 30];
    for i in 0..5 {
        let x = 200.0 + i as f64 * 110.0;
        controls.push(mk(&format!("enc{}", i + 1), "encoder", &format!("Knob {}", i + 1), x, 40.0, 80.0, 80.0, cc(enc_cc[i]), None));
        controls.push(mk(&format!("btn{}", i + 1), "button", &format!("Push {}", i + 1), x + 15.0, 135.0, 50.0, 34.0, cc(btn_cc[i]), None));
        controls.push(mk(&format!("fader{}", i + 1), "slider", &format!("Fader {}", i + 1), x + 15.0, 190.0, 50.0, 260.0, cc(7 + i as u8), None));
    }
    controls.push(mk("joy", "joystick", "Joystick", 30.0, 250.0, 140.0, 140.0, cc(20), cc(21)));
    controls.push(mk("joyBtn", "button", "Joy push", 65.0, 420.0, 70.0, 34.0, cc(22), None));
    AppConfig {
        profiles: vec![Profile {
            id: "default".into(),
            name: "My controller".into(),
            apps: vec![],
            layers: default_layers(),
            controls,
        }],
        active_profile: "default".into(),
        auto_switch: true,
        midi_inputs: vec![],
        spotify_client_id: String::new(),
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
