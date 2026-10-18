//! The pathfinding board: a rectangular grid of open cells, walls and mud,
//! with a start and a goal. UI-free, so it is unit tested on its own.

use crate::rng::Rng;

/// Cost of stepping onto a mud cell (open cells cost 1).
pub const MUD: u8 = 5;
pub const GRID_W: usize = 31;
pub const GRID_H: usize = 19;

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub w: usize,
    pub h: usize,
    walls: Vec<bool>,
    weight: Vec<u8>,
    start: usize,
    goal: usize,
    /// Bumped by every edit that changes something, so recordings made on an
    /// older state can tell they are stale.
    generation: u64,
    /// Seed of the last generated preset, shared by every viewport so each
    /// press gives a new layout.
    seed: u64,
}

impl Grid {
    pub fn new(w: usize, h: usize) -> Self {
        assert!(w >= 4 && h >= 1, "grid needs w >= 4 and h >= 1");
        let mut g = Self {
            w,
            h,
            walls: vec![false; w * h],
            weight: vec![1; w * h],
            start: 0,
            goal: 0,
            generation: 0,
            seed: 0,
        };
        g.start = g.index(1, h / 2);
        g.goal = g.index(w - 2, h / 2);
        g
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    pub fn xy(&self, i: usize) -> (usize, usize) {
        (i % self.w, i / self.w)
    }

    pub fn len(&self) -> usize {
        self.w * self.h
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_wall(&self, i: usize) -> bool {
        self.walls[i]
    }

    pub fn weight(&self, i: usize) -> u8 {
        self.weight[i]
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn goal(&self) -> usize {
        self.goal
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Walls cannot cover the start or goal. Walling a cell removes its mud.
    pub fn set_wall(&mut self, i: usize, wall: bool) {
        if i >= self.len() || i == self.start || i == self.goal {
            return;
        }
        let new_weight = if wall { 1 } else { self.weight[i] };
        if self.walls[i] == wall && self.weight[i] == new_weight {
            return;
        }
        self.walls[i] = wall;
        self.weight[i] = new_weight;
        self.generation += 1;
    }

    /// Walls take no mud, and mud never covers the start or goal.
    pub fn set_weight(&mut self, i: usize, w: u8) {
        if i >= self.len()
            || self.walls[i]
            || i == self.start
            || i == self.goal
            || self.weight[i] == w
        {
            return;
        }
        self.weight[i] = w;
        self.generation += 1;
    }

    pub fn move_start(&mut self, i: usize) -> bool {
        if i >= self.len() || self.walls[i] || i == self.goal {
            return false;
        }
        if i != self.start {
            self.start = i;
            self.generation += 1;
        }
        true
    }

    pub fn move_goal(&mut self, i: usize) -> bool {
        if i >= self.len() || self.walls[i] || i == self.start {
            return false;
        }
        if i != self.goal {
            self.goal = i;
            self.generation += 1;
        }
        true
    }

    /// The grid as one string per row: `.` open, `#` wall, `~` mud, `S` start,
    /// `G` goal.
    pub fn to_rows(&self) -> Vec<String> {
        crate::state::grid_rows(self.w, self.h, |x, y| {
            let i = self.index(x, y);
            match () {
                _ if i == self.start => 'S',
                _ if i == self.goal => 'G',
                _ if self.walls[i] => '#',
                _ if self.weight[i] > 1 => '~',
                _ => '.',
            }
        })
    }

    /// Replace the cells with `rows` (as written by [`Grid::to_rows`]). Rows of
    /// the wrong size, or without exactly one start and one goal, are refused.
    pub fn load_rows(&mut self, rows: &[String]) -> bool {
        let Some(cells) = crate::state::parse_rows(rows, self.w, self.h) else {
            return false;
        };
        let find = |c: char| -> Vec<usize> {
            let mut at = Vec::new();
            for (y, row) in cells.iter().enumerate() {
                for (x, &ch) in row.iter().enumerate() {
                    if ch == c {
                        at.push(y * self.w + x);
                    }
                }
            }
            at
        };
        let (&[start], &[goal]) = (find('S').as_slice(), find('G').as_slice()) else {
            return false;
        };
        for (i, ch) in cells.iter().flatten().enumerate() {
            self.walls[i] = *ch == '#';
            self.weight[i] = if *ch == '~' { MUD } else { 1 };
        }
        self.start = start;
        self.goal = goal;
        self.generation += 1;
        true
    }

    /// Open every cell and drop all mud; the markers stay where they are.
    pub fn clear(&mut self) {
        self.walls.fill(false);
        self.weight.fill(1);
        self.generation += 1;
    }

    /// A perfect maze (recursive backtracker on the odd cells), then the
    /// markers go to the left and right ends of a corridor row.
    pub fn maze(&mut self, seed: u64) {
        let mut rng = Rng::new(seed);
        self.walls.fill(true);
        self.weight.fill(1);
        if self.w >= 3 && self.h >= 3 {
            let mut visited = vec![false; self.len()];
            let first = self.index(1, 1);
            self.walls[first] = false;
            visited[first] = true;
            let mut stack = vec![first];
            while let Some(&cur) = stack.last() {
                let (x, y) = self.xy(cur);
                let mut options = Vec::with_capacity(4);
                if y >= 3 {
                    options.push((x, y - 2));
                }
                if x + 2 < self.w - 1 {
                    options.push((x + 2, y));
                }
                if y + 2 < self.h - 1 {
                    options.push((x, y + 2));
                }
                if x >= 3 {
                    options.push((x - 2, y));
                }
                options.retain(|&(nx, ny)| !visited[self.index(nx, ny)]);
                if options.is_empty() {
                    stack.pop();
                    continue;
                }
                let (nx, ny) = options[rng.index(options.len())];
                let n = self.index(nx, ny);
                let between = self.index((x + nx) / 2, (y + ny) / 2);
                self.walls[n] = false;
                self.walls[between] = false;
                visited[n] = true;
                stack.push(n);
            }
        }
        let row = ((self.h / 2) | 1).min(self.h - 1);
        self.start = self.index(1, row);
        self.goal = self.index(self.w - 2, row);
        self.walls[self.start] = false;
        self.walls[self.goal] = false;
        self.generation += 1;
    }

    /// A maze from the next seed in this grid's own sequence.
    pub fn maze_next(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        self.maze(self.seed);
    }

    /// Random walls from the next seed in this grid's own sequence.
    pub fn scatter_next(&mut self, density: f32) {
        self.seed = self.seed.wrapping_add(1);
        self.scatter(self.seed, density);
    }

    /// Random walls with the given density; the markers stay open.
    pub fn scatter(&mut self, seed: u64, density: f32) {
        let mut rng = Rng::new(seed);
        for i in 0..self.len() {
            self.weight[i] = 1;
            self.walls[i] = i != self.start && i != self.goal && rng.f32() < density;
        }
        self.generation += 1;
    }

    /// Open 4-neighbours in the fixed order up, right, down, left.
    pub fn neighbors(&self, i: usize) -> Vec<usize> {
        let (x, y) = self.xy(i);
        let mut out = Vec::with_capacity(4);
        if y > 0 {
            out.push(self.index(x, y - 1));
        }
        if x + 1 < self.w {
            out.push(self.index(x + 1, y));
        }
        if y + 1 < self.h {
            out.push(self.index(x, y + 1));
        }
        if x > 0 {
            out.push(self.index(x - 1, y));
        }
        out.retain(|&n| !self.walls[n]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reachable(g: &Grid) -> Vec<bool> {
        let mut seen = vec![false; g.len()];
        let mut stack = vec![g.start()];
        seen[g.start()] = true;
        while let Some(i) = stack.pop() {
            for n in g.neighbors(i) {
                if !seen[n] {
                    seen[n] = true;
                    stack.push(n);
                }
            }
        }
        seen
    }

    #[test]
    fn rows_round_trip() {
        let mut g = Grid::new(8, 5);
        g.set_wall(g.index(3, 1), true);
        g.set_weight(g.index(4, 3), MUD);
        let rows = g.to_rows();
        assert_eq!(rows[2], ".S....G.");
        let mut h = Grid::new(8, 5);
        assert!(h.load_rows(&rows));
        assert_eq!(h.to_rows(), rows);
        assert!(h.generation() > 0);
        // Wrong sizes and missing markers are refused.
        assert!(!h.load_rows(&rows[..4]));
        let no_goal: Vec<String> = rows.iter().map(|r| r.replace('G', ".")).collect();
        assert!(!h.load_rows(&no_goal));
    }

    #[test]
    fn new_grid_is_open_with_markers_on_the_middle_row() {
        let g = Grid::new(GRID_W, GRID_H);
        assert_eq!(g.xy(g.start()), (1, GRID_H / 2));
        assert_eq!(g.xy(g.goal()), (GRID_W - 2, GRID_H / 2));
        assert!((0..g.len()).all(|i| !g.is_wall(i) && g.weight(i) == 1));
    }

    #[test]
    fn neighbors_are_open_cells_up_right_down_left() {
        let mut g = Grid::new(5, 5);
        let c = g.index(2, 2);
        assert_eq!(
            g.neighbors(c),
            vec![g.index(2, 1), g.index(3, 2), g.index(2, 3), g.index(1, 2)]
        );
        g.set_wall(g.index(2, 3), true);
        assert_eq!(g.neighbors(c).len(), 3);
        assert_eq!(
            g.neighbors(g.index(0, 0)),
            vec![g.index(1, 0), g.index(0, 1)]
        );
    }

    #[test]
    fn markers_never_go_on_walls_or_onto_each_other() {
        let mut g = Grid::new(GRID_W, GRID_H);
        let (s, t) = (g.start(), g.goal());
        let before = g.generation();
        g.set_wall(s, true);
        g.set_wall(t, true);
        assert!(!g.is_wall(s) && !g.is_wall(t));
        assert_eq!(
            g.generation(),
            before,
            "ignored edits do not invalidate recordings"
        );
        let w = g.index(5, 5);
        g.set_wall(w, true);
        assert!(!g.move_start(w), "onto a wall");
        assert!(!g.move_start(t), "onto the goal");
        assert!(!g.move_goal(g.len()), "off the grid");
        assert_eq!((g.start(), g.goal()), (s, t));
        assert!(g.move_start(g.index(3, 3)));
        assert!(g.generation() > before);
    }

    #[test]
    fn mud_only_on_open_cells_and_walls_clear_it() {
        let mut g = Grid::new(GRID_W, GRID_H);
        let i = g.index(4, 4);
        g.set_weight(i, MUD);
        assert_eq!(g.weight(i), MUD);
        g.set_wall(i, true);
        assert_eq!(g.weight(i), 1);
        g.set_weight(i, MUD);
        assert_eq!(g.weight(i), 1, "walls take no mud");
        let (s, t) = (g.start(), g.goal());
        let before = g.generation();
        g.set_weight(s, MUD);
        g.set_weight(t, MUD);
        assert_eq!((g.weight(s), g.weight(t)), (1, 1), "no mud on the markers");
        assert_eq!(g.generation(), before);
    }

    #[test]
    fn mazes_connect_every_open_cell_and_depend_on_the_seed() {
        let mut a = Grid::new(GRID_W, GRID_H);
        a.maze(1);
        assert!(!a.is_wall(a.start()) && !a.is_wall(a.goal()));
        let seen = reachable(&a);
        assert!(
            (0..a.len()).all(|i| a.is_wall(i) || seen[i]),
            "a perfect maze is connected"
        );
        let mut b = Grid::new(GRID_W, GRID_H);
        b.maze(2);
        assert_ne!(a, b);
        let mut c = Grid::new(GRID_W, GRID_H);
        c.maze(1);
        assert_eq!(a, c, "same seed, same maze");
    }

    #[test]
    fn successive_presets_use_new_seeds() {
        let mut g = Grid::new(GRID_W, GRID_H);
        let walls = |g: &Grid| (0..g.len()).map(|i| g.is_wall(i)).collect::<Vec<_>>();
        g.maze_next();
        let a = walls(&g);
        g.maze_next();
        assert_ne!(a, walls(&g), "a second press changes the maze");
        let b = walls(&g);
        g.scatter_next(0.28);
        let c = walls(&g);
        assert_ne!(b, c);
        g.scatter_next(0.28);
        assert_ne!(c, walls(&g), "a second press changes the walls");
    }

    #[test]
    fn scatter_keeps_the_markers_open() {
        let mut g = Grid::new(GRID_W, GRID_H);
        g.scatter(9, 0.9);
        assert!(!g.is_wall(g.start()) && !g.is_wall(g.goal()));
        let walls = (0..g.len()).filter(|&i| g.is_wall(i)).count();
        assert!(walls > g.len() / 2);
    }
}
