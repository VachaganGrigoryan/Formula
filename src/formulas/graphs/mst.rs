//! A weighted graph (seeded points joined by Delaunay edges) and recorded
//! Kruskal and Prim minimum-spanning-tree runs.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use egui::{Rect, pos2};

use crate::formulas::replay::Replay;
use crate::formulas::voronoi;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub a: usize,
    pub b: usize,
    /// Euclidean length times 100 (points live in the unit square).
    pub w: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Graph {
    points: Vec<[f32; 2]>,
    edges: Vec<Edge>,
    generation: u64,
    /// Seed of the last generated point set, shared by every viewport so each
    /// press of New points gives a new graph.
    seed: u64,
}

/// The most points a graph keeps when loaded from a sheet.
pub const MAX_POINTS: usize = 64;

impl Graph {
    pub fn random(n: usize, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut points: Vec<[f32; 2]> = Vec::new();
        let mut tries = 0;
        while points.len() < n && tries < 1000 {
            tries += 1;
            let p = [rng.range(0.08, 0.92), rng.range(0.08, 0.92)];
            let clear = points
                .iter()
                .all(|q| (q[0] - p[0]).hypot(q[1] - p[1]) >= 0.08);
            if clear {
                points.push(p);
            }
        }
        let edges = delaunay(&points);
        Self {
            points,
            edges,
            generation: 0,
            seed,
        }
    }

    pub fn points(&self) -> &[[f32; 2]] {
        &self.points
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Replaces the graph with a fresh random one. The generation keeps
    /// counting up, so viewports sharing this graph notice and re-record.
    pub fn regenerate(&mut self, n: usize, seed: u64) {
        let generation = self.generation + 1;
        *self = Graph::random(n, seed);
        self.generation = generation;
    }

    /// Regenerates from the next seed in this graph's own sequence.
    pub fn regenerate_next(&mut self, n: usize) {
        self.regenerate(n, self.seed.wrapping_add(1));
    }

    /// The seed of the last generated point set.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Replace the points (e.g. from a sheet): non-finite points are dropped,
    /// the rest are kept in the square, and at most `MAX_POINTS` are kept.
    pub fn set_points(&mut self, points: &[[f32; 2]], seed: u64) {
        let points: Vec<[f32; 2]> = points
            .iter()
            .filter(|p| p[0].is_finite() && p[1].is_finite())
            .take(MAX_POINTS)
            .map(|p| [p[0].clamp(0.02, 0.98), p[1].clamp(0.02, 0.98)])
            .collect();
        self.edges = delaunay(&points);
        self.points = points;
        self.seed = seed;
        self.generation += 1;
    }

    /// Moves point `i`, keeping it in the square and off every other point.
    pub fn move_point(&mut self, i: usize, p: [f32; 2]) {
        let mut p = [p[0].clamp(0.02, 0.98), p[1].clamp(0.02, 0.98)];
        let clear = |pts: &[[f32; 2]], p: [f32; 2]| {
            pts.iter()
                .enumerate()
                .all(|(j, q)| j == i || (q[0] - p[0]).hypot(q[1] - p[1]) >= 0.01)
        };
        // Walk right along the row, wrapping at the edge; if the whole row is
        // blocked, move on to the next row. Bounded: 41 rows of 81 steps.
        let start = p;
        'search: for row in 0..=40 {
            p[1] = start[1] + 0.012 * row as f32;
            if p[1] > 0.98 {
                p[1] = 0.02 + (p[1] - 0.98);
            }
            p[0] = start[0];
            for _ in 0..=80 {
                if clear(&self.points, p) {
                    break 'search;
                }
                p[0] += 0.012;
                if p[0] > 0.98 {
                    p[0] = 0.02;
                }
            }
        }
        self.points[i] = p;
        self.edges = delaunay(&self.points);
        self.generation += 1;
    }
}

fn delaunay(points: &[[f32; 2]]) -> Vec<Edge> {
    let seeds: Vec<egui::Pos2> = points.iter().map(|p| pos2(p[0], p[1])).collect();
    let bounds = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..seeds.len() {
        let poly = voronoi::cell(&seeds, i, bounds, 0.0);
        for j in voronoi::neighbors(&poly) {
            if j != i {
                pairs.push((i.min(j), i.max(j)));
            }
        }
    }
    pairs.sort_unstable();
    pairs.dedup();
    pairs
        .into_iter()
        .map(|(a, b)| Edge {
            a,
            b,
            w: (points[a][0] - points[b][0]).hypot(points[a][1] - points[b][1]) * 100.0,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spanning {
    Kruskal,
    Prim,
}

impl Spanning {
    pub const ALL: [Spanning; 2] = [Spanning::Kruskal, Spanning::Prim];

    pub fn id(self) -> &'static str {
        match self {
            Spanning::Kruskal => "kruskal",
            Spanning::Prim => "prim",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Spanning::Kruskal => "Kruskal's algorithm",
            Spanning::Prim => "Prim's algorithm",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MstOp {
    Consider(usize),
    Accept(usize),
    Reject(usize),
    Union {
        child: usize,
        parent: usize,
    },
    /// A vertex joins Prim's tree.
    Reach(usize),
}

pub fn record(alg: Spanning, g: &Graph) -> Vec<MstOp> {
    match alg {
        Spanning::Kruskal => kruskal(g),
        Spanning::Prim => prim(g),
    }
}

fn kruskal(g: &Graph) -> Vec<MstOp> {
    let n = g.points.len();
    let mut order: Vec<usize> = (0..g.edges.len()).collect();
    order.sort_by(|&x, &y| g.edges[x].w.total_cmp(&g.edges[y].w).then(x.cmp(&y)));
    let mut parent: Vec<usize> = (0..n).collect();
    let mut size = vec![1usize; n];
    let find = |parent: &[usize], mut x: usize| {
        while parent[x] != x {
            x = parent[x];
        }
        x
    };
    let mut ops = Vec::new();
    let mut taken = 0;
    for e in order {
        if taken + 1 >= n {
            break;
        }
        ops.push(MstOp::Consider(e));
        let (ra, rb) = (find(&parent, g.edges[e].a), find(&parent, g.edges[e].b));
        if ra == rb {
            ops.push(MstOp::Reject(e));
            continue;
        }
        ops.push(MstOp::Accept(e));
        taken += 1;
        let (child, root) = if size[ra] < size[rb] {
            (ra, rb)
        } else {
            (rb, ra)
        };
        parent[child] = root;
        size[root] += size[child];
        ops.push(MstOp::Union {
            child,
            parent: root,
        });
    }
    ops
}

fn prim(g: &Graph) -> Vec<MstOp> {
    let n = g.points.len();
    let mut ops = Vec::new();
    if n == 0 {
        return ops;
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, e) in g.edges.iter().enumerate() {
        adj[e.a].push(i);
        adj[e.b].push(i);
    }
    let mut in_tree = vec![false; n];
    let mut heap: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::new();
    in_tree[0] = true;
    ops.push(MstOp::Reach(0));
    for &e in &adj[0] {
        heap.push(Reverse((g.edges[e].w.to_bits(), e)));
    }
    while let Some(Reverse((_, e))) = heap.pop() {
        ops.push(MstOp::Consider(e));
        let Edge { a, b, .. } = g.edges[e];
        if in_tree[a] && in_tree[b] {
            ops.push(MstOp::Reject(e));
            continue;
        }
        let v = if in_tree[a] { b } else { a };
        ops.push(MstOp::Accept(e));
        in_tree[v] = true;
        ops.push(MstOp::Reach(v));
        for &f in &adj[v] {
            let other = if g.edges[f].a == v {
                g.edges[f].b
            } else {
                g.edges[f].a
            };
            if !in_tree[other] {
                heap.push(Reverse((g.edges[f].w.to_bits(), f)));
            }
        }
    }
    ops
}

pub fn accepted(ops: &[MstOp]) -> Vec<usize> {
    ops.iter()
        .filter_map(|op| match *op {
            MstOp::Accept(e) => Some(e),
            _ => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeState {
    Unseen,
    Considering,
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MstState {
    pub edges: Vec<EdgeState>,
    pub parent: Vec<usize>,
    pub in_tree: Vec<bool>,
    pub weight: f32,
    pub accepted: usize,
    pub last: Option<MstOp>,
    lengths: Vec<f32>,
}

impl MstState {
    pub fn new(g: &Graph) -> Self {
        let n = g.points.len();
        Self {
            edges: vec![EdgeState::Unseen; g.edges.len()],
            parent: (0..n).collect(),
            in_tree: vec![false; n],
            weight: 0.0,
            accepted: 0,
            last: None,
            lengths: g.edges.iter().map(|e| e.w).collect(),
        }
    }
}

impl Replay for MstState {
    type Op = MstOp;

    fn apply(&mut self, op: MstOp) {
        match op {
            MstOp::Consider(e) => self.edges[e] = EdgeState::Considering,
            MstOp::Accept(e) => {
                self.edges[e] = EdgeState::Accepted;
                self.weight += self.lengths[e];
                self.accepted += 1;
            }
            MstOp::Reject(e) => self.edges[e] = EdgeState::Rejected,
            MstOp::Union { child, parent } => self.parent[child] = parent,
            MstOp::Reach(v) => self.in_tree[v] = true,
        }
        self.last = Some(op);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regenerating_bumps_the_generation_and_replaces_the_points() {
        let mut g = Graph::random(14, 7);
        g.move_point(0, [0.5, 0.5]);
        let before = g.generation();
        g.regenerate(9, 3);
        assert_eq!(g.generation(), before + 1);
        assert_eq!(g.points().len(), 9);
        assert_eq!(g.points(), Graph::random(9, 3).points());
    }

    fn weight(g: &Graph, edges: &[usize]) -> f32 {
        edges.iter().map(|&e| g.edges()[e].w).sum()
    }

    fn spans(g: &Graph, edges: &[usize]) -> bool {
        let n = g.points().len();
        let mut root: Vec<usize> = (0..n).collect();
        fn find(r: &mut Vec<usize>, x: usize) -> usize {
            if r[x] == x {
                x
            } else {
                let p = r[x];
                let q = find(r, p);
                r[x] = q;
                q
            }
        }
        for &e in edges {
            let (a, b) = (
                find(&mut root, g.edges()[e].a),
                find(&mut root, g.edges()[e].b),
            );
            root[a] = b;
        }
        let r0 = find(&mut root, 0);
        (0..n).all(|v| find(&mut root, v) == r0)
    }

    #[test]
    fn delaunay_edges_are_finite_unique_and_connect_every_point() {
        let g = Graph::random(14, 7);
        assert!(g.points().len() >= 10);
        assert!(
            g.edges()
                .iter()
                .all(|e| e.a < e.b && e.w.is_finite() && e.w > 0.0)
        );
        let mut pairs: Vec<_> = g.edges().iter().map(|e| (e.a, e.b)).collect();
        pairs.dedup();
        assert_eq!(pairs.len(), g.edges().len());
        let all: Vec<usize> = (0..g.edges().len()).collect();
        assert!(spans(&g, &all));
    }

    #[test]
    fn kruskal_and_prim_find_the_same_minimum_weight() {
        for seed in 0..100 {
            let g = Graph::random(12, seed);
            let k = accepted(&record(Spanning::Kruskal, &g));
            let p = accepted(&record(Spanning::Prim, &g));
            let n = g.points().len();
            assert_eq!((k.len(), p.len()), (n - 1, n - 1), "seed {seed}");
            assert!(spans(&g, &k) && spans(&g, &p), "seed {seed}");
            assert!(
                (weight(&g, &k) - weight(&g, &p)).abs() < 1e-3,
                "seed {seed}"
            );
        }
    }

    #[test]
    fn small_graphs_match_brute_force() {
        for seed in 0..30 {
            let g = Graph::random(6, seed);
            let m = g.edges().len();
            let n = g.points().len();
            let mut best = f32::INFINITY;
            for mask in 0u32..(1 << m) {
                if mask.count_ones() as usize != n - 1 {
                    continue;
                }
                let set: Vec<usize> = (0..m).filter(|i| mask >> i & 1 == 1).collect();
                if spans(&g, &set) {
                    best = best.min(weight(&g, &set));
                }
            }
            let k = accepted(&record(Spanning::Kruskal, &g));
            assert!((weight(&g, &k) - best).abs() < 1e-3, "seed {seed}");
        }
    }

    /// Review Focus 4: dragging a point onto another, or lining points up,
    /// keeps the triangulation finite and the MST spanning.
    #[test]
    fn coincident_and_collinear_points_still_give_a_spanning_tree() {
        let mut g = Graph::random(8, 3);
        let target = g.points()[1];
        g.move_point(0, target);
        assert!(g.points()[0] != g.points()[1]);
        for i in 0..g.points().len() {
            g.move_point(i, [0.1 + 0.1 * i as f32, 0.5]);
        }
        let k = accepted(&record(Spanning::Kruskal, &g));
        assert_eq!(k.len(), g.points().len() - 1);
        assert!(spans(&g, &k));
        assert!(g.edges().iter().all(|e| e.w.is_finite()));
    }

    #[test]
    fn replay_accumulates_the_tree_weight() {
        let g = Graph::random(12, 5);
        let ops = record(Spanning::Prim, &g);
        let mut st = MstState::new(&g);
        for &op in &ops {
            crate::formulas::replay::Replay::apply(&mut st, op);
        }
        let k = accepted(&ops);
        assert_eq!(st.accepted, k.len());
        assert!((st.weight - weight(&g, &k)).abs() < 1e-3);
        assert!(st.in_tree.iter().all(|&t| t));
    }

    fn all_apart(g: &Graph) -> bool {
        let p = g.points();
        (0..p.len()).all(|i| (0..i).all(|j| (p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]) >= 0.01))
    }

    #[test]
    fn dragging_onto_a_crowded_row_never_leaves_points_coincident() {
        let mut g = Graph::random(14, 7);
        let n = g.points().len();
        // Pack every point but the last into one tight row, so a nudge has to
        // walk past all of it.
        for i in 0..n - 1 {
            g.move_point(i, [0.5 + 0.015 * i as f32, 0.5]);
            assert!(all_apart(&g), "placing {i}");
        }
        g.move_point(n - 1, [0.5, 0.5]);
        assert!(all_apart(&g), "first drop");
        for round in 0..8 {
            for k in 0..n {
                let target = if round % 2 == 0 {
                    [0.5, 0.5]
                } else {
                    g.points()[(k + 3) % n]
                };
                g.move_point(k, target);
                assert!(all_apart(&g), "round {round} point {k}");
                assert!(g.edges().iter().all(|e| e.w.is_finite() && e.w > 0.0));
            }
        }
    }

    #[test]
    fn dragging_onto_a_row_at_the_right_edge_wraps_to_the_left() {
        let mut g = Graph::random(14, 7);
        let n = g.points().len();
        // A row 0.015 apart ending at the right edge: the walk from its start
        // runs off x = 0.98 and has to wrap to 0.02.
        for i in 0..n - 1 {
            g.move_point(i, [0.98 - 0.015 * i as f32, 0.5]);
        }
        let start = 0.98 - 0.015 * (n - 2) as f32;
        g.move_point(n - 1, [start, 0.5]);
        let p = g.points()[n - 1];
        assert!(p[0] < start, "wrapped to the left of the row: {p:?}");
        for q in g.points() {
            assert!((0.02..=0.98).contains(&q[0]) && (0.02..=0.98).contains(&q[1]));
        }
        assert!(all_apart(&g));
        assert!(g.edges().iter().all(|e| e.w.is_finite() && e.w > 0.0));
    }
}
