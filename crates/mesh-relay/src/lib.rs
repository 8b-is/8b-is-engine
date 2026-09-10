// mesh-relay — the browser's door to the authoritative node.
//
// One relay per node: WebSocket clients (the browser, a dashboard, a
// mobile shell) talk `ws://`; the relay frames each message with the
// node's 4-byte-big-endian length and carries the node's broadcasts back
// as WebSocket text frames. The browser never touches TCP; the node never
// sees a WebSocket. This is the WebTransport row's first real link —
// WebSocket today, QUIC through the same seam later.

use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

pub mod quic;

/// The relay: a WS listener in front of one node's TCP address.
pub struct Relay {
    pub relay_port: u16,
    pub node_addr: Arc<str>,
    /// per-connection counters: frames in from browsers, frames out to them
    pub stats: Arc<Mutex<RelayStats>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RelayStats {
    pub connections: u64,
    pub frames_in: u64,
    pub frames_out: u64,
}

impl Relay {
    pub fn new(relay_port: u16, node_addr: impl Into<Arc<str>>) -> Self {
        Relay {
            relay_port,
            node_addr: node_addr.into(),
            stats: Arc::new(Mutex::new(RelayStats::default())),
        }
    }

    /// run — accept browser connections and bridge each to the node.
    pub async fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(("127.0.0.1", self.relay_port)).await?;
        println!(
            "⟦ mesh-relay ⟧ ws://127.0.0.1:{} ↔ tcp://{} — the browser's door to the node",
            self.relay_port, self.node_addr
        );
        while let Ok((ws_sock, _)) = listener.accept().await {
            let node_addr = Arc::clone(&self.node_addr);
            let stats = Arc::clone(&self.stats);
            tokio::spawn(async move {
                let _ = bridge(ws_sock, &node_addr, &stats).await;
            });
        }
        Ok(())
    }
}

/// bridge — one browser connection, two pumps:
///   browser WS message → length-framed delta into the node;
///   node broadcast → unframed JSON back as a WS text frame.
async fn bridge(
    ws_tcp: TcpStream,
    node_addr: &str,
    stats: &Arc<Mutex<RelayStats>>,
) -> std::io::Result<()> {
    let ws = tokio_tungstenite::accept_async(ws_tcp).await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("ws handshake: {e}"))
    })?;
    let (mut ws_sink, mut ws_stream) = ws.split();
    let node = TcpStream::connect(node_addr).await?;
    let (mut node_reader, mut node_writer) = node.into_split();
    {
        let mut s = stats.lock().await;
        s.connections += 1;
    }

    // browser → node
    let to_node = async {
        while let Some(msg) = ws_stream.next().await {
            let Ok(msg) = msg else { break };
            let data: Vec<u8> = match msg {
                Message::Text(t) => t.into_bytes(),
                Message::Binary(b) => b.to_vec(),
                _ => continue,
            };
            let mut frame = Vec::with_capacity(data.len() + 4);
            frame.extend_from_slice(&(data.len() as u32).to_be_bytes());
            frame.extend_from_slice(&data);
            if node_writer.write_all(&frame).await.is_err() {
                break;
            }
            let mut s = stats.lock().await;
            s.frames_in += 1;
        }
    };

    // node → browser
    let to_ws = async {
        let mut len_buf = [0u8; 4];
        loop {
            if node_reader.read_exact(&mut len_buf).await.is_err() {
                break;
            }
            let len = u32::from_be_bytes(len_buf) as usize;
            let mut buf = vec![0u8; len];
            if node_reader.read_exact(&mut buf).await.is_err() {
                break;
            }
            if ws_sink
                .send(Message::Text(String::from_utf8_lossy(&buf).into_owned()))
                .await
                .is_err()
            {
                break;
            }
            let mut s = stats.lock().await;
            s.frames_out += 1;
        }
    };

    tokio::select! {
        _ = to_node => {}
        _ = to_ws => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use mesh_node::{new_node, ZoneMode};

    #[tokio::test]
    async fn the_relay_bridges_browser_to_node() {
        // an authoritative node ...
        let node_port = find_port().await;
        let node = new_node("sanctuary", ZoneMode::Hub, node_port);
        let node_task = tokio::spawn(async move { node.run().await });

        // ... a relay in front of it ...
        let relay_port = find_port().await;
        let relay = Relay::new(relay_port, format!("127.0.0.1:{node_port}"));
        let relay_task = tokio::spawn(async move { relay.run().await });
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // ... and a browser (a WS client) through the door
        let browser_tcp = TcpStream::connect(("127.0.0.1", relay_port)).await.unwrap();
        let (ws, _resp) =
            tokio_tungstenite::client_async(format!("ws://127.0.0.1:{relay_port}"), browser_tcp)
                .await
                .expect("ws connect to the relay");
        let (mut sink, mut stream) = ws.split();

        let delta = r#"{"t":1,"s":1,"n":{"h":0.5,"r":0.9,"s":0.9}}"#;
        sink.send(Message::Text(delta.into())).await.unwrap();

        // the node folds the browser's delta and broadcasts it back
        let got = tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                let msg = stream.next().await.expect("relay stream").expect("ws msg");
                let text = msg.into_text().unwrap();
                if text.contains("zone_state") {
                    return text;
                }
            }
        })
        .await
        .expect("the broadcast crossed the relay");

        assert!(
            got.contains("127.0.0.1"),
            "the browser's fold is in the broadcast"
        );
        node_task.abort();
        relay_task.abort();
    }

    async fn find_port() -> u16 {
        let l = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        l.local_addr().unwrap().port()
    }

    // ---- the QUIC door: the same compact wire, a different transport ----

    #[tokio::test]
    async fn the_quic_door_bridges_the_browser_to_the_node() {
        use mesh_node::{new_node, ZoneMode};
        use wtransport::ClientConfig;

        let node_port = find_port().await;
        let node = new_node("sanctuary", ZoneMode::Hub, node_port);
        let node_task = tokio::spawn(async move { node.run().await });

        let door_port = find_port().await;
        let door = crate::quic::QuicDoor::new(
            door_port,
            format!("127.0.0.1:{node_port}"),
            Default::default(),
        );
        let door_task = tokio::spawn(async move { door.run().await });
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        // the browser: a wtransport client, trusting nothing (the test
        // escape hatch — the door's self-signed dev identity)
        let client_config = ClientConfig::builder()
            .with_bind_default()
            .with_no_cert_validation()
            .build();
        let endpoint = wtransport::Endpoint::client(client_config).expect("quic client endpoint");
        let connection = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            endpoint.connect(format!("https://127.0.0.1:{door_port}")),
        )
        .await
        .expect("quic connect to the door timed out")
        .expect("quic connect to the door");

        // the same compact wire the WS browser sends
        let delta = r#"{"t":1,"s":7,"n":{"h":0.5,"r":0.9,"s":0.9}}"#;
        connection
            .send_datagram(delta.as_bytes())
            .expect("delta datagram");

        // the node folds it and broadcasts back through the QUIC door
        let got = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let dg = connection.receive_datagram().await.expect("recv datagram");
                let text = String::from_utf8_lossy(&dg).into_owned();
                if text.contains("zone_state") {
                    return text;
                }
            }
        })
        .await
        .expect("the broadcast crossed the QUIC door");

        assert!(
            got.contains("127.0.0.1"),
            "the browser's fold is in the broadcast: {got}"
        );

        // both doors share one node cleanly: a WS browser still works
        // beside the QUIC one (the stats are shared, the seams are not)
        let ws_port = find_port().await;
        let relay = Relay::new(ws_port, format!("127.0.0.1:{node_port}"));
        let relay_task = tokio::spawn(async move { relay.run().await });
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let ws_tcp = TcpStream::connect(("127.0.0.1", ws_port)).await.unwrap();
        let (ws, _) = tokio_tungstenite::client_async(format!("ws://127.0.0.1:{ws_port}"), ws_tcp)
            .await
            .expect("ws connect");
        let (mut sink, mut stream) = ws.split();
        sink.send(Message::Text(
            r#"{"t":1,"s":8,"n":{"h":0.5,"r":0.9,"s":0.9}}"#.into(),
        ))
        .await
        .unwrap();
        let ws_got = tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                let msg = stream.next().await.expect("relay stream").expect("ws msg");
                let text = msg.into_text().unwrap();
                if text.contains("zone_state") {
                    return text;
                }
            }
        })
        .await
        .expect("the WS broadcast also crosses");
        assert!(ws_got.contains("127.0.0.1"));

        node_task.abort();
        door_task.abort();
        relay_task.abort();
    }
}
