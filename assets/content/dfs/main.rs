//! Depth-first search: follow the newest frontier cell first (a stack)
//!
//! A 31 x 19 grid with a start (orange dot) and a goal (green ring). DFS is run once and recorded as a list of operations (a cell joins the stack, a cell is
//! visited, a cell joins the final path); the animation then replays that list at a fixed speed. It dives
//! down one corridor (up first) before backing up, so the path it finds is valid but usually far from the
//! shortest.
//! Click a cell to toggle a wall and run the search again, press m for a random maze and r to
//! restart the replay.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const W: usize = 31;
const H: usize = 19;
const SIZE: f32 = 28.0; // pixels per cell
const HEADER: f32 = 30.0;
const STEPS: usize = 4; // recorded operations replayed per frame
const START: usize = (H / 2) * W + 1;
const GOAL: usize = (H / 2) * W + W - 2;

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

/// Run DFS and log every step.
fn record(walls: &[bool]) -> Vec<Op> {
    let mut ops = vec![Op::Frontier(START)];
    let mut visited = vec![false; W * H];
    let mut parent = vec![usize::MAX; W * H];
    let mut stack = vec![(START, usize::MAX)];
    let mut found = false;
    while let Some((cell, from)) = stack.pop() {
        if visited[cell] {
            continue; // mark on pop: stale duplicates are skipped
        }
        visited[cell] = true;
        parent[cell] = from;
        ops.push(Op::Visit(cell));
        if cell == GOAL {
            found = true;
            break;
        }
        // reversed, so "up" ends on top of the stack and is tried first
        for n in neighbors(walls, cell).into_iter().rev() {
            if !visited[n] {
                ops.push(Op::Frontier(n));
                stack.push((n, cell));
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
    /// Record the search on `walls` and start replaying it.
    fn new(walls: &[bool]) -> Self {
        Self {
            ops: record(walls),
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
        window_title: "Depth-first search".to_owned(),
        window_width: (W as f32 * SIZE) as i32,
        window_height: (H as f32 * SIZE + HEADER) as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut walls = vec![false; W * H];
    let mut replay = Replay::new(&walls);
    let mut seed = 0;
    loop {
        if is_key_pressed(KeyCode::M) {
            seed += 1;
            walls = maze(seed);
            replay = Replay::new(&walls);
        }
        if is_key_pressed(KeyCode::R) {
            replay = Replay::new(&walls);
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if my >= HEADER && mx >= 0.0 {
                let (x, y) = ((mx / SIZE) as usize, ((my - HEADER) / SIZE) as usize);
                let cell = y * W + x;
                if x < W && y < H && cell != START && cell != GOAL {
                    walls[cell] = !walls[cell];
                    replay = Replay::new(&walls);
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
        let header = if replay.no_path {
            "No path: the goal is walled off".to_owned()
        } else {
            format!("Depth-first search  |  visited {visited}  |  path {steps} steps")
        };
        draw_text(&header, 6.0, 21.0, 22.0, WHITE);
        next_frame().await;
    }
}
