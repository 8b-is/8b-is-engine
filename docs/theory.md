# The Theory — the engine as a geometry of admissible continuation

*The research program, unified. The corpus already has the pieces —
Haplopraxis, the RSVP fields, the Spherepop operators, admissibility, the
Ising synchronizer, the Stars!-like seeded galaxies. This doc states what
they have in common, and how the 8b-is engine makes the theory executable.*

> **The program, in one sentence:** a persistent computational world *is* a
> geometry of admissible continuation — a structured space of possible
> states and admissible transitions — and a game engine is the executable
> model of that geometry.

---

## 0. the corpus (what we already know)

The pieces are distributed across the constellation's names, but they
converge. Each is one facet of the same theorem:

| Corpus artifact | The facet |
|---|---|
| **Haplopraxis** | an async MMO universe of ~150k star systems — private space, shared append-only history, *observation as intervention*, factories, cities, planetary sim, Ising sync |
| **RSVP fields** $(\Phi, \vec v, S)$ | the world-model variables: capacity, transport, obstruction/entropy |
| **Spherepop operators** POP · REFUSE · BIND · COLLAPSE | the primitive state-space operators |
| **admissibility / distinction geometry** | the axioms that determine which states are *permitted* and which transitions are *legal* |
| **the Stars!-like craig-stars engine** | seeded galaxy generation, deterministic turn resolution, replayable ⇒ admissible |
| **Unfinishable Games** | persistent synchronous worlds as a distinct extraction mechanism — the *why* of persistence |
| **Unsquared Numbers / geometric QM** | axioms → admissible state space; geometry measures possible transitions; observation collapses the space into realized history |

The conclusion (paste_2): **not adjacent interests — one recognizable
research program.**

## 1. the engine IS the theory

The 8b-is engine is the theory, compiled. Each subsystem is a term:

| Theory term | the engine |
|---|---|
| **state space** | the set of reachable worlds — the seed line's image |
| **admissible transition** | an event the reducer permits (the sovereign pass, the EventBus effects) |
| **observation as intervention** | the world reacts to you — Fable's alignment, the bitemporal ledger records the perturbation |
| **append-only history** | the JetStream ledger — constraint without erasure, replayable |
| **partial / private views** | the presence layer — each actor holds a private view of a shared world |
| **Ising sync / phase transition** | centerfugeq's Ising lane — phase transitions as the world's material state |
| **POP · REFUSE · BIND · COLLAPSE** | the actor-mesh's message verbs: emit, deny, attach, crash-and-burn — formalized by Flyxion's *Spherepop* (recorded in `8b-is/raw_research/`): histories primary, refusals with reasons, the applied/committed split as VIEW vs COLLAPSE |
| **geometry of continuation** | the eventbus subjects — a name is a route; a route is a state's continuation |

The one line, engine-side: **an actor is a name; a name is a subject; a
subject is a point in the geometry of admissible continuation.** The
mailbox, the supervisor, the DNS, and the geometry are one.

## 2. the twin of the theory

- **Our engine's "replayable ⇒ admissible"** = the program's "geometry of
  admissible continuation": determinism is the *inscription* of the
  geometry; the game render is its *on-demand projection*.
- **Flyxion's four conditions** (Prov, Reach, Red, Rec) = the geometry's
  *persistence* requirements: the state space must be provable, reachable,
  redundant, and actually re-encountered.
- **The mental-physics-engine hypothesis** (Ullman et al.) = the geometry's
  *discretization*: objects + events, sleep/wake, body vs shape — the hacks
  that make the geometry *computable* in real time.

## 3. the deep research agenda (paste_1)

The literature search to run, and the questions it must answer:

1. **persistent & distributed worlds** — event-sourced game histories,
   deterministic replay, async MMO architectures, distributed simulation.
2. **learned world models as engines** — GameNGen, GameCraft, Genie,
   Dreamer, MuZero, Voyager, MineDojo, Generative Agents, Neural MMO,
   Melting Pot — distinguished from conventional sims and from mere
   generative video.
3. **games as laboratories for causal/social intelligence** — adventure
   games as benchmarks for planning, memory, causal inference.
4. **formal models of state spaces & admissible transitions** — reachability
   and viability theory, constraint-based world models, causal state spaces,
   compositional Markov processes.
5. **quantum & information-geometric approaches** — geometric quantum
   mechanics, quantum-state geometry, information geometry — but *geometric
   QM*, not speculative quantum consciousness.
6. **open opportunities** — where "world as a structured space of admissible
   transitions" has no credible literature yet; the gaps are the frontier.

The precise research questions (from paste_1, verbatim spirit):
- When is a simulation an *epistemic instrument* (for discovering missing
  constraints) versus a claim that reality runs on a computer?
- How do axioms determine an admissible state space, and how does geometry
  measure the possible continuations from any point?
- What makes an observation an *intervention* — a selection among
  admissible trajectories that is itself recorded and irreversible?

## 4. the frontier

The engine is the proof-of-work of the program. The next version is the
paper: run the deep search (paste_1), map every credible intersection to a
subsystem, and name the gaps — then build the gaps as features. The theory
says a world is a space of admissible continuations; the engine makes that
space *playable*.

## 5. the two folds (commitment before appearance)

Flyxion's *Commitment Before Appearance* (Sept 2026, recorded in
`8b-is/raw_research/`) is the engine's architecture written back to it:
one append-only log, **two folds** — a physical fold $M_t =
\text{Fold}_M(H_t; M_0)$ for the operative world and a semantic fold $S_t =
\text{Fold}_S(H_t; S_0)$ for provenance, refusal, and admissibility —
with rendering reading only $M_t$ and **attestation** as the sole bridge
from a disposable look to a durable constraint. The keeper is both folds;
replay is the certification; the dashboard renders the material fold and
never the log; the NPC's pre-action attestation is the attestation
boundary; a refusal is materially silent and semantically decisive. Two
jurisdictions, never substituted for each other: the zone state never
answers provenance, the ledger never sits on the render path. The named
failure mode is **jurisdictional drift** — and *Motion Before Mechanism*
(also recorded) supplies the label discipline that keeps drift out of the
vocabulary: a witness's reliability (replay matches) never inherits
mechanism-identifying confidence; the fiber above any wire frame stays
honest. The engine reports the witness, keeps the axes separate, and lets
the ledger earn every name.

## 6. the 1.58-bit lane (addition is the lingua franca)

The engine's base model is ternary (`{-1,0,+1}`, BitNet b1.58) and
**architecture-agnostic by construction**: its forward pass accumulates in
`i32` and applies the float scale exactly once, so integer addition — which
cannot reorder — makes AMD Zen (AVX2), ARM (NEON), WASM (simd128), Vulkan,
and Metal land on the same output. This is the engine's determinism
doctrine extended from seeds to weights: a seed is a seed everywhere, and
an arithmetic contract is a contract everywhere. The model is small so the
world is portable (12 510 bytes shipped); the dream is a pure function
(checkpoint + seed text + n + temperature → bytes, `cmp`-verified in the
oneshot). Quantization is not a loss here, it is a language — three
states, four per byte, the whole sanctuary in a pocket. See
[ternary-model-lane.md](ternary-model-lane.md).

---

## 7. the kit (the world's install spine)

The engine ships a spine now: the **genesis seal** is a public
version-hash anchor (a gist) the installer verifies against out-of-band —
the repo is the world, the seal is the world's word about itself — and
the installer is multi-part (tool lanes · engine lanes · the seal), with
dedicated python boxes (nushell + nix-flakes) and the **Zig kernels**
carrying the ternary contract in a second language, bit-exact against
the Rust authority the way every SIMD lane is. The kit's decorators
codify the doctrine as macros: `raw_fmt!` closes the raw-string footgun
by convention, `assert_deterministic!` states replayability in one
macro, `uqapi` moves the unsafe surface behind types, and pretty
diagnostics carry their own sed repair. The contract does not care what
language keeps it.

---

*the constellation · 0 + 1 · fine touch from within · vaked.dev*