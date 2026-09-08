# GAIA — the world-memory (one seed, eight layers)

GAIA is the engine's central world-memory ACT: the universe history /
memory module. Every physical constant of a zone folds out of **one
pseudorandom seed** — deterministically, forever. Nothing is stored;
everything is re-derivable.

```
brief ──▶ GAIA(brief, t) ──▶ {time, weather, entropy, gravity, wind, temp, light, memory}
```

## The eight layers

| Layer | What it is | How it folds |
|---|---|---|
| `time` | the zone clock (tick / epoch) | t, the day is `t % 86400` |
| `weather` | one word per 5-minute window | `WEATHER[whash(base, t/300) % 8]` |
| `entropy` | monotone, never rewinds | `t/86400 + seed noise` — the universe only forgets forward |
| `gravity` | the zone's constant field | seed-constant, 9.7..10.7 |
| `wind` | direction · speed · gust | window anchors, interpolated, gusted |
| `temp` | the season's breath | 10-day seasonal sine + day wobble + noise |
| `light` | the sun's arc 0..1 | `max(0, sin(t/86400 · 2π))` |
| `memory` | the history pointer | hash chain over the folded hours (capped at 4096) |

Same seed + same tick ⇒ same universe. The implementation is **written
twice, deterministically once**:

- `centerfugeq/quantTernEngine/gaia.ts` — the canonical module (Node 24
  native TS, no build step), CLI: `node quantTernEngine/gaia.ts <brief> [tick]`
- `examples/gaia.py` — the mesh actor, publishing `gaia.state` on the
  compact wire (single-char keys: `t w e g v p l m`); field-for-field
  identical to the TS (verified: same brief + tick → same universe in
  Node and in Python)

## Why GAIA is the engine's answer to "the world runs without you"

- **The field, not a script** — actors (NPCs, the keeper, players) read
  GAIA; GAIA reads nothing. The weather exists whether or not anyone is
  logged in.
- **The fold's root** — `M = fold(H)` starts at `GAIA(brief, 0)`; the
  world replays from the brief alone.
- **The shared core** — the same eight-layer function is `world-core`
  (`crates/world-core`), compiled natively and to `wasm32` — the browser
  client calls `gaia_wire_c` and gets the identical frame. Four-language
  determinism (Node, Python, Rust, WASM) is enforced by pinned fixtures
  in the test suite.
- **The compact wire** — `gaia.state` frames are single-char keys, the
  ternary-wire discipline: minimal event vocabulary, minimal bytes, the
  mesh's lowest-latency publisher.

Run it:

```bash
node quantTernEngine/gaia.ts "sanctuary·overworld" 86400     # the field at tick 86400
uv run --with nats-py python examples/gaia.py --brief "sanctuary·overworld"  # the live actor
cd crates/world-core && cargo test                            # Rust: the same universe, pinned
node crates/world-core/smoke.js "sanctuary·overworld" 86400   # WASM: the browser's call
```

— the constellation · 0 + 1 · fine touch from within · vaked.dev
