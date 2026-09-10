// entity.rs — the Rust core's first data structure: the sparse-set entity
// registry.
//
// Archetype-aligned component columns (position, velocity, needs) in
// cache-friendly parallel arrays behind a compact sparse set — the frame
// arena the sim's fauna and the mesh's actors will live in. Kill is a
// swap-remove: constant time, no holes, dense iteration.

/// A dead-simple entity handle: an index, nothing more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

/// The registry: sparse set of live entities + component columns.
#[derive(Debug, Default)]
pub struct Entities {
    sparse: Vec<u32>, // entity id + 1 → dense index (0 = absent)
    dense: Vec<u32>,  // dense index → entity id
    pos: Vec<[f64; 2]>,
    vel: Vec<[f64; 2]>,
    needs: Vec<[f64; 3]>,
    next: u32,
}

impl Entities {
    /// spawn — a new live entity, born comfortable.
    pub fn spawn(&mut self) -> Entity {
        let id = self.next;
        self.next += 1;
        if id as usize >= self.sparse.len() {
            self.sparse.resize(id as usize + 1, 0);
        }
        let idx = self.dense.len();
        self.sparse[id as usize] = idx as u32 + 1;
        self.dense.push(id);
        self.pos.push([0.0, 0.0]);
        self.vel.push([0.0, 0.0]);
        self.needs.push([0.9, 0.9, 0.95]);
        Entity(id)
    }

    pub fn alive(&self, e: Entity) -> bool {
        (e.0 as usize) < self.sparse.len() && self.sparse[e.0 as usize] != 0
    }

    fn idx(&self, e: Entity) -> Option<usize> {
        if self.alive(e) {
            Some(self.sparse[e.0 as usize] as usize - 1)
        } else {
            None
        }
    }

    /// kill — swap-remove: the last entity slides into the freed slot, so
    /// the columns stay dense and iteration stays cache-friendly.
    pub fn kill(&mut self, e: Entity) {
        let Some(idx) = self.idx(e) else { return };
        let last = self.dense.len() - 1;
        if idx != last {
            let moved = self.dense[last];
            self.dense[idx] = moved;
            self.pos[idx] = self.pos[last];
            self.vel[idx] = self.vel[last];
            self.needs[idx] = self.needs[last];
            self.sparse[moved as usize] = idx as u32 + 1;
        }
        self.dense.pop();
        self.pos.pop();
        self.vel.pop();
        self.needs.pop();
        self.sparse[e.0 as usize] = 0;
    }

    pub fn pos(&self, e: Entity) -> Option<[f64; 2]> {
        self.idx(e).map(|i| self.pos[i])
    }
    pub fn set_pos(&mut self, e: Entity, p: [f64; 2]) {
        if let Some(i) = self.idx(e) {
            self.pos[i] = p;
        }
    }
    pub fn vel(&self, e: Entity) -> Option<[f64; 2]> {
        self.idx(e).map(|i| self.vel[i])
    }
    pub fn set_vel(&mut self, e: Entity, v: [f64; 2]) {
        if let Some(i) = self.idx(e) {
            self.vel[i] = v;
        }
    }
    pub fn needs(&self, e: Entity) -> Option<[f64; 3]> {
        self.idx(e).map(|i| self.needs[i])
    }
    pub fn set_needs(&mut self, e: Entity, n: [f64; 3]) {
        if let Some(i) = self.idx(e) {
            self.needs[i] = n;
        }
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.dense.iter().map(|&id| Entity(id))
    }

    /// columns — the archetype-aligned bulk view the tick loop passes over
    /// (ids + position + velocity + needs in lockstep).
    pub fn columns(
        &mut self,
    ) -> (
        &mut Vec<u32>,
        &mut Vec<[f64; 2]>,
        &mut Vec<[f64; 2]>,
        &mut Vec<[f64; 3]>,
    ) {
        (
            &mut self.dense,
            &mut self.pos,
            &mut self.vel,
            &mut self.needs,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_components_round_trip() {
        let mut es = Entities::default();
        let e = es.spawn();
        assert!(es.alive(e));
        assert_eq!(es.needs(e), Some([0.9, 0.9, 0.95]), "born comfortable");
        es.set_pos(e, [1.0, 2.0]);
        es.set_vel(e, [3.0, 4.0]);
        es.set_needs(e, [0.5, 0.8, 0.7]);
        assert_eq!(es.pos(e), Some([1.0, 2.0]));
        assert_eq!(es.vel(e), Some([3.0, 4.0]));
        assert_eq!(es.needs(e), Some([0.5, 0.8, 0.7]));
    }

    #[test]
    fn kill_is_a_swap_remove() {
        let mut es = Entities::default();
        let a = es.spawn();
        let b = es.spawn();
        let c = es.spawn();
        es.set_pos(a, [1.0, 1.0]);
        es.set_pos(b, [2.0, 2.0]);
        es.set_pos(c, [3.0, 3.0]);
        es.kill(b); // the middle one — c slides into its slot
        assert!(!es.alive(b));
        assert!(es.alive(a) && es.alive(c));
        assert_eq!(es.len(), 2);
        // no holes: the component columns stay dense and consistent
        assert_eq!(es.pos(a), Some([1.0, 1.0]));
        assert_eq!(es.pos(c), Some([3.0, 3.0]));
        assert_eq!(es.iter().count(), 2);
    }

    #[test]
    fn columns_run_in_lockstep() {
        let mut es = Entities::default();
        for i in 0..4u32 {
            let e = es.spawn();
            es.set_pos(e, [i as f64, 0.0]);
            es.set_vel(e, [1.0, 1.0]);
            es.set_needs(e, [0.9, 0.9 - i as f64 * 0.1, 0.9]);
        }
        let (ids, pos, vel, needs) = es.columns();
        assert_eq!(ids.len(), 4);
        assert_eq!(pos[2], [2.0, 0.0]);
        assert_eq!(vel[3], [1.0, 1.0]);
        assert_eq!(needs[1], [0.9, 0.8, 0.9]);
    }
}
