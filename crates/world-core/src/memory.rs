// memory.rs — ultra-cogniM8: the two memories, made precise.
//
// The consciousness question, tightened: **collective memory** (the ledger
// H — every attested trace plus every durable refusal, the wall's
// historical memory, the population's memory) versus **individual memory**
// (an actor's own projection of H — the path it personally carried
// forward). The distinction is structural, not philosophical: an
// individual's memory is its admitted states; refusals belong to the
// collective — the world constrains forward and others learn, while no
// single execution carries the record of what it was refused.

use std::collections::BTreeMap;
use serde_json::Value;

/// One actor's memory: the path it attested, tick by tick.
#[derive(Debug, Clone, PartialEq)]
pub struct IndividualMemory {
    pub actor: String,
    pub ticks: Vec<u64>,
    pub states: Vec<[f64; 3]>,
    /// durable refusals this actor incurred (S-only records it did not
    /// carry into its own fold, but the world cannot forget)
    pub refusals: u32,
}

impl IndividualMemory {
    /// path_len — how long the actor's attested history is.
    pub fn path_len(&self) -> usize {
        self.ticks.len()
    }

    /// last_state — the actor's present, from its own memory.
    pub fn last_state(&self) -> Option<[f64; 3]> {
        self.states.last().copied()
    }
}

/// The population's memory: everything the world cannot forget.
#[derive(Debug, Clone, PartialEq)]
pub struct CollectiveMemory {
    pub actors: Vec<String>,
    pub seq: u64,
    pub admissions: u64,
    pub refusals: u64,
    /// the world's clock position (max attested tick)
    pub max_tick: u64,
}

impl CollectiveMemory {
    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }

    /// forgetting_curve — the Ebbinghaus shape: older ticks are coarser in
    /// the collective's view (the coarsening of the observational rule over
    /// time), returning how many ticks survive at a horizon.
    pub fn forgetting_curve(&self, horizon: u64) -> u64 {
        self.max_tick.saturating_sub(horizon)
    }
}

/// parse — fold the ledger bytes (the node's line-delimited
/// {"a","o","w"} records) into the two memories.
pub fn parse(ledger_bytes: &[u8]) -> (CollectiveMemory, Vec<IndividualMemory>) {
    let mut ind: BTreeMap<String, IndividualMemory> = BTreeMap::new();
    let mut actors: Vec<String> = Vec::new();
    let mut seq = 0u64;
    let mut admissions = 0u64;
    let mut refusals = 0u64;
    let mut max_tick = 0u64;

    for line in ledger_bytes.split(|b| *b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let Ok(rec) = serde_json::from_slice::<Value>(line) else { continue };
        let Some(actor) = rec.get("a").and_then(|v| v.as_str()) else { continue };
        let out = rec.get("o").and_then(|v| v.as_str()).unwrap_or("admitted");
        let w = rec.get("w");
        let t = w.and_then(|w| w.get("t")).and_then(|v| v.as_u64()).unwrap_or(0);
        let needs = w
            .and_then(|w| w.get("n"))
            .map(|n| [
                n.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0),
                n.get("r").and_then(|v| v.as_f64()).unwrap_or(0.0),
                n.get("s").and_then(|v| v.as_f64()).unwrap_or(0.0),
            ]);

        seq += 1;
        max_tick = max_tick.max(t);
        if out == "refused" {
            refusals += 1;
        } else {
            admissions += 1;
        }
        if !ind.contains_key(actor) {
            actors.push(actor.to_string());
        }
        let m = ind.entry(actor.to_string()).or_insert_with(|| IndividualMemory {
            actor: actor.to_string(),
            ticks: Vec::new(),
            states: Vec::new(),
            refusals: 0,
        });
        if out == "refused" {
            m.refusals += 1;
        } else if let Some(n) = needs {
            m.ticks.push(t);
            m.states.push(n);
        }
    }

    (
        CollectiveMemory {
            actors,
            seq,
            admissions,
            refusals,
            max_tick,
        },
        ind.into_values().collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // a ledger slice exactly as the node writes it: cast admissions,
    // a player's admission, a poisoned player refusal, then a replay refusal
    const LEDGER: &[u8] = br#"{"a":"ember","o":"admitted","w":{"t":1,"n":{"h":0.9,"r":0.9,"s":0.95},"a":null,"z":1}}
{"a":"keeper","o":"admitted","w":{"t":1,"n":{"h":0.9,"r":0.9,"s":0.95},"a":null,"z":1}}
{"a":"127.0.0.1:1","o":"admitted","w":{"t":2,"n":{"h":0.5,"r":0.9,"s":0.9},"a":null,"z":0}}
{"a":"127.0.0.1:1","o":"refused","w":{"t":3,"n":{"h":0.9,"r":0.9,"s":0.9},"a":"forage the field","z":0}}
{"a":"127.0.0.1:1","o":"refused","w":{"t":2,"n":{"h":0.5,"r":0.9,"s":0.9},"a":null,"z":0}}
"#;

    #[test]
    fn the_two_memories_diverge_on_refusals() {
        let (col, inds) = parse(LEDGER);
        assert_eq!(col.admissions, 3);
        assert_eq!(col.refusals, 2, "refusals are the collective's — durable");
        assert_eq!(col.actor_count(), 3);
        assert_eq!(col.max_tick, 3, "the world's clock continues");

        let player = inds.iter().find(|m| m.actor == "127.0.0.1:1").unwrap();
        // the individual's memory is its path: one admitted state, one tick;
        // the two refusals it suffered stay with the collective, not its path
        assert_eq!(player.path_len(), 1);
        assert_eq!(player.refusals, 2, "it incurred them, but they are not its path");
        assert_eq!(player.last_state(), Some([0.5, 0.9, 0.9]));
    }

    #[test]
    fn individual_memory_projects_only_its_own_path() {
        let (_, inds) = parse(LEDGER);
        let ember = inds.iter().find(|m| m.actor == "ember").unwrap();
        assert_eq!(ember.path_len(), 1);
        assert_eq!(ember.last_state(), Some([0.9, 0.9, 0.95]));
        assert_eq!(ember.refusals, 0);
    }
}
