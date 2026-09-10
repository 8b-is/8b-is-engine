//! lockstep — the world is byte-reproducible: two casts born from the
//! same seed, fed the same ticks, land on the same register.
//!
//! The engine's promise, tested as an invariant rather than a hope: run
//! the same brief through two independent `SimWorld`s for 240 ticks,
//! materialize both into fresh entity arenas, and require that every
//! need, every position, and every keeper delta is *bit-identical*. The
//! world runs without you — and deterministically, wherever it runs.

use world_core::{Entities, SimWorld};

fn fingerprint(world: &mut SimWorld, ticks: u64) -> Vec<u8> {
    let mut out = Vec::new();
    let mut es: Entities = world.materialize();
    for t in 1..=ticks {
        for (name, delta) in world.step(t) {
            for b in name.as_bytes() {
                out.push(*b);
            }
            out.extend_from_slice(&delta.t.to_le_bytes());
            for v in [delta.h, delta.r, delta.s] {
                out.extend_from_slice(&v.to_bits().to_le_bytes());
            }
        }
        world.sync_entities(&mut es);
    }
    // the register, in arena order: needs + position, bit by bit
    let mut ids: Vec<_> = es.iter().collect();
    ids.sort_by_key(|e| e.0);
    for e in ids {
        if let Some(n) = es.needs(e) {
            for v in n {
                out.extend_from_slice(&v.to_bits().to_le_bytes());
            }
        }
        if let Some(p) = es.pos(e) {
            for v in p {
                out.extend_from_slice(&v.to_bits().to_le_bytes());
            }
        }
    }
    out
}

#[test]
fn two_worlds_land_bit_identical() {
    let mut a = SimWorld::from_seed("sanctuary", 7);
    let mut b = SimWorld::from_seed("sanctuary", 7);
    let fa = fingerprint(&mut a, 240);
    let fb = fingerprint(&mut b, 240);
    assert_eq!(fa.len(), fb.len());
    assert_eq!(fa, fb, "the fold must be byte-reproducible");
}

#[test]
fn a_different_seed_is_a_different_world() {
    let mut a = SimWorld::from_seed("sanctuary", 7);
    let mut c = SimWorld::from_seed("the tent", 7);
    let fa = fingerprint(&mut a, 96);
    let fc = fingerprint(&mut c, 96);
    assert_ne!(fa, fc, "the seed is the world's one-way door");
}
