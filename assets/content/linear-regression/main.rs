//! Line of best fit (least squares):
//!   m = sum((x - mean_x) * (y - mean_y)) / sum((x - mean_x)^2),   b = mean_y - m * mean_x.
//!
//! The line through the mean point that makes the squared vertical misses (residuals)
//! as small as possible. Drag points, click to add one, right-click to remove one.
//! Keys: R residuals, S squared errors (drawn as real squares), N new data,
//! Up/Down change the noise of new data.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

struct Fit {
    slope: f32,
    intercept: f32,
    mean: Vec2,
    r2: f32,
}

impl Fit {
    fn predict(&self, x: f32) -> f32 {
        self.slope * x + self.intercept
    }
}

fn sum_squared_residuals(points: &[Vec2], m: f32, b: f32) -> f32 {
    points.iter().map(|p| (p.y - (m * p.x + b)).powi(2)).sum()
}

/// Least-squares fit; None with fewer than two points or when every x is the same.
fn fit(points: &[Vec2]) -> Option<Fit> {
    if points.len() < 2 {
        return None;
    }
    let mean = points.iter().fold(Vec2::ZERO, |s, &p| s + p) / points.len() as f32;
    let sxy: f32 = points.iter().map(|p| (p.x - mean.x) * (p.y - mean.y)).sum();
    let sxx: f32 = points.iter().map(|p| (p.x - mean.x).powi(2)).sum();
    if sxx <= f32::EPSILON {
        return None; // a vertical line has no slope
    }
    let slope = sxy / sxx;
    let intercept = mean.y - slope * mean.x;
    let sst: f32 = points.iter().map(|p| (p.y - mean.y).powi(2)).sum();
    let r2 = if sst > 0.0 { 1.0 - sum_squared_residuals(points, slope, intercept) / sst } else { 1.0 };
    Some(Fit { slope, intercept, mean, r2 })
}

const SPAN: f32 = 10.0; // data space is 0..10 on both axes
const LEFT: f32 = 60.0;
const TOP: f32 = 40.0;
const SIZE: f32 = 480.0;

fn to_screen(p: Vec2) -> Vec2 {
    vec2(LEFT + p.x / SPAN * SIZE, TOP + SIZE - p.y / SPAN * SIZE)
}

fn to_data(s: Vec2) -> Vec2 {
    vec2((s.x - LEFT) / SIZE * SPAN, (TOP + SIZE - s.y) / SIZE * SPAN).clamp(Vec2::ZERO, Vec2::splat(SPAN))
}

fn new_data(noise: f32) -> Vec<Vec2> {
    let slope = rand::gen_range(-0.5, 0.9);
    let intercept = 5.0 - slope * 5.0;
    // The sum of three uniforms is a cheap bell curve.
    let bell = || rand::gen_range(0.0, 1.0) + rand::gen_range(0.0, 1.0) + rand::gen_range(0.0, 1.0) - 1.5;
    (0..18)
        .map(|_| {
            let x = rand::gen_range(0.6, 9.4);
            vec2(x, (slope * x + intercept + bell() * 2.0 * noise).clamp(0.2, 9.8))
        })
        .collect()
}

fn line(a: Vec2, b: Vec2, width: f32, color: Color) {
    draw_line(a.x, a.y, b.x, b.y, width, color);
}

#[macroquad::main("Line of best fit")]
async fn main() {
    rand::srand(9);
    let mut noise = 1.2;
    let mut points = new_data(noise);
    let (mut show_residuals, mut show_squares) = (true, false);
    let mut dragging: Option<usize> = None;
    let amber = Color::from_rgba(240, 180, 90, 255);
    let rose = Color::from_rgba(229, 112, 126, 255);

    loop {
        let mouse = Vec2::from(mouse_position());
        let picked = points.iter().position(|&p| to_screen(p).distance(mouse) < 14.0);
        let in_plot = mouse.x >= LEFT && mouse.x <= LEFT + SIZE && mouse.y >= TOP && mouse.y <= TOP + SIZE;
        if is_mouse_button_pressed(MouseButton::Left) {
            dragging = picked;
            if picked.is_none() && in_plot {
                points.push(to_data(mouse));
            }
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some(k) = picked {
                points.remove(k);
                dragging = None;
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            dragging = None;
        }
        if let Some(k) = dragging {
            points[k] = to_data(mouse);
        }
        if is_key_pressed(KeyCode::R) {
            show_residuals = !show_residuals;
        }
        if is_key_pressed(KeyCode::S) {
            show_squares = !show_squares;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Down) {
            noise = (noise + if is_key_pressed(KeyCode::Up) { 0.25 } else { -0.25 }).clamp(0.0, 3.0);
            points = new_data(noise);
        }
        if is_key_pressed(KeyCode::N) {
            points = new_data(noise);
        }

        clear_background(Color::from_rgba(22, 32, 28, 255));
        for k in 0..=10 {
            let v = k as f32;
            let (a, b) = (to_screen(vec2(v, 0.0)), to_screen(vec2(0.0, v)));
            line(a, to_screen(vec2(v, SPAN)), 1.0, Color::from_rgba(36, 48, 43, 255));
            line(b, to_screen(vec2(SPAN, v)), 1.0, Color::from_rgba(36, 48, 43, 255));
            if k % 2 == 0 {
                draw_text(&k.to_string(), a.x - 4.0, a.y + 18.0, 18.0, GRAY);
                draw_text(&k.to_string(), b.x - 24.0, b.y + 5.0, 18.0, GRAY);
            }
        }

        let result = fit(&points);
        let mut lines: Vec<String> = Vec::new();
        if let Some(f) = &result {
            for &p in &points {
                let (a, b) = (to_screen(p), to_screen(vec2(p.x, f.predict(p.x))));
                if show_squares {
                    // The square of each residual, drawn as an actual square.
                    let side = b.y - a.y;
                    let x = if f.slope >= 0.0 { a.x - side.abs() } else { a.x };
                    draw_rectangle(x, a.y.min(b.y), side.abs(), side.abs(), rose.with_alpha(0.12));
                    draw_rectangle_lines(x, a.y.min(b.y), side.abs(), side.abs(), 1.0, rose.with_alpha(0.5));
                }
                if show_residuals {
                    line(a, b, 2.0, rose);
                }
            }
            // The fitted line always passes through the mean point; clip it to the plot.
            let mean = to_screen(f.mean);
            line(vec2(mean.x, TOP), vec2(mean.x, TOP + SIZE), 1.0, Color::new(0.35, 0.71, 0.90, 0.5));
            line(vec2(LEFT, mean.y), vec2(LEFT + SIZE, mean.y), 1.0, rose.with_alpha(0.5));
            let x0 = ((0.0 - f.intercept) / f.slope).clamp(0.0, SPAN);
            let x1 = ((SPAN - f.intercept) / f.slope).clamp(0.0, SPAN);
            let (lo, hi) = if f.slope.abs() < 1e-6 { (0.0, SPAN) } else { (x0.min(x1), x0.max(x1)) };
            line(to_screen(vec2(lo, f.predict(lo))), to_screen(vec2(hi, f.predict(hi))), 3.0, amber);
            draw_circle(mean.x, mean.y, 6.0, Color::from_rgba(22, 32, 28, 255));
            draw_circle_lines(mean.x, mean.y, 6.0, 2.5, amber);
            draw_text("(mean x, mean y)", mean.x + 9.0, mean.y - 9.0, 18.0, amber);
            let sign = if f.intercept < 0.0 { "-" } else { "+" };
            lines.push(format!("y = {:.2}x {sign} {:.2}", f.slope, f.intercept.abs()));
            lines.push(format!("R^2 = {:.2}", f.r2));
            let sse = sum_squared_residuals(&points, f.slope, f.intercept);
            lines.push(format!("sum residual^2 = {sse:.1}"));
        } else {
            lines.extend(["add points with", "different x", "to fit a line"].map(String::from));
        }
        for (k, p) in points.iter().enumerate() {
            let s = to_screen(*p);
            draw_circle(s.x, s.y, if dragging == Some(k) { 7.0 } else { 5.0 }, WHITE);
        }

        // Side panel: the fit, then the controls.
        let on_off = |b: bool| if b { "on" } else { "off" };
        lines.push(format!("n = {}", points.len()));
        lines.push(String::new());
        lines.push(format!("[R] residuals {}", on_off(show_residuals)));
        lines.push(format!("[S] squared errors {}", on_off(show_squares)));
        lines.push("[N] new data".to_string());
        lines.push(format!("[Up/Down] noise {noise:.2}"));
        let help = ["", "drag: move a point", "click: add a point", "right-click: remove"];
        lines.extend(help.map(String::from));
        for (i, text) in lines.iter().enumerate() {
            draw_text(text, 565.0, 70.0 + 26.0 * i as f32, 20.0, if i < 4 { WHITE } else { GRAY });
        }
        next_frame().await;
    }
}
