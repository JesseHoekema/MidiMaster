//! MIDI input: connects to every (or every selected) input port, keeps watching
//! for devices being plugged in or out, and turns raw bytes into `MidiMsg`.

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::Duration;

use midir::{Ignore, MidiInput, MidiInputConnection};
use serde::Serialize;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MsgKind {
    Cc,
    NoteOn,
    NoteOff,
    Pb,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MidiMsg {
    pub port: String,
    /// 1-16
    pub channel: u8,
    pub kind: MsgKind,
    /// CC number or note number (0 for pitch bend)
    pub number: u8,
    /// 0-127, or 0-16383 for pitch bend
    pub value: u16,
}

pub fn parse(port: &str, bytes: &[u8]) -> Option<MidiMsg> {
    if bytes.len() < 2 {
        return None;
    }
    let status = bytes[0] & 0xF0;
    let channel = (bytes[0] & 0x0F) + 1;
    let d1 = bytes[1] & 0x7F;
    let d2 = bytes.get(2).copied().unwrap_or(0) & 0x7F;
    let (kind, number, value) = match status {
        0xB0 => (MsgKind::Cc, d1, d2 as u16),
        0x90 if d2 > 0 => (MsgKind::NoteOn, d1, d2 as u16),
        0x90 | 0x80 => (MsgKind::NoteOff, d1, 0),
        0xE0 => (MsgKind::Pb, 0, (d2 as u16) << 7 | d1 as u16),
        _ => return None,
    };
    Some(MidiMsg { port: port.to_string(), channel, kind, number, value })
}

/// Names of all MIDI inputs right now.
pub fn list_inputs() -> Vec<String> {
    let Ok(input) = MidiInput::new("MidiMaster scan") else { return vec![] };
    input.ports().iter().filter_map(|p| input.port_name(p).ok()).collect()
}

/// Runs forever on its own thread: (re)connects ports every two seconds.
/// `wanted` returns the configured port filter (empty = all ports).
pub fn run(tx: Sender<MidiMsg>, wanted: impl Fn() -> Vec<String>) {
    let mut conns: HashMap<String, MidiInputConnection<()>> = HashMap::new();
    loop {
        let names = list_inputs();
        let filter = wanted();
        let ok = |n: &String| filter.is_empty() || filter.iter().any(|f| f == n);

        // drop ports that disappeared or were deselected
        conns.retain(|name, _| names.contains(name) && ok(name));

        let new: Vec<String> = names.iter().filter(|n| ok(n) && !conns.contains_key(*n)).cloned().collect();
        for name in &new {
            let Ok(mut input) = MidiInput::new("MidiMaster") else { continue };
            input.ignore(Ignore::SysexAndTime);
            let Some(port) = input.ports().into_iter().find(|p| input.port_name(p).ok().as_ref() == Some(name)) else {
                continue;
            };
            let tx = tx.clone();
            let pname = name.clone();
            if let Ok(conn) = input.connect(
                &port,
                "midimaster-in",
                move |_, bytes, _| {
                    if let Some(m) = parse(&pname, bytes) {
                        let _ = tx.send(m);
                    }
                },
                (),
            ) {
                conns.insert(name.clone(), conn);
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_messages() {
        let m = parse("p", &[0xB0, 16, 100]).unwrap();
        assert_eq!((m.channel, m.kind, m.number, m.value), (1, MsgKind::Cc, 16, 100));
        let m = parse("p", &[0xE2, 0x7F, 0x7F]).unwrap();
        assert_eq!((m.channel, m.kind, m.value), (3, MsgKind::Pb, 16383));
        let m = parse("p", &[0x90, 60, 0]).unwrap();
        assert_eq!(m.kind, MsgKind::NoteOff);
    }
}
