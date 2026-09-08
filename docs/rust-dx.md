# Rust DX — dev + prod building tricks for a faster local loop

*Applied to the vaked-lsp workspace (and any Rust lane of the engine).
Every trick is verified against current cargo/rustc practice; the cheap
ones are already wired in (`.cargo/config.toml`, the profiles below).*

---

## the dev loop — check, not build

| Trick | Why | Command |
|---|---|---|
| **`cargo check` over `cargo build`** | no codegen — the fastest possible feedback; rust-analyzer already does this | `cargo check --all-targets` |
| **watch** | re-check on save | `cargo watch -x check` (watchexec) |
| **nextest** | parallel, per-test-isolated runner — 3–10× faster than libtest | `cargo nextest run` |
| **binstall** | install tool crates (mold, nextest, sccache…) as prebuilt binaries, never compile them | `cargo binstall cargo-nextest mold` |

## the linker — where the seconds actually go

Rust debug builds spend most time in the linker. The fix is a fast linker:

- **macOS**: `rust-lld` (ships with rustup) is the modern default; on
  Apple Silicon add `-C link-arg=-fuse-ld=lld` or use the built-in linker
  of the newest Xcode. `split-debuginfo=unpacked` speeds macOS linking.
- **Linux**: **mold** (`-C link-arg=-fuse-ld=mold`) — seconds, not
  minutes; `lld` as the fallback.
- **Shared cache**: **sccache** dedupes compiled crates across the whole
  machine (and CI) — one warm cache, every branch fast.

```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]   # lld on mac/linux; swap mold in on linux

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld", "-C", "split-debuginfo=unpacked"]
```

## the profiles — dev fast, prod small

```toml
[profile.dev]
opt-level = 1                 # your crate: slightly optimized, still debuggable
debug = "line-tables-only"    # backtraces without the full DWARF cost

[profile.dev.package."*"]     # dependencies: optimize them fully
opt-level = 3                 # (the 200-crate tree only compiles once, cached)

[profile.release]
opt-level = 3
lto = "fat"                   # whole-program: the seed of a small binary
codegen-units = 1             # one unit = max optimization
panic = "abort"               # no unwind tables — smaller + faster
strip = true                  # no symbols in the shipped bin
```

The dev profile keeps *your* crate cheap to rebuild (the thing you touch)
while deps compile at `opt-level 3` once and are cached — so the
edit→check→test loop is sub-second after the first build.

## prod — what actually shrinks the binary

1. `lto = "fat"` + `codegen-units = 1` (above) — the single biggest win.
2. `panic = "abort"` — drops the unwind machinery.
3. `strip = true` — `-trimpath` at the same time for reproducibility.
4. **`cargo bloat` / `cargo llvm-lines`** — find what is fat; `cargo
   machete` finds *unused* deps (each one is compile time you don't need).
5. `#![no_main]` + `-C panic=abort` for tiny tools; `opt-level="z"` +
   `lto` for the size-obsessed.
6. One binary per concern (vaked-lsp / vaked-mcp / vaked-nats are already
   separate bins — shared crate, three thin mains, no dead code shipped).

## the hacks

| Hack | Effect |
|---|---|
| **`cargo check` in rust-analyzer on save** | diagnostics before you finish typing |
| **target dir on fast storage** | `CARGO_TARGET_DIR` → tmpfs/RAM disk on Linux (or an external SSD on mac); the whole dep tree is I/O-bound |
| **`CARGO_INCREMENTAL=1`** | incremental (default in dev); disable for release to avoid the metadata cost |
| **cranelift backend** | `rustup component add rustc-dev` + `cg-clif` — ~2× faster *debug codegen* for the check-free full-build loop |
| **workspaces + `-p`** | `cargo build -p vaked-lsp` rebuilds only one crate, not the tree |
| **`cargo nextest run` over `cargo test`** | isolation + parallelism; failures show the exact command to rerun |
| **`cargo watch -x 'nextest run -E "kind(test)"'`** | TDD without touching the terminal |
| **`CARGO_TERM_COLOR=always` + `bat`** | pretty errors, always |

## CI — the prod mirror

The release pipeline reuses the same profile: `cargo build --release` on
the tag, `panic=abort`+`strip` already in the manifest, and a
`sccache`/`cargo-binstall` bootstrap so CI compiles deps once and caches
them (GitHub Actions cache keyed on `Cargo.lock`). See
`.github/workflows/release.yml`.

---

*the constellation · 0 + 1 · fine touch from within · vaked.dev*