//! Runs actions. One-shot actions run in order on one worker thread.
//! Continuous actions (volume, brightness...) are coalesced: only the newest value
//! per control is applied, at most every `min_interval`, so a fast knob turn never
//! queues up hundreds of slow system calls or web requests.

pub mod audio;
pub mod display;
pub mod keys;

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde_json::json;

use crate::model::Action;
use crate::Core;

struct Pending {
    action: Action,
    value: f64,
    dirty: bool,
    last: Instant,
}

pub struct Runner {
    tx: Mutex<Sender<(Action, f64)>>,
    pending: Mutex<HashMap<String, Pending>>,
}

impl Runner {
    pub fn new() -> (Runner, Receiver<(Action, f64)>) {
        let (tx, rx) = channel();
        (Runner { tx: Mutex::new(tx), pending: Mutex::new(HashMap::new()) }, rx)
    }

    pub fn fire(&self, action: Action, value: f64) {
        let _ = self.tx.lock().send((action, value));
    }

    pub fn set(&self, key: String, action: Action, value: f64) {
        let mut p = self.pending.lock();
        let e = p.entry(key).or_insert(Pending { action: action.clone(), value, dirty: true, last: Instant::now() - Duration::from_secs(1) });
        e.action = action;
        e.value = value;
        e.dirty = true;
    }
}

fn min_interval(a: &Action) -> Duration {
    Duration::from_millis(match a {
        Action::SpotifyVolume | Action::SpotifySeekPosition => 300,
        #[cfg(target_os = "windows")]
        Action::Brightness => 300,
        Action::AppVolume { .. } | Action::ObsInputVolume { .. } => 30,
        _ => 12,
    })
}

pub fn spawn_workers(core: Arc<Core>, rx: Receiver<(Action, f64)>) {
    let c = core.clone();
    std::thread::spawn(move || {
        for (action, value) in rx {
            report(&c, &action, execute(&c, &action, value));
        }
    });
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(8));
        let due: Vec<(Action, f64)> = {
            let mut p = core.runner.pending.lock();
            let now = Instant::now();
            p.values_mut()
                .filter(|e| e.dirty && now - e.last >= min_interval(&e.action))
                .map(|e| {
                    e.dirty = false;
                    e.last = now;
                    (e.action.clone(), e.value)
                })
                .collect()
        };
        for (action, value) in due {
            report(&core, &action, execute(&core, &action, value));
        }
    });
}

fn report(core: &Core, _action: &Action, r: Result<Option<String>, String>) {
    match r {
        Ok(Some(info)) => core.toast("info", &info),
        Ok(None) => {}
        Err(e) => core.toast("error", &e),
    }
}

fn spawn(cmd: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let child = {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd").args(["/C", cmd]).creation_flags(0x0800_0000).spawn()
    };
    #[cfg(not(target_os = "windows"))]
    let child = std::process::Command::new("sh").args(["-c", cmd]).spawn();
    child.map(|_| ()).map_err(|e| e.to_string())
}

fn launch(target: &str) -> Result<(), String> {
    let t = target.trim();
    if std::path::Path::new(t).exists() || t.contains("://") {
        return open::that(t).map_err(|e| e.to_string());
    }
    // An app name like "Spotify" or "Calculator"
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").args(["-a", t]).status();
    #[cfg(target_os = "windows")]
    let r = {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd").args(["/C", "start", "", t]).creation_flags(0x0800_0000).status()
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let r = std::process::Command::new(t).status();
    match r {
        Ok(s) if s.success() => Ok(()),
        _ => Err(format!("Couldn't open \"{t}\"")),
    }
}

/// Carry out one action. `Ok(Some(text))` shows a short message in the app.
pub fn execute(core: &Core, action: &Action, v: f64) -> Result<Option<String>, String> {
    let cid = || core.config.read().spotify_client_id.clone();
    let obs = |kind: &str, data: serde_json::Value| {
        let (url, pw) = {
            let c = core.config.read();
            (c.obs_url.clone(), c.obs_password.clone())
        };
        core.obs.request(&url, &pw, kind, data)
    };
    match action {
        Action::SystemVolume => audio::set_output_volume(v)?,
        Action::SystemMute => audio::toggle_output_mute()?,
        Action::MicVolume => audio::set_input_volume(v)?,
        Action::MicMute => audio::toggle_input_mute()?,
        Action::AppVolume { app } => audio::set_app_volume(app, v)?,
        Action::OutputDeviceNext => return audio::next_output_device().map(|n| Some(format!("Output: {n}"))),
        Action::Brightness => display::set_brightness(v)?,
        Action::Media { key } => keys::media(*key)?,
        Action::SpotifyPlayPause => core.spotify.play_pause(&cid())?,
        Action::SpotifyNext => core.spotify.next(&cid())?,
        Action::SpotifyPrevious => core.spotify.previous(&cid())?,
        Action::SpotifyLike => return core.spotify.like(&cid()).map(Some),
        Action::SpotifyShuffle => return core.spotify.shuffle(&cid()).map(Some),
        Action::SpotifyRepeat => return core.spotify.repeat(&cid()).map(Some),
        Action::SpotifyVolume => core.spotify.volume(&cid(), v)?,
        Action::SpotifySeekBy { seconds } => core.spotify.seek_by(&cid(), *seconds)?,
        Action::SpotifySeekPosition => core.spotify.seek_to(&cid(), v)?,
        Action::Shortcut { keys } => keys::shortcut(keys)?,
        Action::TypeText { text } => keys::type_text(text)?,
        Action::Launch { target } => launch(target)?,
        Action::OpenUrl { url } => open::that(url.trim()).map_err(|e| e.to_string())?,
        Action::Command { command } => spawn(command)?,
        Action::ObsScene { name } => {
            obs("SetCurrentProgramScene", json!({ "sceneName": name }))?;
        }
        Action::ObsToggleMute { input } => {
            let r = obs("ToggleInputMute", json!({ "inputName": input }))?;
            let muted = r["inputMuted"].as_bool().unwrap_or(false);
            return Ok(Some(format!("{input} {}", if muted { "muted" } else { "unmuted" })));
        }
        Action::ObsInputVolume { input } => {
            obs("SetInputVolume", json!({ "inputName": input, "inputVolumeMul": v.clamp(0.0, 1.0) }))?;
        }
        Action::ObsToggleRecord => {
            let r = obs("ToggleRecord", json!({}))?;
            return Ok(Some(if r["outputActive"].as_bool().unwrap_or(false) { "Recording started" } else { "Recording stopped" }.into()));
        }
        Action::ObsToggleStream => {
            let r = obs("ToggleStream", json!({}))?;
            return Ok(Some(if r["outputActive"].as_bool().unwrap_or(false) { "Stream started" } else { "Stream stopped" }.into()));
        }
        Action::LayerNext => return Ok(Some(core.next_layer())),
        Action::LayerSet { layer } => return Ok(Some(core.set_layer(*layer))),
        Action::ProfileNext => return Ok(Some(core.next_profile())),
    }
    Ok(None)
}
