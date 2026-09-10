//! ultra_graph — the relation field, drawn: a brief becomes a seeded
//! tri-state graph of the cast, its affinity + communities folded, and
//! the render written to an SVG.
//!
//! Usage:
//! ```text
//! cargo run -p world-core --example ultra_graph -- "<brief>" out/ultra-graph.svg
//! ```

use std::fs;
use std::process::ExitCode;
use world_core::UltraGraph;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("ultra_graph: need <brief> <out.svg>");
        return ExitCode::FAILURE;
    }
    let graph = UltraGraph::from_seed(&args[0], 12, 0.35);
    let affinity = graph.affinity();
    let communities = graph.community_fold(8);
    match fs::write(&args[1], graph.ultra_svg(&graph.seed_line)) {
        Ok(()) => {
            println!(
                "⟦ the ultra-graph ⟧ {} · {} nodes, {} bonds · {}",
                graph.seed_line,
                graph.nodes.len(),
                graph.edges.len(),
                args[1]
            );
            // the fold, read aloud: each member's net bond + community
            for i in 0..graph.nodes.len() {
                println!(
                    "  {} · affinity {:+} · community {}",
                    graph.nodes[i], affinity[i], communities[i]
                );
            }
        }
        Err(e) => {
            eprintln!("ultra_graph: cannot write {}: {e}", args[1]);
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
