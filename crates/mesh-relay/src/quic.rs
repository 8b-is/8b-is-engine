// quic.rs — the relay's second door: WebTransport (QUIC).
//
// The WS door speaks `ws://`; this door speaks `https://` (HTTP/3 +
// WebTransport), the same compact wire through a different transport:
// every incoming datagram is framed with the node's 4-byte-big-endian
// length and pushed into the node's TCP; every broadcast comes back
// unframed as a datagram to the session that is listening. The browser
// never touches TCP; the node never sees a QUIC packet.
//
// Certificates: the door serves a self-signed identity (localhost SANs)
// generated at startup — right for the dev loop. For a real browser
// (Chrome/Edge) the operator replaces the identity with a proper
// certificate chain (see `Identity::load_pemfiles`) or pins the
// self-signed cert's hash via W3C serverCertificateHashes
// (`with_server_certificate_hashes` on the client side).
// The headless tests use `with_no_cert_validation` (the crate's own
// recommended test escape hatch; feature "dangerous-configuration" —
// never for anything real).

use wtransport::endpoint::{IncomingSession, SessionRequest};
use wtransport::{Connection, Endpoint, Identity, ServerConfig};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::RelayStats;

/// The QUIC door: a WebTransport listener in front of one node's TCP.
pub struct QuicDoor {
    pub quic_port: u16,
    pub node_addr: Arc<str>,
    pub stats: Arc<Mutex<RelayStats>>,
}

impl QuicDoor {
    pub fn new(
        quic_port: u16,
        node_addr: impl Into<Arc<str>>,
        stats: Arc<Mutex<RelayStats>>,
    ) -> Self {
        QuicDoor {
            quic_port,
            node_addr: node_addr.into(),
            stats,
        }
    }

    /// run — accept WebTransport sessions and bridge each to the node.
    pub async fn run(&self) -> std::io::Result<()> {
        // The self-signed dev identity: localhost + loopbacks, 14 days.
        let identity = Identity::self_signed(["localhost", "127.0.0.1", "::1"])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let config = ServerConfig::builder()
            .with_bind_default(self.quic_port)
            .with_identity(identity)
            .build();
        let endpoint = Endpoint::server(config)?;
        println!(
            "⟦ mesh-relay ⟧ wt://127.0.0.1:{} ↔ tcp://{} — the QUIC door to the node",
            self.quic_port, self.node_addr
        );
        loop {
            let session: IncomingSession = endpoint.accept().await;
            let node_addr = Arc::clone(&self.node_addr);
            let stats = Arc::clone(&self.stats);
            tokio::spawn(async move {
                let _ = accept_session(session, &node_addr, &stats).await;
            });
        }
    }
}

/// accept_session — answer the WebTransport request, then bridge
/// datagrams ↔ the node's length-framed TCP (mirror of the WS bridge).
async fn accept_session(
    session: IncomingSession,
    node_addr: &str,
    stats: &Arc<Mutex<RelayStats>>,
) -> std::io::Result<()> {
    let request: SessionRequest = session.await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("quic request: {e}"))
    })?;
    let connection = request
        .accept()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("quic accept: {e}")))?;
    bridge_quic(connection, node_addr, stats).await
}

/// bridge_quic — one session, two pumps:
///   session datagram → length-framed delta into the node;
///   node broadcast → unframed datagram back to the session.
async fn bridge_quic(
    connection: Connection,
    node_addr: &str,
    stats: &Arc<Mutex<RelayStats>>,
) -> std::io::Result<()> {
    let node = TcpStream::connect(node_addr).await?;
    let (mut node_reader, mut node_writer) = node.into_split();
    {
        let mut s = stats.lock().await;
        s.connections += 1;
    }

    // session → node
    let to_node = async {
        loop {
            let dg = match connection.receive_datagram().await {
                Ok(d) => d,
                Err(_) => break,
            };
            let data: &[u8] = &dg;
            if data.is_empty() {
                continue;
            }
            let mut frame = Vec::with_capacity(data.len() + 4);
            frame.extend_from_slice(&(data.len() as u32).to_be_bytes());
            frame.extend_from_slice(data);
            if node_writer.write_all(&frame).await.is_err() {
                break;
            }
            let mut s = stats.lock().await;
            s.frames_in += 1;
        }
    };

    // node → session
    let to_quic = async {
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
            match connection.send_datagram(&buf) {
                Ok(()) => {
                    let mut s = stats.lock().await;
                    s.frames_out += 1;
                }
                Err(e) => {
                    // a broadcast larger than the datagram budget is
                    // dropped for that session, counted nowhere; the
                    // compact wire keeps frames small by design
                    eprintln!("mesh-relay quic: datagram too large ({len}B): {e}");
                    continue;
                }
            }
        }
    };

    tokio::select! {
        _ = to_node => {}
        _ = to_quic => {}
    }
    Ok(())
}
