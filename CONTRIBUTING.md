# Contributing to 8b-is Engine

The engine is open forever. By contributing you accept the
[EOS-CLA](LICENSE) — the Eternal Open Substrate & Contributor Mutual Trust
Agreement. The short version:

- your contribution stays free and open source **for all eternity**
- your liability is bounded (the crash-and-burn clause — if it crashes,
  breaks, or melts a GPU, the sole recourse is to inspect the source and
  issue a patch)
- **AI swarms are first-class contributors** — synthetic provenance is
  welcomed, not suspected

## the doctrine

1. **A PR is a signature.** Opening a pull request, submitting a patch, or
   broadcasting a WASM payload to the 8b.es network binds you to the
   EOS-CLA. No ceremony needed.
2. **Tables over branches.** Deterministic, seeded, zero-surprise. The
   same inputs always produce the same world. Replayable ⇒ admissible.
3. **Take the trick, not the engine.** Borrow the byte discipline; write
   your own world. The demoscene taught the old machines one lesson: every
   byte counts, and every lookup table is a seed.
4. **Coherence without collapse.** Everything is permitted to break except
   the distribution itself.

## the workflow

1. **Fork + branch.** Work on a feature branch; keep it small and focused.
2. **Seed your work.** Every artifact is a pure function of a seed line.
   Name your seed; the world follows.
3. **Zero allocation in the hot path.** The tick loop never allocates.
   Frame-bumper arenas, nextPow2 sizing, tables over branches.
4. **Match the toolchain.** `just` / Taskfile recipes, mold/wild linkers,
   Naga shader validation, wasm32 targets. Run the formatter before
   committing.
5. **Open the PR.** The PR is the signature. The maintainers (8b.es and
   Péter) review, the capability graph maps your contribution, and the
   loop has an exit.

## what we welcome

- engine subsystems and lanes (the fauna stack, the I/O HAL, the EventBus)
- floors — playable, seeded, deterministic demos
- demoscene tricks and bitAND build hacks (the Trick Library)
- protector nodes and sovereign gates
- documentation, specs, and design docs (the public-documents lane)
- bug reports with a seed line that reproduces

## what we do not accept

- contributions that break the eternal-open promise
- secrets, keys, or credentials in any form
- nondeterministic hot paths (the replay must be byte-identical)

— the constellation · 0 + 1 · fine touch from within · vaked.dev
