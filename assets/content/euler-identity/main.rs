//! Euler's formula: e^(i theta) = cos(theta) + i sin(theta).
//!
//! The Taylor series 1 + i theta + (i theta)^2/2! + ... drawn as arrows placed head to
//! tail. Each term is the previous one turned a quarter turn and scaled by theta / k, so
//! the partial sums spiral in to the point at angle theta on the unit circle. At
//! theta = pi the point is -1: e^(i pi) + 1 = 0.
//! Left/Right change theta, Up/Down the number of terms, P jumps to theta = pi.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

/// S_0 ... S_n of sum (i theta)^k / k!, each term from the last one.
fn partial_sums(theta: f32, n: usize) -> Vec<Vec2> {
    let mut term = vec2(1.0, 0.0);
    let mut sum = term;
    let mut out = vec![sum];
    for k in 1..=n {
        let s = theta / k as f32;
        term = vec2(-term.y * s, term.x * s);
        sum += term;
        out.push(sum);
    }
    out
}

fn arrow(from: Vec2, to: Vec2, color: Color) {
    draw_line(from.x, from.y, to.x, to.y, 2.0, color);
    let d = to - from;
    if d.length() > 4.0 {
        let u = d.normalize();
        let n = vec2(-u.y, u.x);
        draw_triangle(to, to - u * 8.0 + n * 3.5, to - u * 8.0 - n * 3.5, color);
    }
}

#[macroquad::main("Euler's formula")]
async fn main() {
    let (mut theta, mut terms) = (2.0_f32, 8usize);
    loop {
        let dt = get_frame_time();
        if is_key_down(KeyCode::Right) {
            theta = (theta + dt).min(TAU);
        }
        if is_key_down(KeyCode::Left) {
            theta = (theta - dt).max(0.0);
        }
        if is_key_pressed(KeyCode::Up) {
            terms = (terms + 1).min(30);
        }
        if is_key_pressed(KeyCode::Down) {
            terms = terms.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::P) {
            theta = PI;
        }
        let sums = partial_sums(theta, terms);
        let reach = sums.iter().map(|p| p.length() * 1.1).fold(1.2, f32::max);
        let center = vec2(screen_width() / 2.0, screen_height() / 2.0);
        let k = screen_width().min(screen_height()) * 0.45 / reach;
        let at = |p: Vec2| center + vec2(p.x * k, -p.y * k);

        clear_background(Color::from_rgba(22, 32, 28, 255));
        draw_circle_lines(center.x, center.y, k, 1.0, Color::from_rgba(85, 99, 93, 255));
        let mut from = center;
        for (i, s) in sums.iter().enumerate() {
            let to = at(*s);
            let color = if i % 2 == 0 { Color::from_rgba(92, 194, 178, 255) } else { Color::from_rgba(227, 169, 59, 255) };
            arrow(from, to, color);
            from = to;
        }
        let target = at(vec2(theta.cos(), theta.sin()));
        draw_circle(target.x, target.y, 6.0, Color::from_rgba(157, 179, 255, 255));
        let last = sums[sums.len() - 1];
        let text = format!("theta = {theta:.3}   sum of {} terms = ({:+.4}, {:+.4})", terms + 1, last.x, last.y);
        draw_text(&text, 16.0, 26.0, 22.0, WHITE);
        draw_text("Left/Right: theta   Up/Down: terms   P: theta = pi", 16.0, screen_height() - 12.0, 18.0, GRAY);
        next_frame().await;
    }
}
