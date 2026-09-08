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

## v1.x — the core engine (the Rust/Go runtime)

- [ ] the Rust core: tick loop, frame arenas, entity structs (wgpu +
  Rapier3D + SDF)
- [ ] the Go multiplexer: sync.Pool, mmap ring buffers, the NATS mesh in
  production form (the actor skeleton compiled, not scripted)
- [ ] the fauna stack in Rust (the floors are the JS twins)
- [ ] the needs/goals scheduler as a first-class reducer (the Dwarf-Fortress
  clock, the mesh-NPC grown up)

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
