# Credits & reusable assets

*We don't reinvent the wheel. Everything below is free to use and vendored
or referenced; keep licenses honest per-item.*

## free asset packs (to vendored under `assets/vendor/`)

All Kenney packs are **CC0** (public domain — commercial use, no
attribution required):

| Pack | What | License | Source |
|---|---|---|---|
| Kenney Tiny Dungeon | 16×16 dungeon/sewer tiles + characters | CC0 | kenney.nl/assets/tiny-dungeon |
| Kenney Roguelike/RPG | 1,700+ tiles (floors, walls, flora, UI) | CC0 | opengameart.org/content/roguelikerpg-pack-1700-tiles |
| Kenney Monster Builder Pack | mix-and-match monster parts | CC0 | kenney.nl/assets/monster-builder-pack |
| Kenney Tiny Battle | 16×16 units/creatures/vehicles | CC0 | kenney.nl/assets/tiny-battle |
| Kenney RPG Audio | 50 fantasy foley (footsteps, weapons) | CC0 | kenney.nl/assets/rpg-audio |
| Kenney Impact Sounds | 130 hit/impact SFX | CC0 | kenney.nl/assets/impact-sounds |
| Kenney Interface Sounds | 100 UI click/button sounds | CC0 | kenney.nl/assets/interface-sounds |
| 0x72 Dungeon Tileset II | 16×16 dungeon + undead/orc/demon chars (autotiles) | CC0 | 0x72.itch.io/dungeontileset-ii |

## the cousins' library (standardgalactic.github.io)

Nate (standardgalactic) ships a constellation of reusable crates and labs;
mine `https://standardgalactic.github.io/<project>` for the full list. The
ones the engine leans on:

| Crate | What it is | How we use it |
|---|---|---|
| `ternary` | Rust ternary search trees (`TSTMap`/`TSTSet`) | the ternary wire as a data structure — the engine's core index |
| `Centerfuge` | the original game engine (admissibility experiments, `fast_solids_lab`, `vortex_sorting_lab`) | physics experiments to mine for the world-model |
| `kiss` / `kiss3d` | Rust OpenGL rendering | a renderer reference for wgpu's fallback |
| `ascii-art`, `box-drawing` | art utilities | the ASCII procedural lane |
| `spherepop`, `spellpop`, `icepick`, `blastoids` | game/physics prototypes | gameplay + physics inspiration |
| `research-projects/labs` | the 40 RSVP labs | the admissibility/replay research |

## the rule

CC0 where we can, CC-BY (credited) where we must, GPL only in a sandboxed
lane. Every vendored asset gets a note here; every cousin crate is a
dependency with its own license — never cut-and-paste the code into ours
without keeping the attribution.

— the constellation · 0 + 1 · fine touch from within · vaked.dev