//! Dijkstra's algorithm: dist(n) = min(dist(n), dist(u) + w(n))
//!
//! A 31 x 19 grid with a start (orange dot) and a goal (green ring). Dijkstra is run once and recorded as a list of operations (a cell joins the priority queue,
//! a cell is settled, a cell joins the final path); the animation then replays that list at a fixed speed.
//! Cells are settled in order of total cost, and mud costs 5 instead of 1, so the path detours around mud
//! when that is cheaper.
//! Click a cell to toggle a wall or mud (press w to switch the brush) and run the search
//! again, press m for a random maze and r to restart the replay.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use macroquad::prelude::*;

const W: usize = 31;
const H: usize = 19;
const SIZE: f32 = 28.0; // pixels per cell
const HEADER: f32 = 30.0;
const STEPS: usize = 4; // recorded operations replayed per frame
const START: usize = (H / 2) * W + 1;
const GOAL: usize = (H / 2) * W + W - 2;
const MUD: u32 = 5; // cost of entering a mud cell

#[derive(Clone, Copy)]
enum Op {
    Frontier(usize),
    Visit(usize),
    Path(usize),
    NoPath,
}

/// Open 4-neighbours in the order up, right, down, left.
fn neighbors(walls: &[bool], i: usize) -> Vec<usize> {
    let (x, y) = ((i % W) as i32, (i / W) as i32);
    [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .filter_map(|(dx, dy)| {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= W as i32 || ny >= H as i32 {
                return None;
            }
            let n = ny as usize * W + nx as usize;
            (!walls[n]).then_some(n)
        })
        .collect()
}

/// Estimated remaining cost: zero (no guess, plain Dijkstra).
fn heuristic(_i: usize) -> u32 {
    0
}

/// Run Dijkstra and log every step.
fn record(walls: &[bool], mud: &[bool]) -> Vec<Op> {
    let mut ops = vec![Op::Frontier(START)];
    let mut dist = vec![u32::MAX; W * H];
    let mut parent = vec![usize::MAX; W * H];
    let mut done = vec![false; W * H];
    dist[START] = 0;
    let mut heap = BinaryHeap::from([Reverse((heuristic(START), heuristic(START), START))]); // (f, h, cell)
    let mut found = false;
    while let Some(Reverse((f, h, cell))) = heap.pop() {
        if done[cell] || f != dist[cell] + h {
            continue; // stale entry: skip it
        }
        done[cell] = true;
        ops.push(Op::Visit(cell));
        if cell == GOAL {
            found = true;
            break;
        }
        for n in neighbors(walls, cell) {
            let nd = dist[cell] + if mud[n] { MUD } else { 1 };
            if nd < dist[n] {
                // relaxation
                dist[n] = nd;
                parent[n] = cell;
                ops.push(Op::Frontier(n));
                heap.push(Reverse((nd + heuristic(n), heuristic(n), n)));
            }
        }
    }
    if !found {
        ops.push(Op::NoPath);
        return ops;
    }
    let mut route = vec![GOAL];
    while parent[*route.last().unwrap()] != usize::MAX {
        route.push(parent[*route.last().unwrap()]);
    }
    ops.extend(route.into_iter().rev().map(Op::Path));
    ops
}

/// A perfect maze by recursive backtracking on the odd cells.
fn maze(seed: u64) -> Vec<bool> {
    rand::srand(seed);
    let mut walls = vec![true; W * H];
    let mut seen = vec![false; W * H];
    let first = W + 1;
    walls[first] = false;
    seen[first] = true;
    let mut stack = vec![first];
    while let Some(&cur) = stack.last() {
        let (x, y) = ((cur % W) as i32, (cur / W) as i32);
        let options: Vec<usize> = [(0, -2), (2, 0), (0, 2), (-2, 0)]
            .into_iter()
            .filter(|&(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                nx >= 1 && ny >= 1 && nx < W as i32 - 1 && ny < H as i32 - 1
                    && !seen[ny as usize * W + nx as usize]
            })
            .map(|(dx, dy)| (y + dy) as usize * W + (x + dx) as usize)
            .collect();
        if options.is_empty() {
            stack.pop();
            continue;
        }
        let n = options[rand::gen_range(0, options.len())];
        walls[(cur + n) / 2] = false;
        walls[n] = false;
        seen[n] = true;
        stack.push(n);
    }
    walls[START] = false;
    walls[GOAL] = false;
    walls
}

/// A block of mud across the straight route: the detour around it is cheaper.
fn default_mud() -> Vec<bool> {
    let mut mud = vec![false; W * H];
    for y in 5..14 {
        for x in 12..19 {
            mud[y * W + x] = true;
        }
    }
    mud
}

/// The search replayed so far.
struct Replay {
    ops: Vec<Op>,
    cursor: usize,
    frontier: Vec<bool>,
    visited: Vec<bool>,
    path: Vec<bool>,
    no_path: bool,
}

impl Replay {
    /// Record the search on `walls` and `mud` and start replaying it.
    fn new(walls: &[bool], mud: &[bool]) -> Self {
        Self {
            ops: record(walls, mud),
            cursor: 0,
            frontier: vec![false; W * H],
            visited: vec![false; W * H],
            path: vec![false; W * H],
            no_path: false,
        }
    }

    fn step(&mut self) {
        let Some(&op) = self.ops.get(self.cursor) else { return };
        self.cursor += 1;
        match op {
            Op::Frontier(c) => self.frontier[c] = true,
            Op::Visit(c) => {
                self.frontier[c] = false;
                self.visited[c] = true;
            }
            Op::Path(c) => self.path[c] = true,
            Op::NoPath => self.no_path = true,
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Dijkstra's algorithm".to_owned(),
        window_width: (W as f32 * SIZE) as i32,
        window_height: (H as f32 * SIZE + HEADER) as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut walls = vec![false; W * H];
    let mut mud = default_mud();
    let mut brush_mud = false;
    let mut replay = Replay::new(&walls, &mud);
    let mut seed = 0;
    loop {
        if is_key_pressed(KeyCode::M) {
            seed += 1;
            walls = maze(seed);
            mud = vec![false; W * H];
            replay = Replay::new(&walls, &mud);
        }
        if is_key_pressed(KeyCode::W) {
            brush_mud = !brush_mud;
        }
        if is_key_pressed(KeyCode::R) {
            replay = Replay::new(&walls, &mud);
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if my >= HEADER && mx >= 0.0 {
                let (x, y) = ((mx / SIZE) as usize, ((my - HEADER) / SIZE) as usize);
                let cell = y * W + x;
                if x < W && y < H && cell != START && cell != GOAL {
                    if brush_mud {
                        if !walls[cell] {
                            mud[cell] = !mud[cell];
                        }
                    } else {
                        walls[cell] = !walls[cell];
                        mud[cell] = false;
                    }
                    replay = Replay::new(&walls, &mud);
                }
            }
        }
        for _ in 0..STEPS {
            replay.step();
        }

        clear_background(Color::from_hex(0x16201c));
        for i in 0..W * H {
            let color = if walls[i] {
                Color::from_hex(0xd8e0dc)
            } else if mud[i] {
                Color::from_hex(0x5a4a2a)
            } else if replay.path[i] {
                Color::from_hex(0xa8c4ff)
            } else if replay.frontier[i] {
                Color::from_hex(0x9b6a2a)
            } else if replay.visited[i] {
                Color::from_hex(0x2c4a6a)
            } else {
                Color::from_hex(0x16201c)
            };
            let (x, y) = ((i % W) as f32 * SIZE, HEADER + (i / W) as f32 * SIZE);
            draw_rectangle(x, y, SIZE, SIZE, color);
            draw_rectangle_lines(x, y, SIZE, SIZE, 1.0, Color::new(1.0, 1.0, 1.0, 0.08));
        }
        let mid = |i: usize| vec2((i % W) as f32 * SIZE + SIZE / 2.0, HEADER + (i / W) as f32 * SIZE + SIZE / 2.0);
        draw_circle(mid(START).x, mid(START).y, SIZE * 0.32, Color::from_hex(0xf0a040));
        draw_circle_lines(mid(GOAL).x, mid(GOAL).y, SIZE * 0.32, 2.5, Color::from_hex(0x4cc38a));

        let visited = replay.visited.iter().filter(|&&v| v).count();
        let steps = replay.path.iter().filter(|&&v| v).count().saturating_sub(1);
        let mud_steps = (0..W * H).filter(|&i| replay.path[i] && mud[i]).count();
        let cost = steps + (MUD as usize - 1) * mud_steps;
        let header = if replay.no_path {
            "No path: the goal is walled off".to_owned()
        } else {
            format!("Dijkstra's algorithm  |  visited {visited}  |  path {steps} steps  |  cost {cost}")
        };
        draw_text(&header, 6.0, 21.0, 22.0, WHITE);
        next_frame().await;
    }
}
