mod actions;
mod engine;
mod midi;
mod model;
mod obs;
mod serial;
mod spotify;

use std::path::PathBuf;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use serde::Serialize;
use serde_json::json;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};

use actions::Runner;
use engine::{Effect, Engine};
use model::{Action, AppConfig};

pub struct Core {
    app: AppHandle,
    pub config: RwLock<AppConfig>,
    config_path: PathBuf,
    engine: Mutex<Engine>,
    pub runner: Runner,
    pub spotify: spotify::Spotify,
    pub obs: obs::Obs,
    controller: serial::Controller,
    /// profile that was active before an automatic per-app switch
    auto_from: Mutex<Option<String>>,
    frontmost: Mutex<String>,
}

impl Core {
    /// Where to reach OBS: from OBS's own settings file (automatic) or from ours.
    pub fn obs_target(&self) -> (String, String) {
        let c = self.config.read();
        if c.obs_auto {
            let l = obs::local_settings();
            if l.found {
                return (format!("ws://127.0.0.1:{}", l.port), if l.auth { l.password } else { String::new() });
            }
        }
        (c.obs_url.clone(), c.obs_password.clone())
    }

    pub fn toast(&self, kind: &str, text: &str) {
        let _ = self.app.emit("toast", json!({ "kind": kind, "text": text }));
    }

    fn save(&self) {
        let cfg = self.config.read();
        if let Some(dir) = self.config_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&self.config_path, serde_json::to_vec_pretty(&*cfg).unwrap()) {
            self.toast("error", &format!("Couldn't save settings: {e}"));
        }
    }

    fn layer_names(&self) -> Vec<String> {
        self.config.read().active().map(|p| p.layers.clone()).unwrap_or_default()
    }

    pub fn set_layer(&self, layer: u8) -> String {
        let names = self.layer_names();
        let l = (layer as usize).min(names.len().saturating_sub(1)) as u8;
        self.engine.lock().layer = l;
        let _ = self.app.emit("layer", l);
        format!("Layer: {}", names.get(l as usize).cloned().unwrap_or_default())
    }

    pub fn next_layer(&self) -> String {
        let n = self.layer_names().len().max(1) as u8;
        let cur = self.engine.lock().layer;
        self.set_layer((cur + 1) % n)
    }

    fn activate_profile(&self, id: &str) -> String {
        let name = {
            let mut cfg = self.config.write();
            let Some(p) = cfg.profiles.iter().find(|p| p.id == id) else { return String::new() };
            let name = p.name.clone();
            cfg.active_profile = id.to_string();
            name
        };
        self.engine.lock().reset();
        self.save();
        let _ = self.app.emit("profile", id);
        format!("Profile: {name}")
    }

    pub fn next_profile(&self) -> String {
        let next = {
            let cfg = self.config.read();
            let i = cfg.profiles.iter().position(|p| p.id == cfg.active_profile).unwrap_or(0);
            cfg.profiles[(i + 1) % cfg.profiles.len()].id.clone()
        };
        *self.auto_from.lock() = None;
        self.activate_profile(&next)
    }

    fn apply(&self, effects: Vec<Effect>) {
        for fx in effects {
            match fx {
                Effect::Value { id, x, y } => {
                    let _ = self.app.emit("control", json!({ "id": id, "x": x, "y": y }));
                }
                Effect::Set { key, action, value } => self.runner.set(key, action, value),
                Effect::Fire { action } => self.runner.fire(action, 1.0),
            }
        }
    }
}

// ------------------------------------------------------------------ threads

fn start_midi(core: Arc<Core>) {
    let (tx, rx) = channel();
    let c = core.clone();
    std::thread::spawn(move || midi::run(tx, move || c.config.read().midi_inputs.clone()));
    std::thread::spawn(move || loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(msg) => {
                let _ = core.app.emit("midi", &msg);
                let fx = {
                    let cfg = core.config.read();
                    let Some(p) = cfg.active() else { continue };
                    core.engine.lock().on_midi(&msg, p)
                };
                core.apply(fx);
            }
            Err(RecvTimeoutError::Timeout) => {
                let fx = {
                    let cfg = core.config.read();
                    let Some(p) = cfg.active() else { continue };
                    core.engine.lock().tick(p)
                };
                core.apply(fx);
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    });
}

/// Watches which app is in front and switches profiles when one is linked to it.
fn start_app_watch(core: Arc<Core>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(800));
        let Ok(w) = active_win_pos_rs::get_active_window() else { continue };
        let name = w.app_name.clone();
        let exe = std::path::Path::new(&w.process_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        // ignore MidiMaster itself, so "last used app" means the app before it
        if name.is_empty() || name.to_lowercase().contains("midimaster") {
            continue;
        }
        {
            let mut f = core.frontmost.lock();
            if *f == name {
                continue;
            }
            *f = name.clone();
        }
        let _ = core.app.emit("frontmost", &name);
        if !core.config.read().auto_switch {
            continue;
        }
        let (target, active) = {
            let cfg = core.config.read();
            let hit = cfg.profiles.iter().find(|p| {
                p.apps.iter().any(|a| {
                    let a = a.trim().to_lowercase();
                    let a = a.strip_suffix(".exe").unwrap_or(&a).to_string();
                    !a.is_empty() && (a == name.to_lowercase() || a == exe.to_lowercase())
                })
            });
            (hit.map(|p| p.id.clone()), cfg.active_profile.clone())
        };
        match target {
            Some(id) if id != active => {
                let mut from = core.auto_from.lock();
                if from.is_none() {
                    *from = Some(active);
                }
                drop(from);
                let msg = core.activate_profile(&id);
                core.toast("info", &msg);
            }
            Some(_) => {}
            None => {
                let back = core.auto_from.lock().take();
                if let Some(id) = back {
                    let msg = core.activate_profile(&id);
                    core.toast("info", &msg);
                }
            }
        }
    });
}

// ------------------------------------------------------------------ commands

type CoreState<'a> = State<'a, Arc<Core>>;

#[tauri::command]
fn get_config(core: CoreState) -> AppConfig {
    core.config.read().clone()
}

#[tauri::command]
fn save_config(core: CoreState, config: AppConfig) {
    let profile_changed = core.config.read().active_profile != config.active_profile;
    *core.config.write() = config;
    if profile_changed {
        *core.auto_from.lock() = None;
        core.engine.lock().reset();
    }
    core.save();
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    layer: u8,
    frontmost: String,
    spotify_connected: bool,
    spotify_redirect: &'static str,
}

#[tauri::command]
fn get_status(core: CoreState) -> Status {
    Status {
        layer: core.engine.lock().layer,
        frontmost: core.frontmost.lock().clone(),
        spotify_connected: core.spotify.connected(),
        spotify_redirect: spotify::REDIRECT_URI,
    }
}

#[tauri::command]
fn control_snapshot(core: CoreState) -> Vec<serde_json::Value> {
    core.engine
        .lock()
        .snapshot()
        .into_iter()
        .filter_map(|e| match e {
            Effect::Value { id, x, y } => Some(json!({ "id": id, "x": x, "y": y })),
            _ => None,
        })
        .collect()
}

#[tauri::command]
fn set_layer(core: CoreState, layer: u8) {
    core.set_layer(layer);
}

#[tauri::command]
fn list_midi_inputs() -> Vec<String> {
    midi::list_inputs()
}

#[tauri::command]
async fn test_action(core: CoreState<'_>, action: Action, value: f64) -> Result<Option<String>, String> {
    let core = core.inner().clone();
    tauri::async_runtime::spawn_blocking(move || actions::execute(&core, &action, value))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn controller_ports() -> Vec<serial::PortInfo> {
    serial::list()
}

#[tauri::command]
fn controller_open(core: CoreState, app: AppHandle, name: String) -> Result<(), String> {
    core.controller.open(&name, app)
}

#[tauri::command]
fn controller_send(core: CoreState, line: String) -> Result<(), String> {
    core.controller.send(&line)
}

#[tauri::command]
fn controller_close(core: CoreState) {
    core.controller.close();
}

#[tauri::command]
async fn spotify_connect(core: CoreState<'_>) -> Result<(), String> {
    let core = core.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let cid = core.config.read().spotify_client_id.clone();
        core.spotify.connect(&cid)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn spotify_disconnect(core: CoreState) {
    core.spotify.disconnect();
}

#[tauri::command]
async fn obs_test(core: CoreState<'_>) -> Result<String, String> {
    let core = core.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        core.obs.disconnect();
        let (url, pw) = core.obs_target();
        let v = core.obs.request(&url, &pw, "GetVersion", json!({}))?;
        let scenes = core.obs.request(&url, &pw, "GetSceneList", json!({}))?;
        let names: Vec<String> = scenes["scenes"]
            .as_array()
            .map(|a| a.iter().filter_map(|s| s["sceneName"].as_str().map(String::from)).collect())
            .unwrap_or_default();
        Ok(format!(
            "Connected to OBS {}. Scenes: {}",
            v["obsVersion"].as_str().unwrap_or("?"),
            if names.is_empty() { "none".into() } else { names.join(", ") }
        ))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn obs_local() -> obs::LocalObs {
    obs::local_settings()
}

/// Scene and source names from OBS, for the drop-downs in the action editor.
#[tauri::command]
async fn obs_lists(core: CoreState<'_>) -> Result<serde_json::Value, String> {
    let core = core.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (url, pw) = core.obs_target();
        let names = |v: &serde_json::Value, list: &str, key: &str| -> Vec<String> {
            v[list].as_array().map(|a| a.iter().filter_map(|s| s[key].as_str().map(String::from)).collect()).unwrap_or_default()
        };
        let scenes = core.obs.request(&url, &pw, "GetSceneList", json!({}))?;
        let inputs = core.obs.request(&url, &pw, "GetInputList", json!({}))?;
        let mut scene_names = names(&scenes, "scenes", "sceneName");
        scene_names.reverse(); // OBS lists them bottom-up
        Ok(json!({ "scenes": scene_names, "inputs": names(&inputs, "inputs", "inputName") }))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_presets() -> Vec<model::Preset> {
    model::presets()
}

// ------------------------------------------------------------------ app

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let config_path = dir.join("config.json");
            let config: AppConfig = std::fs::read(&config_path)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .filter(|c: &AppConfig| !c.profiles.is_empty())
                .unwrap_or_else(model::default_config);
            let start_hidden = config.start_hidden || std::env::args().any(|a| a == "--hidden");

            let (runner, rx) = Runner::new();
            let core = Arc::new(Core {
                app: app.handle().clone(),
                config: RwLock::new(config),
                config_path,
                engine: Mutex::new(Engine::default()),
                runner,
                spotify: spotify::Spotify::new(dir.join("spotify.json")),
                obs: obs::Obs::default(),
                controller: serial::Controller::default(),
                auto_from: Mutex::new(None),
                frontmost: Mutex::new(String::new()),
            });
            app.manage(core.clone());
            actions::spawn_workers(core.clone(), rx);
            start_midi(core.clone());
            start_app_watch(core);

            // tray / menu-bar icon: MidiMaster keeps running when the window is closed
            let show = MenuItem::with_id(app, "show", "Open MidiMaster", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit MidiMaster", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            // one-colour icon; on macOS a "template" icon, so the menu bar shows it
            // white on a dark menu bar (and dark on a light one) like system icons
            TrayIconBuilder::with_id("tray")
                .icon(tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?)
                .icon_as_template(true)
                .tooltip("MidiMaster")
                .menu(&menu)
                .on_menu_event(|app, e| match e.id.as_ref() {
                    "show" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            if start_hidden {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_status,
            control_snapshot,
            set_layer,
            list_midi_inputs,
            test_action,
            controller_ports,
            controller_open,
            controller_send,
            controller_close,
            spotify_connect,
            spotify_disconnect,
            obs_test,
            obs_local,
            obs_lists,
            get_presets,
        ])
        .build(tauri::generate_context!())
        .expect("error while starting MidiMaster")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                show_main(app);
            }
            let _ = (app, event);
        });
}
