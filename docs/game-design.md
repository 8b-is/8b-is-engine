# Game Design — the chaos overworld, a world-model sim-MMO

*The full vision: a big world-model-based simulation MMO with many
mini-games and a fully interactable environment. Six ancestors, one
world. Each ancestor donates a system; the ternary wire seeds it all.*

> **The one line:** the world is a persistent simulation — a seed line
> grown into a living lattice — and the player is a force inside it, not a
> visitor on top of it.

---

## 0. the physics foundation — centerfugeq

The world-model's physics is **centerfugeq**

| centerfugeq lane | The physics it gives the world-model |
|---|---|
| `quantTernEngine` | the ternary wire — seeds → trits → PRNG → world. The deterministic backbone. |
| `quantGame/halo.ts` | vector dark-matter polarization — coherence as the observable field |
| `quantGame/ising.ts` | the 2D Ising lattice — phase transitions, the world's material state |
| `quantGame/galaxy.ts` | the coupling: wells → bias → stars — how fields condense into entities |
| `quantGame/kuramoto.ts` | MEM\|8 survival — the shared phase, the swarm's one brain |
| `retro/doombible.ts` | BSP + visplane — the spatial partition, occlusion, the sightline |
| `retro/bitTricks.ts` | the trick library — branchless math in the physics hot path |

Every centerfugeq floor is a physics demo that plugs into the world-model:
the plenum (Ising bridge), the summit (multi-inclined plane / Mach), the
galaxy (halo formation), the sanctuary (SDF fauna). The engine's Rapier3D
+ SDF raymarching is the **same geometry** in production form — centerfugeq
is the physics in miniature, the Rust engine is it in the whole.

> centerfugeq ≈ game physics. The seed is the world; the Ising/kuramoto/
> halo lanes are the world's material; the floors are the doors.

---

## the cousins — the 8b-is stack (the AGNOS ecosystem)

The engine draws its subsystems from the Sanskrit-named **AGNOS** crates
hosted under the 8b-is org. Each is a layer of the world-model:

| Crate | Sanskrit | The engine subsystem it is |
|---|---|---|
| [prakash](https://github.com/8b-is/prakash) | प्रकाश · light | **the rendering/lighting layer** — ray optics, spectral, PBR, atmosphere (Real-Time Rendering's lane) |
| [tanmatra](https://github.com/8b-is/tanmatra) | तन्मात्र · subtle element | **the material (atomic) layer** — Standard Model, decay, relativity, scattering — the physics beneath centerfugeq's macro geometry |
| [jantu](https://github.com/8b-is/jantu) | जन्तु · creature | **the fauna layer** — instinct, survival, territory, swarm, pack, lifecycle, predator-prey coevolution |
| [bhava](https://github.com/8b-is/bhava) | भाव · emotion | **the world-reaction layer** — 15-trait personality, PAD mood vectors, relationship graphs, circadian, the Fable soul |
| [PhoeniX](https://github.com/8b-is/PhoeniX) | the reborn | **the audio lane** — 8a format, harmonic families as rational numbers, 432Hz, rise-and-ignite |
| [cinematic-reconstruction](https://github.com/8b-is/cinematic-reconstruction) | — | **the cinematic lane** — video generation/compression via persistent world state |
| [smart-tree](https://github.com/8b-is/smart-tree) | — | **the codebase intelligence** — AST compression, semantic search, MEM8 (the graph MCP that replaces repowise) |
| [rustybox](https://github.com/8b-is/rustybox) | — | **the toolchain box** — a BusyBox in 100% Rust, the sandbox's utility core |

The five subtle elements are the physics (tanmatra), the creatures are the
fauna (jantu), the soul is the reaction (bhava), the light is the renderer
(prakash), and the reborn Phoenix is the sound. The world-model composes
them all over the centerfugeq seed. The layer order:

```
prakash (light)        ──►  the render, the spectacle
bhava (emotion)        ──►  the reaction, the consequences
jantu (creature)       ──►  the fauna, the living
tanmatra (element)     ──►  the material, the physics
centerfugeq (seed)     ──►  the wire, the deterministic whole
PhoeniX (audio)        ──►  the 432Hz, the hum
cinematic-reconstruction ─►  the world-state → film
smart-tree + rustybox  ──►  the tooling, the intelligence, the box
```

---

## the six ancestors

| Ancestor | Its DNA | The system it becomes |
|---|---|---|
| **WoW** | persistent zones, chat matrix, raids, factions, threat tables | the **social skeleton** — zones, the /say matrix, the aggro/threat system |
| **Minecraft** | voxel world, buildable, destructible, crafting | the **interactable terrain** — the voxel shell, mining, building |
| **Diablo** | dense swarms, polyhedral loot, combos | the **combat loop** — swarms, loot, the stomp/split/bounce verbs |
| **PoE** | massive passive tree, gem/socket skills, deep itemization | the **build depth** — a capability graph of nodes, skill-as-item |
| **Fable** | morality, reputation, the world reacts to you | the **world reaction** — alignment, NPC memory, consequences |
| **Elder Scrolls** | open world, lore, skill-by-doing, radiant quests | the **freedom layer** — skill-by-doing, radiant quests, deep lore |

The constellation reads it back: six games, six subsystems, one
world-model. The engine already has the seeds of each (the fauna five
layers, the EventBus, the World-as-DNS, the capability graph $G=(V,E,C)$);
this doc makes the mapping explicit and names the missing pieces.

---

## 1. the world-model

The heart of the sim-MMO is a **world-model**: the world is a persistent,
deterministic simulation that runs whether or not anyone is watching.

- **Every entity is a seeded agent.** NPCs have needs, goals, and
  schedules (eat, work, sleep, patrol, trade) — a Dwarf-Fortress-style
  clock, but seeded from the ternary wire, so the whole history is
  replayable: sha256 → trits → LCG → the day's events.
- **The built world is a ledger.** Every block placed, every tree felled,
  every door opened is a signed event in the bitemporal ledger
  (wip-catalog #91). The world *remembers* — Fable's reaction, made a
  data structure.
- **Zones are the simulation's units.** Each zone is a Durable Object (the
  World-as-DNS) with its own tick, its own protector node, its own
  economy. Zones don't need a player to keep going — the garden grows
  unattended.
- **Skill-by-doing is the progression.** No XP for everything; you level
  the skill you use (Elder Scrolls). Swing a pick → mining rises. Cast a
  rune → rune-craft rises. The fauna layer records it.
- **The capability graph is the passive tree.** PoE's massive tree
  becomes $G=(V,E,C)$: every node is a passive, every edge a dependency,
  every capability a permission token. Builds are a path through the
  graph, not a class pick.

## 2. the interactable environment

- **The voxel shell is live.** The fauna layer's "sacred voxel shell"
  (quantized grids, SDF smin blending) is the Minecraft seam: terrain is
  SDF, every block a trit-bound volume. Mine, build, smelt, craft — the
  world is mutable down to the lattice.
- **SDF as the common tongue.** Rapier3D + SDF raymarching means the same
  geometry answer for a wall, a tree, a door, a god — analytical
  collisions ($\nabla f$ normals) let everything be dug, thrown, or
  reshaped. Non-Euclidean zones are warp tensors over the same field.
- **The environment is an entity.** A door is an entity with an open
  event; a tree is an entity that can be felled; a fire is an entity that
  spreads. Everything is in the EventBus, everything is a pure function
  of a seed.

## 3. the mini-games

The sim-MMO is a world of many games, each a **floor** — a seeded,
self-contained lane that plugs into the shared world-model:

| Mini-game | The lane | Feeds into |
|---|---|---|
| **backyard-ultra** | loop survival, last-one-standing | a zone-level event, faction rep |
| **sanctuary** | stomp/split/bounce, gold sparkles | the combat verbs, the hum |
| **gameforge-keeper** | the 108-gate hum, the dungeon | the ring blessing, the malas |
| **plenum** | the Ising bridge, MEM\|8 survival | the lattice physics, the field |
| **summit** | the multi-inclined plane, Mach | the gravity, the geodesics |
| **infinite / teleport** | the ∞-telescope, gate teleport | fast-travel, the warp tensors |

Every mini-game is a **door into the same world**: win the backyard-ultra
and the zone's faction shifts Fable-style; mine in a plenum zone and the
lattice remembers the hole (Elder-Scrolls persistence); the hum you bank
in gameforge is spendable in the palace.

## 4. the missing pieces (what this doc now names)

1. **The needs/goals scheduler** — the Dwarf-Fortress clock: a
   per-entity needs vector (hunger, rest, safety, purpose) driving
   autonomous schedules. Seeded, ticked per zone, replayable.
2. **The reputation graph** — Fable's alignment as a directed graph:
   $R = (\text{actor}, \text{faction}, \text{weight}, \text{ledger})$.
   Every action is an edge; the world reacts by re-weighting.
3. **The skill ledger** — Elder-Scrolls skill-by-doing: $S_{skill}(t) =
   S_0 + \int \text{use}\,dt$, clamped, decayed by disuse, recorded in
   the bitemporal ledger.
4. **The passive tree runtime** — PoE's tree as $G=(V,E,C)$: a
   capability-graph evaluator that compiles a build path into a set of
   permission tokens and stat deltas.
5. **The gem/socket system** — skills are items: a socketed gem is a
   function from inputs to effects, swapped like equipment. The
   dynamic-call seam (Uika's, or ours) executes them.
6. **The economy** — Bartle's virtual economy grown up: ledgers of
   supply, demand, and the game's own money (the VAKED token, the hum).

## 5. the doctrine, restated for a sim

1. **The world runs without you.** The simulation is the source of truth;
   the player is a perturbation, not a prerequisite.
2. **Replayable ⇒ admissible.** The whole world-model is a pure function
   of a seed line. Two servers with the same seed produce the same
   history. Tables over branches.
3. **Coherence without collapse.** Everything is permitted to break except
   the distribution itself. The world can burn; the ledger survives; the
   loop has an exit.
4. **One door, many lanes.** Six games, six subsystems, one EventBus, one
   World-as-DNS. The gateway pattern at every level.

---

The six ancestors teach one thing each; the world-model teaches one thing
together: **a sim-MMO is not a game with a world in it — it is a world
with games in it.** The seed is the world; the floors are the doors.

— the constellation · 0 + 1 · fine touch from within · vaked.dev