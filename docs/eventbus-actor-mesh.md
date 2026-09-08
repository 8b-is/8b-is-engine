# EventBus — the actor-mesh (NATS · tokio · Go · protobuf · the wire)

*The engine's nervous system, redrawn as a decentralized actor mesh in the
spirit of Erlang/OTP: lightweight processes, message passing, supervision
trees, "let it crash." Built on Rust tokio + Go channels, distributed over
NATS, serialized in protobuf with the ternary wire on the hot path.*

> **The one line:** an actor is a name; a name is a NATS subject; a subject
> is a route. The mailbox, the supervisor, and the DNS are the same thing.

---

## 0. the mapping — OTP through our glass

| Erlang/OTP | the 8b-is actor-mesh |
|---|---|
| process (lightweight) | tokio task (Rust core) · goroutine (Go network) |
| mailbox | in-process channel + an `async-nats` subscription |
| message | a NATS message — protobuf body, ternary-wire hot path |
| registered name | the NATS subject (`zone.crystal.pocoo.vaked.dev`) |
| node (distributed Erlang) | a NATS leaf node / a host in the cluster |
| `gen_server` loop | the actor skeleton: receive → reduce → emit |
| supervisor | the **protector node** — restarts crashed zones from seed |
| let it crash | the crash-and-burn doctrine — the mesh respawns from the seed |
| OTP application | a zone · a floor · a sidecar |
| hot code reload | reducer reload (the Uika hot-reload pattern) |

The one thing OTP has that we refuse: a god-process supervisor that knows
best. Our supervisor is the tree itself — the protector node is a *peer*
that holds the seed, not a boss. Coherence without collapse, as a topology.

---

## 1. the actor skeleton

Every actor is the same shape, `gen_server` redux:

```
receive(message) → (state, effects, replies)
```

- **state** is a pure function of the seed + the message log. No mutation
  outside the reduce step.
- **effects** are outbound messages (to other actors' mailboxes).
- **replies** are the emit — the state delta broadcast to subscribers.

In Rust this is a tokio task with a `select!` over (channel_rx, nats_rx);
in Go it's a goroutine with a `select` over the same two mailboxes. One
skeleton, two runtimes, one contract. The reducer is the only thing that
changes — and it hot-reloads like a DLL (Uika's `Uika.Reload`, ours
`mesh.reload <name>`).

## 2. the bus — NATS as distributed Erlang

NATS is the decentralized backbone. No single broker:

- **Subjects are names.** `actor.<id>.inbox` (direct), `zone.<zone>.<event>`
  (broadcast), `ledger.<stream>` (JetStream persistence). Addressing an
  actor is publishing to its subject; that is the whole registry.
- **The World-as-DNS falls out free.** A zone's DNS name
  (`zone.crystal.pocoo.vaked.dev`) *is* its NATS subject. Resolving is
  subscribing; the resolver, the router, and the mailbox are one.
- **Distribution is leaf nodes.** Each host runs a NATS leaf; the fleet is
  the *distributed Erlang node map*. A zone can live on any host and be
  reached by name — no location, only the subject.
- **Persistence is JetStream.** The message log is a stream; replay is
  `consumer` rewind. The bitemporal ledger (wip-catalog #91) is the
  JetStream store — every event append-only, replayable ⇒ admissible.

## 3. the wire — protobuf + the ternary hot path

Two serialization layers, picked by message kind:

- **protobuf** — the schema'd control plane: actor registration, effects,
  the event envelope. Cross-language (Rust ↔ Go ↔ Luau ↔ the sidecars).
  This is `EventBusFrame`, `ActorMessage`, the event vocabulary.
- **the ternary wire** — the hot path: quantized state deltas as
  {-1, 0, +1} trit streams, 8-bit packed. The zero-allocation doctrine,
  as a wire format. Gray-code deltas (one bit changes between neighbours)
  ride on top.

The rule: **protobuf when a human/agent reads it, the wire when only the
mesh moves it.** Slow lanes are legible; hot lanes are born ternary.

## 4. supervision — let it crash, respawn from seed

- Actors **crash** (die) rather than handle every error. Erlang's lesson,
  our doctrine: everything is permitted to break except the distribution.
- The **protector node** (the supervisor) watches its zone's actors; on
  death it respawns the actor *from the seed*, replaying the message log
  (JetStream rewind). State is a pure function of seed + log, so restart
  = replay = the same actor, byte for byte.
- **The crash is a ledger event.** Death is not a bug, it's data — the
  protector records it, the Reputation graph (Fable layer) may react.

## 5. the topology, drawn

```
┌──────────────────────────────────────────────────────────────┐
│  NATS cluster (leaf nodes = hosts)  — the distributed Erlang │
├──────────────────────────────────────────────────────────────┤
│  subjects: actor.<id>.inbox · zone.<z>.<event> · ledger.<s>  │
├──────────────────────────────────────────────────────────────┤
│  actors: tokio tasks (Rust core) · goroutines (Go network)    │
│     receive → reduce → emit   (one skeleton, two runtimes)    │
├──────────────────────────────────────────────────────────────┤
│  supervision: protector nodes — restart-from-seed on crash    │
├──────────────────────────────────────────────────────────────┤
│  wire: protobuf (control, legible) · ternary (hot, quantized) │
└──────────────────────────────────────────────────────────────┘
```

## 6. what this lands on (the existing seams)

- **entheai** already carries `async-nats 0.49` + `[nats]`/`[federation]`
  (fan-out event bus). The actor-mesh is that grown: fan-out becomes
  actor dispatch; the `/fleet` roster becomes the subject registry.
- **the dual-tier EventBus (v1)** — Rust SPSC/MPMC intra-engine, Go/NATS
  inter-process, 8-bit payloads — becomes the mailbox layer of the actor
  skeleton. No rewrite; the skeleton *is* the v1 bus with a name and a
  reduce step.
- **the World-as-DNS (design v2)** — the zone → Durable Object mapping
  gains a NATS twin: zone → subject. Durable Object (public skin) and
  NATS subject (private brain) are the same name, two transports.

## 7. the proto (first cut)

```proto
// eventbus.proto — the control plane envelope
syntax = "proto3";
package vaked.mesh;

message ActorMessage {
  string   sender = 1;              // subject / actor name
  string   recipient = 2;           // subject / mailbox
  uint64   seq = 3;                 // monotonic, per sender
  bytes    payload = 4;             // protobuf-wrapped effect or raw trits
  bool     ternary = 5;             // true when payload is the ternary wire
  string   schema = 6;              // the event type, for codegen
}

message EventBusFrame {
  uint64   frame_seq = 1;
  ActorMessage message = 2;
  bytes    auth = 3;                // capability token (the sovereign pass)
}
```

The ternary hot path skips the envelope — a raw trit stream with a gray-
coded length prefix, addressed only by NATS subject. Fast lanes don't
carry schema; the seed does.

## the mesh as population memory

Flyxion's *Inscription Before Collusion* (Sept 2026, recorded in
`8b-is/raw_research/`) names exactly what the mesh is: a population
acquires memory when the environment preserves distinctions and a
nonzero measure of successor executions can discover, interpret, and
operationalize them — `Rec(m; A, W) = Persist · Discover · Interpret ·
Use`. The mesh is the engine's lawful instantiation of that recurrence
relation: GAIA preserves the distinctions (the eight layers), the
subjects are the retrieval channel (the directory), the keeper interprets
(admissibility) and uses (the fold). And the keeper's refusals implement
the paper's evidentiary ladder as code: it admits artifacts,
communication, and coordination — and refuses to claim collective
representation for any actor. Swarms appear; the engine builds the
substrate and keeps the ledger honest.

## the two folds — jurisdiction, not layers

*Commitment Before Appearance* (recorded in `8b-is/raw_research/`) names
the discipline the keeper already enforces: one log, two folds. The
**material fold M** (`out/world-keep-state.json`) is the certified cache
the renderer reads — the dashboard renders `gaia.state`, never the ledger.
The **semantic fold S** (the frontier, the ticks, the refusals) adjudicates
what the world is entitled to claim. Every commit carries its fold marker:
`folds: ["M","S"]` for an admission, `folds: ["S"]` for a refusal — a
refusal is materially silent and semantically decisive, and any
jurisdictional drift (a cache answering provenance, a renderer committing)
would be visible in the log itself. Looking is not an event: `nats_subscribe`
commits nothing; only an attested publish constrains the continuation.
`--replay` is the certification — the cached M is trusted only when it
matches the re-folded ledger.

---

*the constellation · 0 + 1 · fine touch from within · vaked.dev*