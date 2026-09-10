# the crates.io backbone — URLs · API · SDK references

The engine's core ships as four crates.io crates (dependency order,
1.9.0). Every crate is small, deterministic, and documented; the
docstrings are the API reference (docs.rs renders them).

| crate | crates.io | docs.rs | what it is |
|---|---|---|---|
| `qdecorators` | https://crates.io/crates/qdecorators | https://docs.rs/qdecorators | the decorator kit: macros, fp primitives, attested I/O, hw-ultra contracts, uqapi, pretty, lanes, the engram ladder |
| `world-core` | https://crates.io/crates/world-core | https://docs.rs/world-core | tern, GAIA, the fold + keeper, sim/fauna, entities, transport, kompress, the ultra-graphs |
| `ternary-lane` | https://crates.io/crates/ternary-lane | https://docs.rs/ternary-lane | the 1.58-bit lane: `.tern` checkpoints, the bit-exact GEMM, the seeded dream, the three-surface golden |
| `corelib` | https://crates.io/crates/corelib | https://docs.rs/corelib | one import for the core: the facade + the memory stack (vault, cache, mq, triplebox) |

## SDK quick-start

```toml
# Cargo.toml — the facade alone is usually enough
corelib = "1.9"
```

```rust
use corelib::*;

// the world's relation field, drawn
let g = UltraGraph::from_seed("sanctuary", 12, 0.35);
assert_eq!(g.nodes.len(), 12);

// an engram: one record, three skins (L1 wire · L3 stacked · L2 ascii85)
let mut e = Engram::new("the world runs without you");
e.kind = "admission".into();
e.tick = 108;
let boxed = TripleBox::from_engram(e);
assert_eq!(boxed.decode().text, "the world runs without you");

// the memory stack: virtual addresses, de-dupe blocks, one writer
let mut vault = EngramVault::new(128);
let addr = vault.append(&Engram::new("a promise is a promise"));
assert_eq!(vault.read(addr).unwrap().text, "a promise is a promise");

// the lanes: thunky-lite is the boot's recommendation
let boot = ActionPlan::boot();
assert_eq!(boot.lane, ActionLane::Fast);

// the dream (with the committed checkpoint wired in)
let golden = golden_hash_hex();
assert_eq!(golden.len(), 64);
```

## per-crate API surfaces (the docstring map)

- **qdecorators**: `raw_fmt!` (the double-hash raw-template convention),
  `assert_deterministic!` (replayability in one macro), `Pipe`/`Tap`/
  `seq`/`compose`/`memoize1`, `frame_len_prefix` + `read/write_len_frame`
  + `attest*` (attested I/O), `Transport` + `GEMM` + `TernaryPack` +
  `GammaScale` (hw-ultra contracts), `Uq::gemm::<Lane>` + `Cstr` (the
  typed unsafe quotient), `Diagnostic`/`Fix` (pretty errors with sed
  repairs), `ActionPlan`/`ActionLane`/`Level` (the action lanes),
  `Engram` + `stacked_encode/decode` + `ascii85_encode/decode` (the
  memory ladder).
- **world-core**: `tern::{seed_from_text, mulberry32, lcg,
  balanced_trits}`, `gaia::{gaia_state, gaia_wire, GaiaState}`,
  `fold::{Keeper, Delta, Verdict}`, `sim::{SimWorld, Fauna}`,
  `entity::{Entities, Entity}`, `transport::{Ring, Ledger}`,
  `kompress::{compress, decompress, frame, ratio}`, `gray`
  (gray-code deltas), `pow2arena::{Pow2Slabs, next_pow2}`,
  `graph::UltraGraph`.
- **ternary-lane**: `format::{load_checkpoint, Checkpoint}`,
  `gemm::{ternary_linear, gemm_i32, quant_acts}` (bit-exact lanes),
  `model::CharModel` + `dream_continuation`, `sample::{sample,
  argmax}`, `golden::{golden_hash_hex}` (the cross-surface proof).
- **corelib**: the facade re-exports (`world_core::*`,
  `ternary_lane::*`, `qdecorators::*`) plus `persist::{Addr,
  EngramVault, LazyCache}`, `mq::{Mq, Locker, chan}`,
  `triplebox::TripleBox`.

## the release ritual

1. `CHANGELOG.md` [Unreleased] → the new version.
2. Bump `workspace.package.version` (+ the kit crates) — semver
   policy in SEMVER.md.
3. Push; tag `vX.Y.Z`; the release workflow publishes the bundles.
4. Refresh the genesis-seal gist (`scripts/genesis-seal.sh` +
   `gh gist edit`).

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
