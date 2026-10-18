//! Quadratic formula: where a x^2 + b x + c = 0 crosses the x axis.
//!
//! The parabola y = a x^2 + b x + c with its real roots, its vertex and its axis of
//! symmetry. The discriminant b^2 - 4ac decides how many roots there are: two when it
//! is positive, one double root when it is zero, none when it is negative.
//! Tab picks a, b or c and Up/Down change it.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const EPS: f64 = 1e-12;

/// Real roots, smaller first, computed without cancellation.
fn roots(a: f64, b: f64, c: f64) -> Vec<f64> {
    if a.abs() < EPS {
        return if b.abs() < EPS { vec![] } else { vec![-c / b] };
    }
    let d = b * b - 4.0 * a * c;
    if d.abs() < EPS {
        return vec![-b / (2.0 * a)];
    }
    if d < 0.0 {
        return vec![];
    }
    let q = -0.5 * (b + b.signum() * d.sqrt());
    let (x1, x2) = (q / a, c / q);
    vec![x1.min(x2), x1.max(x2)]
}

#[macroquad::main("Quadratic formula")]
async fn main() {
    let mut coef = [1.0_f64, -1.0, -2.0];
    let names = ["a", "b", "c"];
    let mut selected = 0usize;
    let scale = 60.0_f64;

    loop {
        if is_key_pressed(KeyCode::Tab) {
            selected = (selected + 1) % 3;
        }
        if is_key_down(KeyCode::Up) {
            coef[selected] = (coef[selected] + 0.02).min(5.0);
        }
        if is_key_down(KeyCode::Down) {
            coef[selected] = (coef[selected] - 0.02).max(-5.0);
        }
        let [a, b, c] = coef;
        let (w, h) = (screen_width() as f64, screen_height() as f64);
        let to_screen = |x: f64, y: f64| vec2((w / 2.0 + x * scale) as f32, (h / 2.0 - y * scale) as f32);

        clear_background(Color::from_rgba(22, 32, 28, 255));
        let axis = Color::from_rgba(85, 99, 93, 255);
        draw_line(0.0, (h / 2.0) as f32, w as f32, (h / 2.0) as f32, 1.0, axis);
        draw_line((w / 2.0) as f32, 0.0, (w / 2.0) as f32, h as f32, 1.0, axis);

        let f = |x: f64| a * x * x + b * x + c;
        let trace = Color::from_rgba(157, 179, 255, 255);
        for px in 0..w as usize {
            let (x0, x1) = ((px as f64 - w / 2.0) / scale, (px as f64 + 1.0 - w / 2.0) / scale);
            let (p, q) = (to_screen(x0, f(x0)), to_screen(x1, f(x1)));
            draw_line(p.x, p.y, q.x, q.y, 2.4, trace);
        }
        if a.abs() >= EPS {
            let vx = -b / (2.0 * a);
            let v = to_screen(vx, f(vx));
            let gold = Color::from_rgba(227, 169, 59, 255);
            draw_line(v.x, 0.0, v.x, h as f32, 1.0, gold);
            draw_circle(v.x, v.y, 4.5, gold);
        }
        let found = roots(a, b, c);
        for &x in &found {
            let p = to_screen(x, 0.0);
            draw_circle(p.x, p.y, 6.0, Color::from_rgba(92, 194, 178, 255));
        }

        let d = b * b - 4.0 * a * c;
        let text = if found.is_empty() {
            "no real roots".to_string()
        } else {
            found.iter().map(|x| format!("x = {x:.3}")).collect::<Vec<_>>().join(", ")
        };
        draw_text(&format!("discriminant = {d:.2}: {text}"), 16.0, 24.0, 22.0, WHITE);
        for (i, name) in names.iter().enumerate() {
            let mark = if i == selected { ">" } else { " " };
            draw_text(&format!("{mark} {name} = {:.2}", coef[i]), 16.0, 52.0 + 20.0 * i as f32, 20.0, WHITE);
        }
        draw_text("Tab: next coefficient   Up/Down: change it", 16.0, h as f32 - 12.0, 18.0, GRAY);
        next_frame().await;
    }
}
