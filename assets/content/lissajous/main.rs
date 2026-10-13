//! Lissajous figures: x(t) = A sin(a t + delta pi),  y(t) = B sin(b t).
//!
//! One sine wave drives the horizontal position and another the vertical one. The dots
//! on the top and left tracks show each wave alone; the point they meet at traces the
//! figure. delta drifts slowly forward, so the knot morphs and seems to turn in 3D.
//! Left/Right change a, Up/Down change b, drag sideways to scrub delta, D toggles drift,
//! Space plays or pauses.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

/// The figure's parameters: amplitudes, frequencies, and the phase of x in units of pi.
struct Lissajous {
    amp_x: f32,
    amp_y: f32,
    a: u32,
    b: u32,
    delta: f32,
}

impl Lissajous {
    fn point(&self, t: f32) -> Vec2 {
        vec2(
            self.amp_x * (self.a as f32 * t + self.delta * PI).sin(),
            self.amp_y * (self.b as f32 * t).sin(),
        )
    }
}

fn polyline(points: &[Vec2], thickness: f32, color: Color) {
    for w in points.windows(2) {
        draw_line(w[0].x, w[0].y, w[1].x, w[1].y, thickness, color);
    }
}

fn dashed(a: Vec2, b: Vec2, color: Color) {
    let n = (a.distance(b) / 10.0).max(1.0) as usize;
    for i in 0..n {
        let (p, q) = (a.lerp(b, i as f32 / n as f32), a.lerp(b, (i as f32 + 0.5) / n as f32));
        draw_line(p.x, p.y, q.x, q.y, 1.0, color);
    }
}

#[macroquad::main("Lissajous figures")]
async fn main() {
    let mut fig = Lissajous { amp_x: 1.0, amp_y: 1.0, a: 3, b: 2, delta: 0.5 };
    let (mut t, mut playing, mut drift) = (0.0_f32, true, true);
    let drift_rate = 0.05; // delta change per second, in units of pi
    let mut last_mouse = mouse_position().0;
    let (x_color, y_color) = (Color::from_rgba(239, 107, 91, 255), Color::from_rgba(91, 200, 138, 255));
    let trace = Color::from_rgba(240, 180, 41, 255);

    // Square plot area with gutters on the top and left for the two oscillators.
    let (gutter, plot_x, plot_y, side) = (40.0, 230.0, 70.0, 440.0);
    let to_screen = |p: Vec2| vec2(plot_x + side / 2.0 + p.x * side * 0.45, plot_y + side / 2.0 - p.y * side * 0.45);

    loop {
        let dt = get_frame_time();
        if is_key_pressed(KeyCode::Left) {
            fig.a = fig.a.saturating_sub(1).max(1);
        }
        if is_key_pressed(KeyCode::Right) {
            fig.a = (fig.a + 1).min(9);
        }
        if is_key_pressed(KeyCode::Down) {
            fig.b = fig.b.saturating_sub(1).max(1);
        }
        if is_key_pressed(KeyCode::Up) {
            fig.b = (fig.b + 1).min(9);
        }
        if is_key_pressed(KeyCode::D) {
            drift = !drift;
        }
        if is_key_pressed(KeyCode::Space) {
            playing = !playing;
        }
        // Drag sideways to scrub the phase; drift pauses while dragging.
        let mouse_x = mouse_position().0;
        let dragging = is_mouse_button_down(MouseButton::Left);
        if dragging {
            fig.delta = (fig.delta + (mouse_x - last_mouse) / 300.0).rem_euclid(2.0);
        }
        last_mouse = mouse_x;
        if playing {
            t = (t + dt).rem_euclid(TAU);
        }
        if drift && !dragging {
            fig.delta = (fig.delta + drift_rate * dt).rem_euclid(2.0);
        }

        clear_background(Color::from_rgba(22, 32, 28, 255));
        draw_rectangle_lines(plot_x, plot_y, side, side, 1.0, Color::from_rgba(44, 58, 52, 255));

        // The whole closed figure faintly, and the path traced so far brightly.
        let samples = 720;
        let full: Vec<Vec2> = (0..=samples).map(|j| to_screen(fig.point(TAU * j as f32 / samples as f32))).collect();
        for w in full.windows(2).step_by(2) {
            draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 1.0, DARKGRAY);
        }
        let tip = to_screen(fig.point(t));
        let mut trail = full[..=(t / TAU * samples as f32) as usize].to_vec();
        trail.push(tip);
        polyline(&trail, 2.2, trace);

        // Each oscillator on its own track, and how they combine into the moving point.
        let x_marker = vec2(tip.x, plot_y - gutter / 2.0);
        let y_marker = vec2(plot_x - gutter / 2.0, tip.y);
        draw_line(plot_x, x_marker.y, plot_x + side, x_marker.y, 1.0, Color::new(x_color.r, x_color.g, x_color.b, 0.3));
        draw_line(y_marker.x, plot_y, y_marker.x, plot_y + side, 1.0, Color::new(y_color.r, y_color.g, y_color.b, 0.3));
        dashed(x_marker, tip, x_color);
        dashed(y_marker, tip, y_color);
        draw_circle(x_marker.x, x_marker.y, 5.0, x_color);
        draw_circle(y_marker.x, y_marker.y, 5.0, y_color);
        draw_circle(tip.x, tip.y, 4.0, WHITE);

        draw_text(&format!("{}:{}", fig.a, fig.b), 700.0, 580.0, 44.0, trace);
        let x_text = format!("x(t) = {:.1} sin({}t + {:.2} pi)", fig.amp_x, fig.a, fig.delta);
        draw_text(&x_text, 16.0, 548.0, 22.0, WHITE);
        draw_text(&format!("y(t) = {:.1} sin({}t)", fig.amp_y, fig.b), 16.0, 570.0, 22.0, WHITE);
        let hint = format!("Left/Right: a   Up/Down: b   drag: delta   D: drift {}", if drift { "on" } else { "off" });
        draw_text(&hint, 16.0, 22.0, 18.0, GRAY);
        next_frame().await;
    }
}
