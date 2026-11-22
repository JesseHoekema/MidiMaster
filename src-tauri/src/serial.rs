//! USB-serial link to the MidiMaster Deck's settings (same protocol as the
//! old web page: `get`, `set <name> <value>`, `save`, `mon 1`, ...).

use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use serde::Serialize;
use serialport::{SerialPort, SerialPortType};
use tauri::{AppHandle, Emitter};

/// Adafruit (Earle Philhower core with TinyUSB) and Raspberry Pi vendor IDs
const VIDS: [u16; 2] = [0x239A, 0x2E8A];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    pub name: String,
    pub product: String,
}

pub fn list() -> Vec<PortInfo> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| !p.port_name.starts_with("/dev/tty.")) // macOS lists both cu. and tty.; cu. is the right one
        .filter_map(|p| match p.port_type {
            SerialPortType::UsbPort(u) if VIDS.contains(&u.vid) => Some(PortInfo {
                name: p.port_name,
                // the RP2040-Zero board in the Deck reports itself as "RP2040 Zero"
                product: "MidiMaster Deck".into(),
            }),
            _ => None,
        })
        .collect()
}

#[derive(Default)]
pub struct Controller {
    port: Mutex<Option<Box<dyn SerialPort>>>,
    generation: Arc<AtomicU64>,
}

impl Controller {
    pub fn open(&self, name: &str, app: AppHandle) -> Result<(), String> {
        self.close();
        let mut port = serialport::new(name, 115_200)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| format!("Can't open {name}: {e}. Is another program (like the old web page) using it?"))?;
        let _ = port.write_data_terminal_ready(true);
        let reader = port.try_clone().map_err(|e| e.to_string())?;
        *self.port.lock() = Some(port);

        let gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let generation = self.generation.clone();
        std::thread::spawn(move || {
            let mut r = BufReader::new(reader);
            let mut line = String::new();
            while generation.load(Ordering::SeqCst) == gen {
                line.clear();
                match r.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let l = line.trim();
                        if l.starts_with('{') {
                            let _ = app.emit("controller-line", l.to_string());
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(_) => break,
                }
            }
            if generation.load(Ordering::SeqCst) == gen {
                let _ = app.emit("controller-closed", ());
            }
        });
        Ok(())
    }

    pub fn send(&self, line: &str) -> Result<(), String> {
        let mut p = self.port.lock();
        let p = p.as_mut().ok_or("controller not connected")?;
        p.write_all(format!("{line}\n").as_bytes()).map_err(|e| e.to_string())
    }

    pub fn close(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(mut p) = self.port.lock().take() {
            let _ = p.write_all(b"mon 0\n");
        }
    }
}
