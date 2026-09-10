// mesh-relay — the browser's door to the authoritative node.
//
//   mesh-relay --relay-port 9222 --node-port 7777
//
// WebSocket clients talk ws://127.0.0.1:9222; every message is framed into
// the node's TCP, every broadcast comes back as a WS text frame.

use mesh_relay::Relay;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut relay_port = 9222u16;
    let mut node_port = 7777u16;
    if let Some(i) = args.iter().position(|a| a == "--relay-port") {
        relay_port = args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or(relay_port);
    }
    if let Some(i) = args.iter().position(|a| a == "--node-port") {
        node_port = args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or(node_port);
    }
    let node_addr = format!("127.0.0.1:{node_port}");
    let mut quic_port = 0u16;
    if let Some(i) = args.iter().position(|a| a == "--quic-port") {
        quic_port = args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or(quic_port);
    }
    let relay = Relay::new(relay_port, node_addr.clone());
    if quic_port != 0 {
        // both doors, one node: the WS door and the QUIC door bridge the
        // same compact wire, sharing the same counters
        let stats = Arc::clone(&relay.stats);
        let quic = mesh_relay::quic::QuicDoor::new(quic_port, node_addr, stats);
        tokio::select! {
            r = relay.run() => r,
            q = quic.run() => q,
        }
        .map_err(|e| {
            eprintln!("mesh-relay failed: {e}");
            std::process::exit(1);
        })
        .ok();
    } else {
        let _ = tokio::join!(relay.run()).0;
    }
}
