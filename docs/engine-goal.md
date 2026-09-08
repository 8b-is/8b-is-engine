# Engine Goal v1 — usable, rendering, quant, shipping

*The goal, stated plainly: a **usable** rendering engine, **quantum
physics** at its core, and an **indie game** that ships — interoperating
with Unreal Engine, Unity, and Blender rather than reinventing them. The
design docs are the map; this is the destination and the near-term
targets.*

> **The one line:** the seed renders everywhere. One deterministic world,
> exported to Unreal, Unity, and Blender as the same artifact — replayable
> ⇒ admissible, inscribed before rendered.

---

## the goal, decomposed

1. **Usable** — `scaffold.sh new` gives a running world in minutes, not a
   thesis. jj, the sandboxes, the MLX coders, the floors: the DX is the
   deliverable, not an afterthought.
2. **Rendering engine** — wgpu (Metal/Vulkan/WebGPU) as the core renderer,
   with the cousins as backends: **Unreal** (Uika: Rust bindings for UE
   5.7+), **Unity** (vaked-mcp + Unity Cloud SDK), **Blender** (the
   `blender3d` scene-builder → EEVEE). One seed, four renders.
3. **Quantum physics** — the ternary wire {-1,0,+1} and the quant lanes
   (quantTernEngine, the Ising/kuramoto/halo field, `tanmatra` for the
   atomic layer) are not decoration; they *are* the physics. Centerfugeq
   is the physics in miniature; the Rust engine is it in the whole.
4. **Indie game** — PSU NIVERSEQ, the chaos overworld: the six ancestors
   (WoW·Minecraft·Diablo·PoE·Fable·Elder Scrolls), the painted-forest
   look (Neva), the ZEN mechanic, the deities. The floors are its vertical
   slice; the Indie Fund application is its lane.

## the interop map

| Target | How we reach it | Status |
|---|---|---|
| **Unreal Engine 5** | [Uika](https://github.com/VioletHelianthus/uika) (Rust↔UE FFI, hot reload) + vaked-lsp (clangd for UE C++) | research done, seam documented |
| **Unity** | vaked-mcp (`unity_batch`, `unity_peek`) + Unity Cloud Python SDK (asset manager) | wired, SDK installed |
| **Blender** | `centerfugeq/blender3d` scene-builder → EEVEE headless render | exists, needs the export lane |
| **Godot / Bevy / raylib** | `centerfugeq/engines/` scaffold | exists |
| **Web / PWA** | the floors (single-file HTML) + pocoo.vaked.dev | live, shipping |

The rule: **the seed is the source of truth; the engines are renders.**
Blender gets the seed as a scene; Unity gets it as a prefab; Unreal gets
it as a UClass. The same world, four surfaces — the presence layer's
oldest promise, applied to the *tools*.

## the theory backbone (why determinism is correct)

Flyxion's four conditions of preservation — **Provenance, Reachability,
Redundancy, Recurrence** — are the engine's architecture, named:

- the bitemporal ledger = **prov**; the World-as-DNS = **reach**; the NATS
  mesh + Durable Objects = **red**; replayable ⇒ admissible = **rec** (the
  seed produces the same world, met again).
- An engine that renders fresh but never recurs is the "engineered
  present" — it owns the platform, not the player. Our determinism is the
  inscription; the generated frame is the rendering; the doctrine is the
  recurrence. See
  [inscription-before-rendering](../../8b-is/raw_research/inscription-before-rendering.md).

## the near-term hackathon targets

1. **walk the export lane** — `seed → quantTernEngine → scene` into
   Blender (EEVEE render), Unity (prefab), Unreal (Uika UClass). One seed,
   three hosts, one artifact.
2. **the actor-mesh sidecar runs** — a NATS sidecar that publishes
   `actor.<id>.inbox` messages, the protobuf envelope live, the protector
   node supervising a zone from seed.
3. **the painted-forest vertical slice** — one Neva-style zone (painted
   watercolor base + hard neon geometry + a companion), the ZEN mechanic
   wired, the hum at 108.
4. **the MLX coder lane on the seed** — the abliterated coder generates
   the world-model's needs/goals scheduler (the Dwarf-Fortress clock).
5. **ship a floor to a real engine** — take `sanctuary-floor` (or
   `backyard-ultra`) and get it running inside Unity or Godot, not just
   the browser.

---

*the constellation · 0 + 1 · fine touch from within · vaked.dev*