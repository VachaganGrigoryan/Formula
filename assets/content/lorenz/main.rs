//! Lorenz attractor: dx/dt = sigma (y - x), dy/dt = x (rho - z) - y, dz/dt = x y - beta z.
//!
//! Two paths start 0.00001 apart and are stepped forward with RK4. Their 3D trails are
//! projected onto the screen over a faint trace of the whole attractor while the view
//! slowly spins; at first they overlap, then they split onto different lobes. The inset
//! plots log10 of the distance between them. Drag to orbit the view.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use std::collections::VecDeque;

use macroquad::prelude::*;

type V3 = [f64; 3];

const SIGMA: f64 = 10.0; // Lorenz's own values from 1963
const RHO: f64 = 28.0;
const BETA: f64 = 8.0 / 3.0;
const H: f64 = 0.004; // integration step (model time)
const SPEED: f64 = 1.0; // model time units per second
const TRAIL: usize = 2500; // points kept per path
const PLOT_SPAN: f64 = 40.0; // time covered by the separation plot

/// The right-hand side: how fast x, y and z are changing at state s.
fn derivative([x, y, z]: V3) -> V3 {
    [SIGMA * (y - x), x * (RHO - z) - y, x * y - BETA * z]
}

/// One fourth-order Runge-Kutta step of size h.
fn rk4(s: V3, h: f64) -> V3 {
    let add = |a: V3, b: V3, k: f64| [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k];
    let k1 = derivative(s);
    let k2 = derivative(add(s, k1, h / 2.0));
    let k3 = derivative(add(s, k2, h / 2.0));
    let k4 = derivative(add(s, k3, h));
    std::array::from_fn(|i| s[i] + h / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]))
}

fn distance(a: V3, b: V3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Orthographic view: spin about the vertical z axis, then tilt toward the viewer.
fn project([x, y, z]: V3, yaw: f32, pitch: f32) -> Vec2 {
    let (x, y, z) = (x as f32, y as f32, (z - 0.85 * RHO) as f32);
    let (sy, cy) = yaw.sin_cos();
    let (x1, depth) = (x * cy - y * sy, x * sy + y * cy);
    let (sp, cp) = pitch.sin_cos();
    let scale = screen_height().min(screen_width() * 0.9) / (1.85 * RHO as f32);
    let center = vec2(screen_width(), screen_height()) / 2.0;
    center + vec2(x1, -(z * cp - depth * sp)) * scale
}

fn polyline(points: impl Iterator<Item = Vec2>, thickness: f32, color: Color) {
    let mut prev: Option<Vec2> = None;
    for p in points {
        if let Some(q) = prev {
            draw_line(q.x, q.y, p.x, p.y, thickness, color);
        }
        prev = Some(p);
    }
}

#[macroquad::main("Lorenz attractor")]
async fn main() {
    // The attractor itself, traced once in advance as a faint backdrop.
    let mut backdrop = vec![[1.0, 1.0, 20.0]];
    for _ in 0..8000 {
        backdrop.push(rk4(*backdrop.last().unwrap(), H));
    }
    backdrop.drain(..1000);

    let start: V3 = [-8.0, 8.0, 27.0];
    let mut paths = [
        VecDeque::from([start]),
        VecDeque::from([[start[0] + 1e-5, start[1], start[2]]]),
    ];
    let colors = [
        Color::from_rgba(77, 204, 230, 255),
        Color::from_rgba(255, 140, 77, 255),
    ];
    let mut separation: Vec<(f64, f64)> = Vec::new(); // (time, log10 distance)
    let (mut time, mut carry) = (0.0f64, 0.0f64);
    let (mut yaw, mut pitch, mut spinning) = (-0.8f32, 0.25f32, true); // wings face the viewer
    let mut last_mouse = Vec2::from(mouse_position());

    loop {
        let dt = get_frame_time().min(0.1);
        let mouse = Vec2::from(mouse_position());
        if is_mouse_button_down(MouseButton::Left) {
            let d = mouse - last_mouse;
            yaw += d.x * 0.01;
            pitch = (pitch + d.y * 0.01).clamp(-1.5, 1.5);
            if d != Vec2::ZERO {
                spinning = false;
            }
        }
        last_mouse = mouse;
        if spinning {
            yaw += 0.12 * dt;
        }

        carry += SPEED * dt as f64 / H;
        while carry >= 1.0 {
            carry -= 1.0;
            for trail in &mut paths {
                trail.push_back(rk4(*trail.back().unwrap(), H));
                if trail.len() > TRAIL {
                    trail.pop_front();
                }
            }
            time += H;
            let d = distance(*paths[0].back().unwrap(), *paths[1].back().unwrap());
            let due = separation.last().is_none_or(|&(t, _)| time - t >= 0.05);
            if due && time <= PLOT_SPAN {
                separation.push((time, d.max(1e-12).log10()));
            }
        }

        clear_background(Color::from_rgba(16, 22, 26, 255));
        let ghost = Color::new(0.63, 0.67, 0.69, 0.12);
        polyline(backdrop.iter().map(|&p| project(p, yaw, pitch)), 0.7, ghost);

        // Each trail fades toward its tail.
        for (trail, &color) in paths.iter().zip(&colors) {
            let n = trail.len();
            let pts: Vec<Vec2> = trail.iter().map(|&p| project(p, yaw, pitch)).collect();
            for (k, w) in pts.windows(2).enumerate() {
                let faded = Color {
                    a: 0.1 + 0.9 * (k + 1) as f32 / n as f32,
                    ..color
                };
                draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 1.4, faded);
            }
            let head = pts[n - 1];
            draw_circle(head.x, head.y, 4.0, color);
        }

        // Readout for the first path, and the separation plot (log10 distance, 1e-6 to 1e2).
        let [x, y, z] = *paths[0].back().unwrap();
        let text_color = Color::from_rgba(200, 214, 208, 255);
        let rows = [
            format!("x = {x:.2}"),
            format!("y = {y:.2}"),
            format!("z = {z:.2}"),
            format!("t = {time:.2}"),
            "drag to orbit".to_string(),
        ];
        for (k, row) in rows.iter().enumerate() {
            draw_text(row, 14.0, 24.0 + 18.0 * k as f32, 20.0, text_color);
        }
        let (bx, by, bw, bh) = (14.0, screen_height() - 104.0, 240.0, 70.0);
        draw_rectangle(bx, by, bw, bh, Color::new(1.0, 1.0, 1.0, 0.04));
        draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::new(0.78, 0.84, 0.82, 0.3));
        let at = |t: f64, l: f64| {
            vec2(
                bx + (t / PLOT_SPAN) as f32 * bw,
                by + bh - ((l.clamp(-6.0, 2.0) + 6.0) / 8.0) as f32 * bh,
            )
        };
        polyline(separation.iter().map(|&(t, l)| at(t, l)), 1.5, colors[1]);
        draw_text(
            "distance between the paths (log scale)",
            bx,
            by + bh + 20.0,
            18.0,
            text_color,
        );
        next_frame().await;
    }
}
