//! A* over the tile grid. Buffers are reused between searches and reset in
//! O(1) with a generation stamp, so per-call cost is only the nodes it touches.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::{flag, World};

#[derive(Copy, Clone, PartialEq)]
struct Node {
    f: f32,
    idx: u32,
}
impl Eq for Node {}
impl Ord for Node {
    fn cmp(&self, o: &Self) -> Ordering {
        o.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal).then_with(|| o.idx.cmp(&self.idx))
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

pub struct PathFinder {
    w: i32,
    h: i32,
    g: Vec<f32>,
    parent: Vec<u32>,
    stamp: Vec<u32>,
    closed: Vec<u32>,
    epoch: u32,
    heap: BinaryHeap<Node>,
}

const DIRS: [(i32, i32, f32); 8] = [
    (1, 0, 1.0), (-1, 0, 1.0), (0, 1, 1.0), (0, -1, 1.0),
    (1, 1, 1.414), (-1, 1, 1.414), (1, -1, 1.414), (-1, -1, 1.414),
];

impl PathFinder {
    pub fn new(w: i32, h: i32) -> Self {
        let n = (w * h) as usize;
        PathFinder {
            w,
            h,
            g: vec![0.0; n],
            parent: vec![0; n],
            stamp: vec![0; n],
            closed: vec![0; n],
            epoch: 0,
            heap: BinaryHeap::with_capacity(1024),
        }
    }

    /// Generic search. `step(ax, ay, bx, by)` returns the cost of moving from
    /// a to b, or None if impassable. `diagonal` enables 8-way movement.
    pub fn search(
        &mut self,
        start: (i32, i32),
        goal: (i32, i32),
        max_nodes: usize,
        diagonal: bool,
        mut step: impl FnMut(i32, i32, i32, i32) -> Option<f32>,
    ) -> Option<Vec<(i32, i32)>> {
        if start == goal {
            return Some(vec![start]);
        }
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.stamp.fill(0);
            self.closed.fill(0);
            self.epoch = 1;
        }
        let epoch = self.epoch;
        self.heap.clear();
        let w = self.w;
        let id = |x: i32, y: i32| (y * w + x) as usize;
        let heur = |x: i32, y: i32| {
            let dx = (x - goal.0).abs() as f32;
            let dy = (y - goal.1).abs() as f32;
            // Octile distance; the small inflation trades optimality for speed.
            (dx + dy + (1.414 - 2.0) * dx.min(dy)) * 1.02
        };
        let s = id(start.0, start.1);
        self.g[s] = 0.0;
        self.stamp[s] = epoch;
        self.parent[s] = s as u32;
        self.heap.push(Node { f: heur(start.0, start.1), idx: s as u32 });
        let ndirs = if diagonal { 8 } else { 4 };
        let mut expanded = 0;
        while let Some(Node { idx, .. }) = self.heap.pop() {
            let i = idx as usize;
            if self.closed[i] == epoch {
                continue;
            }
            self.closed[i] = epoch;
            let x = (i as i32) % w;
            let y = (i as i32) / w;
            if (x, y) == goal {
                let mut out = vec![(x, y)];
                let mut c = i;
                while c != s {
                    c = self.parent[c] as usize;
                    out.push(((c as i32) % w, (c as i32) / w));
                }
                out.reverse();
                return Some(out);
            }
            expanded += 1;
            if expanded > max_nodes {
                return None;
            }
            let gi = self.g[i];
            for &(dx, dy, base) in &DIRS[..ndirs] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= self.w || ny >= self.h {
                    continue;
                }
                let ni = id(nx, ny);
                if self.closed[ni] == epoch {
                    continue;
                }
                let Some(c) = step(x, y, nx, ny) else { continue };
                let ng = gi + c * base;
                if self.stamp[ni] != epoch || ng < self.g[ni] {
                    self.stamp[ni] = epoch;
                    self.g[ni] = ng;
                    self.parent[ni] = i as u32;
                    self.heap.push(Node { f: ng + heur(nx, ny), idx: ni as u32 });
                }
            }
        }
        None
    }

    /// Path for walking agents: respects walls, levels and stairs, and
    /// prefers roads.
    pub fn walk(&mut self, world: &World, start: (i32, i32), goal: (i32, i32), max_nodes: usize) -> Option<Vec<(i32, i32)>> {
        self.search(start, goal, max_nodes, true, |ax, ay, bx, by| {
            if !world.can_step(ax, ay, bx, by) {
                return None;
            }
            let f = world.flags(bx, by);
            Some(if f & (flag::ROAD | flag::PLAZA | flag::BRIDGE) != 0 { 0.55 } else { 1.0 })
        })
    }
}

/// Simplify a tile path into fewer waypoints by dropping collinear points.
pub fn simplify(path: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if path.len() <= 2 {
        return path.to_vec();
    }
    let mut out = vec![path[0]];
    for k in 1..path.len() - 1 {
        let (a, b, c) = (path[k - 1], path[k], path[k + 1]);
        if (b.0 - a.0, b.1 - a.1) != (c.0 - b.0, c.1 - b.1) {
            out.push(b);
        }
    }
    out.push(*path.last().unwrap());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_path_around_wall() {
        let mut pf = PathFinder::new(10, 10);
        // Wall at x=5 except y=9.
        let p = pf
            .search((0, 0), (9, 0), 10_000, false, |_, _, bx, by| if bx == 5 && by < 9 { None } else { Some(1.0) })
            .expect("path");
        assert_eq!(p.first(), Some(&(0, 0)));
        assert_eq!(p.last(), Some(&(9, 0)));
        assert!(p.iter().all(|&(x, y)| !(x == 5 && y < 9)));
        // Must detour down to y=9 and back up: 9 + 9 + 9 steps + start.
        assert_eq!(p.len(), 28);
    }

    #[test]
    fn no_path_when_sealed() {
        let mut pf = PathFinder::new(10, 10);
        let p = pf.search((0, 0), (9, 0), 10_000, true, |_, _, bx, _| if bx == 5 { None } else { Some(1.0) });
        assert!(p.is_none());
    }

    #[test]
    fn repeated_searches_are_independent() {
        let mut pf = PathFinder::new(20, 20);
        for _ in 0..50 {
            let p = pf.search((0, 0), (19, 19), 100_000, true, |_, _, _, _| Some(1.0)).unwrap();
            assert_eq!(p.len(), 20);
        }
    }
}
