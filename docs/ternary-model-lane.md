# the 1.58-bit lane — AMD CPUs and 1-bit base models as first-class citizens

`crates/ternary` is the engine's **model lane**: BitNet b1.58
`{-1, 0, +1}` base models treated the way the engine treats everything —
small, seeded, deterministic, tested, public, and portable to every
surface the world runs on. This document is the lane's constitution:
what the arithmetic contract is, why it is bit-exact across architectures,
how the checkpoint is born, and how the dream is read.

## the one-sentence contract

> The forward pass accumulates in `i32` integer arithmetic, and the
> single float scale is applied **once**, at the end.

The proof is machine-run on all three surfaces: the golden hash
(64 hex chars pinning the model's first 32 forward logits) is asserted
bit-identical on aarch64/NEON, x86-64/AVX2 (CI), and wasm32/simd128
(`scripts/wasm-golden.mjs`) — and when the wasm lane once was not, the
golden failed loudly and the stride bug was found. One arithmetic
contract, three machines, one string.

Consequences, in order:

- **AMD CPUs are first-class.** The AVX2 lane runs on any x86-64 core —
  AMD Zen and Intel alike. The same binary produces the same output on
  an AMD laptop, an Intel server, an ARM core (NEON), a WASM sandbox
  (simd128), and — by the same contract — the GPU kernels. There is no
  Apple gate, no CUDA-only path, no "metal board". The engine's arithmetic
  does not know what vendor it is running on.
- **1-bit models are first-class.** Weights live the whole way as
  `{-1, 0, +1}`, packed two bits each, four to a byte. Not a "compressed
  approximation of a bigger model" — the model itself is ternary. A
  16-million-parameter model is 4 MiB on disk; the engine's base model is
  12 510 bytes.
- **Deterministic by construction.** Integer addition is associative —
  it does not care which order you add in. The GPU's parallel grouping,
  the SIMD lane's partial sums, and the scalar loop all land on the same
  `i32`, and therefore the same `f32` output. No floating-point
  weather.

## the pieces

| module | what it is |
|--------|-----------|
| `pack` | the 2-bit tri-state store: `0b00→0, 0b01→+1, 0b10→-1`, `0b11` reserved (strict refuses, lenient zeroes). 4 weights per byte. |
| `gemm` | the integer GEMM: scalar reference + AVX2 (`x86_64`) + NEON (`aarch64`) + simd128 (`wasm32`) lanes, `ternary_linear` with the one-scale-at-the-end rule, the absmean `γ` quantizer. |
| `format` | the `.tern` checkpoint: `TERN1.58` magic, sectioned body, sha256 trailer. The file is the truth. |
| `model` | the forward: embed → (LayerNorm → ternary linear → ReLU, residual) × (L−1) → head → logits, with a hidden carry so the dream conditions on its whole history. |
| `sample` | temperature-scaled sampling on the engine's own `mulberry32` — a seed is a seed everywhere. |
| `shaders/` | the GPU citizens: `ternary_gemm_vulkan.comp` (Vulkan compute — AMD RADV's native consumer lane) and `ternary_gemm_msl.metal` (Apple). Written to the same integer contract; the host applies `s_x · γ` once. |

## the checkpoint

`assets/ternary/sanctuary-1.58.tern` is the engine's base model, trained by
`tools/train_ternary.py` (uv, numpy, fully seeded):

- corpus: `assets/ternary/corpus.txt` — the sanctuary's doctrine, ~2.6 KB
- architecture: emb(128) + 2 residual ternary layers + ternary head
- training: quantization-aware (straight-through estimator), one
  full-corpus pass per step, Adam, gradient clipping; the forward uses
  the exact engine math (dynamic `i16` activation quant, `i32` GEMM)
  so train and dream cannot drift
- manifest: `sanctuary-1.58.json` records the vocab, the full-corpus eval
  CE, and the seed.

Reproduce, verify, dream:

```bash
uv run tools/train_ternary.py                # retrain (deterministic output)
uv run tools/train_ternary.py --verify       # congruence: file ↔ manifest ↔ engine
uv run tools/train_ternary.py --dream "the world runs without" 200
```

The Rust side pins the same promises in tests: the committed checkpoint is
bundled (`include_bytes!`) and a golden logits hash guards cross-arch
stability — if the world ever drifts on some machine, CI sees it.

## the overflow question (asked honestly)

Activations are quantized to `[-16000, 16000]` (headroom under i16). The
i32 accumulation is exact while `16000 · n_in < 2 147 483 647`, i.e. for
inner dimensions up to ~131 000. The lane's models are dim-128 stacks —
three hundred times under the bound. `gemm_i32` asserts the bound, and a
test guards it.

## the dream

```bash
cargo run -p ternary --example dream -- \
  assets/ternary/sanctuary-1.58.tern assets/ternary/sanctuary-1.58.json \
  "the world runs without" 240 0.8 out/sanctuary-dream.txt
```

The dream is a pure function: checkpoint + seed text + n + temperature →
bytes. Same inputs, same bytes, on every surface — the e2e oneshot
reproduces it and requires `cmp` equality. The world runs without you, and
so does its dream.

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
