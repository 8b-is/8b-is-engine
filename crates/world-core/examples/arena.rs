// arena.rs — the world, drawn: a cast from the seed, stepped, rendered to
// SVG. The E2E oneshot's final artifact — the world visible as a picture.
//
//   cargo run -p world-core --example arena -- "sanctuary" out/arena.svg

use std::fs;
use world_core::arena_svg;
use world_core::SimWorld;

fn main() {
    let brief = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sanctuary".into());
    let out = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "out/arena.svg".into());

    let mut w = SimWorld::from_seed(&brief, 6);
    let mut es = w.materialize();
    // the world runs without you: step it into a recognizable state
    for t in 1..240u64 {
        w.step(t);
        w.sync_entities(&mut es);
    }
    let names: Vec<String> = w.fauna.iter().map(|f| f.name.clone()).collect();
    let svg = arena_svg(&es, &names, 640, 400);
    if let Some(parent) = std::path::Path::new(&out).parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&out, &svg).expect("write arena svg");
    println!(
        "⟦ the world, drawn ⟧ {out} · {brief} · {} fauna",
        names.len()
    );
}
