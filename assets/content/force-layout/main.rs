//! Springs and repulsion: F_s = -k (d - x0) on every edge, F_r = kr / d^2 between every pair.
//!
//! A random graph of four clusters relaxes into a readable layout. Edge color shows the
//! spring's strain (red stretched, blue squeezed). Drag a node to pin it and watch the
//! graph rebalance; hover a node to see its spring and repulsion forces.
//! Keys: S shuffles the positions, N makes a new graph.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;
use macroquad::rand::gen_range;

const K: f32 = 0.04; // spring stiffness
const REST: f32 = 60.0; // spring rest length
const KR: f32 = 3000.0; // repulsion strength
const GRAVITY: f32 = 0.004;
const DAMPING: f32 = 0.85;
const MAX_SPEED: f32 = 12.0;

/// Hooke's law along an edge: negative pulls the two ends together.
fn spring(k: f32, d: f32, rest: f32) -> f32 {
    -k * (d - rest)
}

/// Coulomb-style repulsion: always pushes apart, falls off as 1 / d^2.
fn repulsion(kr: f32, d: f32) -> f32 {
    kr / (d * d)
}

struct Node {
    pos: Vec2,
    vel: Vec2,
    cluster: usize,
}

/// Spring and repulsion forces on every node.
fn forces(nodes: &[Node], edges: &[(usize, usize)]) -> (Vec<Vec2>, Vec<Vec2>) {
    let mut springs = vec![Vec2::ZERO; nodes.len()];
    let mut repel = vec![Vec2::ZERO; nodes.len()];
    for &(i, j) in edges {
        let delta = nodes[i].pos - nodes[j].pos;
        let d = delta.length().max(1e-3);
        let f = delta / d * spring(K, d, REST);
        springs[i] += f;
        springs[j] -= f;
    }
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            let delta = nodes[i].pos - nodes[j].pos;
            let d = delta.length().max(4.0);
            let f = delta / d * repulsion(KR, d);
            repel[i] += f;
            repel[j] -= f;
        }
    }
    (springs, repel)
}

/// One damped integration step; `pinned` stays put. Returns the kinetic energy.
fn step(nodes: &mut [Node], edges: &[(usize, usize)], pinned: Option<usize>) -> f32 {
    let (springs, repel) = forces(nodes, edges);
    let mut kinetic = 0.0;
    for (i, n) in nodes.iter_mut().enumerate() {
        if Some(i) == pinned {
            n.vel = Vec2::ZERO;
            continue;
        }
        n.vel = (n.vel + springs[i] + repel[i] - n.pos * GRAVITY) * DAMPING;
        n.vel = n.vel.clamp_length_max(MAX_SPEED); // stiff springs can't fling a node away
        n.pos += n.vel;
        kinetic += 0.5 * n.vel.length_squared();
    }
    kinetic
}

/// Four clusters, each a random tree plus a few extra edges, chained together.
fn random_graph() -> (Vec<Node>, Vec<(usize, usize)>) {
    let (mut nodes, mut edges, mut firsts) = (Vec::new(), Vec::new(), Vec::new());
    for c in 0..4 {
        let (start, size) = (nodes.len(), gen_range(5, 10));
        firsts.push(start);
        for i in 0..size {
            let pos = vec2(gen_range(-150.0, 150.0), gen_range(-110.0, 110.0));
            nodes.push(Node {
                pos,
                vel: Vec2::ZERO,
                cluster: c,
            });
            if i > 0 {
                edges.push((start + i, start + gen_range(0, i)));
            }
        }
        for _ in 0..size / 3 {
            let (a, b) = (start + gen_range(0, size), start + gen_range(0, size));
            if a != b && !edges.contains(&(a, b)) && !edges.contains(&(b, a)) {
                edges.push((a, b));
            }
        }
        if c > 0 {
            edges.push((firsts[c - 1], start));
        }
    }
    (nodes, edges)
}

fn arrow(from: Vec2, v: Vec2, color: Color) {
    let v = v.clamp_length_max(120.0);
    if v.length() < 1.0 {
        return;
    }
    let to = from + v;
    let back = -v.normalize() * 10.0;
    let side = vec2(-back.y, back.x) * 0.5;
    draw_line(from.x, from.y, to.x, to.y, 2.5, color);
    draw_triangle(to, to + back + side, to + back - side, color);
}

#[macroquad::main("Springs and repulsion")]
async fn main() {
    let cluster_colors = [
        Color::from_rgba(42, 157, 143, 255),
        Color::from_rgba(233, 196, 106, 255),
        Color::from_rgba(126, 167, 224, 255),
        Color::from_rgba(195, 139, 214, 255),
    ];
    let (mut nodes, mut edges) = random_graph();
    let mut dragging: Option<usize> = None;

    loop {
        let center = vec2(screen_width(), screen_height()) / 2.0;
        let mouse = Vec2::from(mouse_position()) - center;
        let pick =
            |nodes: &[Node], radius: f32| nodes.iter().position(|n| n.pos.distance(mouse) < radius);
        if is_mouse_button_pressed(MouseButton::Left) {
            dragging = pick(&nodes, 20.0);
        }
        if is_mouse_button_released(MouseButton::Left) {
            dragging = None;
        }
        if let Some(i) = dragging {
            nodes[i].pos = mouse;
        }
        if is_key_pressed(KeyCode::N) {
            (nodes, edges) = random_graph();
        }
        if is_key_pressed(KeyCode::S) {
            for n in &mut nodes {
                n.pos = vec2(gen_range(-200.0, 200.0), gen_range(-140.0, 140.0));
                n.vel = Vec2::ZERO;
            }
        }

        let mut kinetic = 0.0;
        for _ in 0..2 {
            kinetic = step(&mut nodes, &edges, dragging);
        }
        let half = center - vec2(12.0, 12.0);
        for n in &mut nodes {
            n.pos = n.pos.clamp(-half, half);
        }

        clear_background(Color::from_rgba(22, 32, 28, 255));
        // Edges: gray at rest length, red when stretched, blue when squeezed.
        for &(i, j) in &edges {
            let (a, b) = (center + nodes[i].pos, center + nodes[j].pos);
            let strain = ((a.distance(b) - REST) / REST).clamp(-1.0, 1.0);
            let gray = vec3(0.5, 0.55, 0.53);
            let target = if strain > 0.0 {
                vec3(0.9, 0.31, 0.31)
            } else {
                vec3(0.31, 0.55, 0.94)
            };
            let c = gray.lerp(target, strain.abs());
            draw_line(
                a.x,
                a.y,
                b.x,
                b.y,
                1.5 + 2.0 * strain.abs(),
                Color::new(c.x, c.y, c.z, 1.0),
            );
        }
        let hovered = pick(&nodes, 16.0);
        for (i, n) in nodes.iter().enumerate() {
            let p = center + n.pos;
            let r = if Some(i) == dragging || Some(i) == hovered {
                9.0
            } else {
                7.0
            };
            draw_circle(p.x, p.y, r + 2.0, Color::from_rgba(22, 32, 28, 255));
            draw_circle(p.x, p.y, r, cluster_colors[n.cluster % 4]);
        }

        // Forces on the focused node, scaled up so they are visible.
        if let Some(i) = dragging.or(hovered) {
            let (springs, repel) = forces(&nodes, &edges);
            let p = center + nodes[i].pos;
            arrow(p, springs[i] * 12.0, Color::from_rgba(255, 107, 107, 255));
            arrow(p, repel[i] * 12.0, Color::from_rgba(78, 205, 196, 255));
        }
        let muted = Color::from_rgba(138, 154, 146, 255);
        draw_text(
            "drag a node   red: spring   teal: repulsion   S: shuffle   N: new graph",
            14.0,
            24.0,
            18.0,
            muted,
        );
        let status = if kinetic < 0.01 { "settled" } else { "moving" };
        let text = format!("kinetic energy {kinetic:.3}  ({status})");
        draw_text(&text, 14.0, screen_height() - 14.0, 20.0, muted);
        next_frame().await;
    }
}
