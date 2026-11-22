//! Mapping engine: matches MIDI messages to the controls of the active profile,
//! keeps each control's value, and decides which actions should run.
//! It has no side effects itself; it returns `Effect`s that `lib.rs` carries out.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::midi::{MidiMsg, MsgKind};
use crate::model::{Action, ActionSpec, Axis, BindKind, Binding, Control, Profile, Trigger};

const LONG_PRESS: Duration = Duration::from_millis(600);
const DOUBLE_PRESS: Duration = Duration::from_millis(350);

#[derive(Default, Clone)]
struct CtlState {
    x: f64,
    y: f64,
    pressed: bool,
    press_at: Option<Instant>,
    last_press: Option<Instant>,
    long_fired: bool,
}

#[derive(Debug, Clone)]
pub enum Effect {
    /// Show this value in the live view (raw 0-1 values, before range/curve)
    Value { id: String, x: f64, y: f64 },
    /// Continuous action: apply the latest value (coalesced by key)
    Set { key: String, action: Action, value: f64 },
    /// One-shot action
    Fire { action: Action },
}

#[derive(Default)]
pub struct Engine {
    msb: HashMap<(u8, u8), u8>,
    states: HashMap<String, CtlState>,
    pub layer: u8,
}

impl Engine {
    pub fn reset(&mut self) {
        self.states.clear();
        self.layer = 0;
    }

    /// Current raw values of all controls, for the UI after a reload.
    pub fn snapshot(&self) -> Vec<Effect> {
        self.states
            .iter()
            .map(|(id, s)| Effect::Value { id: id.clone(), x: s.x, y: s.y })
            .collect()
    }

    pub fn on_midi(&mut self, msg: &MidiMsg, profile: &Profile) -> Vec<Effect> {
        // Remember 14-bit MSBs for every CC; the LSB message completes the value.
        if msg.kind == MsgKind::Cc && msg.number < 32 {
            self.msb.insert((msg.channel, msg.number), msg.value as u8);
        }
        let mut out = Vec::new();
        for c in &profile.controls {
            for (axis, b) in [(Axis::X, c.binding), (Axis::Y, c.binding_y)] {
                let Some(b) = b else { continue };
                if let Some(v) = self.value_for(&b, msg) {
                    let v = if (axis == Axis::X && c.invert) || (axis == Axis::Y && c.invert_y) { 1.0 - v } else { v };
                    self.update(c, axis, v, &mut out);
                }
            }
        }
        out
    }

    /// Call every ~50 ms: long presses.
    pub fn tick(&mut self, profile: &Profile) -> Vec<Effect> {
        let mut out = Vec::new();
        let now = Instant::now();
        for c in &profile.controls {
            let Some(st) = self.states.get_mut(&c.id) else { continue };
            if st.pressed && !st.long_fired && st.press_at.is_some_and(|t| now - t >= LONG_PRESS) {
                st.long_fired = true;
                let layer = self.layer;
                fire_matching(c, layer, Axis::X, Trigger::LongPress, &mut out);
            }
        }
        out
    }

    fn value_for(&self, b: &Binding, m: &MidiMsg) -> Option<f64> {
        if b.channel != 0 && b.channel != m.channel {
            return None;
        }
        match (b.kind, m.kind) {
            (BindKind::Cc, MsgKind::Cc) if m.number == b.number => Some(m.value as f64 / 127.0),
            // 14-bit: the value is complete when the LSB (number + 32) arrives
            (BindKind::Cc14, MsgKind::Cc) if b.number < 32 && m.number == b.number + 32 => {
                let msb = *self.msb.get(&(m.channel, b.number)).unwrap_or(&0) as u16;
                Some(((msb << 7) | m.value) as f64 / 16383.0)
            }
            (BindKind::Pb, MsgKind::Pb) => Some(m.value as f64 / 16383.0),
            (BindKind::Note, MsgKind::NoteOn) if m.number == b.number => Some(1.0),
            (BindKind::Note, MsgKind::NoteOff) if m.number == b.number => Some(0.0),
            _ => None,
        }
    }

    fn update(&mut self, c: &Control, axis: Axis, v: f64, out: &mut Vec<Effect>) {
        let layer = self.layer;
        let st = self.states.entry(c.id.clone()).or_default();
        let prev = if axis == Axis::X { st.x } else { st.y };
        if axis == Axis::X { st.x = v } else { st.y = v }
        out.push(Effect::Value { id: c.id.clone(), x: st.x, y: st.y });

        // continuous actions
        let shaped = c.min + (c.max - c.min) * v.clamp(0.0, 1.0).powf(c.curve.max(0.05));
        for s in active(c, layer, axis) {
            if s.trigger == Trigger::Value && s.action.is_continuous() {
                out.push(Effect::Set { key: format!("{}/{}", c.id, s.id), action: s.action.clone(), value: shaped });
            }
        }

        // direction
        if v > prev + 1e-9 {
            fire_matching(c, layer, axis, Trigger::Increase, out);
        } else if v < prev - 1e-9 {
            fire_matching(c, layer, axis, Trigger::Decrease, out);
        }

        // press / release (X axis only; a joystick's button is its own control)
        if axis == Axis::X {
            let now = Instant::now();
            let pressed = v >= 0.5;
            if pressed && !st.pressed {
                st.pressed = true;
                st.press_at = Some(now);
                st.long_fired = false;
                let double = st.last_press.is_some_and(|t| now - t <= DOUBLE_PRESS);
                st.last_press = if double { None } else { Some(now) };
                fire_matching(c, layer, axis, Trigger::Press, out);
                if double {
                    fire_matching(c, layer, axis, Trigger::DoublePress, out);
                }
            } else if !pressed && st.pressed {
                st.pressed = false;
                fire_matching(c, layer, axis, Trigger::Release, out);
            }
        }
    }
}

fn active<'a>(c: &'a Control, layer: u8, axis: Axis) -> impl Iterator<Item = &'a ActionSpec> {
    c.actions
        .iter()
        .filter(move |s| s.axis == axis && s.layer.map_or(true, |l| l == layer))
}

fn fire_matching(c: &Control, layer: u8, axis: Axis, trigger: Trigger, out: &mut Vec<Effect>) {
    for s in active(c, layer, axis) {
        if s.trigger == trigger {
            out.push(Effect::Fire { action: s.action.clone() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::default_config;

    fn msg(kind: MsgKind, number: u8, value: u16) -> MidiMsg {
        MidiMsg { port: "t".into(), channel: 1, kind, number, value }
    }

    #[test]
    fn cc_moves_control_and_fires_continuous() {
        let mut cfg = default_config();
        let p = &mut cfg.profiles[0];
        let fader = p.controls.iter_mut().find(|c| c.id == "fader1").unwrap();
        fader.actions.push(ActionSpec { id: "a".into(), layer: None, trigger: Trigger::Value, axis: Axis::X, action: Action::SystemVolume });
        let mut e = Engine::default();
        let fx = e.on_midi(&msg(MsgKind::Cc, 7, 127), p);
        assert!(fx.iter().any(|f| matches!(f, Effect::Set { value, .. } if (*value - 1.0).abs() < 1e-9)));
    }

    #[test]
    fn button_press_release_double() {
        let mut cfg = default_config();
        let p = &mut cfg.profiles[0];
        let b = p.controls.iter_mut().find(|c| c.id == "btn1").unwrap();
        for (i, t) in [Trigger::Press, Trigger::Release, Trigger::DoublePress].into_iter().enumerate() {
            b.actions.push(ActionSpec { id: i.to_string(), layer: None, trigger: t, axis: Axis::X, action: Action::LayerNext });
        }
        let mut e = Engine::default();
        let fires = |fx: Vec<Effect>| fx.iter().filter(|f| matches!(f, Effect::Fire { .. })).count();
        assert_eq!(fires(e.on_midi(&msg(MsgKind::Cc, 17, 127), p)), 1); // press
        assert_eq!(fires(e.on_midi(&msg(MsgKind::Cc, 17, 0), p)), 1); // release
        assert_eq!(fires(e.on_midi(&msg(MsgKind::Cc, 17, 127), p)), 2); // press + double
    }

    #[test]
    fn cc14_combines_msb_lsb() {
        let mut cfg = default_config();
        let p = &mut cfg.profiles[0];
        p.controls[0].binding = Some(Binding { kind: BindKind::Cc14, channel: 1, number: 16 });
        let mut e = Engine::default();
        assert!(e.on_midi(&msg(MsgKind::Cc, 16, 127), p).is_empty()); // MSB alone: wait for LSB
        let fx = e.on_midi(&msg(MsgKind::Cc, 48, 127), p);
        assert!(matches!(fx[0], Effect::Value { x, .. } if (x - 1.0).abs() < 1e-9));
    }
}
