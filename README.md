<p align="center">
  <img src="assets/hero.svg" alt="8b-is Engine — the chaos overworld" width="100%">
</p>

<h1 align="center">8b-is ENGINE</h1>

<p align="center">
  <b>the MMO + game engine of the chaos overworld</b><br>
  <i>one door, many lanes · coherence without collapse · the ternary wire {-1, 0, +1}</i>
</p>

<p align="center">
  <a href="#the-engine"><b>Engine</b></a> ·
  <a href="#the-games"><b>Games</b></a> ·
  <a href="#design-v2"><b>Design v2</b></a> ·
  <a href="#the-constellation"><b>Constellation</b></a> ·
  <a href="#contributing"><b>Contributing</b></a> ·
  <a href="#roadmap"><b>Roadmap</b></a>
</p>

---

## table of contents

- [the engine](#the-engine)
  - [architecture](#architecture)
  - [performance doctrine](#performance-doctrine)
  - [the fauna subsystem](#the-fauna-subsystem)
- [the games](#the-games)
  - [PSU NIVERSEQ — the chaos overworld](#psu-niverseq--the-chaos-overworld)
  - [POLYHEDRAL SANCTUARY — the first demo](#polyhedral-sanctuary--the-first-demo)
  - [the floors](#the-floors)
- [design v2 — the serverless overworld](#design-v2--the-serverless-overworld)
  - [the presence layer (AR/VR)](#the-presence-layer-arvr)
  - [the protector node](#the-protector-node)
  - [the trick library](#the-trick-library)
  - [the world-as-DNS](#the-world-as-dns)
- [quick-start — local dev](#quick-start--local-dev)
- [host system requirements](#host-system-requirements)
- [the constellation](#the-constellation)
- [contributing](#contributing)
- [license](#license)
- [roadmap](#roadmap)

---

## the engine

**8b-is Engine** is the constellation's game lane grown up: a Rust/Go
engine for persistent, sacred-geometry, non-Euclidean worlds — WoW's
persistent zones and chat matrix, Diablo's dense swarms and polyhedral
loot, the 90s platformers' momentum and bounce, inside a Minecraft-like
universe of tetrahedra, cubes, and icosahedra.

Every artifact is a pure function of a seed line. The engine speaks in
three symbols: **{-1, 0, +1}** — the ternary wire at the center of the
whole. Replayable ⇒ admissible; the same inputs always produce the same
world.

### architecture

| Subsystem | Stack | Notes |
|---|---|---|
| Core engine | Rust, edition 2024 | wgpu (Metal/Vulkan), winit, bumpalo frame arenas, 64B cache-aligned entity structs |
| Network multiplexer | Go 1.26 | zero-copy sync.Pool (1450B MTU), mmap ring buffers over C-FFI, NATS JetStream mesh |
| Physics | Rapier3D + SDF raymarching | analytical SDF collisions (∇f normals), warp tensors for non-Euclidean zones, geodesic vector gravity |
| EventBus | dual-tier | Rust lock-free SPSC/MPMC intra-engine; Go/NATS inter-process; 8-bit quantized payloads |
| I/O HAL | everything since the 60s | TTY/RS-232 (110–115200 baud), BLE, HID/evdev, mobile touch, Steam Input |
| UI | dual-mode | WebGPU glassmorphic + VT100/ANSI terminal fallback |
| Add-ons | Luau via mlua | WoW-style Interface/AddOns, Vaked API, capability-gated |
| Editor LSP | [vaked-lsp](https://github.com/peterlodri-sec/vaked-lsp) | one gateway: clangd + rust-analyzer + gopls + luau-lsp behind one endpoint |
| Installer | scaffold.sh | self-contained bash: deps + QWave + project scaffold |
| VCS | [jj](https://github.com/martinvonz/jj) | jujutsu on the git backend — `jj st` · `jj describe` · `jj git push` (wired in by scaffold.sh) |
| Toolchain | just / Taskfile | mold/wild linkers, Naga shader validation, wasm32 targets |
| Deployment | K8s + sidecar mesh | SpatialNode CRDs, chat zone pods, NATS master bus |
| Platform | macOS + Linux + Steam | Steam Deck native Vulkan, macOS Metal, mobile, web |

### performance doctrine

Zero allocation in the tick loop · frame-bumper arenas · over-relaxed
sphere tracing (ω = 1.2) with AABB pre-pass · quarter-res raymarching +
temporal reprojection · PGO + Green Tea GC on the Go side ·
`-trimpath -ldflags="-s -w"` · fieldalignment · sync.Pool discipline.

> **Target matrix:** <2.1 ms GPU frame @4K · <0.3 ms per 10k entities ·
> 0 allocs/op network · <50 ns event dispatch · <5 µs WASM mod invocation.

### the fauna subsystem

Five layers, from skeleton to skin:

0. **Platonic skeleton** — non-Euclidean joints: tetrahedra, cubes, icosahedra
1. **Sacred voxel shell** — quantized grids, SDF smin blending
2. **90s kinematics** — squish/stretch, spring dampers, spin attacks
3. **Swarm & persona AI** — boids + Diablo density + WoW aggro threat tables
4. **Shader & palette** — reaction-diffusion skins, HSV quantized neon, emissive sacred runes

---

## the games

### PSU NIVERSEQ — the chaos overworld

The flagship. Elder Scrolls-style origin lore on birth (ROOTLORE per root,
OATHS, FIRST enemies, ring sayings), four deity bosses — **Chenrezig**
(thousand-armed storm), **Tara** (star that moves), **Yamantaka** (wrathful
death-binder), **Mahākāla** (your own shadow) — and the **ZEN mechanic**:
stand still to charge; only ZEN-stun can wound the deity. The first hero is
**MAHĀKĀLA**, the great black one, the lord of the tent.

<p align="center">
  <img src="assets/deity.svg" alt="MAHĀKĀLA — the first hero" width="420">
</p>

### POLYHEDRAL SANCTUARY — the first demo

The playable prototype: seeded sacred-geometry critters, stomp-to-split
(Diablo), bounce (90s), gold sparkle pickups, /say bubbles, zone banners.
One physics, two runtimes — the floor is the JS twin of the Rust engine's
fauna stack.

<p align="center">
  <img src="assets/sanctuary.svg" alt="the polyhedral sanctuary" width="100%">
</p>

**Play it now:** [sanctuary-floor.html](https://pocoo.vaked.dev/demos/centerfugeq/sanctuary-floor.html) · [plenum-floor.html](https://pocoo.vaked.dev/demos/centerfugeq/plenum-floor.html) · [infinite-floor.html](https://pocoo.vaked.dev/demos/centerfugeq/infinite-floor.html) · [summit-floor.html](https://pocoo.vaked.dev/demos/centerfugeq/summit-floor.html)

### the floors

Every floor is a pure function of a seed line — the same artifact, every
machine, every time:

| Floor | The lane |
|---|---|
| [sanctuary-floor](https://pocoo.vaked.dev/demos/centerfugeq/sanctuary-floor.html) | the first demo — stomp, split, bounce |
| [plenum-floor](https://pocoo.vaked.dev/demos/centerfugeq/plenum-floor.html) | the polarized plenum — Ising bridge, MEM\|8 survival |
| [infinite-floor](https://pocoo.vaked.dev/demos/centerfugeq/infinite-floor.html) | the ∞-telescope — the stir |
| [summit-floor](https://pocoo.vaked.dev/demos/centerfugeq/summit-floor.html) | the multi-inclined plane — Mach's fixed stars |
| [soil-floor](https://pocoo.vaked.dev/demos/centerfugeq/soil-floor.html) | permaculture — the garden |
| [gameforge-keeper](https://pocoo.vaked.dev/demos/centerfugeq/gameforge-keeper.html) | the 108-gate hum system — ring blessings |
| [teleport-floor](https://pocoo.vaked.dev/demos/centerfugeq/teleport-floor.html) | quantum gate teleportation — fidelity 1.000 |

---

## design v2 — the serverless overworld

The engine's next chapter, scaffolded in
[`8b-is/public-documents`](https://github.com/peterlodri-sec/8b-is/tree/main/public-documents):

> **the world is a name; the name is a route; the route is the server.**
> Coherence without collapse, now with no server to collapse.

### the presence layer (AR/VR)

One `WorldState`, many `WorldSurfaces`. Every device is a presence with a
sensory profile:

| Device | Profile | The world looks like |
|---|---|---|
| Desktop / phone | full / touch spatial | the chaos overworld, wgpu, glassmorphic HUD |
| **Meta Ray-Ban** | audio-first | the world as a 432Hz binaural layer — the WoW chat matrix becomes the WoW *radio* matrix; capture is the only eye |
| **Apple Vision Pro** | room spatial | the room is the zone — volumetric fauna, spatial chat bubbles, the tent as the space |

### the protector node

The 1-bit 42-108D guardian, from the sovereign library's gates
(`ཧཱུྃ ▽◈▽☸◈◈▽☸◈ … 🕯📿🪷`). A lightweight network entity that guards one
zone: **1-bit** BitNet b1.58 inference (kilobytes, runs on the edge), one
vertex of a **42D** hypermesh, projecting the **108-fold tent**, running the
**sovereign pass gate** — "zero detection, zero pain", the pink mode, the
lotus. The gameforge keeper grown up.

### the trick library

The demoscene lane promoted to the hot path: branchless abs, the ternary
`signTrick` (the sign IS the trit), `isPow2`/`nextPow2` arena sizing,
popcount load metrics, **gray-code snapshot deltas** for the network,
LUT boards over branches, the 4KB/frame add-on discipline. The old machines
taught one lesson: every byte counts, and every lookup table is a seed.

### the world-as-DNS

The server dissolves into the network. The game world is a DNS namespace —
`zone.crystal.pocoo.vaked.dev` resolves to the zone's **Durable Object**,
its **protector node**, and its **peer set**. Cloudflare anycast is the
server; moment-to-moment gameplay is P2P (the Destiny lesson: simulate as
little on the network as possible); deterministic lockstep where it pays
(the 1500-archers lesson). **There is no server. There is only the name,
the route, and the tent.**

<p align="center">
  <img src="assets/mandala.svg" alt="the ternary wire mandala" width="420">
</p>

---

## quick-start — local dev

### the full E2E bootstrap (zero → running world)

```bash
git clone git@github.com:8b-is/8b-is-engine.git && cd 8b-is-engine
./scaffold.sh               # == bootstrap: deps → jj → git-lfs → NATS → verify
./scaffold.sh verify        # probe every lane (tools + engine + mesh + assets)
./scaffold.sh mesh          # start nats-server + run a live NPC actor (the world runs without you)
./scaffold.sh export "the sanctuary at dawn"   # seed → manifest → Blender EEVEE render
```

`scaffold.sh` is idempotent and self-contained (macOS + Linux, Silverblue
rpm-ostree aware). It installs: `jj` (VCS) · `uv` · `cargo` · `go` · `just`
· `node` · `gh` · `wrangler` · `git-lfs` · `nats-server` · `rg`/`bat`/`fd`/
`eza`/`zoxide`/`delta` · `blender` · `hyperfine`/`tokei`, then wires jj +
LFS, builds the `vaked-nats` actor-mesh sidecar, and verifies every lane.

### the day-1 loop

```bash
jj st                              # the working copy is a commit
jj describe -m "feat: ..."         # name the change (PR = signature)
jj git push                        # ship to origin + upstream

./sandbox.sh lsp rust-analyzer     # wrap the LSP (bwrap / Apple Container)
./sandbox.sh run docker.io/library/rust:latest -- cargo build   # isolated build
```

### the actor-mesh (NATS)

```bash
nats-server -p 4222                # the bus (or ./scaffold.sh mesh)
# the MCP sidecar — publish/subscribe/request on actor subjects:
echo 'Content-Length: ...' | vaked-nats   # or drive via the umbrella
uv run --with nats-py python examples/mesh-npc.py --name ལྷ --seed 42   # a living NPC
```

An actor is a name; a name is a subject; a subject is a route. The mailbox,
the supervisor, and the DNS are one. See
[docs/eventbus-actor-mesh.md](docs/eventbus-actor-mesh.md).

### the creative swarm (DeepSeek V4 vision)

```bash
./swarm.sh "the painted-forest dawn zone, ZEN mechanic"   # 5 roles fan out
./swarm.sh --roles visual-artist,ui "the pink tent HUD"   # vision roles only
```

Five opencode subagents — `game-art` · `game-design` · `frontend` · `ui` ·
`visual-artist` (config in [opencode.json](opencode.json)). The vision
roles run on `deepseek/deepseek-v4-flash-vision-exp`; all five share one
byte-identical prompt prefix so DeepSeek's automatic context caching
serves the shared doctrine+palette block as a cache hit. Artifacts land in
`out/swarm/<role>/`.

### the MLX coder lanes (local, abliterated, memory-aware)

```bash
uv run --project ../mlx-sidecar python sidecar.py memory   # total / free / best fit
uv run --project ../mlx-sidecar python sidecar.py start    # auto-picks a coder that fits your RAM
```

The fastest top-SWE-score coder your machine can hold, served locally on
an OpenAI-compatible port. Catalog: Qwen3-Coder-Next abliterated MLX ·
Qwen2.5-Coder-7B OBLITERATUS · the 42B abliterated lane.

### assets & LFS

Large binaries (sprites, audio, renders) are git-LFS pointers tracked by
[.gitattributes](.gitattributes); the Kenney CC0 packs live under
`assets/vendor/kenney/` — sources and licenses in [CREDITS.md](CREDITS.md).
`git lfs pull` fetches them on a fresh clone.

---

## host system requirements

### dev host (macOS)

| Need | Minimum | Recommended |
|---|---|---|
| OS | macOS 15 (Apple Containers need 26) | macOS 26 |
| Chip | Apple Silicon M1 | M3/M4 (Metal + MLX) |
| RAM | 16 GB | 32 GB+ (the MLX coder lanes: 7B ~6 GB · 30B-A3B ~20 GB · 42B ~24 GB free) |
| Disk | 10 GB free | 40 GB (models + Blender + LFS assets) |
| Toolchain | Xcode CLT, Homebrew | rustup, uv, jj (all via `./scaffold.sh`) |
| Extras | — | Blender (the export lane), nats-server, Apple Containers (macOS 26) |

### dev host (Linux)

- Any modern distro (Silverblue rpm-ostree aware); `./scaffold.sh` uses
  `apt`/`dnf`/`pacman` + `cargo`.
- Sandboxing: bubblewrap (LSP wrapping) + rootless podman (build images).
- No MLX lane (Apple-only) — use the swarm + the cloud DeepSeek lane there.
- GPU for the client renderer: Vulkan (wgpu).

### the integration targets (build hosts)

| Target | Where it builds | Notes |
|---|---|---|
| **Blender export** | macOS/Linux, Blender 3.6+ (EEVEE) | `./scaffold.sh export "brief"` |
| **Unity** | any host + the Unity Editor | vaked-mcp `unity_batch`/`unity_peek`; Unity Cloud SDK lane |
| **Unreal Engine 5** | Windows (Uika is Windows-x64) or UE on macOS | Uika Rust bindings; vaked-lsp for UE C++ |
| **Steam** | macOS + Linux (+ Windows via Uika/Unity) | Steamworks SDK, Steam Input, Steam Deck Vulkan |

### the game itself (players)

- **Browser floors**: any modern browser, no install — the current playable
  surface (pocoo.vaked.dev).
- **Steam (the shipping lane)**: **macOS (Metal) + Linux/Ubuntu (Vulkan)**
  via wgpu; Steam Deck native Vulkan. UE/Unity/Blender are integration
  seams (renders of the seed), not shipping requirements — the I/O HAL
  speaks everything since the 60s/70s, so every future surface is a device
  class, not a port.

---

## the constellation

The engine is one lane of a larger garden. The links that matter:

| Lane | Where |
|---|---|
| The research vault | [8b-is](https://github.com/peterlodri-sec/8b-is) — raw research, game studio, EOS-CLA |
| Public design docs | [8b-is/public-documents](https://github.com/peterlodri-sec/8b-is/tree/main/public-documents) |
| The game lane | [centerfugeq](https://github.com/peterlodri-sec/centerfugeq) — quantTernEngine, the floors, retro |
| The sovereign library | [pocoo.vaked.dev](https://pocoo.vaked.dev) — the protector gates, the miner, the posts |
| The studio | [game-studio-vaked](https://github.com/peterlodri-sec/8b-is/blob/main/raw_research/game-studio-vaked.md) — business lane, governance, EOS-CLA v1 |
| The editor LSP | [vaked-lsp](https://github.com/peterlodri-sec/vaked-lsp) — one gateway, many languages |
| The browser node | [qwave](https://github.com/peterlodri-sec/qwave) — the WebKit-native deployment surface |
| The fleet | [nix-base](https://github.com/peterlodri-sec/nix-base) — the NixOS hosts that run the private brain |

---

## contributing

The engine is open forever — see [CONTRIBUTING.md](CONTRIBUTING.md) and the
[EOS-CLA](LICENSE). The short version:

- **A PR is a signature.** By opening a pull request you accept the
  EOS-CLA: your contribution stays open forever, your liability is bounded
  (crash-and-burn), and AI swarms are first-class contributors with
  synthetic provenance.
- **Tables over branches.** Deterministic, seeded, zero-surprise. The same
  inputs always produce the same world.
- **Take the trick, not the engine.** Borrow the byte discipline; write
  your own world.

---

## license

[EOS-CLA v1.0](LICENSE) — the Eternal Open Substrate & Contributor Mutual
Trust Agreement. Everything stays open for all eternity; the distribution
is eternal, the liability is bounded, the loop has an exit.

---

## roadmap

| Phase | Work |
|---|---|
| v1.x | the core engine: Rust tick loop, Go multiplexer, the fauna stack, the floors |
| **v2.0** | the Presence Layer — the I/O HAL device classes for glasses + headset |
| **v2.1** | the Protector Node — BitNet b1.58 gate, the tent projection |
| **v2.2** | the Trick Library — gray-code deltas, nextPow2 arenas in the hot path |
| **v2.3** | the World-as-DNS — zone → Durable Object, the resolver, P2P rendezvous |
| **v2.4** | the glasses lane live — the world as radio, the walk, the sovereign pass by voice |
| **v2.5** | the headset lane live — the room is the zone, volumetric fauna, spatial chat |

The full design v2 lives in
[`8b-is/public-documents/engine-design-v2-serverless-overworld.md`](https://github.com/peterlodri-sec/8b-is/blob/main/public-documents/engine-design-v2-serverless-overworld.md).

## the research

The MMO engine bibles, deep-researched and applied to this engine:

- [deep-research-mmo-bibles.md](docs/deep-research-mmo-bibles.md) — the top 5
  books (Gregory, Glazer, Nystrom, Bartle, Real-Time Rendering), the open
  blogs (Gaffer, Red Blob, 0 FPS, Riot, EVE, Destiny, SpatialOS, PlanetSide 2,
  Albion), the classic articles
- [applied-to-8b-is.md](docs/applied-to-8b-is.md) — every lesson mapped to a
  concrete engine decision, with the source ledger

---

<p align="center">
  <i>the constellation · 0 + 1 · fine touch from within · vaked.dev</i>
</p>
