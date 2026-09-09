# pipeline — Layer 1: ingestion and staging

Text in, typed-and-checksummed assets out. The engine's first layer:
GDD/lore text → `ZoneManifest` (RON) → staged assets with SHA256
attestation — ready for the client and server workspace to consume.

```
brief ──▶ gdd_parser ──▶ (asset_gen: FLUX art + vision QA) ──▶ stager ──▶ assets/staged/
```

## the modules

| Module | Role |
|---|---|
| `gdd_parser` | text → `ZoneManifest` (archetypes, items, board, the GAIA field) — deterministic by default, LLM-adapter optional |
| `asset_gen` | wrappers for the local lanes (FLUX.2-klein concept art, Qwen2.5-VL QA) |
| `retopo` | mesh intake validation: GLB magic/version, GLTF mesh counts |
| `stager` | temp-then-rename copies + SHA256 checksums + stage manifest |
| `lib` | the async orchestrator: parse → art → stage, sequential, non-blocking |

## run it

```bash
cargo run -p pipeline -- run "the bazaar of the 108 gates, gold and cyan"
cargo run -p pipeline -- run docs/gdd/painted-forest.md --art   # + concept art + the eye
cargo run -p pipeline -- verify                                 # re-checksum assets/staged
cargo run -p pipeline -- stage out/terrain.ron assets/staged
cargo run -p pipeline -- mesh assets/vendor/wolf.glb
cargo test -p pipeline
```

## configuration — external tool APIs

The pipeline is **deterministic by default** and never commits a secret.
External backends (LLMs, local image models) are wired through
environment variables:

| Variable | What it wires |
|---|---|
| `VAKED_PIPELINE_LLM_CMD` | a command (space-split) that reads GDD text on stdin and prints strict JSON on stdout — your local LLM wrapper, an API proxy, anything. Unset → the deterministic expander (GAIA + the seed). |
| `VAKED_MLX_SIDECAR_DIR` | the mlx-sidecar uv project (default `../vaked-lsp/mlx-sidecar`) — the FLUX.2-klein and Qwen2.5-VL lanes |
| `VAKED_PIPELINE_STAGE_DIR` | where staged assets land (default `assets/staged`) |

Why env-only: the engine's rule is one door, many lanes, and the lane
stays dark when its engine is not running — a missing sidecar or an unset
adapter degrades to the deterministic path, never to a hard failure and
never to a committed key.

## the doctrines in force

- **Determinism is the truth.** The brief IS the seed; `world-core` folds
  the same universe in Node, Python, Rust, and WASM. The LLM path is an
  assistant to the seed, not a second source of randomness.
- **Attestation, not promises.** Every staged file carries a SHA256 that
  `verify` re-checks; corruption is detected, not assumed away.
- **Proposals before commitments.** Concept art and QA verdicts are
  proposals; the manifest is the admission; the stage directory is the
  mid-pipeline boundary the keeper's world consumes.

— the constellation · 0 + 1 · fine touch from within · vaked.dev
