# enthea architecture — the VM is a verification target, not a fast runtime

This document locks the role of the register bytecode VM. It settles the
question the doom debug forced: **why does a VM exist here at all?**

## The one-sentence contract

> The VM is the trustworthy reference that *verifies*; native is the fast
> lane that must *pass the gate*; the arena is the memory model they share.

## What each layer is for

| Layer | Role | Non-role |
|-------|------|----------|
| The shrine (16 Boolean letters as opcodes) | the executable alphabet — functional completeness is literal | the performance story |
| The language (`fn`) | a pure surface over the letters: multi-param functions, recursion, tail calls — a real compiler target | a runtime |
| **The VM** | **the self-hosting target + the audit oracle** — deterministic, inspectable, bounded | **NOT the inference runtime** |
| The arena | the decoupled memory model: quantized cells, capability regions, program-below/data-above | a malloc replacement |
| Native | the fast path on the box | allowed to run *after* the gate, not before |
| The gate (parity check) | native output must match VM output on golden inputs | a suggestion |

## Why this split is correct

1. **Inference does not run on the VM.** The pupil runs on MLX (native
   Metal). A bytecode interpreter is 10–100× slower than native and adds
   nothing but loss. If the VM were "the runtime", it would be the wrong
   tool and the whole project would deserve the question Peter asked.
2. **The VM is where trust lives.** Bytecode is inspectable: frames, the
   honesty ledger, the golden-logit checksum. The audit's "Rust runner as
   the reference forward" is the same idea — an interpreter/oracle you
   believe, that the fast path must agree with.
3. **Stage 2 (self-hosting) needs a target.** "enthea compiles enthea" is
   only concrete if the compiler emits something small, stable, and
   deterministic. That is the VM. The language exists so the compiler can
   eventually be written in it — and the VM is what it compiles to.
4. **The arena is not the VM's.** The arena (quantized store, capability
   regions, the `{-1,0,+1}` data plane) is the memory model; the VM is one
   executor on it. A native executor on the same arena is a different
   executor, not a different architecture.

## Tiered execution

- **Reference**: the VM runs the compiler's output. Correct, deterministic,
  auditable. This is what `enthea doom`, the shrine proofs, and the parity
  fixtures run on.
- **Fast lane**: a native codegen tier (per box) that lowers the same
  program to host code. Same arena, same semantics.
- **The gate**: for golden inputs, the fast lane must reproduce the VM's
  registers + arena writes exactly (the golden-logit parity). Until a lane
  passes the gate it is not trusted.

## Consequences

- New opcodes are added to the VM only when the language needs them, and
  each gets a fixture (the address lane: `ldib` unsigned, `lix` signed,
  `aadd`/`asub` raw int16 — the doom surfaced exactly these).
- The register file is dynamically resizable and allocation is monotonic:
  a free-list was removed because "free is just a queue" — corruption by
  construction is impossible, and the calling convention is r0..rn for
  multi-parameter functions (the language extension this doc ships with).
- The doom (`enthea doom`) is the living example: a maze walker written in
  the language, running on the VM — game logic, not beside it.
