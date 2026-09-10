//! graph — the ultra-graphs: the world's relation field as a structure.
//!
//! The engine's cast is a roster, and a roster without bonds is a
//! handful of names. The ultra-graph is the seeded, tri-state relation
//! field over the cast: nodes are the names, and every chosen edge is
//! one of the wire's own three symbols — `+1` a friendly bond, `-1`
//! a hostile one, `0` a neutral acquaintance. One seed, one graph;
//! the same brief, the same relations, on every surface.
//!
//! Two folds ride on the structure, both deterministic:
//!
//! * **affinity** — each node's net bond: friendly minus hostile edges
//!   (the roster's love/harm lanes, read as a sum).
//! * **communities** — a seeded label propagation over the friendly
//!   edges only: hostility never binds a community, it only repels it.
//!   Iteration order is fixed (sorted nodes, fixed rounds), so the
//!   partition is as replayable as the graph itself.
//!
//! The render is a pure function too (`ultra_svg`): circle layout, sign-
//! colored edges, community-tinted fills — a disposable look over a
//! durable structure, never a history.

use crate::tern::{mulberry32, seed_from_text};

/// A tiny deterministic name pool for the fold's members (the cast's
/// own onomatopoeia, in the wire's spirit).
const NAMES: &[&str] = &[
    "ember",
    "stone",
    "reed",
    "cinder",
    "moss",
    "tide",
    "drum",
    "lantern",
    "thistle",
    "emberfold",
    "hollow",
    "gravel",
    "silt",
    "willow",
    "breath",
    "anvil",
];

/// The ultra-graph: nodes + tri-state edges, seeded.
#[derive(Debug, Clone)]
pub struct UltraGraph {
    /// The seed line the graph was folded from.
    pub seed_line: String,
    pub nodes: Vec<String>,
    /// (a, b, w) with a < b, w ∈ {-1, 0, +1}, deterministic order.
    pub edges: Vec<(u32, u32, i8)>,
}

impl UltraGraph {
    /// from_seed — a cast of `n` names with seeded tri-state bonds:
    /// every ordered pair (a < b) is a bond with probability `p_edge`,
    /// its weight drawn from the ternary stream (mulberry32 under the
    /// brief's seed).
    pub fn from_seed(brief: &str, n: usize, p_edge: f64) -> Self {
        let base = seed_from_text(brief);
        let mut rng = mulberry32(base as u32);
        let mut lcg_s = base as u32;
        let nodes: Vec<String> = (0..n)
            .map(|i| {
                let name = NAMES[(i + (lcg_s as usize)) % NAMES.len()];
                lcg_s = lcg_s.wrapping_mul(1664525).wrapping_add(1013904223);
                if i < NAMES.len() {
                    name.to_string()
                } else {
                    format!("{name}·{}", i)
                }
            })
            .collect();
        let mut edges = Vec::new();
        for a in 0..n {
            for b in (a + 1)..n {
                if rng() < p_edge {
                    let w = rng();
                    let w = if w < 0.34 {
                        -1i8
                    } else if w < 0.67 {
                        0i8
                    } else {
                        1i8
                    };
                    edges.push((a as u32, b as u32, w));
                }
            }
        }
        UltraGraph {
            seed_line: format!("{}·{n}·{p_edge}", brief),
            nodes,
            edges,
        }
    }

    /// affinity — each node's net bond: friendly minus hostile edges.
    /// The roster's love/harm lanes, read as a single sum.
    pub fn affinity(&self) -> Vec<i32> {
        let mut net = vec![0i32; self.nodes.len()];
        for &(a, b, w) in &self.edges {
            net[a as usize] += w as i32;
            net[b as usize] += w as i32;
        }
        net
    }

    /// community_fold — deterministic seeded label propagation over the
    /// friendly edges only (hostility repels, never binds). Fixed order
    /// and fixed rounds ⇒ the partition is a pure function of the seed.
    pub fn community_fold(&self, rounds: u32) -> Vec<u32> {
        let n = self.nodes.len();
        let mut label: Vec<u32> = (0..n as u32).collect();
        // friendly adjacency (sorted neighborhoods, insertion order kept)
        let mut friendly: Vec<Vec<u32>> = vec![Vec::new(); n];
        for &(a, b, w) in &self.edges {
            if w == 1 {
                friendly[a as usize].push(b);
                friendly[b as usize].push(a);
            }
        }
        for _ in 0..rounds {
            for i in 0..n {
                let mut best = label[i];
                for &j in &friendly[i] {
                    let lj = label[j as usize];
                    if lj < best {
                        best = lj;
                    }
                }
                label[i] = best;
            }
            // the label ids compress to [0, c): a stable renumbering so
            // the partition does not depend on boot order
            let mut map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
            let mut next = 0u32;
            for l in label.iter_mut() {
                let e = map.entry(*l).or_insert_with(|| {
                    let v = next;
                    next += 1;
                    v
                });
                *l = *e;
            }
        }
        label
    }

    /// degree_centrality — who is most connected: the friendly (+1) and
    /// hostile (-1) bonds counted per node (the neutral zero stays
    /// silent). Freeman's degree centrality, tri-state edition.
    pub fn degree_centrality(&self) -> Vec<(u32, u32)> {
        let n = self.nodes.len();
        let mut friends = vec![0u32; n];
        let mut foes = vec![0u32; n];
        for &(a, b, w) in &self.edges {
            if w == 1 {
                friends[a as usize] += 1;
                friends[b as usize] += 1;
            } else if w == -1 {
                foes[a as usize] += 1;
                foes[b as usize] += 1;
            }
        }
        (0..n).map(|i| (friends[i], foes[i])).collect()
    }

    /// closeness — the shortest-path sums over the friendly edges (BFS
    /// from every node, deterministic order): how close a member is to
    /// the rest of ITS component, in bonds. Hostility never shortens a
    /// path; it only isolates. A node in a shattered graph is closest
    /// to its own shard (the honest distance: the others are not in
    /// the room).
    pub fn closeness(&self) -> Vec<f32> {
        let n = self.nodes.len();
        let mut friendly: Vec<Vec<usize>> = vec![Vec::new(); n];
        for &(a, b, w) in &self.edges {
            if w == 1 {
                friendly[a as usize].push(b as usize);
                friendly[b as usize].push(a as usize);
            }
        }
        let mut out = Vec::with_capacity(n);
        for start in 0..n {
            let mut dist = vec![usize::MAX; n];
            dist[start] = 0;
            let mut q = std::collections::VecDeque::new();
            q.push_back(start);
            while let Some(u) = q.pop_front() {
                for &v in &friendly[u] {
                    if dist[v] == usize::MAX {
                        dist[v] = dist[u] + 1;
                        q.push_back(v);
                    }
                }
            }
            let sum: usize = dist.iter().filter(|&&d| d != usize::MAX).map(|&d| d).sum();
            out.push(if sum == 0 { 0.0 } else { 1.0 / sum as f32 });
        }
        out
    }

    /// ultra_svg — the disposable look for the durable structure:
    /// circle layout (fixed by node count), color by bond (plus cyan,
    /// minus pink, zero dim), fill tinted by community.
    pub fn ultra_svg(&self, label: &str) -> String {
        let n = self.nodes.len();
        let (w, h) = (640, 400);
        let (cx, cy, r) = (320.0, 190.0, 140.0);
        let pos: Vec<(f64, f64)> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / n.max(1) as f64
                    - std::f64::consts::FRAC_PI_2;
                (cx + r * a.cos(), cy + r * a.sin())
            })
            .collect();
        let communities = self.community_fold(8);
        let hues = [55i32, 200, 350, 150, 280, 0]; // the family palette
        let mut out = crate::raw_fmt!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><rect width="100%" height="100%" fill="#0b0f19"/><text x="320" y="22" font-size="11" fill="#e8d8c8" text-anchor="middle" font-family="monospace">{label}</text>"##
        );
        for &(a, b, w) in &self.edges {
            let (ax, ay) = pos[a as usize];
            let (bx, by) = pos[b as usize];
            let col = match w {
                1 => "#00f0ff",
                -1 => "#ff007f",
                _ => "#3a4a5a",
            };
            out.push_str(&crate::raw_fmt!(
                r##"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{col}" stroke-width="0.8" opacity="0.6"/>"##,
                ax, ay, bx, by
            ));
        }
        for i in 0..n {
            let (x, y) = pos[i];
            let hue = hues[(communities[i] as usize) % hues.len()];
            out.push_str(&crate::raw_fmt!(
                r##"<circle cx="{:.1}" cy="{:.1}" r="7" fill="oklch(.55 .12 {hue})" stroke="#e8d8c8" stroke-width="0.5"/><text x="{:.1}" y="{:.1}" font-size="7" fill="#e8d8c8" text-anchor="middle" font-family="monospace">{}·{:02}</text>"##,
                x, y, x, y + 14.0, self.nodes[i], i
            ));
        }
        out.push_str("</svg>");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_brief_is_the_same_graph() {
        let a = UltraGraph::from_seed("sanctuary", 12, 0.35);
        let b = UltraGraph::from_seed("sanctuary", 12, 0.35);
        assert_eq!(a.nodes, b.nodes);
        assert_eq!(a.edges, b.edges);
        assert_eq!(a.affinity(), b.affinity());
        assert_eq!(a.community_fold(8), b.community_fold(8));
        assert_eq!(a.ultra_svg("fold"), b.ultra_svg("fold"));
    }

    #[test]
    fn a_different_brief_is_a_different_world() {
        let a = UltraGraph::from_seed("sanctuary", 12, 0.35);
        let c = UltraGraph::from_seed("the tent", 12, 0.35);
        assert_ne!(a.edges, c.edges, "the seed must split the bonds");
    }

    #[test]
    fn the_alphabet_is_ternary() {
        let g = UltraGraph::from_seed("sanctuary", 20, 0.6);
        assert!(g
            .edges
            .iter()
            .all(|&(a, b, w)| a < b && (-1..=1).contains(&w)));
        let max_edges = 20 * 19 / 2;
        assert!(g.edges.len() <= max_edges);
    }

    #[test]
    fn affinity_is_the_net_bond() {
        let g = UltraGraph::from_seed("pair", 2, 1.0);
        // p_edge = 1.0 and the ternary draw decides the single bond's sign
        let w = g.edges[0].2 as i32;
        assert_eq!(g.affinity(), vec![w, w]);
    }

    #[test]
    fn communities_cover_every_node_and_are_stable() {
        let g = UltraGraph::from_seed("sanctuary", 16, 0.5);
        let c1 = g.community_fold(8);
        let c2 = g.community_fold(8);
        assert_eq!(c1, c2);
        assert_eq!(c1.len(), 16);
        assert!(c1.iter().all(|&l| l < 16), "renumbered to a compact range");
    }

    #[test]
    fn degree_centrality_counts_friends_and_foes() {
        // p_edge 1.0 on two nodes: one bond, tri-state drawn
        let g = UltraGraph::from_seed("pair", 2, 1.0);
        let w = g.edges[0].2;
        let dc = g.degree_centrality();
        let expected = if w == 1 {
            (1u32, 0u32)
        } else if w == -1 {
            (0u32, 1u32)
        } else {
            (0u32, 0u32)
        };
        assert_eq!(dc, vec![expected, expected]);
    }

    #[test]
    fn closeness_is_bfs_deterministic() {
        let a = UltraGraph::from_seed("sanctuary", 12, 0.5);
        let b = UltraGraph::from_seed("sanctuary", 12, 0.5);
        let c1 = a.closeness();
        let c2 = b.closeness();
        assert_eq!(c1, c2, "the same seed, the same closeness");
        // a friendly chain of three: endpoints sum 1+2=3 → 1/3, the
        // center sums 1+1=2 → 1/2 (build a deterministic friendly line)
        // closeness lives in (0, 1]: 1/sum, sum ≥ 1 for anyone with a
        // reachable self; a lone shard-mate reads 0.0 (nobody else in
        // the room)
        for seed in ["sanctuary", "the tent", "the yard"] {
            let g = UltraGraph::from_seed(seed, 9, 0.6);
            let cc = g.closeness();
            assert!(cc.iter().all(|&c| c >= 0.0 && c <= 1.0), "{seed}: in range");
            assert_eq!(cc.len(), 9);
        }
        // and the pair from the degree test: a friendly single bond
        // gives both members closeness 1/1 = 1.0
        let g = UltraGraph::from_seed("pair", 2, 1.0);
        match g.edges[0].2 {
            1 => assert_eq!(g.closeness(), vec![1.0, 1.0]),
            -1 | 0 => {
                // no friendly bond: two lone shards, each 0.0
                assert_eq!(g.closeness(), vec![0.0, 0.0])
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_svg_is_a_tagged_render() {
        let g = UltraGraph::from_seed("sanctuary", 9, 0.4);
        let svg = g.ultra_svg("the fold");
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("the fold"));
        assert!(svg.contains("</svg>"));
        // every node is drawn — 9 circles
        assert_eq!(svg.matches("<circle").count(), 9);
        // every edge is drawn
        assert_eq!(svg.matches("<line").count(), g.edges.len());
    }
}
