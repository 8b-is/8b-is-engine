//! dream — the 1.58-bit lane's read-out: load a `.tern` base model, warm
//! it on a seed text, and dream `n` tokens with the engine's own PRNG.
//!
//! Usage:
//! ```text
//! cargo run -p ternary --example dream -- \
//!   <checkpoint.tern> <manifest.json> <seed text> <n tokens> <temperature> <out.txt>
//! ```
//!
//! The dream is a pure function of its inputs: the same checkpoint, the
//! same seed text, the same n and temperature produce the same bytes on
//! an AMD Zen laptop, an Apple core, a WASM sandbox, and a server farm —
//! the PRNG is `mulberry32(seed_from_text(prompt))`, the engine's own
//! wire family. `temperature < 1e-3` reads the greedy argmax.

use std::collections::HashMap;
use std::fs;
use std::process::ExitCode;
use ternary::format::load_checkpoint;
use ternary::{pack_mulberry, sample, CharModel};
use world_core::tern::seed_from_text;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        eprintln!(
            "dream: need <checkpoint.tern> <manifest.json> <seed> <n> <temperature> <out.txt>"
        );
        return ExitCode::FAILURE;
    }
    let (cp_path, manifest_path, seed_text, n, temperature, out_path) =
        (&args[0], &args[1], &args[2], &args[3], &args[4], &args[5]);

    let n_tokens: usize = match n.parse() {
        Ok(v) => v,
        Err(_) => return usage_error("n must be a count"),
    };
    let temp: f32 = match temperature.parse() {
        Ok(v) => v,
        Err(_) => return usage_error("temperature must be a number"),
    };

    let cp_bytes = match fs::read(cp_path) {
        Ok(b) => b,
        Err(e) => return usage_error(&format!("cannot read checkpoint: {e}")),
    };
    let cp = match load_checkpoint(&cp_bytes) {
        Ok(c) => c,
        Err(e) => return usage_error(&e),
    };

    let chars = match load_chars(manifest_path, cp.vocab) {
        Ok(chars) => chars,
        Err(e) => return usage_error(&e),
    };
    let to_id: HashMap<char, usize> = chars.iter().enumerate().map(|(i, c)| (*c, i)).collect();

    let model = CharModel::from_checkpoint(&cp);
    let mut hidden = vec![0f32; model.dim];

    // Warm the carry on the seed text (unknown chars are dropped — the
    // dream begins where the model's alphabet begins).
    let prompt_tokens: Vec<usize> = seed_text
        .chars()
        .filter_map(|c| to_id.get(&c).copied())
        .collect();
    if prompt_tokens.is_empty() {
        return usage_error("the seed text must contain at least one vocab char");
    }
    for &t in &prompt_tokens {
        let _ = model.forward(t, &mut hidden);
    }

    // One PRNG stream for the whole dream, seeded from the prompt: a seed
    // is a seed everywhere.
    let mut rng = pack_mulberry(seed_from_text(seed_text) as u32);
    let mut cur = *prompt_tokens.last().unwrap();
    let mut dreamed = String::new();
    for _ in 0..n_tokens {
        let logits = model.forward(cur, &mut hidden);
        cur = sample(&logits, temp, &mut rng);
        dreamed.push(chars[cur]);
    }

    let out = format!("{}{}", seed_text.to_string(), dreamed);
    match fs::write(out_path, out.trim_end()) {
        Ok(_) => {
            println!(
                "⟦ the dream ⟧ {} · {} chars, t={} · {}",
                cp_path,
                seed_text.chars().count() + n_tokens,
                temp,
                out_path
            );
            println!("{}", out.trim_end());
        }
        Err(e) => return usage_error(&format!("cannot write {}: {e}", out_path)),
    }
    ExitCode::SUCCESS
}

/// The manifest's `chars` field is the trainer's vocab, in index order.
/// The field is a JSON string, so escapes (`\n`, `\"`, `\\`, `\t`) are
/// undone — the manifest's vocab is the actual character alphabet.
fn load_chars(path: &str, vocab: usize) -> Result<Vec<char>, String> {
    let manifest = fs::read_to_string(path).map_err(|e| format!("cannot read manifest: {e}"))?;
    let key = "\"chars\":";
    let idx = manifest.find(key).ok_or("manifest has no chars field")? + key.len();
    let rest = &manifest[idx..];
    let start = rest.find('"').ok_or("manifest: malformed chars")? + 1;
    let end = rest[start..].find('"').ok_or("manifest: malformed chars")? + start;
    let raw = &rest[start..end];
    let mut chars: Vec<char> = Vec::with_capacity(raw.len());
    let mut it = raw.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => chars.push('\n'),
                Some('t') => chars.push('\t'),
                Some('r') => chars.push('\r'),
                Some('\\') => chars.push('\\'),
                Some('"') => chars.push('"'),
                Some(other) => return Err(format!("manifest: unhandled escape \\{other}")),
                None => return Err("manifest: dangling escape".into()),
            }
        } else {
            chars.push(c);
        }
    }
    if chars.len() != vocab {
        return Err(format!(
            "manifest vocab {} != checkpoint vocab {}",
            chars.len(),
            vocab
        ));
    }
    Ok(chars)
}

fn usage_error(msg: &str) -> ExitCode {
    eprintln!("dream: {msg}");
    ExitCode::FAILURE
}
