//! engram_demo — the engine's logs, as engrams: a two-way alias.
//!
//! Every log line is an engram (kind = log), linked to the line before
//! it; every engram is a TripleBox — the same memory in three skins
//! (L1 wire, L3 stacked blob, L2 ascii85). Decode walks the diamond
//! back to the wire. One record, many names: the log line, the wire,
//! the blob, the printable skin — aliases of one memory.
//!
//! Run: cargo run -p corelib --example engram_demo

use corelib::{Engram, TripleBox};

fn main() {
    // the engine's own voice — the doctrine lines that would hit a log
    let log_lines: Vec<&str> = vec![
        "the world runs without you",
        "tick 108 · the keeper folded GAIA",
        "somebody refused — the reason is the document",
        "a promise is a promise",
        "the dream is byte-equal on every surface",
    ];

    // each line → an engram, linked to its predecessor (the memory's
    // own graph: every record after the first names the one before it)
    let mut prev = String::new();
    let mut boxes: Vec<TripleBox> = Vec::new();
    for (i, line) in log_lines.iter().enumerate() {
        let mut e = Engram::new(*line);
        e.kind = "log".into();
        e.ctx = "the engine".into();
        e.tick = 100 + i as u64;
        if i > 0 {
            e.links = vec![prev.clone()];
        }
        prev = format!("seg:0/off:{}", i);
        boxes.push(TripleBox::from_engram(e));
    }

    println!(
        "⟦ engrams ⟧ {} log lines → {} engrams, chained by links",
        boxes.len(),
        boxes.len()
    );
    for (i, b) in boxes.iter().enumerate() {
        let l3 = b.l3();
        println!(
            "  [{}] L1 {:<44} L3 {:>6}B L2 {:<28} alias seg:0/off:{}",
            i,
            format!("{:?}", b.l1().text),
            l3.len(),
            &b.l2()[..28.min(b.l2().len())],
            i
        );
    }

    // the diamond decode: every box walks back to the wire, exactly
    let mut decoded_ok = true;
    for (i, b) in boxes.iter().enumerate() {
        let back = b.decode();
        decoded_ok &= back.text == log_lines[i];
    }
    println!(
        "⟦ triplebox ⟧ decode L2 → L3 → L1: {} — one record, three skins, many names",
        if decoded_ok { "exact" } else { "DRIFTED" }
    );

    // the chain: follow the links — one path through the log
    let mut path: Vec<String> = Vec::new();
    for b in &boxes {
        if let Some(link) = b.l1().links.first() {
            path.push(link.clone());
        }
    }
    println!("⟦ links ⟧ each line names the one before — the log, as a graph");
}
