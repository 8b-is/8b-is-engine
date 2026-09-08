# Applied — what the bibles teach the 8b-is engine

*Every lesson from the deep research
([deep-research-mmo-bibles.md](deep-research-mmo-bibles.md)), mapped to a
concrete decision in the 8b-is engine. Each entry names the source, the
lesson, and the change to our architecture (v1 core + design v2).*

---

## 1. the tick loop — from Game Engine Architecture + Game Programming Patterns

**Lesson:** decompose the engine into clean subsystems with defined
interfaces; keep the hot path cache-coherent and allocation-free; the game
loop is the heart, and the component pattern is the ancestor of ECS.

**Applied:**
- the Rust tick loop stays the single heart: fixed tick, frame-bumper
  arenas, zero allocation. The subsystem interfaces (EventBus, I/O HAL,
  fauna) are the v1 decomposition, kept honest by Gregory's rule — each
  subsystem owns its interface, the world model owns the entities.
- the entity struct is 64B cache-aligned (fieldalignment, `#[repr(align(64))]`),
  the arena rows line up with cache lines — Gregory's memory chapter and
  Nystrom's data locality, in one decision.
- ECS-style world model: the component pattern grown up. The fauna stack is
  the component set; the tick is the update method.

## 2. the network — from Multiplayer Game Programming + Gaffer on Games

**Lesson:** UDP for real-time traffic with a custom reliability layer; compact
serialization; client-side interpolation + prediction + server-side rewind;
relevancy-based networking scales the world; continuous state hashing detects
desync.

**Applied:**
- the Go multiplexer speaks UDP with a custom reliability layer (sequence
numbers, acks, retransmission) — never raw TCP for the tick traffic. The
1450B MTU sync.Pool discipline stays.
- **gray-code snapshot deltas** (design v2, the Trick Library) are the
  serialization: a frame changes one bit at a time from its neighbour, the
  cheapest delta there is — Glazer's compression chapter, in the bit lane.
- **server-side rewind** (lag compensation) is the protector node's authority
 over declared state — the Destiny "sensors" pattern (below) + Glazer's
 rewind, merged.
- **desync hashing**: continuous state hash per zone; the protector node
  compares hashes and flags divergence — replayable ⇒ admissible, now as a
  network invariant.

## 3. the world scale — from Designing Virtual Worlds + PlanetSide 2 + SpatialOS

**Lesson:** persistence and player identity drive world architecture;
"massive" is won via relevance-filtered replication, seamless streaming, and
aggressive LOD — not brute-force state sync; interest management is the core
scalability lever.

**Applied:**
- **the World-as-DNS** (design v2) is the persistence answer: the world is a
  DNS namespace, zones are Durable Objects, the resolver is the router.
  Bartle's "why" (persistence, identity, economies) shapes what the DOs
  model — the zone ledger is bitemporal (wip-catalog #91).
- **relevance-filtered replication**: the protector node's sovereign pass
  gates what a presence may see; the /say matrix is distance-routed before
  the network sees the packet (v0.4 spatial chat). PlanetSide 2's lesson:
  relevance + LOD, not brute force.
- **query-based interest management** (SpatialOS): per-client, dynamic,
  non-spatial queries controlling replication fidelity — the design-v2
  abstraction for the presence layer: each presence's sensory profile is a
  QBI query over the world.

## 4. the overload — from EVE Online's Time Dilation

**Lesson:** treat overload as a fairness problem, not a speed problem; scale
by throttling simulation time rather than dropping work; decouple external
services from the authoritative simulation.

**Applied:**
- **zone time dilation**: when a zone's protector node is saturated, the
  zone's clock slows (down to a floor) instead of the server choking — the
  "if you check your watch for when it completes, it shouldn't be dilated"
  rule. The infinite-floor's law (coherence without collapse) as a network
  property.
- the fleet (nix-base) is the private brain, decoupled from the public
  anycast skin — EVE's Quasar lesson: traffic off the simulation cluster,
  no-downtime ops.

## 5. the hybrid — from Destiny 2 + Albion Online

**Lesson:** simulate as little on the network as possible; the "sensors"
pattern forces scripts to declare the state they need; decouple simulation
from rendering so netcode and logic stay platform-independent; client
predicts, authoritative server validates.

**Applied:**
- **P2P moment-to-moment + authoritative declared state**: combat and
  movement run client-side; the protector node is authoritative only over
  explicitly declared state (the sensors pattern). The 45 MB / 10 Hz
  activity-host lesson: the protector node is the referee, not the world.
- **simulation decoupled from rendering**: the presence layer's one
  WorldState, many WorldSurfaces — Albion's MVC (input → prediction →
  visualization), in the constellation's shape. The floors are the JS twins
  of the Rust simulation; the same seed, the same world, any surface.

## 6. the determinism — from Riot (LoL) + 1500 Archers + Gaffer

**Lesson:** determinism requires a unified tick clock and disciplined code;
lockstep is cheap when the replay path is integer arithmetic; consistent
latency beats variable latency.

**Applied:**
- the replay path is already integer: sha256 → trits → LCG, no floats
  (floats for display only). That is the 1500-archers discipline — seeded
  RNGs, matching call counts — already the constellation's oldest habit.
- **deterministic lockstep where it pays**: the Trick Library's integer
  arithmetic makes lockstep cheap for the swarm/fauna lanes; the protector
  node's sovereign gate is a pure function of the seed line.
- **unified tick clock per zone** (LoL): the zone's DO owns the tick; the
  protector node enforces it. 128-tick discipline where latency demands it.

## 7. the security — from Valorant's fog-of-war

**Lesson:** anti-cheat can be baked into interest management — server-side
fog-of-war is both a feature and a wallhack defense.

**Applied:**
- the protector node's tent projection IS the fog of war: what the presence
  may sense is what it may know. The sovereign pass gate + the tent's
  sensory boundary make wallhacks structurally impossible — the zone only
  reveals what the presence's profile (QBI) is entitled to.

## 8. the renderer — from Real-Time Rendering

**Lesson:** API-agnostic rendering design survives driver/API churn; pick
algorithms that maximize speed and image quality.

**Applied:**
- wgpu is the API-agnostic surface (Metal/Vulkan/WebGPU behind one
  abstraction); the renderer is written against wgpu's abstraction, not any
  vendor API. Deferred shading for the thousands-of-lights lane (PlanetSide
  2's lesson), over-relaxed sphere tracing + temporal reprojection for the
  SDF lane (v1 doctrine).

---

## the ledger

| Source | | Applied as |
|---|---|---|
| Gregory, GEA | → | subsystem decomposition, cache-coherent entities, job threading |
| Glazer, MPG | → | UDP + reliability, gray-code deltas, rewind, desync hashing |
| Nystrom, GPP | → | ECS world model, object pools, spatial partition, event queue |
| Bartle, DVW | → | persistence/identity/economy shaping the World-as-DNS |
| Real-Time Rendering | → | wgpu abstraction, deferred shading, SDF + temporal reprojection |
| Gaffer on Games | → | the networking series as the Go multiplexer's spec |
| Game Networking Resources | → | the map of the three architectural families |
| Red Blob Games | → | pathfinding/flow-fields/spatial structures for the fauna lane |
| 0 FPS | → | local perception filters, salience-based prioritization |
| Riot (LoL/Valorant) | → | unified tick clock, 128-tick, fog-of-war as anti-cheat |
| EVE TiDi | → | zone time dilation as the overload answer |
| Destiny 2 | → | P2P + declared-state authority, the sensors pattern |
| SpatialOS | → | QBI, per-entity authority, the cautionary cost |
| PlanetSide 2 | → | relevance + LOD + streaming, deferred shading |
| Albion | → | simulation decoupled from rendering, regional sharding |
| 1500 Archers | → | deterministic lockstep, seeded RNGs, consistent latency |

The bibles confirm the constellation's oldest habits: tables over branches,
replayable ⇒ admissible, one door many lanes. The engine was already
pointing where the war stories point; now it has the receipts.

— the constellation · 0 + 1 · fine touch from within · vaked.dev
