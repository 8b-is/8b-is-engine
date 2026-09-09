//! mesh-node — the zone node: a zero-lock Tokio event loop.
//!
//! The server loop from the hub-and-instance architecture session,
//! compiled against the engine's own world:
//!
//! * **one log, two folds** — GAIA folds the zone's eight layers (the
//!   material field everyone reads), the keeper folds each incoming delta
//!   into M/Q/H with durable refusals (world-core::fold);
//! * **the compact wire** — the same `{"t","n"{h,r,s},"a","z"}` JSON the
//!   Python actors publish, carried in 4-byte-big-endian length frames;
//! * **hub vs instance** — the same binary, two tick profiles: HUB at ten
//!   ticks per second (cities: social flow, broadcast), INSTANCE at thirty
//!   (dungeons: combat, deterministic);
//! * **zero-lock main loop** — client tasks push `ServerCommand`s into one
//!   `mpsc` channel; `tokio::select!` ticks the world without blocking I/O.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use world_core::fold::{Delta, Keeper};
use world_core::SimWorld;

/// The zone's tick profile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoneMode {
    /// cities — persistent social hubs, 10 TPS
    Hub,
    /// dungeons — ephemeral instances, 30 TPS
    Instance,
}

impl ZoneMode {
    pub fn ticks_per_second(&self) -> u64 {
        match self {
            ZoneMode::Hub => 10,
            ZoneMode::Instance => 30,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            ZoneMode::Hub => "hub",
            ZoneMode::Instance => "instance",
        }
    }
}

/// Commands the client loops send to the main tick loop.
#[derive(Debug)]
pub enum ServerCommand {
    ClientConnected { addr: SocketAddr, tx: mpsc::Sender<bytes::Bytes> },
    ClientDisconnected { addr: SocketAddr },
    Delta { addr: SocketAddr, payload: Vec<u8> },
}

use bytes::Bytes;

/// The zone node: owns the keeper + the client map, runs the tick loop.
pub struct MeshNode {
    pub brief: String,
    pub mode: ZoneMode,
    pub port: u16,
    keeper: Arc<Mutex<Keeper>>,
    clients: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Bytes>>>>,
    /// GAIA's clock — the world's own tick counter
    world_tick: Arc<Mutex<u64>>,
    /// the zone's own inhabitants — the world runs without you
    sim: Arc<Mutex<SimWorld>>,
}

impl MeshNode {
    /// spawn — bind the listener (lazily on first accept) and run the loop.
    pub async fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(("127.0.0.1", self.port)).await?;
        println!(
            "⟦ mesh-node ⟧ {} · {} zone on :{} — the world runs without you",
            self.brief,
            self.mode.name(),
            self.port
        );
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<ServerCommand>(10_000);
        // the main loop: tick the world at the zone's rate, fold deltas
        let keeper = Arc::clone(&self.keeper);
        let clients = Arc::clone(&self.clients);
        let world_tick = Arc::clone(&self.world_tick);
        let sim = Arc::clone(&self.sim);
        let brief = self.brief.clone();
        let tps = self.mode.ticks_per_second();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000 / tps));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let wt = *world_tick.lock().await + 1;
                        *world_tick.lock().await = wt;
                        let mut k = keeper.lock().await;
                        // the cast lives first — fauna fold under their own
                        // names before any player's delta
                        for (name, d) in sim.lock().await.step(wt) {
                            k.adjudicate(&name, &d);
                        }
                        let state = framed(zone_json(&brief, k.seq, &k.zone, wt));
                        let cs = clients.lock().await;
                        for tx in cs.values() {
                            let _ = tx.send(Bytes::from(state.clone())).await;
                        }
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            ServerCommand::ClientConnected { addr, tx } => {
                                println!("  player on: {addr}");
                                clients.lock().await.insert(addr, tx);
                            }
                            ServerCommand::ClientDisconnected { addr } => {
                                clients.lock().await.remove(&addr);
                            }
                            ServerCommand::Delta { addr, payload } => {
                                if let Ok(delta) = wire_delta(&payload) {
                                    let mut k = keeper.lock().await;
                                    // adjudication IS the fold: an admission
                                    // moves the material field, a refusal is
                                    // materially silent and semantically bound
                                    k.adjudicate(addr.to_string().as_str(), &delta);
                                }
                            }
                        }
                    }
                }
            }
        });
        // the accept loop
        loop {
            let (socket, addr) = listener.accept().await?;
            let cmd_tx = cmd_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_client(socket, addr, cmd_tx).await {
                    eprintln!("  connection error {addr}: {e}");
                }
            });
        }
    }
}

/// handle_client — length-prefixed frames in, a writer task out.
async fn handle_client(
    socket: TcpStream,
    addr: SocketAddr,
    cmd_tx: mpsc::Sender<ServerCommand>,
) -> std::io::Result<()> {
    let (tx, mut rx) = mpsc::channel::<Bytes>(100);
    cmd_tx
        .send(ServerCommand::ClientConnected { addr, tx })
        .await
        .map_err(|_| std::io::Error::other("channel closed"))?;
    let (mut reader, mut writer) = socket.into_split();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if writer.write_all(&msg).await.is_err() {
                break;
            }
        }
    });
    // the wire: 4-byte big-endian length + the compact JSON payload
    let mut len_buf = [0u8; 4];
    loop {
        if reader.read_exact(&mut len_buf).await.is_err() {
            break;
        }
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        if reader.read_exact(&mut payload).await.is_err() {
            break;
        }
        if cmd_tx
            .send(ServerCommand::Delta { addr, payload })
            .await
            .is_err()
        {
            break;
        }
    }
    let _ = cmd_tx.send(ServerCommand::ClientDisconnected { addr }).await;
    Ok(())
}

/// wire_delta — the compact mesh wire → the keeper's Delta. Single-char
/// keys (t/n/a/z), the ternary-wire discipline.
fn wire_delta(payload: &[u8]) -> Result<Delta, String> {
    let v: serde_json::Value = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let n = v.get("n").ok_or("no needs field")?;
    let h = n.get("h").and_then(|x| x.as_f64()).unwrap_or(0.0);
    let r = n.get("r").and_then(|x| x.as_f64()).unwrap_or(0.0);
    let s = n.get("s").and_then(|x| x.as_f64()).unwrap_or(0.0);
    Ok(Delta {
        t: v.get("t").and_then(|x| x.as_u64()).unwrap_or(0),
        h,
        r,
        s,
        action: v.get("a").and_then(|x| x.as_str()).map(str::to_string),
        asleep: v.get("z").and_then(|x| x.as_u64()).unwrap_or(0) == 1,
    })
}

/// framed — the wire's envelope: 4-byte big-endian length + the payload.
/// Every byte on the socket obeys the same framing, broadcasts included.
fn framed(json: String) -> Vec<u8> {
    let mut f = Vec::with_capacity(json.len() + 4);
    f.extend_from_slice(&(json.len() as u32).to_be_bytes());
    f.extend_from_slice(json.as_bytes());
    f
}

/// zone_json — the field every client reads: the seed, GAIA's clock, and
/// the unfolded operative state (the material fold, rendered).
fn zone_json(brief: &str, seq: u64, zone: &HashMap<String, [f64; 3]>, tick: u64) -> String {
    let gaia = world_core::gaia::gaia_state(brief, tick);
    let mut z = serde_json::Map::new();
    for (actor, needs) in zone {
        let mut n = serde_json::Map::new();
        for (k, v) in ["h", "r", "s"].iter().zip(needs.iter()) {
            n.insert((*k).to_string(), serde_json::Value::from(*v));
        }
        z.insert(actor.clone(), serde_json::Value::Object(n));
    }
    serde_json::json!({
        "zone": brief,
        "seq": seq,
        "tick": tick,
        "gaia": {
            "weather": gaia.weather,
            "gravity": gaia.gravity,
            "entropy": gaia.entropy,
            "memory": gaia.memory,
        },
        "zone_state": z,
    })
    .to_string()
}

/// new_node — a zone from a brief: GAIA's constant field, the keeper empty,
/// the cast spawned from the same seed.
pub fn new_node(brief: &str, mode: ZoneMode, port: u16) -> MeshNode {
    MeshNode {
        brief: brief.to_string(),
        mode,
        port,
        keeper: Arc::new(Mutex::new(Keeper::default())),
        clients: Arc::new(Mutex::new(HashMap::new())),
        world_tick: Arc::new(Mutex::new(0)),
        sim: Arc::new(Mutex::new(SimWorld::from_seed(brief, 4))),
    }
}

// tests — the compact wire parses; the state renders GAIA + the fold
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_round_trips() {
        let d = wire_delta(b"{\"t\":7,\"n\":{\"h\":0.2,\"r\":0.9,\"s\":0.9},\"a\":\"forage the field\",\"z\":0}").unwrap();
        assert_eq!(d.t, 7);
        assert_eq!(d.h, 0.2);
        assert_eq!(d.action.as_deref(), Some("forage the field"));
        assert!(!d.asleep);
    }

    #[test]
    fn state_carries_gaia_and_the_fold() {
        let mut k = Keeper::default();
        k.adjudicate("player-1", &wire_delta(b"{\"t\":1,\"n\":{\"h\":0.9,\"r\":0.9,\"s\":0.9}}").unwrap());
        let json = zone_json("sanctuary", k.seq, &k.zone, 30);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let expected = world_core::gaia::gaia_state("sanctuary", 30);
        assert_eq!(v["gaia"]["weather"].as_str().unwrap(), expected.weather);
        assert!(v["zone_state"]["player-1"].is_object());
    }

    #[tokio::test]
    async fn the_loop_folds_and_broadcasts() {
        // spawn a node on an ephemeral port, connect, send a delta, read
        // the hub broadcast — the zero-lock loop end to end
        let port = find_port().await;
        let node = new_node("sanctuary", ZoneMode::Hub, port);
        let handle = tokio::spawn(async move { node.run().await });
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let payload = b"{\"t\":1,\"n\":{\"h\":0.5,\"r\":0.9,\"s\":0.9}}";
        stream
            .write_all(&(payload.len() as u32).to_be_bytes())
            .await
            .unwrap();
        stream.write_all(payload).await.unwrap();

        // the next hub tick should carry the folded zone state
        let mut buf = Vec::new();
        let mut len_buf = [0u8; 4];
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                stream.read_exact(&mut len_buf).await.unwrap();
                let len = u32::from_be_bytes(len_buf) as usize;
                buf.clear();
                buf.resize(len, 0);
                stream.read_exact(&mut buf).await.unwrap();
                let text = String::from_utf8_lossy(&buf);
                if text.contains("tick") && text.contains("zone_state") {
                    return text.to_string();
                }
            }
        })
        .await
        .expect("broadcast timeout");
        let text = String::from_utf8_lossy(&buf).to_string();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(
            v["zone_state"].as_object().map(|o| o.len() >= 1).unwrap_or(false),
            "the delta was folded into the broadcast: {text}"
        );

        handle.abort();
    }

    async fn find_port() -> u16 {
        use std::net::TcpListener;
        TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port()
    }
}