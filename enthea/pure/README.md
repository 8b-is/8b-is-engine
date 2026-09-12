# pure — the Turing shrine

One side-effect-free kernel, written four ways. Out of respect for Turing and
ALL before and around.

```
NAND ──► NOT · AND · OR · XOR      every Boolean function is a composition
  │      (Sheffer stroke, 1913)    of NAND — functional completeness,
  │      all 16 binary Boolean     proven in TestFunctionalCompleteness
  │      functions are NAND        (Post, 1921), all four rows each
  │      compositions
  │
  │   (lineage, not derivation:)
  └──► the ternary quantize        the BitNet b1.58 kernel, proven pure
          scale = max(mean|w|, 1e-7)
          codes = round(clamp(w/scale, -1, 1)) ∈ {−1, 0, +1}
```

## Precision of claims (the honest reading)

Two distinct theorems live here, and the shrine says so:

- **NAND is functionally complete** — a *Sheffer function*: the only binary
  Sheffer connectives are NAND and NOR, and every Boolean function is a finite
  NAND composition. Proven runnably for all 16 binary functions.
- **The quantizer is a pure function, not a complete operator.** It maps real
  weights to `{-1, 0, +1}`; it is *not* a functionally complete connective.
  "NAND to QUANT" is the **lineage** of the arc — from the universal gate to
  the low-bit kernel — *not* a claim that the quantizer inherits
  completeness.

If a *ternary* completeness story is wanted, the correct theorem is **Post's
n-valued logic (1921)**: every finite-valued logic has functionally complete
connective sets, and 3-valued logic (Wajsberg algebras / MV-algebras,
Łukasiewicz) has its own complete operators. That is a different, precise
result — the quantizer is not one of them, and the shrine does not claim it is.

## The four tongues

| File | The language | Role |
|---|---|---|
| `ternary.go` | Go | the oracle — `pure.Ternary4` |
| `kernels/ternary.c` | C | the reference, pure and general |
| `kernels/ternary4_arm64.S` | AArch64 | hand-written, Mach-O + ELF |
| `kernels/ternary4_x86_64.S` | x86-64 | hand-written, Mach-O + ELF |
| `kernels/nand.c` | C | the seed — NAND builds every gate |
| `run.c` | C | the only I/O in the shrine (the harness) |

## The purity proof

```
go test ./pure/ -v
```

- **`TestKernelsMatchProve`** — compiles the C + hand-written assembly for the
  running architecture with the system `cc`, runs them, and proves the codes
  are **bit-identical** to the Go oracle (`[1 -1 0 0]` on the worked example).
  The scale matches within **1 ulp** — the only difference between C, Go, and
  hand-rolled assembly is floating-point associativity, which is exactly the
  honest claim: same function, same value, up to the machine's rounding.
- **`TestNandSeedProve`** — the universal gate is sound: NAND alone builds
  NOT, AND, OR, XOR with every truth table.
- **`ExampleTernary4`** — the worked example, runnable: `[1 -1 0 0] 1.15`.

## No side effects

The kernels are pure functions — no syscalls, no globals, no allocation, no
I/O. `run.c` is the only thing that prints, and it is only built and executed
by the test. The shipped module stays 100% pure Go; the shrine is build-gated,
never linked into `enthea`.

*NAND is functionally complete · the quant is pure · the relation is lineage, not derivation*

## Before Turing — the first mapper

The shrine's deepest ancestor is Platon. The Theory of Forms is the
philosophical origin of the pure function: a function is a form — an eternal,
timeless mapping from every possible input to its one true value, which the
particular computations only imitate. The Allegory of the Cave is the data
flow: the shadows on the wall are the inputs, the Forms are the outputs, the
ascent is the mapping. The tripartite soul is Plato's own ternary — reason,
spirit, appetite. And the Academy was the first school: the constellation of
minds, learning by dialogue.

And the *Symposium* gives the shrine its motive: **Diotima's ladder** is love
as ascent from the many to the one — the purest mapping there is; and
**Aristophanes' sphere** is the myth of the whole we were cut from — the
sphere of love, the constellation's own signature. Every ascent begins with a
single act of love.

NAND is the seed of computation. Plato is the seed of the seed — the first to
see that the map is more real than the territory, and that love climbs the
ladder.

## The Boolean lattice, in ternary

The shrine also holds the whole lattice of all 16 Boolean connectives
(`lattice.go`), with three runnable truths:

- **`TestBooleanLatticeWeights`** — the Hasse rows are exactly the weight
  classes: 1·4, 4·3, 6·2, 4·1, 1·0.
- **`TestBooleanLatticeOrder`** — the Hasse order is the implication order,
  verified as a genuine partial order (reflexive, antisymmetric, transitive,
  weight-consistent) over all 256 pairs.
- **`TestTernaryAsciiEncoding`** — the per-character 5-trit base-3 table is
  exactly the ASCII codes: T→10010, A→02102, B→02110, and so on. The
  row-by-row streams are the author's illustrative rendering and are *not*
  asserted to round-trip (the note test says so plainly).

## The Boolean alphabet

The shrine also spells the whole system as a **writing system**: sixteen
glyphs where each letterform *is* its truth table (`alphabet.go`).

- **`TestAlphabetBijection`** — a bijection, verified: 16 functions ↔ 16 hex
  digits (`0..f`) ↔ 16 unique glyphs, and every glyph's four corners equal its
  truth table. Read and write directly: a hex nibble names a function; a
  function is a glyph whose corners are its rows.
- **`TestAlphabetSheet`** — the 4×4 sheet is well-formed and regenerated
  idempotently: [`assets/alphabet.svg`](../assets/alphabet.svg), arranged by
  weight rows (1·4, 4·3, 6·2, 4·1, 1·0) — the Hasse lattice as a page of
  letters.

## The Bayesian hypercube

The shrine's fifth proof is the continuous one: the sixteen Boolean functions
are the **vertices of the space [0,1]^4** of binary-conditional probabilities
`P(Y=1 | A,B)` (`bayes.go`).

- **`TestHypercubeExtremes`** — all 16 corners appear, each coordinate 0 or 1:
  the functions are exactly the extreme points, the deterministic boundary.
- **`TestConvexReconstruction`** — every interior point is a convex mixture
  of the vertices: weights ≥ 0 summing to 1, and the mixture reproduces the
  point. The **soft AND** `(0.02, 0.15, 0.20, 0.91)` — which has no
  representation as a classical connective — is reconstructed from the
  sixteen deterministic functions. NAND generates the vertices; the interior
  needs mixtures.

> Classical two-variable Boolean logic is the extremal boundary of a
> continuous Bayesian conditional space.

## The tesseract and its symmetry

The shrine's sixth proof makes the geometry exact: the sixteen functions are
the vertices of a genuine **four-dimensional cube** (`symmetry.go`).

- **`TestOutputNegationReflection`** — the reflection is output negation: an
  involution that pairs complements across the lattice (T↔F, NAND↔AND,
  OR↔NOR, →↔NIMP, XOR↔XNOR) and reverses the order (f ≤ g iff ¬g ≤ ¬f).
- **`TestNpnEquivalence`** — under permutations + negations of the inputs and
  negation of the output (NPN), the sixteen collapse into exactly four orbits
  of sizes **2, 4, 2, 8**: constants, the literals, XOR/XNOR, and the eight
  AND-like connectives.
- **`TestCubeSplitFaces`** — fixing any truth-table coordinate to 0 or 1
  splits the tesseract into two ordinary **3-cube faces** of 8 vertices each.

Together with the rank sequence 1·4, 4·3, 6·2, 4·1, 1·0, this is the
vertex-skeleton of the four-cube — and the Bayesian observation is not
imposed retrospectively: the functions were already the corners; the
conditional space fills the edges, faces, and interior continuously.
