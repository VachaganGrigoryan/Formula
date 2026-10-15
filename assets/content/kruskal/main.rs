//! Kruskal's algorithm: sort edges by weight; add each one that joins two different trees.
//!
//! 14 random points joined by every pair closer than 0.35 (the seed is retried
//! until the graph is connected). The algorithm is run once and recorded as a
//! list of operations; the animation then replays that list. Vertices are colored by their union-find tree. Click the
//! window to draw a new set of points.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 14;
const RADIUS: f32 = 0.35;
const SIZE: f32 = 560.0; // pixels per side of the unit square
const HEADER: f32 = 30.0;
const STEPS: usize = 1; // recorded operations replayed per tick
const TICK: f64 = 0.12; // seconds per tick

#[derive(Clone, Copy)]
struct Edge {
    a: usize,
    b: usize,
    w: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Status {
    Unseen,
    Consider,
    Accept,
    Reject,
}

#[derive(Clone, Copy)]
enum Op {
    Edge(usize, Status),
    Union(usize, usize),
}

fn find(parent: &[usize], mut v: usize) -> usize {
    while parent[v] != v {
        v = parent[v];
    }
    v
}

/// Random points and every pair within RADIUS; retry until connected.
fn make_graph(mut seed: u64) -> (Vec<Vec2>, Vec<Edge>) {
    loop {
        rand::srand(seed);
        let pts: Vec<Vec2> = (0..N)
            .map(|_| vec2(rand::gen_range(0.08, 0.92), rand::gen_range(0.08, 0.92)))
            .collect();
        let mut edges = Vec::new();
        for a in 0..N {
            for b in a + 1..N {
                let w = pts[a].distance(pts[b]) * 100.0;
                if w <= RADIUS * 100.0 {
                    edges.push(Edge { a, b, w });
                }
            }
        }
        let mut parent: Vec<usize> = (0..N).collect();
        for e in &edges {
            let (ra, rb) = (find(&parent, e.a), find(&parent, e.b));
            parent[ra] = rb;
        }
        let first = find(&parent, 0);
        if (0..N).all(|v| find(&parent, v) == first) {
            return (pts, edges);
        }
        seed += 1000;
    }
}

/// Run Kruskal and log every step.
fn record(edges: &[Edge]) -> Vec<Op> {
    let mut order: Vec<usize> = (0..edges.len()).collect();
    order.sort_by(|&x, &y| edges[x].w.total_cmp(&edges[y].w).then(x.cmp(&y)));
    let mut parent: Vec<usize> = (0..N).collect();
    let mut size = vec![1usize; N];
    let mut ops = Vec::new();
    let mut taken = 0;
    for e in order {
        if taken + 1 >= N {
            break;
        }
        ops.push(Op::Edge(e, Status::Consider));
        let (ra, rb) = (find(&parent, edges[e].a), find(&parent, edges[e].b));
        if ra == rb {
            ops.push(Op::Edge(e, Status::Reject));
            continue;
        }
        ops.push(Op::Edge(e, Status::Accept));
        taken += 1;
        let (child, root) = if size[ra] < size[rb] { (ra, rb) } else { (rb, ra) };
        parent[child] = root;
        size[root] += size[child];
        ops.push(Op::Union(child, root));
    }
    ops
}

/// The run replayed so far.
struct Replay {
    ops: Vec<Op>,
    cursor: usize,
    status: Vec<Status>,
    forest: Vec<usize>,
    weight: f32,
    accepted: usize,
}

impl Replay {
    fn new(edges: &[Edge]) -> Self {
        Self {
            ops: record(edges),
            cursor: 0,
            status: vec![Status::Unseen; edges.len()],
            forest: (0..N).collect(),
            weight: 0.0,
            accepted: 0,
        }
    }

    fn step(&mut self, edges: &[Edge]) {
        let Some(&op) = self.ops.get(self.cursor) else { return };
        self.cursor += 1;
        match op {
            Op::Union(child, root) => self.forest[child] = root,
            Op::Edge(e, s) => {
                self.status[e] = s;
                if s == Status::Accept {
                    self.weight += edges[e].w;
                    self.accepted += 1;
                }
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Kruskal's algorithm".to_owned(),
        window_width: SIZE as i32,
        window_height: (SIZE + HEADER) as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut seed = 7;
    let (mut pts, mut edges) = make_graph(seed);
    let mut replay = Replay::new(&edges);
    let mut last = get_time();
    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            seed += 1;
            (pts, edges) = make_graph(seed);
            replay = Replay::new(&edges);
        }
        if get_time() - last >= TICK {
            last = get_time();
            for _ in 0..STEPS {
                replay.step(&edges);
            }
        }

        clear_background(Color::from_hex(0x16201c));
        let at = |v: usize| vec2(pts[v].x * SIZE, HEADER + pts[v].y * SIZE);
        for pass in [Status::Unseen, Status::Reject, Status::Accept, Status::Consider] {
            for (i, e) in edges.iter().enumerate() {
                if replay.status[i] != pass {
                    continue;
                }
                let (color, width) = match pass {
                    Status::Consider => (Color::from_hex(0xf0a040), 3.0),
                    Status::Accept => (Color::from_hex(0xa8c4ff), 3.0),
                    Status::Reject => (Color::from_hex(0x7a8a84), 1.2),
                    Status::Unseen => (Color::from_hex(0x3a4a44), 1.0),
                };
                draw_line(at(e.a).x, at(e.a).y, at(e.b).x, at(e.b).y, width, color);
            }
        }
        for v in 0..N {
            let palette = [0x4cc38a, 0xf0a040, 0xa8c4ff, 0xd8e0dc, 0x7a8a84, 0x2c8a6a];
            let fill = Color::from_hex(palette[find(&replay.forest, v) % palette.len()]);
            draw_circle(at(v).x, at(v).y, 8.0, fill);
            draw_circle_lines(at(v).x, at(v).y, 8.0, 1.5, Color::from_hex(0xd8e0dc));
        }
        let header = format!(
            "Kruskal's algorithm  |  tree weight {:.0}  |  edges {} / {}",
            replay.weight,
            replay.accepted,
            N - 1
        );
        draw_text(&header, 6.0, 21.0, 22.0, WHITE);
        next_frame().await;
    }
}
