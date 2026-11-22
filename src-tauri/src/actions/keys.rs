//! Media keys, keyboard shortcuts and typing text (via enigo).
//! On macOS this needs Accessibility permission (System Settings > Privacy & Security).

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::model::MediaKey;

fn enigo() -> Result<Enigo, String> {
    Enigo::new(&Settings::default()).map_err(|e| format!("keyboard control not allowed: {e}"))
}

pub fn media(key: MediaKey) -> Result<(), String> {
    let k = match key {
        MediaKey::PlayPause => Key::MediaPlayPause,
        MediaKey::Next => Key::MediaNextTrack,
        MediaKey::Previous => Key::MediaPrevTrack,
        MediaKey::Stop => stop_key()?,
        MediaKey::VolumeUp => Key::VolumeUp,
        MediaKey::VolumeDown => Key::VolumeDown,
        MediaKey::Mute => Key::VolumeMute,
    };
    enigo()?.key(k, Direction::Click).map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn stop_key() -> Result<Key, String> {
    Ok(Key::MediaStop)
}
#[cfg(not(target_os = "windows"))]
fn stop_key() -> Result<Key, String> {
    Err("there is no stop key on this system; use play/pause".into())
}

/// Parse "cmd+shift+a", "ctrl+alt+delete", "f5", "space"...
pub fn parse_shortcut(s: &str) -> Result<Vec<Key>, String> {
    s.split('+')
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .map(|p| {
            Ok(match p.as_str() {
                "cmd" | "command" | "win" | "super" | "meta" => Key::Meta,
                "ctrl" | "control" => Key::Control,
                "shift" => Key::Shift,
                "alt" | "option" | "opt" => Key::Alt,
                "space" => Key::Space,
                "enter" | "return" => Key::Return,
                "tab" => Key::Tab,
                "esc" | "escape" => Key::Escape,
                "backspace" => Key::Backspace,
                "delete" | "del" => Key::Delete,
                "up" => Key::UpArrow,
                "down" => Key::DownArrow,
                "left" => Key::LeftArrow,
                "right" => Key::RightArrow,
                "home" => Key::Home,
                "end" => Key::End,
                "pageup" => Key::PageUp,
                "pagedown" => Key::PageDown,
                f if f.len() >= 2 && f.starts_with('f') && f[1..].parse::<u8>().is_ok() => {
                    match f[1..].parse::<u8>().unwrap() {
                        1 => Key::F1, 2 => Key::F2, 3 => Key::F3, 4 => Key::F4, 5 => Key::F5, 6 => Key::F6,
                        7 => Key::F7, 8 => Key::F8, 9 => Key::F9, 10 => Key::F10, 11 => Key::F11, 12 => Key::F12,
                        n => return Err(format!("unknown key F{n}")),
                    }
                }
                c if c.chars().count() == 1 => Key::Unicode(c.chars().next().unwrap()),
                other => return Err(format!("unknown key \"{other}\"")),
            })
        })
        .collect()
}

pub fn shortcut(s: &str) -> Result<(), String> {
    let keys = parse_shortcut(s)?;
    let (last, mods) = keys.split_last().ok_or("empty shortcut")?;
    let mut en = enigo()?;
    let r = (|| {
        for k in mods {
            en.key(*k, Direction::Press)?;
        }
        en.key(*last, Direction::Click)?;
        Ok::<_, enigo::InputError>(())
    })();
    for k in mods.iter().rev() {
        let _ = en.key(*k, Direction::Release);
    }
    r.map_err(|e| e.to_string())
}

pub fn type_text(t: &str) -> Result<(), String> {
    enigo()?.text(t).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_shortcuts() {
        assert_eq!(parse_shortcut("cmd+shift+a").unwrap(), vec![Key::Meta, Key::Shift, Key::Unicode('a')]);
        assert_eq!(parse_shortcut("F5").unwrap(), vec![Key::F5]);
        assert!(parse_shortcut("ctrl+banana").is_err());
    }
}
