//! OBS Studio control through obs-websocket 5 (built into OBS 28+:
//! Tools > WebSocket Server Settings).

use std::net::TcpStream;
use std::time::Duration;

use base64::Engine as _;
use parking_lot::Mutex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

type Sock = WebSocket<MaybeTlsStream<TcpStream>>;

#[derive(Default)]
pub struct Obs {
    sock: Mutex<Option<Sock>>,
    next_id: Mutex<u64>,
}

fn read_json(ws: &mut Sock) -> Result<Value, String> {
    loop {
        match ws.read().map_err(|e| e.to_string())? {
            Message::Text(t) => return serde_json::from_str(&t).map_err(|e| e.to_string()),
            Message::Close(_) => return Err("OBS closed the connection".into()),
            _ => {}
        }
    }
}

fn connect(url: &str, password: &str) -> Result<Sock, String> {
    let (mut ws, _) = tungstenite::connect(url).map_err(|e| format!("Can't reach OBS at {url} ({e}). Is OBS open with the WebSocket server on?"))?;
    if let MaybeTlsStream::Plain(s) = ws.get_ref() {
        let _ = s.set_read_timeout(Some(Duration::from_secs(3)));
    }
    let hello = read_json(&mut ws)?;
    let mut identify = json!({ "rpcVersion": 1, "eventSubscriptions": 0 });
    if let Some(auth) = hello["d"].get("authentication") {
        let b64 = base64::engine::general_purpose::STANDARD;
        let secret = b64.encode(Sha256::digest(format!("{password}{}", auth["salt"].as_str().unwrap_or(""))));
        let resp = b64.encode(Sha256::digest(format!("{secret}{}", auth["challenge"].as_str().unwrap_or(""))));
        identify["authentication"] = json!(resp);
    }
    ws.send(Message::Text(json!({ "op": 1, "d": identify }).to_string())).map_err(|e| e.to_string())?;
    let r = read_json(&mut ws).map_err(|_| "OBS refused the connection. Check the password in Settings.".to_string())?;
    if r["op"] != 2 {
        return Err("OBS didn't accept MidiMaster".into());
    }
    Ok(ws)
}

impl Obs {
    pub fn request(&self, url: &str, password: &str, kind: &str, data: Value) -> Result<Value, String> {
        let mut sock = self.sock.lock();
        for attempt in 0..2 {
            if sock.is_none() {
                *sock = Some(connect(url, password)?);
            }
            let ws = sock.as_mut().unwrap();
            let id = {
                let mut n = self.next_id.lock();
                *n += 1;
                n.to_string()
            };
            let msg = json!({ "op": 6, "d": { "requestType": kind, "requestId": id, "requestData": data } });
            let result = ws.send(Message::Text(msg.to_string())).map_err(|e| e.to_string()).and_then(|_| loop {
                let v = read_json(ws)?;
                if v["op"] == 7 && v["d"]["requestId"] == id.as_str() {
                    break Ok(v["d"].clone());
                }
            });
            match result {
                Ok(d) => {
                    if d["requestStatus"]["result"].as_bool() == Some(true) {
                        return Ok(d["responseData"].clone());
                    }
                    return Err(format!("OBS: {}", d["requestStatus"]["comment"].as_str().unwrap_or("request failed")));
                }
                Err(e) => {
                    *sock = None; // connection broken: reconnect once
                    if attempt == 1 {
                        return Err(e);
                    }
                }
            }
        }
        unreachable!()
    }

    pub fn disconnect(&self) {
        *self.sock.lock() = None;
    }
}
