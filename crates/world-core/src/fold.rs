// fold.rs — the zone keeper's adjudication, as a pure reducer.
//
// The same admission rules the Python keeper (examples/world-keep.py)
// enforces, compiled: an event is admissible only if it can be a
// continuation of the actor's attested history. Refusals are durable —
// never patched, never erased — so fold(seed, H) = M is idempotent.

use std::collections::HashMap;

/// The compact wire delta: t = tick, n = needs {h,r,s}, a = action,
/// z = asleep. Event-vocabulary minimality: only these keys count.
#[derive(Debug, Clone)]
pub struct Delta {
    pub t: u64,
    pub h: f64,
    pub r: f64,
    pub s: f64,
    pub action: Option<String>,
    pub asleep: bool,
}

/// What the keeper decided.
#[derive(Debug, PartialEq)]
pub enum Verdict {
    /// the event is a provable continuation of the actor's past
    Admitted,
    /// the event cannot continue its past — refused, and why
    Refused(&'static str),
}

const THRESH: [(char, f64); 3] = [('h', 0.35), ('r', 0.3), ('s', 0.4)];
const ACTIONS: [(&str, char); 3] = [
    ("forage the field", 'h'),
    ("sleep in the tent", 'r'),
    ("flee to the gate", 's'),
];

/// The zone's operative state M: last attested needs + last tick per actor.
#[derive(Default)]
pub struct Keeper {
    pub zone: HashMap<String, [f64; 3]>,
    pub last_tick: HashMap<String, u64>,
    pub admitted: u64,
    pub refused: u64,
}

impl Keeper {
    /// Adjudicate one delta. Admission = provable continuation.
    pub fn adjudicate(&mut self, actor: &str, d: &Delta) -> Verdict {
        if !self.zone.contains_key(actor) {
            self.zone.insert(actor.to_string(), [d.h, d.r, d.s]);
            self.last_tick.insert(actor.to_string(), d.t);
            self.admitted += 1;
            return Verdict::Admitted; // birth — first attested state
        }
        // the tick is monotonic per actor: a re-delivered delta is a
        // replay, refused durable (idempotent fold)
        if d.t <= *self.last_tick.get(actor).unwrap_or(&0) {
            self.refused += 1;
            return Verdict::Refused("tick not monotonic — cannot continue its past");
        }
        if !(0.0..=1.0).contains(&d.h) || !(0.0..=1.0).contains(&d.r) || !(0.0..=1.0).contains(&d.s) {
            self.refused += 1;
            return Verdict::Refused("needs out of range");
        }
        if let Some(action) = &d.action {
            let Some((_, need)) = ACTIONS.iter().find(|(a, _)| a == action) else {
                self.refused += 1;
                return Verdict::Refused("unknown action — outside the vocabulary");
            };
            // the action must be entitled by the attestation itself:
            // acting while attesting comfort is poisoning
            let attested = match need {
                'h' => d.h,
                'r' => d.r,
                _ => d.s,
            };
            let threshold = THRESH.iter().find(|(n, _)| n == need).map(|(_, v)| *v).unwrap();
            if attested > threshold {
                self.refused += 1;
                return Verdict::Refused("action without need");
            }
        }
        self.zone.insert(actor.to_string(), [d.h, d.r, d.s]);
        self.last_tick.insert(actor.to_string(), d.t);
        self.admitted += 1;
        Verdict::Admitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delta(t: u64, h: f64, action: Option<&str>) -> Delta {
        Delta { t, h, r: 0.9, s: 0.9, action: action.map(str::to_string), asleep: false }
    }

    #[test]
    fn poison_is_refused_and_entitled_is_admitted() {
        let mut k = Keeper::default();
        assert_eq!(k.adjudicate("ལྷ", &delta(1, 0.9, None)), Verdict::Admitted);
        // forage while attesting comfort — poisoning
        assert_eq!(
            k.adjudicate("ལྷ", &delta(2, 0.9, Some("forage the field"))),
            Verdict::Refused("action without need")
        );
        // the refused event did not move the fold
        assert_eq!(k.admitted, 1);
        assert_eq!(k.refused, 1);
        // forage with the need proven in the attestation — admitted
        assert_eq!(
            k.adjudicate("ལྷ", &delta(3, 0.2, Some("forage the field"))),
            Verdict::Admitted
        );
        assert_eq!(k.admitted, 2);
    }

    #[test]
    fn replays_are_refused_not_patched() {
        let mut k = Keeper::default();
        assert_eq!(k.adjudicate("ལྷ", &delta(10, 0.8, None)), Verdict::Admitted);
        // the same tick delivered again — a replay
        assert_eq!(
            k.adjudicate("ལྷ", &delta(10, 0.8, None)),
            Verdict::Refused("tick not monotonic — cannot continue its past")
        );
        // the fold is idempotent: M unchanged, the refusal is the record
        assert_eq!(k.zone["ལྷ"], [0.8, 0.9, 0.9]);
        assert_eq!(k.last_tick["ལྷ"], 10);
    }
}
