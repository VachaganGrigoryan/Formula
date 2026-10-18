//! Pythagorean theorem: a proof by rearrangement.
//!
//! Four copies of a right triangle with legs a and b sit in the corners of a square of
//! side a + b, leaving a tilted square of side c uncovered. Three of them then slide,
//! one after another, until the uncovered part is a square of side a and one of side b.
//! The big square and the triangles never changed, so c^2 = a^2 + b^2.
//! Left/Right change a, Up/Down change b.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

type Tri = [[f32; 2]; 3];

/// The four triangles around the c^2 square: right angle, end of leg a, end of leg b.
fn tilted(a: f32, b: f32) -> [Tri; 4] {
    let s = a + b;
    [
        [[0.0, 0.0], [a, 0.0], [0.0, b]],
        [[s, 0.0], [s, a], [a, 0.0]],
        [[s, s], [b, s], [s, a]],
        [[0.0, s], [0.0, b], [b, s]],
    ]
}

fn smoothstep(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

fn triangles(a: f32, b: f32, t: f32) -> [Tri; 4] {
    let mut tris = tilted(a, b);
    let moves = [[0.0, a], [0.0, 0.0], [-b, 0.0], [a, -b]];
    for (turn, i) in [3usize, 0, 2].into_iter().enumerate() {
        let u = smoothstep(t * 3.0 - turn as f32);
        for p in &mut tris[i] {
            p[0] += moves[i][0] * u;
            p[1] += moves[i][1] * u;
        }
    }
    tris
}

/// Hold at 0, move to 1, hold, move back.
fn cycle(phase: f32) -> f32 {
    let u = (phase % 10.0) / 10.0;
    if u < 0.1 {
        0.0
    } else if u < 0.45 {
        (u - 0.1) / 0.35
    } else if u < 0.6 {
        1.0
    } else {
        1.0 - (u - 0.6) / 0.4
    }
}

#[macroquad::main("Pythagorean theorem")]
async fn main() {
    let (mut a, mut b) = (3.0_f32, 4.0_f32);
    let mut phase = 0.0_f32;
    loop {
        let dt = get_frame_time();
        phase += dt;
        if is_key_down(KeyCode::Right) {
            a = (a + dt).min(6.0);
        }
        if is_key_down(KeyCode::Left) {
            a = (a - dt).max(0.5);
        }
        if is_key_down(KeyCode::Up) {
            b = (b + dt).min(6.0);
        }
        if is_key_down(KeyCode::Down) {
            b = (b - dt).max(0.5);
        }
        let s = a + b;
        let t = cycle(phase);
        let side = screen_width().min(screen_height()) - 120.0;
        let k = side / s;
        let (ox, oy) = (60.0, 60.0 + side);
        let at = |p: [f32; 2]| vec2(ox + p[0] * k, oy - p[1] * k);

        clear_background(Color::from_rgba(22, 32, 28, 255));
        draw_rectangle(ox, oy - side, side, side, Color::from_rgba(227, 169, 59, 70));
        for tri in triangles(a, b, t) {
            let [p, q, r] = tri.map(at);
            draw_triangle(p, q, r, Color::from_rgba(22, 32, 28, 255));
            let blue = Color::from_rgba(157, 179, 255, 255);
            draw_triangle_lines(p, q, r, 1.5, blue);
        }
        if t < 0.5 {
            let c = at([s / 2.0, s / 2.0]);
            draw_text("c^2", c.x - 14.0, c.y, 30.0, WHITE);
        } else {
            let pa = at([a / 2.0, a / 2.0]);
            let pb = at([a + b / 2.0, a + b / 2.0]);
            draw_text("a^2", pa.x - 14.0, pa.y, 30.0, WHITE);
            draw_text("b^2", pb.x - 14.0, pb.y, 30.0, WHITE);
        }
        let text = format!("a = {a:.2}, b = {b:.2}:  a^2 + b^2 = {:.2} = c^2", a * a + b * b);
        draw_text(&text, 16.0, 30.0, 22.0, WHITE);
        draw_text("Left/Right: a   Up/Down: b", 16.0, screen_height() - 12.0, 18.0, GRAY);
        next_frame().await;
    }
}
