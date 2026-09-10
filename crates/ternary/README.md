[![crates.io](https://img.shields.io/crates/v/ternary-lane.svg)](https://crates.io/crates/ternary-lane)
[![docs.rs](https://img.shields.io/docsrs/ternary-lane.svg)](https://docs.rs/ternary-lane)

# ternary-lane — the 1.58-bit lane (the 8b-is engine's base-model crate)

BitNet b1.58 `{-1,0,+1}` base models as first-class citizens:
tri-state packing (4 per byte), the `.tern` checkpoint (sections +
sha256 trailer), and the integer GEMM that is **bit-exact across every
surface** (accumulate in `i32`, scale once — integer addition cannot
reorder), carrying the engine's `sanctuary-1.58.tern` base model and its
seeded dream.

Docs: [docs.rs/ternary-lane](https://docs.rs/ternary-lane) — API:
`format` (the checkpoint), `gemm` (the lanes), `model` (the forward),
`sample` (the dream's draw), `golden` (the three-surface proof).

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
