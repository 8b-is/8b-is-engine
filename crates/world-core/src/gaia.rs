// gaia.rs — the world-memory: one seed, eight layers, deterministic forever.
//
// GAIA is the engine's central world-memory module. Every physical
// constant of a zone folds out of ONE pseudorandom seed: weather,
// universe entropy, time, gravity, wind, temperature, light, memory.
// Same brief + same tick ⇒ same universe.
//
// Bit-identical to quantTernEngine/gaia.ts (Node) and examples/gaia.py
// (Python) — this crate is the shared core all three surfaces link.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const LAYERS: [&str; 8] = [
    "time", "weather", "entropy", "gravity", "wind", "temp", "light", "memory",
];
pub const WEATHER: [&str; 8] = [
    "calm", "mist", "wind", "rain", "storm", "clear", "overcast", "frost",
];

/// The eight layers, folded from one brief at one tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaState {
    /// time — the zone's tick / epoch
    #[serde(rename = "t")]
    pub t: u64,
    /// weather — a word from the zone's own vocabulary
    #[serde(rename = "w")]
    pub weather: String,
    /// entropy — monotone, never rewinds
    #[serde(rename = "e")]
    pub entropy: f64,
    /// gravity — the zone's constant field
    #[serde(rename = "g")]
    pub gravity: f64,
    /// wind — direction, speed, gust
    #[serde(rename = "v")]
    pub wind: [f64; 3],
    /// temperature — the season's breath
    #[serde(rename = "p")]
    pub temp: f64,
    /// light — the sun's arc, 0..1
    #[serde(rename = "l")]
    pub light: f64,
    /// memory — the history pointer (hash chain)
    #[serde(rename = "m")]
    pub memory: String,
}

/// whash — deterministic window hash: one value per (base, epoch) pair.
/// The exact 32-bit sequence JS and Python use.
fn whash(base: u32, epoch: u32) -> u32 {
    let mut h = base ^ epoch.wrapping_mul(2654435761);
    h = (h ^ (h >> 16)).wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(3266489917);
    h ^ (h >> 16)
}

/// base_of — sha256('gaia·' + brief)[:8] as a u32. The same root number
/// every implementation derives.
fn base_of(brief: &str) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(format!("gaia·{brief}").as_bytes());
    let d = hasher.finalize();
    let bytes: &[u8] = d.as_slice();
    let hex: String = bytes[..4].iter().map(|b| format!("{b:02x}")).collect();
    u32::from_str_radix(&hex, 16).expect("8 hex chars")
}

/// JS-style Math.round(x*100)/100 — half-up to two decimals.
fn r2(x: f64) -> f64 {
    (x * 100.0 + 0.5).floor() / 100.0
}

/// gaia_state — the fold: brief + tick → the eight layers.
pub fn gaia_state(brief: &str, tick: u64) -> GaiaState {
    let base = base_of(brief);
    let t = tick;

    let weather = WEATHER[(whash(base, (t / 300) as u32) as usize) % WEATHER.len()].to_string();
    let entropy = r2(t as f64 / 86400.0 + (whash(base, 0) % 1000) as f64 / 1000.0);
    let gravity = r2(9.7 + (whash(base, 1) % 100) as f64 / 100.0);

    let w0 = whash(base, (t / 60) as u32) % 360;
    let w1 = whash(base, (t / 60) as u32 + 1) % 360;
    let f = (t % 60) as f64 / 60.0;
    let dir = ((w0 as f64 + (((w1 + 360 - w0) % 360) as f64 * f)).round()) % 360.0;
    let speed = 1 + (whash(base, (t / 60) as u32 + 7) % 12);
    let gust = r2(speed as f64 + (whash(base, t as u32) % 100) as f64 / 100.0 * 4.0);
    let wind = [dir, speed as f64, gust];

    let temp = r2(15.0
        + 8.0 * ((t as f64 / 864000.0) * std::f64::consts::TAU).sin()
        + 4.0 * ((t as f64 / 86400.0) * std::f64::consts::TAU).sin()
        + ((whash(base, (t % 3600) as u32) % 100) as f64 / 100.0 - 0.5) * 2.0);
    let light = r2(((t as f64 / 86400.0) * std::f64::consts::TAU)
        .sin()
        .max(0.0));

    // memory — the history pointer: a hash chain over the folded hours
    let folds = (t / 3600) as u32;
    let mut mem = whash(base, 0);
    for i in 1..=folds.min(4096) {
        mem = whash(mem, i);
    }
    let memory = format!("{mem:08x}");

    GaiaState {
        t,
        weather,
        entropy,
        gravity,
        wind,
        temp,
        light,
        memory,
    }
}

/// gaia_wire — the compact mesh frame (single-char keys, the ternary
/// wire discipline). Field order matches gaia.ts / gaia.py.
pub fn gaia_wire(s: &GaiaState) -> String {
    serde_json::to_string(s).expect("gaia state serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    // fixtures captured from the verified Node and Python runs —
    // cross-language determinism is the contract, not a preference
    #[test]
    fn matches_node_and_python_at_t0() {
        let s = gaia_state("sanctuary·overworld", 0);
        assert_eq!(s.weather, "storm");
        assert_eq!(s.entropy, 0.37);
        assert_eq!(s.gravity, 9.74);
        assert_eq!(s.wind, [212.0, 4.0, 6.88]);
        assert_eq!(s.temp, 15.44);
        assert_eq!(s.light, 0.0);
        assert_eq!(s.memory, "499c32bc");
    }

    #[test]
    fn matches_node_and_python_at_t86400() {
        let s = gaia_state("sanctuary·overworld", 86400);
        assert_eq!(s.weather, "rain");
        assert_eq!(s.entropy, 1.37);
        assert_eq!(s.gravity, 9.74);
        assert_eq!(s.wind, [27.0, 11.0, 13.4]);
        assert_eq!(s.temp, 20.14);
        assert_eq!(s.light, 0.0);
        assert_eq!(s.memory, "167893c7");
    }

    #[test]
    fn entropy_is_monotone_and_gravity_is_constant() {
        let a = gaia_state("sanctuary·overworld", 0);
        let b = gaia_state("sanctuary·overworld", 86400);
        assert!(b.entropy > a.entropy);
        assert_eq!(a.gravity, b.gravity);
    }

    #[test]
    fn wire_is_compact() {
        let s = gaia_state("sanctuary·overworld", 0);
        let wire = gaia_wire(&s);
        assert!(wire.contains("\"t\":0"));
        assert!(wire.contains("\"w\":\"storm\""));
        assert!(wire.len() < 120, "the mesh frame stays small: {wire}");
    }
}
