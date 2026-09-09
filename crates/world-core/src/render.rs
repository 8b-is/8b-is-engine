// render.rs — the arena's first rendering: the cast drawn where it stands.
//
// Each fauna is a polygon at its seeded position, sized by its hunger
// (a hungrier creature looms larger), tinted from the constellation
// palette by the hash of its name, and tagged. The renderer reads the
// frame arena only — disposable appearance, never history (the doctrine:
// Render(M_t, ω_t), never H).

use super::entity::Entities;

const PALETTE: [&str; 5] = ["#00f0ff", "#ff007f", "#00ff66", "#ffd700", "#b48bff"];

/// hash — the constellation's name hash (31·h + c), for tint and shape.
fn hash(name: &str) -> u32 {
    let mut h = 7u32;
    for b in name.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    h
}

/// arena_svg — the world drawn from the frame arena. `names` zips with the
/// arena's entities in spawn order; `width`/`height` are the viewport.
pub fn arena_svg(es: &Entities, names: &[String], width: u32, height: u32) -> String {
    let w = width.max(64) as f64;
    let h = height.max(64) as f64;
    let mut out = String::with_capacity(1024);
    out.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"##
    ));
    out.push_str(r##"<rect width="100%" height="100%" fill="#0b0f19"/>"##);

    for (e, name) in es.iter().zip(names.iter()) {
        let pos = es.pos(e).unwrap_or([0.0, 0.0]);
        let needs = es.needs(e).unwrap_or([0.9, 0.9, 0.95]);
        // map the arena's [-100, 100] into the viewport
        let x = (pos[0] + 100.0) / 200.0 * (w - 40.0) + 20.0;
        let y = (pos[1] + 100.0) / 200.0 * (h - 40.0) + 20.0;
        // size by hunger: a hungrier creature looms larger
        let hunger = 1.0 - needs[0];
        let r = 6.0 + hunger * 14.0;
        let k = hash(name);
        let col = PALETTE[(k as usize) % PALETTE.len()];
        let sides = if k % 2 == 0 { 3 } else { 6 };
        let mut pts = String::new();
        for i in 0..sides {
            let a = std::f64::consts::TAU * i as f64 / sides as f64 + 0.3;
            pts.push_str(&format!("{:.1},{:.1} ", x + a.cos() * r, y + a.sin() * r));
        }
        out.push_str(&format!(
            r##"<polygon points="{pts}" fill="{col}" opacity="0.85" stroke="#e8d8c8" stroke-width="0.6"/>"##
        ));
        out.push_str(&format!(
            r##"<text x="{x:.1}" y="{y:.1}" font-size="8" fill="#e8d8c8" text-anchor="middle" font-family="monospace">{name}</text>"##
        ));
    }
    out.push_str("</svg>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::SimWorld;

    #[test]
    fn the_arena_renders_with_every_fauna() {
        let mut w = SimWorld::from_seed("the painted forest", 4);
        let mut es = w.materialize();
        w.step(1);
        w.sync_entities(&mut es);
        let names: Vec<String> = w.fauna.iter().map(|f| f.name.clone()).collect();
        let svg = arena_svg(&es, &names, 320, 240);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        for n in &names {
            assert!(svg.contains(&format!(">{n}<")), "every fauna is tagged");
        }
        assert!(svg.contains(">ember<"));
    }

    #[test]
    fn rendering_is_deterministic() {
        let mut w = SimWorld::from_seed("sanctuary", 3);
        let mut es = w.materialize();
        w.step(2);
        w.sync_entities(&mut es);
        let names: Vec<String> = w.fauna.iter().map(|f| f.name.clone()).collect();
        let a = arena_svg(&es, &names, 320, 240);
        let b = arena_svg(&es, &names, 320, 240);
        assert_eq!(a, b, "same arena, same world, same picture");
    }
}
