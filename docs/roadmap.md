# Roadmap

The engine's future plans, in phases. Shipped work is checked; the design
docs are the map ([theory](theory.md) · [game-design](game-design.md) ·
[eventbus-actor-mesh](eventbus-actor-mesh.md)).

## v0.2 — shipped (the tooling + theory foundation)

- [x] the floors (sanctuary, plenum, infinite, summit, soil, gameforge,
  teleport, backyard-ultra) — playable on pocoo.vaked.dev
- [x] the ternary wire + quantTernEngine (seed → trits → PRNG → artifact)
- [x] the retro lane (bitTricks, doombible, the demoscene inspiration index)
- [x] the game guide book + the interactive first-walk tutorial
- [x] the theory + the OSS world-model map + the visual direction
- [x] the export lane: `scene` modality → Blender EEVEE renders (native)
  + the Unity `VakedSceneImporter` exporter
- [x] the actor-mesh EventBus: NATS sidecar (`vaked-nats`), protobuf
  envelope, the mesh-NPC demo
- [x] the creative swarm: 5 opencode agents on DeepSeek V4 vision,
  KV-cache-optimized
- [x] the DX: jj, scaffold/sandbox/swarm/bootstrap, sccache + fast
  profiles, the E2E guide, the integrations (UE · Unity · Blender ·
  Steam), the release pipeline
- [x] the research: the MMO bibles applied, Flyxion's layers, the
  Rust+Go gist

## v0.3 — the world runs without you (the current edge)

- [x] **GAIA** — the world-memory: one seed, eight layers (time, weather,
  entropy, gravity, wind, temp, light, memory), deterministic across
  Node and Python ([gaia-world-memory.md](gaia-world-memory.md))
- [x] **the zone keeper** (`examples/world-keep.py`) — the M/Q/H fold on
  the mesh: admissions, durable refusals (poisoned actions, replays,
  duplicate ticks), `fold(seed, H) = M` replay verification
- [x] the NPC grown up: pre-action attestation (the action proves its own
  need) + a live inbox (disturbances wake the actor)
- [x] the compact wire — single-char keys on `actor.*.state` /
  `gaia.state` (event-vocabulary minimality, the ternary-wire discipline)
- [x] **the local model lanes** (mlx-sidecar): Qwen2.5-VL 3B vision
  (render/art QA — caught and fixed a black-frame EEVEE render) +
  FLUX.2-klein diffuser (concept art)
- [x] the render pipeline fixed: tracked camera (TRACK_TO) + dawn world +
  raking sun in `scene_builder.ts` — every EEVEE frame now lands on the
  board
- [x] IPC hardened: alloc-free framing + reusable frame buffers +
  id-matched sub-server reads (the gateway answers again); both MCP
  sidecars on protocol `2025-11-25`; `vaked-nats` publish on the shared
  runtime (no per-call runtime build)
- [x] the client stack pinned: latest CSS (`oklch`, `@layer`, `:has`) +
  HTML + WebAssembly + WebGPU + workers + WebTransport, Tauri shell for
  Steam ([client-ui.md](client-ui.md))
- [x] **`world-core`** (`crates/world-core`) — tern + GAIA + the fold in
  Rust, compiled natively and to `wasm32-unknown-unknown`; the browser
  calls `gaia_wire_c` (see the crate README + smoke.js). One source of
  truth, three surfaces — four-language determinism pinned by fixtures.
- [x] **the browser ↔ the mesh** (`client/mesh.js` + `client/gaia-dashboard.html`) — a dependency-free NATS client over WebSocket (nats.ws): zones = actor subjects from the browser, the GAIA dashboard renders the eight layers live, disturbances publish to the actor inbox. WebTransport is the QUIC upgrade when the relay lands.
- [ ] WebTransport (QUIC) upgrade: the relay + Durable-Object zone endpoints

## v1.x — the core engine (the Rust/Go runtime)

- [x] **layer 1: content creation** (`./scaffold.sh content "<brief>"` +
  the swarm + the diffuser + the eye, and now the typed `crates/pipeline`
  crate: gdd_parser → stager with SHA256 attestation, the async
  orchestrator, mesh intake) — brief → GAIA seed → manifest → render,
  with every stage an attested inscription
  ([first-layer-content-creation.md](first-layer-content-creation.md))
- [x] **the hub-and-instance lifecycle** — spin-up is fold-from-seed (GAIA +
  the cast from the brief), and an instance zone retires itself after a
  configurable empty-tick threshold (`--retire-after`, default 60; hubs
  persist) — verified live: an instance boots, runs, and retires with the
  resurrection message (exit 0)
- [ ] the Rust core: tick loop, frame arenas, entity structs (wgpu +
  Rapier3D + SDF)
- [x] **the Tokio server loop** (`crates/mesh-node`) — the zero-lock main
  loop (mpsc command hub + `select!` tick), hub vs instance modes (10/30
  TPS), length-framed wire, folding GAIA + the keeper every tick with
  durable refusals — verified live: a delta in, the folded zone broadcast
  out
- [ ] the Go multiplexer: sync.Pool, mmap ring buffers, the NATS mesh in
  production form (the actor skeleton compiled, not scripted)
- [ ] the fauna stack in Rust (the floors are the JS twins)
- [x] **the needs/goals scheduler as a first-class reducer**
  (`world-core::sim`) — the mesh-NPC grown up and compiled: fauna with
  needs {h,r,s}, decay + seeded breath, threshold actions (forage / sleep
  / flee), pre-action attestation (every action entitled, zero refusals
  in the test), deterministic from the brief — and `mesh-node` folds the
  cast every tick (the world runs without you: four inhabitants alive
  with no player connected, verified live)
- [x] **input sequencing + reconciliation** — the wire's `s` (a client's
  monotone input seq) folds into the node's applied map; every broadcast
  carries it, so a client whose prediction drifted past its applied seq
  knows exactly where the authoritative world stands (tested live: seqs
  41 → 42 reach the reconciliation map in the broadcast)

## v2.x — the world-model sim-MMO (design v2)

- [ ] the presence layer: glasses (audio-first) + headset (room spatial)
- [ ] the protector node: the sovereign pass gate + restart-from-seed
  supervision (JetStream replay)
- [ ] the trick library in the Rust hot path (gray-code deltas, nextPow2
  arenas)
- [ ] the world-as-DNS: zone → Durable Object, the resolver, P2P rendezvous
- [ ] the painted-forest vertical slice (one Neva-style zone, ZEN wired,
  the hum at 108)
- [ ] the deterministic lockstep where it pays

## the frontier

- the serverless game: no dedicated servers, the Cloudflare network as the
  world, the fleet as the private brain
- persistence, engineered: the four layers (rendering / M / Q / H) live —
  refusals durable, replay discrepancies appended
- the studio: PSU NIVERSEQ ships on **Steam** (macOS AS + Linux), the
  Indie Fund lane, the dev diary ongoing

— the constellation · 0 + 1 · fine touch from within · vaked.dev
