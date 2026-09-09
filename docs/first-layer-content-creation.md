# Layer 1 — input and content creation: ideation → the mid-pipeline

The first layer of the engine is the content pipeline: from a brief (a
sentence) to the mid-pipeline artifact (an attested manifest + concept art
+ a first render). Everything downstream — the zones, the NPCs, the
instances — consumes what this layer admits. Two disciplines from the
recorded theory govern it: every stage is an **attested inscription** (the
keeper's rule — nothing enters the ledger without being answerable), and
every label is a **named region, not a mechanism** (the witness/fiber rule
— "anabolic" style words describe the wire, never claim the cause).

## the stages

```
brief ──▶ seed (GAIA) ──▶ swarm fanout ──▶ concept art ──▶ vision QA
                                  │
                                  ▼
                manifest (gen.ts) ──▶ modality builders ──▶ render (EEVEE / Unity)
```

| Stage | Tool | Artifact | Attestation |
|---|---|---|---|
| 1. the brief | the human | a sentence | the input — already an inscription |
| 2. the seed | `gaia.ts` / `gaia.py` / `world-core` | the 8-layer field at t=0 | deterministic — same brief, same universe, four languages |
| 3. the fanout | `swarm.sh` (5 opencode agents) | `out/swarm/<role>/` — art, design, ui | each agent's file is a proposal; none is admitted yet |
| 4. concept art | the FLUX.2-klein diffuser lane | `assets/concepts/flux-*.png` | seeded by the brief — a rendering of the seed, not the world |
| 5. the eye | the Qwen2.5-VL vision lane | a QA verdict on the render | the first admissibility check — a black frame is refused here, not later |
| 6. the manifest | `quantTernEngine/gen.ts scene` | `out/game-0x<seed>.json` | the admitted brief, compiled to the wire |
| 7. the modality | `scene_builder.ts` (Blender / Unity) | `.scene.py` / `VakedSceneImporter.cs` | the manifest's projection onto an export seam |
| 8. the render | Blender EEVEE / Unity | `out/*.png` | disposable appearance — `Render(M_t, ω_t)`, never history |

The mid-pipeline boundary sits at the **manifest**: before it, everything
is content creation (ideation, proposals, renders of the seed); after it,
everything is the world (the mesh, the keeper, the folds). The manifest is
the attestation that bridges the two: a brief becomes a world only when it
is admitted.

## the one-command pipeline

`scaffold.sh content "<brief>"` runs the local, deterministic half of the
layer (the lanes that need no paid model):

```bash
./scaffold.sh content "the pink tent at dawn, 108 gates"     # seed → manifest → EEVEE render
./scaffold.sh content --art "the painted forest, ZEN wired"  # + FLUX concept art + vision QA
./swarm.sh "the sanctuary vertical slice"                    # the paid-model fanout (DeepSeek)
```

## the typed implementation

`crates/pipeline` is this layer compiled: `gdd_parser` turns GDD text into
the typed `ZoneManifest` (archetypes, items, the ternary board, the GAIA
field) — deterministic by default, an env-wired LLM adapter optional;
`stager` moves every artifact into `assets/staged/` with SHA256
attestation and a stage manifest (`pipeline verify` re-checksums);
`retopo` validates mesh intake; the `pipeline run` CLI orchestrates the
stages sequentially on Tokio. The scaffold's `content` command and the
crate agree on the same brief → same seed — verified.

```bash
cargo run -p pipeline -- run "the bazaar of the 108 gates"   # parse → stage
cargo run -p pipeline -- run docs/gdd/zone.md --art          # + concept art + the eye
cargo run -p pipeline -- verify                              # corruption check
```

## the doctrine this layer enforces

1. **One brief, one seed, one universe.** The brief IS the seed; GAIA is
   the door. Nothing downstream may invent a second source of randomness
   the brief didn't imply.
2. **Proposals are not commitments.** The swarm's output, the concept art,
   and even the first render are proposals until the manifest admits them.
   The vision QA is the gate that catches a proposal whose rendering
   failed (the black-frame incident: the eye refused it, the builder was
   fixed, the render was re-admitted).
3. **Labels describe regions.** "sanctuary at dawn" is a region of the
   brief's image, not a mechanism claim about the world. The world's
   mechanisms — weather, entropy, gravity — are GAIA's layers, and they
   are read, not named.
4. **The mid-pipeline is where jurisdiction changes.** Before the
   manifest: disposable renders, free iteration. After the manifest: the
   keeper's admissibility, the ledger, the folds. A render never commits;
   a manifest never renders.

## the hub-and-instance consequence

Layer 1 feeds both zone kinds from the same manifest: a **hub** is a brief
about a city ("the bazaar of the 108 gates") folded into a persistent
zone at 10–15 TPS of social flow; an **instance** is a brief about a
dungeon folded into an ephemeral zone at 30–60 TPS of combat. The content
layer does not care which — the same brief, seed, art, and manifest serve
both; the difference is declared at spawn time, not at creation time.

— the constellation · 0 + 1 · fine touch from within · vaked.dev
