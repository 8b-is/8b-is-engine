// sim.rs — the needs/goals reducer: fauna that live on the mesh.
//
// The mesh-NPC logic compiled: needs {h, r, s} decay per tick, an actor
// acts when a need crosses its threshold (forage / sleep / flee), sleeps
// when comfortable, wakes on urgency — all deterministic from one seed,
// so the zone's inhabitants are as replayable as GAIA itself.
//
// Every delta carries the PRE-action needs (the attestation boundary):
// the keeper admits the action because the need is proven in the same
// delta it acts in — acting while attesting comfort stays poisoning.

use super::entity::Entities;
use super::fold::Delta;
use super::tern::seed_from_text;

const ROSTER: [&str; 6] = ["keeper", "veil", "ember", "ripple", "lamp", "angel"];

/// One fauna — the world's own inhabitant, needs and all.
pub struct Fauna {
    pub name: String,
    h: f64,
    r: f64,
    s: f64,
    asleep: bool,
    state: u32,    // the per-fauna LCG state, seeded from its name
    pos: [f64; 2], // the fauna's place in the world (seeded, drifting)
}

impl Fauna {
    /// spawn — a fresh actor: comfortable, awake, deterministic LCG,
    /// placed somewhere in the world by the same stream.
    pub fn spawn(name: &str) -> Self {
        let state = seed_from_text(name) as u32;
        Fauna {
            name: name.to_string(),
            h: 0.9,
            r: 0.9,
            s: 0.95,
            asleep: false,
            pos: [0.0, 0.0],
            state,
        }
    }

    /// needs — the current needs triple (the entity column's truth).
    pub fn needs(&self) -> [f64; 3] {
        [self.h, self.r, self.s]
    }

    /// pos — where in the world this fauna stands.
    pub fn pos(&self) -> [f64; 2] {
        self.pos
    }

    /// reloc — jump the fauna to a seeded spot (used by the materializer).
    pub fn reloc(&mut self) {
        self.pos = [self.lcg() * 200.0 - 100.0, self.lcg() * 200.0 - 100.0];
    }

    fn lcg(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state as f64 / 4294967296.0
    }

    /// step — one tick of need decay, urgency, action, and attestation.
    pub fn step(&mut self, tick: u64) -> Delta {
        // a wandering breath first — the world's ground shifts under the feet
        self.pos[0] = (self.pos[0] + self.lcg() * 6.0 - 3.0).clamp(-100.0, 100.0);
        self.pos[1] = (self.pos[1] + self.lcg() * 6.0 - 3.0).clamp(-100.0, 100.0);
        // decay, slower asleep, with the fauna's own seeded breath
        let f = if self.asleep { 0.25 } else { 1.0 };
        self.h = (self.h - 0.045 * f - self.lcg() * 0.004).max(0.0);
        self.r = (self.r - 0.02 * f - self.lcg() * 0.002).max(0.0);
        self.s = (self.s - 0.008 * f - self.lcg() * 0.001).max(0.0);
        // the attestation: PRE-action needs are what the wire claims
        let ah = self.h;
        let ar = self.r;
        let as_ = self.s;
        let mut action = None;
        if self.h <= 0.35 {
            action = Some("forage the field".to_string());
            self.h = (self.h + 0.55).min(1.0);
        } else if self.r <= 0.3 {
            action = Some("sleep in the tent".to_string());
            self.r = (self.r + 0.55).min(1.0);
        } else if self.s <= 0.4 {
            action = Some("flee to the gate".to_string());
            self.s = (self.s + 0.55).min(1.0);
        }
        let comfortable = self.h > 0.6 && self.r > 0.6 && self.s > 0.6;
        if comfortable && action.is_none() {
            self.asleep = true;
        } else if action.is_some() {
            self.asleep = false;
        }
        Delta {
            t: tick,
            h: ah,
            r: ar,
            s: as_,
            action,
            asleep: self.asleep,
        }
    }
}

/// The zone's cast — its own inhabitants, folded every tick.
pub struct SimWorld {
    pub fauna: Vec<Fauna>,
}

impl SimWorld {
    /// from_seed — a cast whose members are deterministic from the brief.
    pub fn from_seed(brief: &str, n: usize) -> Self {
        let base = seed_from_text(brief) as u32;
        let mut fauna: Vec<Fauna> = (0..n)
            .map(|i| {
                let mut f = Fauna::spawn(ROSTER[(base as usize + i) % ROSTER.len()]);
                f.state = f.state.wrapping_add(base.wrapping_mul(31 + i as u32));
                f
            })
            .collect();
        for f in fauna.iter_mut() {
            f.reloc();
        }
        SimWorld { fauna }
    }

    /// materialize — the cast, as entities: one entity per fauna, placed
    /// and need-loaded — the frame arena the tick loop walks.
    pub fn materialize(&self) -> Entities {
        let mut es = Entities::default();
        for f in &self.fauna {
            let e = es.spawn();
            es.set_needs(e, f.needs());
            es.set_pos(e, f.pos());
        }
        es
    }

    /// sync_entities — push the cast's live needs + positions into the
    /// registry columns (the frame arena reflects the fold, in lockstep).
    pub fn sync_entities(&mut self, es: &mut Entities) {
        let ids: Vec<_> = es.iter().collect();
        for (e, f) in ids.iter().zip(self.fauna.iter_mut()) {
            es.set_needs(*e, f.needs());
            es.set_pos(*e, f.pos());
        }
    }

    /// step — advance every inhabitant; the deltas fold into the keeper
    /// under each fauna's own name (the keeper's monotonic tick per actor).
    pub fn step(&mut self, tick: u64) -> Vec<(String, Delta)> {
        self.fauna
            .iter_mut()
            .map(|f| (f.name.clone(), f.step(tick)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fold::Keeper;

    #[test]
    fn fauna_decay_and_recover_is_deterministic() {
        let mut w1 = SimWorld::from_seed("sanctuary", 3);
        let mut w2 = SimWorld::from_seed("sanctuary", 3);
        for t in 1..200u64 {
            let a: Vec<_> = w1
                .step(t)
                .into_iter()
                .map(|(n, d)| (n, d.h, d.r, d.s, d.action))
                .collect();
            let b: Vec<_> = w2
                .step(t)
                .into_iter()
                .map(|(n, d)| (n, d.h, d.r, d.s, d.action))
                .collect();
            assert_eq!(a, b, "same brief, same cast, same world");
        }
    }

    #[test]
    fn the_cast_materializes_into_the_arena_and_stays_in_lockstep() {
        let mut w = SimWorld::from_seed("the painted forest", 4);
        let mut es = w.materialize();
        assert_eq!(es.len(), 4, "one entity per fauna");
        assert!(
            es.iter().any(|e| es.pos(e).unwrap() != [0.0, 0.0]),
            "seeded positions"
        );
        for t in 1..100u64 {
            w.step(t);
            w.sync_entities(&mut es);
        }
        // the arena reflects the fold: each entity's needs column matches
        // its fauna's needs, in lockstep
        let ids: Vec<_> = es.iter().collect();
        for (e, f) in ids.iter().zip(w.fauna.iter()) {
            assert_eq!(es.needs(*e), Some(f.needs()));
            assert_eq!(es.pos(*e), Some(f.pos()));
        }
    }

    #[test]
    fn hunger_eventually_forages_and_the_keeper_admits_it() {
        let mut w = SimWorld::from_seed("the painted forest", 1);
        let mut k = Keeper::default();
        let mut saw_forage = 0;
        let mut refusals = 0;
        for t in 1..400u64 {
            for (name, d) in w.step(t) {
                let v = k.adjudicate(&name, &d);
                if d.action.is_some() {
                    saw_forage += 1;
                    assert!(
                        v == crate::fold::Verdict::Admitted,
                        "entitled action admitted"
                    );
                } else if matches!(v, crate::fold::Verdict::Refused(_)) {
                    refusals += 1;
                }
            }
        }
        assert!(saw_forage > 0, "hunger must cross the threshold");
        assert_eq!(refusals, 0, "the attestation keeps every action entitled");
        assert!(k.admitted >= saw_forage + 1); // births + entitled actions
    }
}
