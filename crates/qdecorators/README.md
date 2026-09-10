# qdecorators — the 8b-is decorator kit

Bleeding-edge Rust ergonomics from the **8b-is engine**, published
(crates.io, semver). Decorator macros, functional primitives, attested
I/O framing, and the hardware-ultra abstraction contracts of the 1.58-bit
lane.

### decorators
- `raw_fmt!` — format over raw templates with the double-hash convention
  (the `r#"…"#` footgun: any attribute quote next to a hex color cuts the
  literal in half). The `check-raw-strings` guard enforces it.
- `assert_deterministic!` — run an expression twice, demand bit-equality.
  The engine's replay discipline as one macro.

### functional primitives
- `Pipe` / `Tap` / `tap_mut` — left-to-right reads and in-place repairs
- `seq` — fixed-round iteration (the tick loop, as a value)
- `compose` — inside-out function joining
- `memoize1` — deterministic memoization (same input, same value, never
  recomputed)

### attested I/O
- `frame_len_prefix` / `read_len_frame` / `write_len_frame` — the
  wire's 4-byte big-endian framing, reusable anywhere
- `attest` / `attest_frames` — SHA-256 fingerprints, the stager's
  checksum discipline as a library

### the engram stack
- `Engram` — the memory unit; `l1_json()` (snake_case wire), the
  **STACCED** stacked binary (`stacked_encode`/`stacked_decode`, FNV
  trailer), and the ASCII85 printable skin — the L1 → L3 → L2 ladder,
  bidirectional.

### the action lanes
- `ActionLane::{Fast, Root}` (Fast = thunky-lite, the boot
  recommendation), `Level::L1|L2|L3` with o1-style reasoning budgets,
  `Compute::{Plain, R1, O1}` — the OpenAI-compatible grid, typed.

### hardware-ultra contracts
- `Transport` — the lanes: Scalar, Avx2, Neon, Simd128, Vulkan, Msl
- `GEMM` / `TernaryPack` / `GammaScale` — the integer-accumulation,
  tri-state, absmean-scale contracts `world-core::gemm` satisfies

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
