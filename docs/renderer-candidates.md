# Renderer candidates from the local source inventory

Reviewed 2026-09-23. **Selection remains open.** Source inspection is not proof
of a running deployment. The prior suggestion that this engine must own the
renderer was too definite; SpherePOP is a concrete candidate.

A bounded filename scan across `/Volumes/source` was followed by targeted source
inspection. Vendor/build trees and many forks were excluded from the broad scan;
specific SpherePOP forks were inspected separately. Some protected metadata
directories and malformed ignore patterns were skipped. This is a useful
shortlist, not a claim to have audited every renderer on the volume. No renderer
was installed, updated, fetched or started for this review.

## Source-backed shortlist

Paths below are relative to `/Volumes/source`, so duplicate checkouts are clear.

| Candidate | Local source / entry point | What exists | What is not established |
| --- | --- | --- | --- |
| SpherePOP world | `8s/standardgalactic/spherepop/game-engine.html` | Canvas + DOM/perspective rendering, labelled spheres, animated world/player loop | Current Aye connection, private-container identity/privacy enforcement |
| SpherePOP field view | `8s/standardgalactic/spherepop/field-dynamics-simulator.html` | Phi/vector/entropy canvases and animation | Live MEM8 salience or model-activation telemetry |
| SpherePOP history comparison | `8s/standardgalactic/spherepop/history-comparator.html` | Side-by-side histories and SVG event stepping | Phoenix verification/authority integration |
| CenterfugeQ Neon City 42 | `8b-is/centerfugeq/demos/neon-city-42.html` | Canvas city, avatars, movement and above-head memory bubbles | Bubble text is a fixed array; the locally drawn synced label is not a verified status |
| CenterfugeQ mSphere | `8b-is/centerfugeq/quantGame/spherepop.ts`, imported by `gameforge.ts` | Executable simulation used by generated games; Python validation twin under `python/spherepop_sandbox/` | This model alone is not a graphical renderer or a continuity authority |
| Aye salience card | `8b-is/aye/web/salience-card/index.html` and `app.js` | Image upload, Canvas salience display, PNG export | It measures image-processing behavior, not a language model's hidden attention |
| Aye Q8 visual bridge | `8b-is/aye/crates/q8/src/visual.rs` | Eight-byte visual state from peak amplitude, jitter, salience and topology | No separate GPU renderer consumer was found in the reviewed Aye source |
| Aye native status surface | `8b-is/aye/crates/ayeos-kernel/src/framebuffer.rs` | Framebuffer text, glyphs, dashboard rows and scrolling | General avatar/world scene rendering |
| MEM8 Explorer | `8b-is/mem8-explorer/src/lib/components/Visualizer.svelte` and `WaveAnalyzer.svelte` | Svelte/Canvas wave and timeline displays | Reviewed values are synthetic, with no live MEM8 link found |
| 8b-is-engine | `8b-is/8b-is-engine/crates/world-core/src/render.rs`, example `arena.rs` | Rust entity-to-SVG renderer; browser mesh dashboard | Mandatory renderer choice or authenticated private-world readiness |
| CenterfugeQ Blender export | `8b-is/centerfugeq/blender3d/scene_builder.ts` | Generates a Blender scene/render script | Interactive world UI |
| Prakash | `8b-is/prakash/src/ray_trace.cyr` | Cyrius optics/ray-tracing/PBR primitives | An avatar application; reuse needs its GPL-3.0-only terms considered |

Related but different: Aye's `q8/src/renderer.rs` synthesizes audio; Q8-Caster
has media render functions but its reviewed window code disables egui rendering.
`phoenix-player` is empty locally, `cinematic-reconstruction` is research material,
and `space-bender` contains programming exercises. Aye's `enginerenderer` is an
empty gitlink in this checkout, without a usable `.gitmodules` mapping; it was
not fetched or treated as an existing renderer implementation.

## SpherePOP versions and semantics

- Upstream checkout: `8s/standardgalactic/spherepop`, remote
  `standardgalactic/spherepop`, inspected HEAD `ed5e2de` dated 2026-09-10.
- Fork: `8b-is/forks/spherepop`, origin `8b-is/spherepop`, inspected HEAD
  `2e95a54` dated 2026-05-21.
- `game-engine.html`, `spherepop.html` and `field-dynamics-simulator.html` have
  identical file hashes in those two checkouts. Avoid evaluating duplicates as
  independent implementations. `history-comparator.html` is in the newer checkout.
- `/Volumes/source/sg` contains `ideal` and `repo-reports`; it is not the main
  SpherePOP checkout or a symlink to it.

The C interpreter under `compiler/` is a computational/event model. Its topology
listener in `src/runtime/evaluator.c` is a no-op attachment point, not proof of an
attached renderer. Its README records Bind/Collapse differences between the
interpreter and canonical fixtures. Keep semantics compatibility explicit.

`spherepop.html` uses `Function(expression)` and inserts evaluation results into
`innerHTML`. That demonstration cannot receive private-world/user expressions
as a trusted evaluator without isolation and a restricted data contract.
Neon City's bubble text and synchronized label are illustrative. Its random
initialization and per-frame movement also mean that a general README claim of
determinism is not evidence of deterministic interactive replay for this page.

## Useful composition, without committing to a backend

Keep three choices independent: event/simulation semantics, authorized view
protocol, and visual/audio rendering backend. SpherePOP browser views or Neon
City could provide an initial interaction prototype; the salience card and
history comparator could provide diagnostic views. The engine could host an
adapter, but no backend is selected by this document.

Evaluate candidates against the same contract: recipient-specific scene input,
safe text, protected privacy/exit controls, bounded rendering resources, versioned
schema, upgrade failure/rollback behavior and truthful provenance. A diagnostic
overlay must distinguish input discarded before scoring from scored input
rejected later and from content merely hidden by the renderer. See the
[world/privacy proposal](aye-world-renderer.md).
