//! Drawn by circles: Fourier epicycles.
//!
//! x_N(t) = a0 + sum_{k=1..N} r_k cos(k t - phi_k)      (and the same for y)
//!
//! A hand-drawn outline (a cat's head) is resampled evenly, and each coordinate is
//! expanded into a Fourier series. Every harmonic is a circle spinning k times per turn;
//! chained largest first, the top chain's tip gives x and the left chain's tip gives y,
//! and where they meet the outline is drawn. Drag the slider to change the number of
//! terms N and watch the details melt away. Space plays or pauses.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;
use std::f32::consts::TAU;

const SAMPLES: usize = 256; // points taken along the outline
const MAX_TERMS: usize = 80; // highest harmonic computed

/// A closed outline, written by hand: a cat's head with two pointed ears (y up).
const OUTLINE: [[f32; 2]; 20] = [
    [0.0, -0.9], [0.35, -0.82], [0.62, -0.62], [0.78, -0.32], [0.82, 0.0], [0.78, 0.3], [0.7, 0.52],
    [0.82, 1.05], [0.4, 0.72], [0.2, 0.75], [0.0, 0.76], [-0.2, 0.75], [-0.4, 0.72], [-0.82, 1.05],
    [-0.7, 0.52], [-0.78, 0.3], [-0.82, 0.0], [-0.78, -0.32], [-0.62, -0.62], [-0.35, -0.82],
];

/// One harmonic: a circle of radius r turning k times per period.
#[derive(Clone, Copy)]
struct Term {
    k: usize,
    r: f32,
    phase: f32,
}

struct Series {
    mean: f32,
    terms: Vec<Term>,
}

impl Series {
    /// Partial sum with the first n harmonics at angle t.
    fn eval(&self, n: usize, t: f32) -> f32 {
        self.mean + self.terms[..n].iter().map(|q| q.r * (q.k as f32 * t - q.phase).cos()).sum::<f32>()
    }
}

/// m points evenly spaced by arc length along the closed polyline.
fn resample(poly: &[Vec2], m: usize) -> Vec<Vec2> {
    let mut p = poly.to_vec();
    p.push(poly[0]);
    let mut len = vec![0.0_f32];
    for i in 1..p.len() {
        len.push(len[i - 1] + p[i].distance(p[i - 1]));
    }
    let mut j = 1;
    (0..m)
        .map(|i| {
            let s = len[len.len() - 1] * i as f32 / m as f32;
            while j < len.len() - 1 && len[j] < s {
                j += 1;
            }
            let seg = len[j] - len[j - 1];
            p[j - 1].lerp(p[j], if seg > 0.0 { (s - len[j - 1]) / seg } else { 0.0 })
        })
        .collect()
}

/// Center the bounding box, scale the longer side to 1, and flip y for the screen.
fn normalize(p: &[Vec2]) -> Vec<Vec2> {
    let lo = p.iter().fold(Vec2::splat(f32::MAX), |a, &b| a.min(b));
    let hi = p.iter().fold(Vec2::splat(f32::MIN), |a, &b| a.max(b));
    let side = (hi - lo).max_element();
    p.iter().map(|&q| (q - (lo + hi) / 2.0) / side * vec2(1.0, -1.0)).collect()
}

/// DFT of evenly spaced samples: a cos kt + b sin kt = r cos(kt - phase).
fn fourier(v: &[f32], terms: usize) -> Series {
    let m = v.len() as f32;
    let terms = (1..=terms)
        .map(|k| {
            let (mut a, mut b) = (0.0_f32, 0.0_f32);
            for (j, x) in v.iter().enumerate() {
                let (s, c) = (TAU * k as f32 * j as f32 / m).sin_cos();
                a += x * c;
                b += x * s;
            }
            Term { k, r: 2.0 / m * a.hypot(b), phase: b.atan2(a) }
        })
        .collect();
    Series { mean: v.iter().sum::<f32>() / m, terms }
}

/// Draw one chain of circles, largest first. With swap, cosine drives the vertical axis.
fn chain(s: &Series, n: usize, t: f32, origin: Vec2, scale: f32, swap: bool, color: Color) -> Vec2 {
    let mut active = s.terms[..n].to_vec();
    active.sort_by(|a, b| b.r.total_cmp(&a.r));
    let mut p = origin;
    for q in active {
        let r = q.r * scale;
        let (sn, cs) = (q.k as f32 * t - q.phase).sin_cos();
        let d = if swap { vec2(r * sn, r * cs) } else { vec2(r * cs, r * sn) };
        if r > 0.6 {
            draw_circle_lines(p.x, p.y, r, 1.0, Color::new(color.r, color.g, color.b, 0.3));
            draw_line(p.x, p.y, p.x + d.x, p.y + d.y, 1.0, color);
        }
        p += d;
    }
    p
}

#[macroquad::main("Drawn by circles")]
async fn main() {
    let outline: Vec<Vec2> = OUTLINE.iter().map(|&[x, y]| vec2(x, y)).collect();
    let points = normalize(&resample(&outline, SAMPLES));
    let series_x = fourier(&points.iter().map(|p| p.x).collect::<Vec<_>>(), MAX_TERMS);
    let series_y = fourier(&points.iter().map(|p| p.y).collect::<Vec<_>>(), MAX_TERMS);
    let (scale, center) = (240.0, vec2(520.0, 340.0));
    let to_screen = |x: f32, y: f32| center + vec2(x, y) * scale;
    let (slider_x0, slider_x1, slider_y) = (150.0, 650.0, 572.0);
    let (mut target, mut n, mut grow, mut t, mut playing) = (40, 1, Some(0.0_f32), 0.0_f32, true);
    let (x_color, y_color) = (Color::from_rgba(239, 107, 91, 255), Color::from_rgba(91, 200, 138, 255));
    let trace = Color::from_rgba(240, 180, 41, 255);

    loop {
        let dt = get_frame_time().min(0.1);
        // Drag on the slider to pick N; it stops the grow animation.
        let (mx, my) = mouse_position();
        if is_mouse_button_down(MouseButton::Left) && (my - slider_y).abs() < 16.0 {
            let u = ((mx - slider_x0) / (slider_x1 - slider_x0)).clamp(0.0, 1.0);
            target = 1 + (u * (MAX_TERMS - 1) as f32).round() as usize;
            n = target;
            grow = None;
        }
        if let Some(elapsed) = grow.as_mut() {
            // Ease from 1 term up to the target over 1.8 s.
            *elapsed += dt;
            let u = (*elapsed / 1.8).min(1.0);
            n = (1.0 + (1.0 - (1.0 - u).powi(3)) * (target - 1) as f32).round() as usize;
            if u >= 1.0 {
                grow = None;
            }
        }
        if is_key_pressed(KeyCode::Space) {
            playing = !playing;
        }
        if playing {
            t = (t + 0.5 * dt) % TAU;
        }
        clear_background(Color::from_rgba(22, 32, 28, 255));

        // The original outline (faint dots) and the N-term curve, filled as a triangle fan.
        for p in points.iter().step_by(2) {
            let s = to_screen(p.x, p.y);
            draw_circle(s.x, s.y, 1.0, DARKGRAY);
        }
        let recon: Vec<Vec2> = (0..=SAMPLES)
            .map(|j| {
                let a = TAU * j as f32 / SAMPLES as f32;
                to_screen(series_x.eval(n, a), series_y.eval(n, a))
            })
            .collect();
        let middle = to_screen(series_x.mean, series_y.mean);
        for w in recon.windows(2) {
            draw_triangle(middle, w[0], w[1], Color::new(trace.r, trace.g, trace.b, 0.12));
        }
        let x_tip = chain(&series_x, n, t, to_screen(series_x.mean, -0.8), scale, false, x_color);
        let y_tip = chain(&series_y, n, t, to_screen(-0.85, series_y.mean), scale, true, y_color);
        let pen = vec2(x_tip.x, y_tip.y);
        draw_line(x_tip.x, x_tip.y, pen.x, pen.y, 1.0, Color::new(x_color.r, x_color.g, x_color.b, 0.6));
        draw_line(y_tip.x, y_tip.y, pen.x, pen.y, 1.0, Color::new(y_color.r, y_color.g, y_color.b, 0.6));
        let mut trail = recon[..=(t / TAU * SAMPLES as f32) as usize].to_vec();
        trail.push(pen);
        for w in trail.windows(2) {
            draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 2.2, trace);
        }
        draw_circle(pen.x, pen.y, 3.5, WHITE);

        // The terms slider.
        let knob = slider_x0 + (n - 1) as f32 / (MAX_TERMS - 1) as f32 * (slider_x1 - slider_x0);
        draw_line(slider_x0, slider_y, slider_x1, slider_y, 6.0, Color::from_rgba(44, 58, 52, 255));
        draw_line(slider_x0, slider_y, knob, slider_y, 6.0, trace);
        draw_text("Terms N", 60.0, slider_y + 6.0, 20.0, WHITE);
        draw_text(&n.to_string(), 690.0, slider_y + 11.0, 40.0, trace);
        draw_text("drag the slider   Space: play/pause", 16.0, 26.0, 20.0, GRAY);
        next_frame().await;
    }
}
