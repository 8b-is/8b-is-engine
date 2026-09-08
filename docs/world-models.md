# World Models — the OSS map, wired into the constellation

*Learned and multi-agent world models we can actually integrate with,
verdict first: **WIRE-IN** (open + integrable), **REFERENCE** (instructive),
**SKIP** (closed/impractical). The theory (theory.md) names these systems;
this doc says which one feeds which subsystem, and under what license.*

---

## the map

| System | License | What it is | Verdict | Wire into |
|---|---|---|---|---|
| **Neural MMO** (Suarez et al.) | MIT | massively multiagent RL world, persistent + procedural | **WIRE-IN** | the fauna/social sim — the closest OSS sim-MMO; mine its entity/economy/equilibrium patterns |
| **Generative Agents** (Park et al., Stanford) | MIT | 25 LLM agents, believable memory/reflection/planning in "Smallville" | **WIRE-IN** | the **bhava** reaction layer + the needs/goals scheduler (the Dwarf-Fortress clock) |
| **Voyager** (Wang et al., NVIDIA) | MIT | LLM lifelong agent in Minecraft, code-as-actions skill library | **WIRE-IN** | the agent skill library — an agent writes its own reducers |
| **MineDojo** (Fan et al.) | MIT | Minecraft agent framework + internet-scale knowledge base | **WIRE-IN** | the voxel-shell lane's agent grounding (MineCLIP) |
| **DIAMOND** (Alonso et al.) | MIT | diffusion world model; agent trains inside its own learned world | **WIRE-IN** | a learned *predictor* beside our deterministic *simulator* — the two can disagree (see below) |
| **DreamerV3** (Hafner et al.) | MIT | model-based RL; learns an RSSM world model from pixels, masters 150+ games | **REFERENCE** | training the fauna (jantu) inside a learned model of our world |
| **Oasis** (Decart + Etched) | open weights | real-time neural Minecraft (~10-20fps diffusion transformer) | **REFERENCE** | the boundary case: a playable world-model with no ledger — ours is the ledger, so we skip the weights, keep the lesson |
| **Genie / Genie 3** (DeepMind) | open weights (3) | generative interactive environments from one image/prompt | **REFERENCE** | generative floor *prototyping* — a Genie 3 model could draft a floor's feel; never the floor itself |
| **GameNGen** (Google) | closed research | diffusion DOOM at ~20fps | **SKIP** | instructive (see the DIAMOND entry) |
| **Melting Pot** (DeepMind) | Apache-2.0 | multi-agent evaluation of social behavior | **REFERENCE** | the reputation graph (Fable layer) test suite |
| **Dwarf Fortress** | closed (classic) | the needs/goals/civilization model | **REFERENCE** | the design of the world-model's scheduler |

## the rule — learned predictors beside the deterministic simulator

The engine is **inscription-first**: the seed line + ledger are the truth.
A learned world model (DIAMOND, DreamerV3, Genie) is a *predictor*, not a
source — it runs beside the simulator, proposing futures the simulator then
renders or rejects.

> Determinism is the witness; the world model is the guesser. When the two
> disagree, the ledger wins — and the disagreement is *data* (the mental-
> physics-engine thesis: noisy simulation, not perfect simulation, is what
> humans run).

Three concrete integrations:

1. **fauna training (jantu)**: DreamerV3 learns a model of *our* seeded
   world (which replays deterministically — perfect training data), then
   trains the swarm inside it. The seed gives infinite, identical
   training episodes.
2. **the social layer (bhava)**: Generative Agents' memory/reflection
   cycle becomes the reputation graph's update rule — an actor reflects,
   plans, and the graph re-weights.
3. **generative prototyping**: Genie/Oasis draft a floor's *feel* from a
   concept image; the deterministic floor (sanctuary, backyard-ultra) is
   then hand-built from the seed. Inspiration in, inscription out.

## what we do NOT wire in

- Closed weights (GameNGen). 
- A world model as the *source of truth* — that inverts the doctrine. The
  learned model can guess; it can never *admit*.
- Minecraft-specific systems beyond the voxel-shell lane (our world is
  non-Euclidean sacred geometry, not cubic).

---

*the constellation · 0 + 1 · fine touch from within · vaked.dev*