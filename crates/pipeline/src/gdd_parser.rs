// gdd_parser.rs — GDD/lore text into strongly-typed zone specs.
//
// Two expanders, one trait:
//   * DeterministicExpander (default) — the brief IS the seed: GAIA folds
//     the zone's eight layers, the ternary wire draws the board, and the
//     bestiary/gear vocabularies are picked by a seeded PRNG. No network,
//     no key, replayable forever.
//   * CommandExpander — a user-provided adapter command (local LLM or API
//     wrapper) that expands text into the same strict JSON. Configured via
//     env; the engine never commits keys.

use serde::{Deserialize, Serialize};
use world_core::gaia::{gaia_state, GaiaState};
use world_core::tern::{balanced_trits, mulberry32, seed_from_text};

/// One kind of fauna — the bestiary archetype (cube-wolf & friends).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityArchetype {
    pub name: String,
    pub color: String,
    /// polygon sides of the sacred form (4, 5, ...)
    pub shape: u32,
    pub size: f32,
}

/// One piece of gear — the stat-bearing item definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemDefinition {
    pub name: String,
    pub stat: String,
    pub value: u32,
    /// rarity tier 0..=2 (the RAR palette)
    pub rarity: u32,
}

/// The zone manifest — the admitted brief, compiled to the wire.
/// The mid-pipeline boundary: everything downstream consumes this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoneManifest {
    pub brief: String,
    pub seed_line: String,
    pub width: u32,
    /// the ternary board: {-1, 0, +1} per tile
    pub board: Vec<i8>,
    /// the world-memory at t=0 — the same universe every language folds
    pub gaia: GaiaState,
    pub archetypes: Vec<EntityArchetype>,
    pub items: Vec<ItemDefinition>,
}

/// A GDD-expansion failure, always with a reason (the axis, not a score).
#[derive(Debug)]
pub enum ExpandError {
    Io(std::io::Error),
    BadJson(String),
    Command(String),
}

impl std::fmt::Display for ExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandError::Io(e) => write!(f, "io: {e}"),
            ExpandError::BadJson(e) => write!(f, "bad json: {e}"),
            ExpandError::Command(e) => write!(f, "expander command: {e}"),
        }
    }
}

impl std::error::Error for ExpandError {}

/// The expansion interface: text in, a zone spec out.
pub trait GddExpander: Send + Sync {
    fn expand(&self, text: &str) -> Result<ZoneManifest, ExpandError>;
}

const BESTIARY: [(&str, &str, u32, f32); 3] = [
    ("cube-wolf", "#00f0ff", 4, 22.0),
    ("icosa-owl", "#ff007f", 5, 18.0),
    ("dodeca-bear", "#00ff66", 5, 30.0),
];
const ITEMS: [(&str, &str, u32, u32); 6] = [
    ("Icosahedron Relic", "magnet", 10, 0),
    ("Golden Ratio Matrix", "swift", 1, 2),
    ("Crown of the Ratna", "stomp", 2, 1),
    ("Vajra Arm", "arms", 2, 1),
    ("Garland of 108", "stomp", 1, 0),
    ("Hollow Star Sigil", "jump", 1, 2),
];

/// The engine's own expander: no LLM, no network — the brief is the seed.
pub struct DeterministicExpander;

impl GddExpander for DeterministicExpander {
    fn expand(&self, text: &str) -> Result<ZoneManifest, ExpandError> {
        let brief = first_line(text);
        let seed = seed_from_text(&brief);
        let mut rng = mulberry32(seed as u32);

        let board = balanced_trits(seed, 256)
            .into_iter()
            .map(|t| t as i8)
            .collect::<Vec<_>>();

        let mut archetypes = Vec::with_capacity(3);
        for &(name, color, shape, size) in &BESTIARY {
            if rng() < 0.8 {
                archetypes.push(EntityArchetype {
                    name: name.to_string(),
                    color: color.to_string(),
                    shape,
                    size,
                });
            }
        }

        let mut items = Vec::with_capacity(3);
        for &(name, stat, value, rarity) in &ITEMS {
            if rng() < 0.5 {
                items.push(ItemDefinition {
                    name: name.to_string(),
                    stat: stat.to_string(),
                    value,
                    rarity,
                });
            }
        }

        let gaia = gaia_state(&brief, 0);
        Ok(ZoneManifest {
            seed_line: format!("{seed:016x}"),
            brief: brief.clone(),
            width: 16,
            board,
            gaia,
            archetypes,
            items,
        })
    }
}

/// A user-provided adapter: a command that reads GDD text on stdin and
/// prints the strict JSON spec on stdout (a local LLM wrapper, an API
/// proxy — whatever the operator wires). Never the default; never a key.
pub struct CommandExpander {
    pub cmd: Vec<String>,
}

impl GddExpander for CommandExpander {
    fn expand(&self, text: &str) -> Result<ZoneManifest, ExpandError> {
        use std::io::Write;
        let mut child = std::process::Command::new(&self.cmd[0])
            .args(&self.cmd[1..])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ExpandError::Command(e.to_string()))?;
        child
            .stdin
            .as_mut()
            .expect("piped stdin")
            .write_all(text.as_bytes())
            .map_err(|e| ExpandError::Io(e))?;
        drop(child.stdin.take());
        let out = child.wait_with_output().map_err(|e| ExpandError::Io(e))?;
        if !out.status.success() {
            return Err(ExpandError::Command(
                String::from_utf8_lossy(&out.stderr).trim().to_string(),
            ));
        }
        serde_json::from_slice(&out.stdout).map_err(|e| ExpandError::BadJson(e.to_string()))
    }
}

/// the brief is the first non-empty line, trimmed — one sentence, one seed
fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("the sanctuary at dawn")
        .to_string()
}

/// parse_text — the synchronous entry point used by the orchestrator and
/// the tests: pick the expander from config, return the typed manifest.
pub fn parse_text(text: &str, expander: &dyn GddExpander) -> Result<ZoneManifest, ExpandError> {
    expander.expand(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_GDD: &str = r#"# GDD — the painted forest zone
the painted forest at dawn, ZEN wired

The zone is a dawn-lit forest floor. Fauna: cube-wolves patrol the
canopy edge; dodeca-bears sleep near the hollow star. Gear drops:
garlands of 108 and vajra arms. The wire reads {-1, 0, +1}.
"#;

    #[test]
    fn deterministic_expander_is_replayable() {
        let a = DeterministicExpander.expand(SAMPLE_GDD).unwrap();
        let b = DeterministicExpander.expand(SAMPLE_GDD).unwrap();
        assert_eq!(a, b, "same brief, same seed, same universe");
        assert_eq!(a.brief, "the painted forest at dawn, ZEN wired");
        assert_eq!(a.board.len(), 256);
        assert!(a.board.iter().all(|t| (-1..=1).contains(t)));
        assert!(!a.archetypes.is_empty());
        // the world-memory folded in: gravity is the zone's constant
        assert!(a.gaia.gravity > 0.0);
        assert_eq!(a.gaia.t, 0);
    }

    #[test]
    fn manifest_round_trips_ron() {
        let m = DeterministicExpander.expand(SAMPLE_GDD).unwrap();
        let s = ron::ser::to_string_pretty(&m, ron::ser::PrettyConfig::default()).unwrap();
        let back: ZoneManifest = ron::from_str(&s).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn different_briefs_never_collide() {
        let a = DeterministicExpander
            .expand("the pink tent at dawn")
            .unwrap();
        let b = DeterministicExpander
            .expand("the hollow star at dusk")
            .unwrap();
        assert_ne!(a.seed_line, b.seed_line);
        assert_ne!(a.board, b.board);
    }
}
