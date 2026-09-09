// mesh-relay — the browser's door to the authoritative node.
//
//   mesh-relay --relay-port 9222 --node-port 7777
//
// WebSocket clients talk ws://127.0.0.1:9222; every message is framed into
// the node's TCP, every broadcast comes back as a WS text frame.

use mesh_relay::Relay;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut relay_port = 9222u16;
    let mut node_port = 7777u16;
    if let Some(i) = args.iter().position(|a| a == "--relay-port") {
        relay_port = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(relay_port);
    }
    if let Some(i) = args.iter().position(|a| a == "--node-port") {
        node_port = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(node_port);
    }
    let relay = Relay::new(relay_port, format!("127.0.0.1:{node_port}"));
    if let Err(e) = relay.run().await {
        eprintln!("mesh-relay failed: {e}");
        std::process::exit(1);
    }
}
