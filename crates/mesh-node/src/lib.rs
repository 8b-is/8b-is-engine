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
use serde::Serialize;
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
    /// input sequencing: the last input seq each player applied (drift = the
    /// gap between a client's prediction and the authoritative zone state)
    applied: Arc<Mutex<HashMap<SocketAddr, AppliedEntry>>>,
    /// lifecycle: an instance zone retires after this many empty ticks
    /// (hub zones are persistent — the seed resurrects instances anytime)
    retire_after: u64,
    /// the durable log: every adjudication is appended to the mmap ledger,
    /// so the world survives restarts — fold(seed, H) = M, re-folded on boot
    ledger: Arc<Mutex<world_core::Ledger>>,
    /// the frame arena: the cast materialized as entities, synced each tick
    /// (positions + needs columns — the future render's world)
    arena: Arc<Mutex<world_core::Entities>>,
}

/// The commit record for one player's input — consumed vs committed, made
/// distinct: `in` is the client's proposed sequence, `seq` is the world's
/// ledger position where the transition (admission or refusal) was bound,
/// and `out` says which fold the event landed in. A refusal is durable: it
/// still owns a ledger position, the material fold just did not move.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedEntry {
    /// the client's proposed input sequence
    pub r#in: u64,
    /// the world's committed ledger position (the keeper's seq)
    pub seq: u64,
    /// which fold the event entered: "admitted" (M+S) or "refused" (S only)
    pub out: &'static str,
    /// the world tick the event was folded on
    pub tick: u64,
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
        let keeper = Arc::clone(&self.keeper);
        let clients = Arc::clone(&self.clients);
        let world_tick = Arc::clone(&self.world_tick);
        let applied = Arc::clone(&self.applied);
        let sim = Arc::clone(&self.sim);
        let ledger = Arc::clone(&self.ledger);
        let arena = Arc::clone(&self.arena);
        let brief = self.brief.clone();
        let tps = self.mode.ticks_per_second();
        let retire_after = self.retire_after;
        let is_instance = self.mode == ZoneMode::Instance;
        // one loop, everything: accepts, the world's tick, the commands
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000 / tps));
        let mut idle = 0u64;
        loop {
            tokio::select! {
                res = listener.accept() => {
                    let (socket, addr) = res?;
                    let cmd_tx = cmd_tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(socket, addr, cmd_tx).await {
                            eprintln!("  connection error {addr}: {e}");
                        }
                    });
                }
                _ = interval.tick() => {
                    let wt = *world_tick.lock().await + 1;
                    *world_tick.lock().await = wt;
                    // lifecycle: an instance zone retires when it stays empty;
                    // spin-up was fold-from-seed, so the seed resurrects it
                    let empty = clients.lock().await.is_empty();
                    if empty { idle += 1 } else { idle = 0 }
                    if is_instance && retire_after < u64::MAX && idle >= retire_after {
                        println!("⟦ instance retired ⟧ empty for {idle} ticks — the seed can resurrect {brief} anytime");
                        return Ok(());
                    }
                    let mut k = keeper.lock().await;
                    // the cast lives first — fauna fold under their own
                    // names before any player's delta, and every fold is
                    // appended to the durable ledger (the world's log)
                    let cast = sim.lock().await.step(wt);
                    for (name, d) in &cast {
                        let v = k.adjudicate(name, d);
                        let out = match v {
                            world_core::fold::Verdict::Admitted => "admitted",
                            world_core::fold::Verdict::Refused(_) => "refused",
                        };
                        ledger_append(&ledger, name, d, out).await;
                    }
                    // the frame arena reflects the fold, in lockstep
                    let mut s = sim.lock().await;
                    let mut a = arena.lock().await;
                    s.sync_entities(&mut a);
                    let applied_snapshot = applied.lock().await.clone();
                    let state = framed(zone_json(&brief, k.seq, &k.zone, wt, &applied_snapshot));
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
                                let verdict = k.adjudicate(addr.to_string().as_str(), &delta);
                                // consumed vs committed: the entry records the
                                // WORLD's ledger position, not the client's
                                // number — a refusal owns a position too
                                let input_seq = serde_json::from_slice::<serde_json::Value>(&payload)
                                    .ok()
                                    .and_then(|v| v.get("s").and_then(|x| x.as_u64()))
                                    .unwrap_or(0);
                                let committed = k.seq;
                                let out = match verdict {
                                    world_core::fold::Verdict::Admitted => "admitted",
                                    world_core::fold::Verdict::Refused(_) => "refused",
                                };
                                applied.lock().await.insert(
                                    addr,
                                    AppliedEntry {
                                        r#in: input_seq,
                                        seq: committed,
                                        out,
                                        tick: *world_tick.lock().await,
                                    },
                                );
                                ledger_append(&ledger, addr.to_string().as_str(), &delta, out).await;
                            }
                        }
                    }
                }
            }
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

/// zone_json — the field every client reads: the seed, GAIA's clock, the
/// unfolded operative state, and the reconciliation map (each player's last
/// applied input seq — a client whose prediction drifted past this knows
/// exactly where the authoritative world stands).
fn zone_json(
    brief: &str,
    seq: u64,
    zone: &HashMap<String, [f64; 3]>,
    tick: u64,
    applied: &HashMap<SocketAddr, AppliedEntry>,
) -> String {
    let gaia = world_core::gaia::gaia_state(brief, tick);
    let mut z = serde_json::Map::new();
    for (actor, needs) in zone {
        let mut n = serde_json::Map::new();
        for (k, v) in ["h", "r", "s"].iter().zip(needs.iter()) {
            n.insert((*k).to_string(), serde_json::Value::from(*v));
        }
        z.insert(actor.clone(), serde_json::Value::Object(n));
    }
    let mut ap = serde_json::Map::new();
    for (addr, e) in applied {
        ap.insert(addr.to_string(), serde_json::to_value(e).unwrap_or_default());
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
        "applied": ap,
    })
    .to_string()
}

/// new_node — a zone from a brief: GAIA's constant field, the keeper empty,
/// the cast spawned from the same seed.
/// delta_wire — the compact mesh wire from a Delta (the ledger's record).
fn delta_wire(d: &Delta) -> serde_json::Value {
    serde_json::json!({
        "t": d.t,
        "n": { "h": d.h, "r": d.r, "s": d.s },
        "a": d.action,
        "z": if d.asleep { 1 } else { 0 },
    })
}

/// ledger_append — one adjudication, made durable: actor + wire record.
async fn ledger_append(ledger: &Arc<Mutex<world_core::Ledger>>, actor: &str, d: &Delta, out: &str) {
    let rec = serde_json::json!({ "a": actor, "o": out, "w": delta_wire(d) });
    let mut l = ledger.lock().await;
    let mut buf = serde_json::to_vec(&rec).unwrap_or_default();
    buf.push(b'\n'); // the ledger is line-delimited — the fold splits on it
    let _ = l.append(&buf);
}

pub fn new_node(brief: &str, mode: ZoneMode, port: u16) -> MeshNode {
    let ledger_path = std::env::var("VAKED_MESH_LEDGER")
        .unwrap_or_else(|_| format!("out/mesh-node-{port}.log"));
    if let Some(parent) = std::path::Path::new(&ledger_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let ledger = world_core::Ledger::open(std::path::Path::new(&ledger_path))
        .unwrap_or_else(|e| panic!("ledger open failed: {e}"));
    let node = MeshNode {
        brief: brief.to_string(),
        mode,
        port,
        keeper: Arc::new(Mutex::new(Keeper::default())),
        clients: Arc::new(Mutex::new(HashMap::new())),
        world_tick: Arc::new(Mutex::new(0)),
        sim: Arc::new(Mutex::new(SimWorld::from_seed(brief, 4))),
        applied: Arc::new(Mutex::new(HashMap::new())),
        retire_after: match mode {
            ZoneMode::Hub => u64::MAX,
            ZoneMode::Instance => 60,
        },
        ledger: Arc::new(Mutex::new(ledger)),
        arena: Arc::new(Mutex::new(world_core::Entities::default())),
    };
    // the cast takes its place in the arena: one entity per fauna
    *node.arena.try_lock().unwrap() = node.sim.try_lock().unwrap().materialize();
    // restart-safe: re-fold the durable log into the keeper — the world
    // that was, becomes the world that is
    let mut max_tick = 0u64;
    if let Ok(bytes) = node.ledger.try_lock().unwrap().read_all() {
        for line in bytes.split(|b| *b == b'\n') {
            if line.is_empty() {
                continue;
            }
            if let Ok(rec) = serde_json::from_slice::<serde_json::Value>(line) {
                if let (Some(a), Some(w)) = (rec.get("a").and_then(|v| v.as_str()), rec.get("w")) {
                    if let Ok(d) = wire_delta(&serde_json::to_vec(w).unwrap_or_default()) {
                        node.keeper.try_lock().unwrap().adjudicate(a, &d);
                        max_tick = max_tick.max(d.t);
                    }
                }
            }
        }
    }
    // Phoenix: the strategy returns, the process does not rewind. The
    // world's clock continues where the ledger left off, so the cast's new
    // ticks are continuations — never replays refused by the keeper.
    *node.world_tick.try_lock().unwrap() = max_tick;
    node
}

impl MeshNode {
    /// with_retire — override the empty-tick retirement threshold.
    pub fn with_retire(mut self, ticks: u64) -> Self {
        self.retire_after = ticks;
        self
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
        let json = zone_json("sanctuary", k.seq, &k.zone, 30, &HashMap::new());
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

    #[tokio::test]
    async fn a_refusal_is_a_committed_record() {
        // consumed vs committed, made distinct: a poison input (forage while
        // attesting comfort) is REFUSED, yet it still owns a ledger position
        // in the applied map — the refusal is durable, the material fold
        // just did not move
        let port = find_port().await;
        let node = new_node("sanctuary", ZoneMode::Instance, port);
        let handle = tokio::spawn(async move { node.run().await });
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        // a benign birth first (the first delta is the actor's attestation
        // of existence), then the poison — which must be refused durable
        let birth = br#"{"t":1,"s":6,"n":{"h":0.9,"r":0.9,"s":0.9}}"#;
        stream.write_all(&(birth.len() as u32).to_be_bytes()).await.unwrap();
        stream.write_all(birth).await.unwrap();
        let poison = br#"{"t":2,"s":7,"n":{"h":0.9,"r":0.9,"s":0.9},"a":"forage the field"}"#;
        stream.write_all(&(poison.len() as u32).to_be_bytes()).await.unwrap();
        stream.write_all(poison).await.unwrap();

        let mut buf = Vec::new();
        let mut len_buf = [0u8; 4];
        let entry = tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                stream.read_exact(&mut len_buf).await.unwrap();
                let len = u32::from_be_bytes(len_buf) as usize;
                buf.clear();
                buf.resize(len, 0);
                stream.read_exact(&mut buf).await.unwrap();
                let text = String::from_utf8_lossy(&buf).to_string();
                let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                if let Some(e) = v["applied"].as_object().and_then(|m| m.values().find(|e| e["in"] == 7)) {
                    return e.clone();
                }
            }
        })
        .await
        .expect("refusal record timeout");

        assert_eq!(entry["out"], "refused");
        assert!(entry["seq"].as_u64().unwrap() > 0, "a refusal owns a ledger position");
        handle.abort();
    }

    async fn find_port() -> u16 {
        use std::net::TcpListener;
        TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port()
    }

    #[tokio::test]
    async fn the_world_survives_restart() {
        // the durable ledger: a player's fold is appended; a fresh node on
        // the same log re-folds it — fold(seed, H) = M across restarts
        let dir = std::env::temp_dir().join(format!("mesh-node-restart-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ledger_path = dir.join("world.log");
        std::env::set_var("VAKED_MESH_LEDGER", &ledger_path);

        let port = find_port().await;
        let node1 = new_node("sanctuary", ZoneMode::Hub, port);
        let handle1 = tokio::spawn(async move { node1.run().await });
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let p = br#"{"t":1,"s":1,"n":{"h":0.5,"r":0.9,"s":0.9}}"#;
        stream.write_all(&(p.len() as u32).to_be_bytes()).await.unwrap();
        stream.write_all(p).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        handle1.abort();

        // restart on the same ledger — the world that was, becomes
        let node2 = new_node("sanctuary", ZoneMode::Hub, port + 1);
        let k2 = node2.keeper.lock().await;
        assert!(k2.seq > 0, "the ledger re-folded into the keeper");
        assert!(k2.zone.len() >= 1, "the player's fold survived the restart");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn an_empty_instance_retires() {
        // an instance with no players retires itself — the seed resurrects
        let port = find_port().await;
        let node = new_node("the hollow instance", ZoneMode::Instance, port).with_retire(3);
        let mut handle = tokio::spawn(async move { node.run().await });
        let res = tokio::time::timeout(std::time::Duration::from_secs(3), &mut handle).await;
        assert!(res.is_ok(), "an empty instance must retire itself");
        assert!(res.unwrap().unwrap().is_ok());
    }

    #[tokio::test]
    async fn input_sequencing_reconciles() {
        // a client's input seqs fold into the reconciliation map — the
        // broadcast carries the last applied seq per player
        let port = find_port().await;
        let node = new_node("sanctuary", ZoneMode::Instance, port);
        let handle = tokio::spawn(async move { node.run().await });
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let p1 = br#"{"t":41,"s":41,"n":{"h":0.5,"r":0.9,"s":0.9}}"#;
        stream.write_all(&(p1.len() as u32).to_be_bytes()).await.unwrap();
        stream.write_all(p1).await.unwrap();

        // read broadcasts until the applied map reports seq 41
        let mut buf = Vec::new();
        let mut len_buf = [0u8; 4];
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                stream.read_exact(&mut len_buf).await.unwrap();
                let len = u32::from_be_bytes(len_buf) as usize;
                buf.clear();
                buf.resize(len, 0);
                stream.read_exact(&mut buf).await.unwrap();
                let text = String::from_utf8_lossy(&buf).to_string();
                let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                if v["applied"].as_object().map(|m| m.values().any(|e| e["in"] == 41 && e["out"] == "admitted")).unwrap_or(false) {
                    return v;
                }
            }
        })
        .await
        .expect("reconciliation timeout");

        let p2 = br#"{"t":42,"s":42,"n":{"h":0.5,"r":0.9,"s":0.9}}"#;
        stream.write_all(&(p2.len() as u32).to_be_bytes()).await.unwrap();
        stream.write_all(p2).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                stream.read_exact(&mut len_buf).await.unwrap();
                let len = u32::from_be_bytes(len_buf) as usize;
                buf.clear();
                buf.resize(len, 0);
                stream.read_exact(&mut buf).await.unwrap();
                let text = String::from_utf8_lossy(&buf).to_string();
                let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                if v["applied"].as_object().map(|m| m.values().any(|e| e["in"] == 42 && e["out"] == "admitted")).unwrap_or(false) {
                    return;
                }
            }
        })
        .await
        .expect("second reconciliation timeout");

        handle.abort();
    }
}