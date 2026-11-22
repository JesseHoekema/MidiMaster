//! Spotify Web API with PKCE login (no client secret needed).
//! The user creates a free app at developer.spotify.com, adds the redirect URI
//! below, and pastes the Client ID into MidiMaster's settings.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use parking_lot::Mutex;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::blocking::Client;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const REDIRECT_URI: &str = "http://127.0.0.1:43821/callback";
const SCOPES: &str = "user-read-playback-state user-modify-playback-state user-read-currently-playing user-library-read user-library-modify";

#[derive(Serialize, Deserialize, Clone)]
struct Tokens {
    access: String,
    refresh: String,
    expires_at: u64,
}

pub struct Spotify {
    path: PathBuf,
    tokens: Mutex<Option<Tokens>>,
    http: Client,
    /// (fetched at, progress ms, duration ms) of the current song, for seeking
    track: Mutex<Option<(Instant, u64, u64)>>,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

impl Spotify {
    pub fn new(path: PathBuf) -> Self {
        let tokens = std::fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok());
        Spotify {
            path,
            tokens: Mutex::new(tokens),
            http: Client::builder().timeout(Duration::from_secs(8)).build().unwrap(),
            track: Mutex::new(None),
        }
    }

    pub fn connected(&self) -> bool {
        self.tokens.lock().is_some()
    }

    pub fn disconnect(&self) {
        *self.tokens.lock() = None;
        let _ = std::fs::remove_file(&self.path);
    }

    fn store(&self, t: Tokens) {
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&self.path, serde_json::to_vec(&t).unwrap());
        *self.tokens.lock() = Some(t);
    }

    fn token_request(&self, form: &[(&str, &str)]) -> Result<Value, String> {
        let r = self
            .http
            .post("https://accounts.spotify.com/api/token")
            .form(form)
            .send()
            .map_err(|e| e.to_string())?;
        let ok = r.status().is_success();
        let v: Value = r.json().map_err(|e| e.to_string())?;
        if !ok {
            return Err(format!("Spotify login failed: {}", v["error_description"].as_str().unwrap_or("unknown error")));
        }
        Ok(v)
    }

    fn save_token_response(&self, v: &Value, old_refresh: Option<String>) -> Result<(), String> {
        let access = v["access_token"].as_str().ok_or("no access token")?.to_string();
        let refresh = v["refresh_token"].as_str().map(String::from).or(old_refresh).ok_or("no refresh token")?;
        let expires_at = now() + v["expires_in"].as_u64().unwrap_or(3600);
        self.store(Tokens { access, refresh, expires_at });
        Ok(())
    }

    /// Opens the browser and waits (up to 3 minutes) for the user to log in.
    pub fn connect(&self, client_id: &str) -> Result<(), String> {
        if client_id.trim().is_empty() {
            return Err("Enter your Spotify Client ID first".into());
        }
        let verifier: String = rand::thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        let state: String = rand::thread_rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect();

        let listener = TcpListener::bind("127.0.0.1:43821").map_err(|e| format!("port 43821 is busy: {e}"))?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;

        let mut auth = url::Url::parse("https://accounts.spotify.com/authorize").unwrap();
        auth.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", client_id.trim())
            .append_pair("scope", SCOPES)
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("code_challenge_method", "S256")
            .append_pair("code_challenge", &challenge)
            .append_pair("state", &state);
        open::that(auth.as_str()).map_err(|e| e.to_string())?;

        let deadline = Instant::now() + Duration::from_secs(180);
        let code = loop {
            if Instant::now() > deadline {
                return Err("Spotify login timed out".into());
            }
            let Ok((mut stream, _)) = listener.accept() else {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            };
            let _ = stream.set_nonblocking(false);
            let mut line = String::new();
            let _ = BufReader::new(&stream).read_line(&mut line);
            let path = line.split_whitespace().nth(1).unwrap_or("/");
            let u = url::Url::parse(&format!("http://localhost{path}")).map_err(|e| e.to_string())?;
            let q: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
            let (msg, result) = match (q.get("code"), q.get("state")) {
                (Some(c), Some(s)) if *s == state => ("MidiMaster is connected to Spotify. You can close this tab.", Ok(c.clone())),
                _ if u.path() != "/callback" => continue,
                _ => ("Spotify login was cancelled.", Err(q.get("error").cloned().unwrap_or("cancelled".into()))),
            };
            let body = format!("<html><body style=\"font-family:sans-serif;padding:40px\"><h2>{msg}</h2></body></html>");
            let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            break result.map_err(|e| format!("Spotify login failed: {e}"))?;
        };

        let v = self.token_request(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", REDIRECT_URI),
            ("client_id", client_id.trim()),
            ("code_verifier", &verifier),
        ])?;
        self.save_token_response(&v, None)
    }

    fn access_token(&self, client_id: &str, force_refresh: bool) -> Result<String, String> {
        let t = self.tokens.lock().clone().ok_or("Spotify isn't connected (Settings > Spotify)")?;
        if !force_refresh && t.expires_at > now() + 60 {
            return Ok(t.access);
        }
        let v = self.token_request(&[("grant_type", "refresh_token"), ("refresh_token", &t.refresh), ("client_id", client_id.trim())])?;
        self.save_token_response(&v, Some(t.refresh))?;
        Ok(self.tokens.lock().as_ref().unwrap().access.clone())
    }

    fn api(&self, client_id: &str, method: Method, path: &str) -> Result<Option<Value>, String> {
        for attempt in 0..2 {
            let token = self.access_token(client_id, attempt > 0)?;
            let r = self
                .http
                .request(method.clone(), format!("https://api.spotify.com/v1{path}"))
                .bearer_auth(token)
                .header("Content-Length", "0")
                .send()
                .map_err(|e| e.to_string())?;
            let status = r.status().as_u16();
            if status == 401 && attempt == 0 {
                continue;
            }
            let text = r.text().unwrap_or_default();
            let json: Option<Value> = serde_json::from_str(&text).ok();
            return match status {
                200..=299 => Ok(json),
                404 => Err("Spotify isn't playing on any device. Start playback in Spotify first.".into()),
                403 => Err("Spotify refused this. Playback control needs Spotify Premium.".into()),
                429 => Err("Spotify: too many requests, slow down".into()),
                _ => Err(format!(
                    "Spotify error {status}: {}",
                    json.as_ref().and_then(|j| j["error"]["message"].as_str()).unwrap_or("unknown")
                )),
            };
        }
        unreachable!()
    }

    fn player(&self, cid: &str) -> Result<Value, String> {
        self.api(cid, Method::GET, "/me/player")?
            .ok_or_else(|| "Spotify isn't playing on any device. Start playback in Spotify first.".into())
    }

    pub fn play_pause(&self, cid: &str) -> Result<(), String> {
        let playing = self.player(cid)?["is_playing"].as_bool().unwrap_or(false);
        self.api(cid, Method::PUT, if playing { "/me/player/pause" } else { "/me/player/play" }).map(|_| ())
    }
    pub fn next(&self, cid: &str) -> Result<(), String> {
        self.api(cid, Method::POST, "/me/player/next").map(|_| ())
    }
    pub fn previous(&self, cid: &str) -> Result<(), String> {
        self.api(cid, Method::POST, "/me/player/previous").map(|_| ())
    }
    pub fn shuffle(&self, cid: &str) -> Result<String, String> {
        let on = !self.player(cid)?["shuffle_state"].as_bool().unwrap_or(false);
        self.api(cid, Method::PUT, &format!("/me/player/shuffle?state={on}"))?;
        Ok(format!("Shuffle {}", if on { "on" } else { "off" }))
    }
    pub fn repeat(&self, cid: &str) -> Result<String, String> {
        let next = match self.player(cid)?["repeat_state"].as_str().unwrap_or("off") {
            "off" => "context",
            "context" => "track",
            _ => "off",
        };
        self.api(cid, Method::PUT, &format!("/me/player/repeat?state={next}"))?;
        Ok(format!("Repeat {}", match next { "context" => "all", "track" => "one song", _ => "off" }))
    }
    pub fn volume(&self, cid: &str, v: f64) -> Result<(), String> {
        let pct = (v.clamp(0.0, 1.0) * 100.0).round() as u32;
        self.api(cid, Method::PUT, &format!("/me/player/volume?volume_percent={pct}")).map(|_| ())
    }

    fn track_times(&self, cid: &str, max_age: Duration) -> Result<(u64, u64), String> {
        if let Some((at, prog, dur)) = *self.track.lock() {
            if at.elapsed() < max_age {
                return Ok((prog + at.elapsed().as_millis() as u64, dur));
            }
        }
        let p = self.player(cid)?;
        let prog = p["progress_ms"].as_u64().unwrap_or(0);
        let dur = p["item"]["duration_ms"].as_u64().ok_or("nothing is playing")?;
        *self.track.lock() = Some((Instant::now(), prog, dur));
        Ok((prog, dur))
    }
    pub fn seek_by(&self, cid: &str, secs: i32) -> Result<(), String> {
        let (prog, dur) = self.track_times(cid, Duration::ZERO)?;
        let pos = (prog as i64 + secs as i64 * 1000).clamp(0, dur as i64);
        *self.track.lock() = None;
        self.api(cid, Method::PUT, &format!("/me/player/seek?position_ms={pos}")).map(|_| ())
    }
    pub fn seek_to(&self, cid: &str, v: f64) -> Result<(), String> {
        let (_, dur) = self.track_times(cid, Duration::from_secs(5))?;
        let pos = (v.clamp(0.0, 1.0) * dur as f64) as u64;
        self.api(cid, Method::PUT, &format!("/me/player/seek?position_ms={pos}")).map(|_| ())
    }

    /// Adds the current song to Liked Songs, or removes it if it's already there.
    pub fn like(&self, cid: &str) -> Result<String, String> {
        let p = self.player(cid)?;
        let id = p["item"]["id"].as_str().ok_or("nothing is playing")?.to_string();
        let name = p["item"]["name"].as_str().unwrap_or("song").to_string();
        let saved = self
            .api(cid, Method::GET, &format!("/me/tracks/contains?ids={id}"))?
            .and_then(|v| v[0].as_bool())
            .unwrap_or(false);
        let method = if saved { Method::DELETE } else { Method::PUT };
        self.api(cid, method, &format!("/me/tracks?ids={id}"))?;
        Ok(if saved { format!("Removed \"{name}\" from Liked Songs") } else { format!("Liked \"{name}\"") })
    }
}
