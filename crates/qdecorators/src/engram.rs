//! engram — the memory unit, and the three-step descent to a blob.
//!
//! An **engram** is the engine's minimal durable record: a text, a kind,
//! a context, a seed, a tick, tags, a weight. It lives in three skins,
//! one ladder:
//!
//! * **L1 · the snake_case JSON-ish wire** — the human-readable face:
//!   every key snake_case (`text`, `seed_line`, `tick`…), determinism
//!   preserved, zero ambiguity about what a key means.
//! * **L3 · the STACCED binary** — the compact stacked form: a blob may
//!   STACK many engrams (records are length-prefixed, one after
//!   another), each with a fixed header (magic, kind, flags, seed, tick,
//!   weight) followed by its variable payloads — the ledger's flavor in
//!   miniature, FNV-1a-trailed per blob so corruption is heard.
//! * **L2 · ASCII85** — the blob in printable dress (4 bytes → 5
//!   characters, the PDF inheritance): for logs, emails, and any wire
//!   that eats only text. `ceil(4n/5)` growth, round-trip exact.
//!
//! The ladder is bidirectional: `L1 → (serde) → L3 stack → L2 ascii85`,
//! and each step decodes back to the step above it. The world runs
//! without you; its memories ship in whichever skin the surface wants.

use serde::{Deserialize, Serialize};

/// The engine's minimal durable record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Engram {
    /// the record's text — the document the keeper would keep
    pub text: String,
    /// what kind of record it is (admission, refusal, attest, dream…)
    pub kind: String,
    /// the context the record was folded in
    pub ctx: String,
    /// the seed that folded it — a seed is a seed everywhere
    pub seed: u64,
    /// the world's tick when it was inscribed
    pub tick: u64,
    /// the tags that index it
    pub tags: Vec<String>,
    /// the record's weight — a float, honest about its own importance
    pub weight: f32,
}

impl Engram {
    pub fn new(text: impl Into<String>) -> Self {
        Engram {
            text: text.into(),
            kind: String::new(),
            ctx: String::new(),
            seed: 0,
            tick: 0,
            tags: Vec::new(),
            weight: 0.0,
        }
    }

    /// L1 — the snake_case JSON-ish wire (keys are `rename_all`-ed
    /// snake_case by serde; the module's contract).
    pub fn l1_json(&self) -> String {
        serde_json::to_string(self).expect("engram json")
    }
}

// ----------------------------------------------------------------------
// L1 · the wire
// ----------------------------------------------------------------------

impl Engram {
    /// L1 decode — the inverse of `l1_json`.
    pub fn from_l1(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

// ----------------------------------------------------------------------
// L3 · the STACCED binary
// ----------------------------------------------------------------------

/// The stacked-blob magic.
pub const ENGRAM_MAGIC: &[u8; 4] = b"ENG1";
/// The current stacked format revision.
pub const ENGRAM_FORMAT: u8 = 1;

fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in bytes {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// L3 — stack many engrams into ONE compact blob; the trailer is
/// FNV-1a over the body, so a corrupted stack is refused, not absorbed.
pub fn stacked_encode(engrams: &[Engram]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(ENGRAM_MAGIC);
    out.push(ENGRAM_FORMAT);
    out.extend_from_slice(&(engrams.len() as u32).to_le_bytes());
    for e in engrams {
        out.extend_from_slice(&e.seed.to_le_bytes());
        out.extend_from_slice(&e.tick.to_le_bytes());
        out.extend_from_slice(&e.weight.to_le_bytes());
        for field in [&e.text, &e.kind, &e.ctx] {
            out.extend_from_slice(&(field.len() as u32).to_le_bytes());
            out.extend_from_slice(field.as_bytes());
        }
        out.extend_from_slice(&(e.tags.len() as u32).to_le_bytes());
        for t in &e.tags {
            out.extend_from_slice(&(t.len() as u32).to_le_bytes());
            out.extend_from_slice(t.as_bytes());
        }
    }
    out.extend_from_slice(&fnv1a32(&out).to_le_bytes());
    out
}

/// L3 decode — the inverse of `stacked_encode`; refuses bad magic,
/// bad lengths, bad checksum.
pub fn stacked_decode(bytes: &[u8]) -> Result<Vec<Engram>, String> {
    if bytes.len() < 4 + 1 + 4 + 4 {
        return Err("engram: too small to be a stack".into());
    }
    if &bytes[..4] != ENGRAM_MAGIC {
        return Err("engram: bad magic".into());
    }
    let fmt = bytes[4];
    if fmt != ENGRAM_FORMAT {
        return Err(format!("engram: unsupported format {fmt}"));
    }
    let body_end = bytes.len() - 4;
    let want = fnv1a32(&bytes[..body_end]);
    let have = u32::from_le_bytes(bytes[body_end..].try_into().unwrap());
    if want != have {
        return Err("engram: checksum mismatch — the stack was touched".into());
    }
    let count = u32::from_le_bytes(bytes[5..9].try_into().unwrap()) as usize;
    let mut p = 9usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if p + 8 + 8 + 4 > body_end {
            return Err("engram: truncated record header".into());
        }
        let seed = u64::from_le_bytes(bytes[p..p + 8].try_into().unwrap());
        p += 8;
        let tick = u64::from_le_bytes(bytes[p..p + 8].try_into().unwrap());
        p += 8;
        let weight = f32::from_le_bytes(bytes[p..p + 4].try_into().unwrap());
        p += 4;
        let mut read_str = |p: &mut usize| -> Result<String, String> {
            if *p + 4 > body_end {
                return Err("engram: truncated length".into());
            }
            let len = u32::from_le_bytes(bytes[*p..*p + 4].try_into().unwrap()) as usize;
            *p += 4;
            if *p + len > body_end {
                return Err("engram: field overruns the body".into());
            }
            let s = String::from_utf8(bytes[*p..*p + len].to_vec()).map_err(|e| e.to_string())?;
            *p += len;
            Ok(s)
        };
        let text = read_str(&mut p)?;
        let kind = read_str(&mut p)?;
        let ctx = read_str(&mut p)?;
        let tags: Vec<String> = {
            if p + 4 > body_end {
                return Err("engram: truncated tag count".into());
            }
            let n = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()) as usize;
            p += 4;
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(read_str(&mut p)?);
            }
            v
        };
        out.push(Engram {
            text,
            kind,
            ctx,
            seed,
            tick,
            tags,
            weight,
        });
    }
    if p != body_end {
        return Err("engram: trailing bytes after the last record".into());
    }
    Ok(out)
}

// ----------------------------------------------------------------------
// L2 · ASCII85
// ----------------------------------------------------------------------

/// ascii85_encode — 4 bytes → 5 characters (the PDF inheritance), no
/// `z` shorthand, no `<~ ~>` wrapper: the byte-true canonical skin.
pub fn ascii85_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(4) * 5);
    let mut i = 0;
    while i + 4 <= data.len() {
        let v = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        out.push_str(&group85(v));
        i += 4;
    }
    if i < data.len() {
        let mut buf = [0u8; 4];
        buf[..data.len() - i].copy_from_slice(&data[i..]);
        let v = u32::from_be_bytes(buf);
        let g = group85(v);
        let keep = data.len() - i + 1; // chars for the used bytes
        out.push_str(&g[..keep]);
    }
    out
}

fn group85(v: u32) -> String {
    if v == 0 {
        return "!!!!!".into();
    }
    let mut n = v;
    let mut chars = [b'u'; 5];
    for j in (0..5).rev() {
        chars[j] = b'!' + (n % 85) as u8;
        n /= 85;
    }
    String::from_utf8(chars.to_vec()).expect("ascii85 chars")
}

/// ascii85_decode — the inverse; refuses non-[!-u] characters and
/// malformed tails.
pub fn ascii85_decode(s: &str) -> Result<Vec<u8>, String> {
    let bytes = s.as_bytes();
    if bytes.len() % 5 == 1 {
        return Err("ascii85: a dangling character".into());
    }
    let mut out = Vec::with_capacity(bytes.len() / 5 * 4);
    let mut i = 0;
    while i + 5 <= bytes.len() {
        let g = &bytes[i..i + 5];
        out.extend_from_slice(&ungroup85(g)?);
        i += 5;
    }
    if i < bytes.len() {
        // the tail: 2..4 chars → 1..3 bytes
        let mut g = [b'u'; 5];
        g[..bytes.len() - i].copy_from_slice(&bytes[i..]);
        let full = ungroup85(&g)?;
        let keep = bytes.len() - i - 1; // bytes the tail encoded
        out.extend_from_slice(&full[..keep]);
    }
    Ok(out)
}

fn ungroup85(g: &[u8]) -> Result<[u8; 4], String> {
    let mut v: u32 = 0;
    for &c in g {
        if !(b'!'..=b'u').contains(&c) {
            return Err(format!("ascii85: character out of range: {c}"));
        }
        v = v.wrapping_mul(85).wrapping_add((c - b'!') as u32);
    }
    Ok(v.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Engram> {
        vec![
            Engram {
                text: "the world runs without you".into(),
                kind: "admission".into(),
                ctx: "sanctuary".into(),
                seed: 41592,
                tick: 108,
                tags: vec!["fold".into(), "keeper".into(), "wire".into()],
                weight: 1.5,
            },
            Engram {
                text: "sometimes he trains in his love".into(),
                kind: "dream".into(),
                ctx: "the training yard".into(),
                seed: 7,
                tick: 888,
                tags: vec!["heart".into(), "promise".into()],
                weight: 0.25,
            },
        ]
    }

    #[test]
    fn l1_keys_are_snake_case() {
        let json = sample()[0].l1_json();
        for key in ["text", "kind", "ctx", "seed", "tick", "tags", "weight"] {
            assert!(
                json.contains(&format!("\"{key}\":")),
                "missing {key}: {json}"
            );
        }
        // no camelCase leakers
        assert!(!json.contains("seedLine") && !json.contains("context"));
    }

    #[test]
    fn l1_round_trips() {
        for e in sample() {
            assert_eq!(Engram::from_l1(&e.l1_json()).unwrap(), e);
        }
    }

    #[test]
    fn l3_stacks_and_unstacks() {
        let got = stacked_decode(&stacked_encode(&sample())).unwrap();
        assert_eq!(got, sample());
    }

    #[test]
    fn l3_the_checksum_hears_tampering() {
        let mut blob = stacked_encode(&sample());
        let n = blob.len();
        blob[n - 5] ^= 0x01; // flip a byte inside the last record
        assert!(stacked_decode(&blob).is_err());
    }

    #[test]
    fn l3_bad_magic_and_lengths_are_refused() {
        let blob = stacked_encode(&sample());
        let mut bad = blob.clone();
        bad[0] = b'X';
        assert!(stacked_decode(&bad).is_err());
        let mut cut = blob.clone();
        cut.truncate(blob.len() - 8);
        assert!(stacked_decode(&cut).is_err());
    }

    #[test]
    fn l2_ascii85_round_trips_binary() {
        let blob = stacked_encode(&sample());
        let wire = ascii85_encode(&blob);
        assert_eq!(ascii85_decode(&wire).unwrap(), blob);
        // a fully binary chunk, high bytes included
        let bin = [0u8, 255, 128, 1, 4, 5, 0, 0, 9];
        assert_eq!(ascii85_decode(&ascii85_encode(&bin)).unwrap(), bin);
    }

    #[test]
    fn l2_rejects_garbage() {
        // an out-of-range character ('~' is not in [!-u])
        assert!(ascii85_decode("abcde~").is_err());
        // a dangling single character (5n+1)
        assert!(ascii85_decode("!!!!!a").is_err());
        assert!(ascii85_decode("!!!!").is_ok()); // a valid 3-byte tail
    }

    #[test]
    fn the_ladder_is_bidirectional() {
        let wire = ascii85_encode(&stacked_encode(&sample()));
        let back = stacked_decode(&ascii85_decode(&wire).unwrap()).unwrap();
        assert_eq!(back, sample());
    }
}
