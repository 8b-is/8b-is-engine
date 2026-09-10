# Changelog

All notable changes to the 8b-is engine (the backbone), following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) — the policy
is spelled out in [SEMVER.md](SEMVER.md). Released tags correspond 1:1
to versions below.

## [Unreleased]

- mem8 — the hypermesh quad: the 8-byte MEMNET memory cell (bilinear
  phase interpolation, i32 overflow-proof, observer-relative origo, boolean
  gates) with the Zig twin bit-exact across the 255-seed battery.


## [1.9.0] — the crates.io backbone — 2026-09-10

### Added

- **the crates.io lane** — `qdecorators` → `world-core` → `ternary-lane`
  → `corelib` at 1.9.0 (the lane crate renamed `ternary-lane`;
  `ternary` is an old crates.io name), driven by the `publish` workflow
  with the `CARGO_REGISTRY_TOKEN` secret and the `.crate` tarballs kept
  as release artifacts.
- the memory stack finalized: `Engram.links` (the memory's own graph),
  `corelib::persist` (`Addr` virtual addresses, content-hash de-dupe
  blocks, the single `&mut self` writer, segment splits), `LazyCache`
  (decode-on-first-ask, session `forget`), `corelib::mq` (`Mq` FIFO,
  `chan()`, `Locker` the single-writer door), and `TripleBox` — the
  L1/L3/L2 diamond with lazy memoized skins.
- the engram demo (log lines → chained engrams → triplebox skins, exact diamond decode) and the dream-penetration eval (polars, the mirror's catalog[dream])
- the genesis seal carries the live version; `scaffold genesis` warns
  when the remote seal's version disagrees with the local tree.

### Changed

- workspace + kit crates at **1.9.0** (post-1.0 policy: MINOR =
  features, MAJOR = breaking contracts).
- the Zig kernels compile `-fPIC` — the x86_64 Linux linker accepts the
  static lib (the CI `R_X86_64` relocations cleared).

## [0.7.0] — the kit edge — 2026-09-10

### Added

- **`qdecorators`** — the publishable kit crate: `raw_fmt!` (the
  double-hash raw-template convention) + its `check-raw-strings` footgun
  guard, `assert_deterministic!`, functional primitives (pipe/tap/seq/
  compose/memoize1), attested I/O framing (`frame_len_prefix` +
  `read/write_len_frame` + `attest*`), and the hardware-ultra contracts
  (`Transport`, `GEMM`, `TernaryPack`, `GammaScale`).
- **`corelib`** — the one-import facade over world-core + ternary +
  qdecorators.
- **`uqapi`** — the typed unsafe quotient: SIMD lanes as types
  (`Uq::gemm::<Lane>`), `Cstr` RAII over the FFI string bridge, and the
  borrow-domain concerns behind types.
- **pretty diagnostics** — Python-traceback-style errors whose `Fix`
  carries a pasteable `sed` repair (the compiler's "try this," made
  machine-runnable).
- **`lanes`** — the two action lanes (Fast · thunky-lite / Root),
  L1-L3 o1-style compute budgets, the known `r1` mode.
- **The Zig kernels** — the ternary GEMM + tri-state pack in Zig,
  default-on (`build-obj` + `zig ar`), reached typed through the C ABI;
  Rust↔Zig bit-exactness tested against the scalar authority.
- **The genesis seal** — the public version-hash gist + the multi-part
  installer (`scaffold.sh` part 01 tool lanes · 02 engine lanes · 03 the
  seal) attesting version, artifacts, and the golden out-of-band.
- **The python sandboxes** (nushell + nix-flakes) — `sandbox compile |
  repl | run`, wired as `./scaffold.sh py-sandbox`.
- **The ultra-graphs** — `world-core::graph`: seeded tri-state relation
  fields over the cast, affinity + community folds, `ultra_svg`.
- **engram → blob** — the memory-unit ladder in `qdecorators::engram`:
  L1 snake_case JSON wire, L3 STACCED stacked binary (FNV-trailed), L2
  ASCII85 printable skin — bidirectional, corruption-hearing.
- **`lanes`** — the two action lanes (Fast · thunky-lite / Root) with
  L1-L3 o1-style compute budgets and the known `r1` mode.
- **`crates/ternary-lane::dream_continuation` + the dream, in the browser** —
  the wasm ABI dreams inside the wasm; `client/dream-dashboard.html`;
  the dream bytes are byte-equal to the native dream.
- **The relay's QUIC door** — WebTransport through the same compact
  wire (wtransport, datagrams ↔ length-framed TCP).
- Runner tooling: `scripts/agy` (+ `agy-setup` install/auth/ensure —
  the M1 brain wrapper), `scripts/genesis-seal.sh`, `scripts/build-wasm.sh`.

### Changed

- Pipeline `run` no longer re-stages existing assets (list-only
  enumeration); `pipeline graph` describes the layer-1 DAG.
- The workspace converges on rustfmt; zero warnings.
- `ternary` lib drops its world-core dependency for the wasm surface
  (the PRNG family pinned bit-identical in tests).

### Fixed

- The wasm simd128 GEMM stride bug (half the inputs were skipped) —
  caught by the three-surface golden.
- `mesh-relay` release builds in CI (the container's missing sccache is
  overridden).
- The floors harness in CI now actually checks out centerfugeq + pocoo.

## [0.6.2] — the dream, drawn in the browser — 2026-09-10

### Added

- `ternary_dream_c` (wasm ABI) + `client/dream-dashboard.html` + the
  three-surface golden (aarch64 NEON / x86-64 AVX2 / wasm simd128).
- The commit-batch on top of 0.3.0's lane (see the release body).

## [0.3.0] — the world runs without you — 2026-09-09

### Added

- `world-core`: tern, GAIA, the fold + keeper, sim/fauna, entity arena,
  MEMNET memories, kompress (zstd), tick, transport (ring + ledger),
  render — 4-language determinism pinned by fixtures.
- `mesh-node` (hub/instance zones, zero-lock loop, durable ledger,
  Phoenix continuity), `mesh-relay` (WS door), `pipeline` (ingestion).
- The floors, the swarm, the export lane, the CI oneshot.

## [0.2.0] — the floors + the tooling foundation — 2026-09-08

### Added

- The playable floors, the ternary wire + quantTernEngine, the theory
  vault, the export lane, the actor-mesh EventBus, the creative swarm,
  the DX (jj, scaffold, sccache), the MMO bibles research.

## [0.1.0] — the first release — 2026-09-07

### Added

- The actor-mesh EventBus, visual direction, credits, quick-start,
  git-lfs.

[Unreleased]: https://github.com/peterlodri-sec/8b-is-engine/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/peterlodri-sec/8b-is-engine/releases/tag/v0.7.0
[0.6.2]: https://github.com/peterlodri-sec/8b-is-engine/releases/tag/v0.6.2
[0.3.0]: https://github.com/peterlodri-sec/8b-is-engine/releases/tag/v0.3.0
[0.2.0]: https://github.com/peterlodri-sec/8b-is-engine/releases/tag/v0.2.0
[0.1.0]: https://github.com/peterlodri-sec/8b-is-engine/releases/tag/v0.1.0
