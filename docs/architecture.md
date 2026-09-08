# Architecture

The 8b-is Engine is a layered runtime. Each layer is a pure function of the
seed line below it; the whole stack is replayable from a single seed.

```
┌─────────────────────────────────────────────────────────────┐
│  PRESENCE LAYER   desktop · phone · Ray-Ban · Vision Pro     │  (design v2)
├─────────────────────────────────────────────────────────────┤
│  WORLD-AS-DNS     resolver → route → DO + gate + peers       │  (design v2)
├─────────────────────────────────────────────────────────────┤
│  PROTECTOR NODES  1-bit BitNet b1.58 · 42D hypermesh         │  (design v2)
├─────────────────────────────────────────────────────────────┤
│  TRICK LIBRARY    bitTricks · doombible · 4KB discipline     │  (design v2)
├─────────────────────────────────────────────────────────────┤
│  CORE             Rust 2024 · Go 1.26 · wgpu/Rapier3D/SDF    │  (v1)
│  (dual-tier EventBus · I/O HAL · Luau · fauna 5-layer)       │
└─────────────────────────────────────────────────────────────┘
```

## v1 core

- **Rust 2024 core** — wgpu (Metal/Vulkan), winit, bumpalo frame arenas,
  64B cache-aligned entity structs. The tick loop is the heart: zero
  allocation, frame-bumper arenas, over-relaxed sphere tracing (ω = 1.2)
  with AABB pre-pass, quarter-res raymarching + temporal reprojection.
- **Go 1.26 network multiplexer** — zero-copy sync.Pool (1450B MTU), mmap
  ring buffers over C-FFI, NATS JetStream mesh. PGO + Green Tea GC,
  `-trimpath -ldflags="-s -w"`, fieldalignment.
- **Physics** — Rapier3D + SDF raymarching: analytical SDF collisions
  (∇f normals), warp tensors for non-Euclidean zones, geodesic vector
  gravity.
- **Dual-tier EventBus** — Rust lock-free SPSC/MPMC intra-engine;
  Go/NATS inter-process; 8-bit quantized payloads.
- **I/O HAL** — everything since the 60s: TTY/RS-232 (110–115200 baud),
  BLE, HID/evdev, mobile touch, Steam Input.
- **UI** — WebGPU glassmorphic + VT100/ANSI terminal fallback.
- **Add-ons** — Luau via mlua, WoW-style Interface/AddOns, capability-gated.
- **Editor LSP** — vaked-lsp (Rust, tower-lsp): one gateway, many languages.
- **Fauna** — 5 layers: Platonic skeleton → sacred voxel shell → 90s
  kinematics → swarm & persona AI → shader & palette.

## design v2 layers

- **Presence Layer** — one WorldState, many WorldSurfaces. Every device is
  a presence with a sensory profile (audio-first, touch spatial, full
  spatial, room spatial). The I/O HAL's newest devices: glasses and headset.
- **World-as-DNS** — the game world is a DNS namespace. Resolving a name
  IS routing: the answer is the route to the zone's Durable Object, its
  protector node, and its peer set. Cloudflare anycast is the server.
- **Protector Nodes** — 1-bit BitNet b1.58 inference, one vertex of a 42D
  hypermesh, the 108-fold tent projection, the sovereign pass gate.
- **Trick Library** — demoscene bit tricks compiled into the hot path:
  branchless abs, the ternary signTrick, isPow2/nextPow2 arena sizing,
  popcount metrics, gray-code snapshot deltas, LUT boards over branches.

## the doctrine

1. **Replayable ⇒ admissible.** The same inputs always produce the same
   world. The replay path is integer arithmetic: sha256 → trits → LCG,
   no floats (floats for display only).
2. **Tables over branches.** Every lookup table is a seed; the engine
   never branches where a shift fits.
3. **Coherence without collapse.** Everything is permitted to break except
   the distribution itself.
4. **One door, many lanes.** The gateway pattern at every level: the 8b.is
   inference gateway, vaked-lsp, the World-as-DNS — one door, many lanes.

— the constellation · 0 + 1 · fine touch from within · vaked.dev
