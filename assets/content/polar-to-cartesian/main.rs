//! Polar to Cartesian: x = r cos(theta), y = r sin(theta).
//!
//! A point given by a radius r and an angle theta, with the x and y components it
//! produces drawn along the axes while theta sweeps around. Press M to switch to the
//! rose curve r = cos(k theta), drawn one angle at a time (Up/Down change k).
//! Click or drag to place the point (this uses the inverse, atan2 and hypot);
//! Space plays or pauses.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

/// (r, theta) -> (x, y), theta in radians, counterclockwise from the +x axis.
fn to_cartesian(r: f32, theta: f32) -> Vec2 {
    vec2(r * theta.cos(), r * theta.sin())
}

/// (x, y) -> (r, theta) with theta in [0, 2 pi).
fn to_polar(x: f32, y: f32) -> (f32, f32) {
    (x.hypot(y), y.atan2(x).rem_euclid(TAU))
}

#[derive(PartialEq)]
enum Mode {
    Point,
    Rose,
}

const CENTER: Vec2 = vec2(400.0, 270.0);
const SCALE: f32 = 220.0;

/// Math space (y up) to screen space (y down).
fn to_screen(p: Vec2) -> Vec2 {
    vec2(CENTER.x + p.x * SCALE, CENTER.y - p.y * SCALE)
}

fn polyline(points: &[Vec2], thickness: f32, color: Color) {
    for w in points.windows(2) {
        draw_line(w[0].x, w[0].y, w[1].x, w[1].y, thickness, color);
    }
}

fn dashed(a: Vec2, b: Vec2, color: Color) {
    let n = (a.distance(b) / 8.0) as usize;
    for i in 0..n {
        let (p, q) = (a.lerp(b, i as f32 / n as f32), a.lerp(b, (i as f32 + 0.5) / n as f32));
        draw_line(p.x, p.y, q.x, q.y, 1.0, color);
    }
}

#[macroquad::main("Polar to Cartesian")]
async fn main() {
    let (mut mode, mut r, mut theta, mut k, mut playing) = (Mode::Point, 0.8_f32, 0.75_f32, 5, true);
    let (x_color, y_color) = (Color::from_rgba(239, 107, 91, 255), Color::from_rgba(91, 200, 138, 255));
    let (trace, grid) = (Color::from_rgba(240, 180, 41, 255), Color::from_rgba(44, 58, 52, 255));

    loop {
        if is_key_pressed(KeyCode::Space) {
            playing = !playing;
        }
        if is_key_pressed(KeyCode::M) {
            mode = if mode == Mode::Point { Mode::Rose } else { Mode::Point };
        }
        if is_key_pressed(KeyCode::Up) {
            k = (k + 1).min(9);
        }
        if is_key_pressed(KeyCode::Down) {
            k = (k - 1).max(1);
        }
        // Click or drag: the inverse conversion turns the pointer into (r, theta).
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (pr, pt) = to_polar((mx - CENTER.x) / SCALE, (CENTER.y - my) / SCALE);
            theta = pt;
            if mode == Mode::Point {
                r = pr.min(1.0);
            }
            playing = false;
        }
        if playing {
            theta = (theta + 0.45 * get_frame_time()).rem_euclid(TAU);
        }
        // The radius actually plotted: free in point mode, cos(k theta) on the rose.
        let radius = |a: f32| if mode == Mode::Point { r } else { (k as f32 * a).cos() };

        clear_background(Color::from_rgba(22, 32, 28, 255));

        // Polar grid: rings of equal r, spokes of equal theta every 30 degrees.
        for ring in 1..=4 {
            draw_circle_lines(CENTER.x, CENTER.y, SCALE * ring as f32 / 4.0, 1.0, grid);
        }
        for spoke in 0..12 {
            let a = spoke as f32 * PI / 6.0;
            let end = to_screen(to_cartesian(1.0, a));
            draw_line(CENTER.x, CENTER.y, end.x, end.y, 1.0, grid);
            if spoke % 3 == 0 {
                let at = to_screen(to_cartesian(1.12, a));
                draw_text(&format!("{}", spoke * 30), at.x - 10.0, at.y + 5.0, 18.0, GRAY);
            }
        }

        if mode == Mode::Rose {
            let steps = 720;
            let full: Vec<Vec2> = (0..=steps)
                .map(|j| {
                    let a = TAU * j as f32 / steps as f32;
                    to_screen(to_cartesian(radius(a), a))
                })
                .collect();
            for w in full.windows(2).step_by(2) {
                draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 1.0, DARKGRAY);
            }
            let drawn = (theta / TAU * steps as f32).ceil() as usize;
            let trail: Vec<Vec2> = (0..=drawn)
                .map(|j| {
                    let a = (TAU * j as f32 / steps as f32).min(theta);
                    to_screen(to_cartesian(radius(a), a))
                })
                .collect();
            polyline(&trail, 2.2, trace);
        }

        let rad = radius(theta);
        let p = to_cartesian(rad, theta);
        let s = to_screen(p);
        let (foot_x, foot_y) = (to_screen(vec2(p.x, 0.0)), to_screen(vec2(0.0, p.y)));

        // The angle, swept from the positive x axis.
        let arc: Vec<Vec2> = (0..=48)
            .map(|j| to_screen(to_cartesian(0.16, theta * j as f32 / 48.0)))
            .collect();
        polyline(&arc, 1.5, LIGHTGRAY);

        // The two components the conversion produces, and the radius itself.
        draw_line(CENTER.x, CENTER.y, foot_x.x, foot_x.y, 4.0, x_color);
        draw_line(CENTER.x, CENTER.y, foot_y.x, foot_y.y, 4.0, y_color);
        dashed(s, foot_x, x_color);
        dashed(s, foot_y, y_color);
        draw_line(CENTER.x, CENTER.y, s.x, s.y, 2.0, trace);
        draw_circle(s.x, s.y, 6.5, trace);

        let deg = theta.to_degrees();
        draw_text(&format!("r = {rad:+.2}   theta = {deg:.0} deg"), 20.0, 530.0, 22.0, WHITE);
        draw_text(&format!("x = r cos(theta) = {:+.2}", p.x), 20.0, 552.0, 22.0, x_color);
        draw_text(&format!("y = r sin(theta) = {:+.2}", p.y), 20.0, 574.0, 22.0, y_color);
        let hint = if mode == Mode::Point { "M: rose".to_string() } else { format!("M: point  Up/Down: k = {k}") };
        draw_text(&format!("drag to place  Space: play  {hint}"), 400.0, 574.0, 18.0, GRAY);
        next_frame().await;
    }
}
