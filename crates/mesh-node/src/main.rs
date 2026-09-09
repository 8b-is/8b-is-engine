// mesh-node — the zone node CLI.
//
//   mesh-node hub "sanctuary"                    # a city, 10 TPS, :7777
//   mesh-node instance "the 108 gates" --port 7778   # a dungeon, 30 TPS
//
// Client wire: 4-byte big-endian length + the compact mesh JSON
// ({"t","n"{h,r,s},"a","z"}). The node broadcasts the folded zone state
// on every tick — the world runs without you.

use mesh_node::{new_node, ZoneMode};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = match args.first().map(String::as_str) {
        Some("hub") => ZoneMode::Hub,
        Some("instance") => ZoneMode::Instance,
        _ => {
            eprintln!("usage: mesh-node <hub|instance> \"<brief>\" [--port N]");
            std::process::exit(2);
        }
    };
    let brief = args.get(1).cloned().unwrap_or_else(|| "the sanctuary at dawn".to_string());
    let mut port = 7777u16;
    if let Some(i) = args.iter().position(|a| a == "--port") {
        port = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(port);
    }
    let node = new_node(&brief, mode, port);
    if let Err(e) = node.run().await {
        eprintln!("mesh-node failed: {e}");
        std::process::exit(1);
    }
}