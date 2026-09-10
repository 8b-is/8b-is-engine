# The client stack — latest CSS + HTML + WebAssembly, and what completes them

The engine's client frontend is a **browser-shaped client, not a browser
compromise**. The value of the 8b-is engine is the living world (GAIA,
the actor mesh, the replayable history) — pixels are a plug-in. This page
pins the plug.

## The stack

| Layer | Choice | Why it is the latest / SOTA move |
|---|---|---|
| HTML | Custom elements + `popover` + `dialog` | Declarative UI without a framework; popover for tooltips/lore, dialog for the ternary door |
| CSS | Native modern CSS: `oklch()` color, `@layer`, `:has()`, container queries, scroll-driven animations, view transitions | Framework-free design system; perceptual color, cascade control, parent-aware styling — no preprocessor |
| Core | **WebAssembly** (Rust → `wasm32-unknown-unknown`, SIMD) | quantTernEngine / GAIA / the ternary PRNG compiled once, shared by client AND server — same seed math everywhere |
| World loop | Web Worker + `SharedArrayBuffer` + `Atomics` | The tick runs off the main thread; render never blocks on the sim; lockstep-ready (Atomics.wait/notify) |
| Rendering | **WebGPU** (wgpu) with a 2D-canvas fallback | Modern GPU API; wgpu is the same crate the Rust core uses natively — one renderer, three targets |
| Networking | **WebTransport (QUIC)** with `nats.ws` WebSocket fallback | Lowest-latency transport for the mesh; NATS subjects = zones; one subject = one actor's inbox. The relay's QUIC door (`mesh-relay --quic-port N`, wtransport) is live and tested — the same compact wire, datagrams both ways |
| Modules | Native ES modules + import maps, zero build step | Node 24 strips types; the floors ship as plain `.html` + `.ts` today — this stays the dev loop |
| Steam shell | **Tauri** (WebView + Rust) | The exact same HTML/WASM client ships as a native macOS + Linux binary: Steam depots, overlay, achievements, no rewrite |

## The build (how the game gets built today → tomorrow)

```
brief ──▶ quantTernEngine/gen.ts ──▶ manifest (deterministic)
               │                         │
               ▼                         ▼
        GAIA(seed, t)            modalities:
        eight layers             scene_builder.ts → Blender EEVEE / Unity importer
        (weather, entropy,       floors (.html) — the playable prototypes
         gravity, time …)        vaked-nats mesh — the live world
                                    │
                                    ▼
release.yml (tags) → Rust binaries · pocoo Pages (demos) · ─▶ Tauri (Steam)
```

The missing piece today is the **shared WASM core**: right now
`quantTernEngine` is TypeScript and the Rust lanes live in `vaked-lsp`.
The v1.x move is one Rust crate (tern + GAIA + the reducer) compiled to
both `aarch64-apple-darwin`/`x86_64-linux` (server, Steam client) and
`wasm32` (browser, WebView). One source of truth, three surfaces.

**Shipped:** `crates/world-core` (tern + GAIA + the fold, native + wasm32,
four-language determinism pinned) and `client/mesh.js` — the first
surfaces of this stack are real artifacts now.

## To be Steam-distributed SOTA

1. **One core**: Rust crate `world-core` (ternary PRNG, GAIA, fold/replay)
   → WASM for the client, native for the server. No logic drift.
2. **One shell**: Tauri wraps the existing HTML/CSS/WASM client — Steam
   depots for macOS + Linux, achievements + cloud saves via
   `steamworks-rs`, Workshop as the player-content (Q-layer) surface.
3. **One mesh**: client ↔ WebTransport/WebSocket → NATS. Zones are actor
   subjects; GAIA publishes `gaia.state`; the keeper folds M/Q/H.
   Serverless world = Cloudflare Durable Objects per zone (v2.x), the
   mesh protocol unchanged.
4. **One loop**: `requestAnimationFrame` render + fixed-tick worker on
   `SharedArrayBuffer` — determinism where it pays, smoothness everywhere.
5. **One history**: `fold(seed, H) = M` verified on every client join —
   a joining player replays the ledger, never receives a snapshot blob.

## Explicitly NOT chosen

- **Unreal / Unity as the client frontend** — full engines for the render
  layer would dominate the project and kill the iteration speed the
  floors run on. UE/Unity stay as **export seams** (the scene modality
  renders through them), not as the shipping client. If a cinematic
  single-player mode ever ships, UE5 is the plug for that mode alone.
- **A bundler** — import maps keep the dev loop instant; a bundler
  appears only inside the Tauri build if tree-shaking starts to matter.
- **A CSS framework** — `@layer` + `oklch` + container queries are the
  framework now.

— the constellation · 0 + 1 · fine touch from within · vaked.dev
