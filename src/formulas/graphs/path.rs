//! Grid path searches (BFS, DFS, Dijkstra, A*) that record their work as an
//! operation log, which a viewport later replays step by step.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

use super::grid::Grid;
use crate::formulas::replay::Replay;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    Bfs,
    Dfs,
    Dijkstra,
    AStar,
}

impl Search {
    pub const ALL: [Search; 4] = [Search::Bfs, Search::Dfs, Search::Dijkstra, Search::AStar];

    pub fn id(self) -> &'static str {
        match self {
            Search::Bfs => "bfs",
            Search::Dfs => "dfs",
            Search::Dijkstra => "dijkstra",
            Search::AStar => "a-star",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Search::Bfs => "Breadth-first search",
            Search::Dfs => "Depth-first search",
            Search::Dijkstra => "Dijkstra's algorithm",
            Search::AStar => "A* search",
        }
    }

    /// Whether the search accounts for cell weights (mud).
    pub fn weighted(self) -> bool {
        matches!(self, Search::Dijkstra | Search::AStar)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathOp {
    Frontier { cell: usize, dist: u32 },
    Visit(usize),
    Path(usize),
    NoPath,
}

const UNSEEN: u32 = u32::MAX;

fn manhattan(grid: &Grid, a: usize, b: usize) -> u32 {
    let ((ax, ay), (bx, by)) = (grid.xy(a), grid.xy(b));
    (ax.abs_diff(bx) + ay.abs_diff(by)) as u32
}

/// Run `search` from the grid's start to its goal and log every step.
pub fn record(search: Search, grid: &Grid) -> Vec<PathOp> {
    let (start, goal) = (grid.start(), grid.goal());
    let mut ops = Vec::new();
    let mut parent: Vec<Option<usize>> = vec![None; grid.len()];
    let mut dist = vec![UNSEEN; grid.len()];
    let mut visited = vec![false; grid.len()];
    let mut found = false;

    dist[start] = 0;
    ops.push(PathOp::Frontier {
        cell: start,
        dist: 0,
    });

    match search {
        Search::Bfs => {
            let mut queue = VecDeque::from([start]);
            while let Some(c) = queue.pop_front() {
                visited[c] = true;
                ops.push(PathOp::Visit(c));
                if c == goal {
                    found = true;
                    break;
                }
                for n in grid.neighbors(c) {
                    if dist[n] == UNSEEN {
                        dist[n] = dist[c] + 1;
                        parent[n] = Some(c);
                        ops.push(PathOp::Frontier {
                            cell: n,
                            dist: dist[n],
                        });
                        queue.push_back(n);
                    }
                }
            }
        }
        Search::Dfs => {
            let mut stack = vec![(start, None, 0u32)];
            while let Some((c, from, d)) = stack.pop() {
                if visited[c] {
                    continue;
                }
                visited[c] = true;
                parent[c] = from;
                dist[c] = d;
                ops.push(PathOp::Visit(c));
                if c == goal {
                    found = true;
                    break;
                }
                // Reverse order so "up" is explored first, except that a goal
                // next door is always taken at once (pushed last, popped first).
                let mut next = grid.neighbors(c);
                next.reverse();
                next.sort_by_key(|&n| n == goal);
                for n in next {
                    if !visited[n] {
                        ops.push(PathOp::Frontier {
                            cell: n,
                            dist: d + 1,
                        });
                        stack.push((n, Some(c), d + 1));
                    }
                }
            }
        }
        Search::Dijkstra | Search::AStar => {
            let a_star = search == Search::AStar;
            let h = |n: usize| if a_star { manhattan(grid, n, goal) } else { 0 };
            let mut heap = BinaryHeap::new();
            heap.push(Reverse((h(start), h(start), start)));
            while let Some(Reverse((f, hc, c))) = heap.pop() {
                if visited[c] || f != dist[c] + hc {
                    continue;
                }
                visited[c] = true;
                ops.push(PathOp::Visit(c));
                if c == goal {
                    found = true;
                    break;
                }
                for n in grid.neighbors(c) {
                    let nd = dist[c] + u32::from(grid.weight(n));
                    if nd < dist[n] {
                        dist[n] = nd;
                        parent[n] = Some(c);
                        ops.push(PathOp::Frontier { cell: n, dist: nd });
                        heap.push(Reverse((nd + h(n), h(n), n)));
                    }
                }
            }
        }
    }

    if found {
        let mut path = vec![goal];
        while let Some(p) = parent[*path.last().expect("path is never empty")] {
            path.push(p);
        }
        ops.extend(path.into_iter().rev().map(PathOp::Path));
    } else {
        ops.push(PathOp::NoPath);
    }
    ops
}

/// Moves for BFS/DFS; the summed weight of every cell entered after the
/// start for Dijkstra and A*.
pub fn path_cost(grid: &Grid, search: Search, path: &[usize]) -> u32 {
    let entered = path.iter().skip(1);
    if search.weighted() {
        entered.map(|&c| u32::from(grid.weight(c))).sum()
    } else {
        entered.count() as u32
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathState {
    pub dist: Vec<Option<u32>>,
    pub frontier: Vec<bool>,
    pub visited: Vec<bool>,
    pub path: Vec<usize>,
    pub no_path: bool,
    pub visits: usize,
    pub last: Option<PathOp>,
}

impl PathState {
    pub fn new(cells: usize) -> Self {
        Self {
            dist: vec![None; cells],
            frontier: vec![false; cells],
            visited: vec![false; cells],
            path: Vec::new(),
            no_path: false,
            visits: 0,
            last: None,
        }
    }
}

impl Replay for PathState {
    type Op = PathOp;

    fn apply(&mut self, op: PathOp) {
        match op {
            PathOp::Frontier { cell, dist } => {
                self.frontier[cell] = true;
                self.dist[cell] = Some(dist);
            }
            PathOp::Visit(cell) => {
                self.frontier[cell] = false;
                self.visited[cell] = true;
                self.visits += 1;
            }
            PathOp::Path(cell) => self.path.push(cell),
            PathOp::NoPath => self.no_path = true,
        }
        self.last = Some(op);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas::graphs::grid::{Grid, MUD};
    use crate::formulas::replay::Replay;
    use crate::rng::Rng;

    fn path_of(ops: &[PathOp]) -> Vec<usize> {
        ops.iter()
            .filter_map(|op| match op {
                PathOp::Path(c) => Some(*c),
                _ => None,
            })
            .collect()
    }

    fn valid(g: &Grid, path: &[usize]) -> bool {
        path.first() == Some(&g.start())
            && path.last() == Some(&g.goal())
            && path.iter().all(|&c| !g.is_wall(c))
            && path.windows(2).all(|w| g.neighbors(w[0]).contains(&w[1]))
    }

    #[test]
    fn bfs_finds_a_manhattan_shortest_path_on_an_open_grid() {
        let g = Grid::new(11, 7);
        let path = path_of(&record(Search::Bfs, &g));
        let ((sx, sy), (tx, ty)) = (g.xy(g.start()), g.xy(g.goal()));
        assert!(valid(&g, &path));
        assert_eq!(path.len() - 1, sx.abs_diff(tx) + sy.abs_diff(ty));
    }

    #[test]
    fn a_walled_off_goal_ends_in_no_path() {
        let mut g = Grid::new(11, 7);
        for n in g.neighbors(g.goal()) {
            g.set_wall(n, true);
        }
        for s in Search::ALL {
            let ops = record(s, &g);
            assert_eq!(ops.last(), Some(&PathOp::NoPath), "{s:?}");
            assert!(path_of(&ops).is_empty());
        }
    }

    /// Review Focus 1: the markers can never share a cell, so the shortest
    /// possible search is between adjacent markers.
    #[test]
    fn adjacent_markers_give_a_two_cell_path() {
        let g = Grid::new(4, 3);
        assert_eq!((g.xy(g.start()), g.xy(g.goal())), ((1, 1), (2, 1)));
        for s in Search::ALL {
            assert_eq!(path_of(&record(s, &g)), vec![g.start(), g.goal()], "{s:?}");
        }
    }

    #[test]
    fn dfs_finds_a_valid_path_through_a_maze() {
        let mut g = Grid::new(31, 19);
        g.maze(4);
        let path = path_of(&record(Search::Dfs, &g));
        assert!(valid(&g, &path));
    }

    #[test]
    fn dijkstra_and_a_star_agree_on_the_cost_of_weighted_grids() {
        for seed in 0..100 {
            let mut g = Grid::new(21, 13);
            g.scatter(seed, 0.25);
            let mut rng = Rng::new(seed + 1000);
            for i in 0..g.len() {
                if rng.f32() < 0.2 {
                    g.set_weight(i, MUD);
                }
            }
            let d = path_of(&record(Search::Dijkstra, &g));
            let a = path_of(&record(Search::AStar, &g));
            assert_eq!(d.is_empty(), a.is_empty(), "seed {seed}");
            if !d.is_empty() {
                assert!(valid(&g, &d) && valid(&g, &a), "seed {seed}");
                assert_eq!(
                    path_cost(&g, Search::Dijkstra, &d),
                    path_cost(&g, Search::AStar, &a),
                    "seed {seed}"
                );
            }
        }
    }

    #[test]
    fn a_star_visits_no_more_cells_than_dijkstra_on_an_open_grid() {
        let g = Grid::new(31, 19);
        let visits = |s| {
            record(s, &g)
                .iter()
                .filter(|op| matches!(op, PathOp::Visit(_)))
                .count()
        };
        assert!(visits(Search::AStar) < visits(Search::Dijkstra));
    }

    #[test]
    fn replaying_the_log_rebuilds_the_final_state() {
        let mut g = Grid::new(31, 19);
        g.maze(8);
        let ops = record(Search::Bfs, &g);
        let mut st = PathState::new(g.len());
        for &op in &ops {
            st.apply(op);
        }
        assert_eq!(st.path, path_of(&ops));
        assert!(st.visited[g.start()] && !st.no_path);
        assert_eq!(st.dist[g.goal()], Some((st.path.len() - 1) as u32));
    }
}
