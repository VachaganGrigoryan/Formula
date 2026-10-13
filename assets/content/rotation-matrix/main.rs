//! 2D rotation: x' = x cos(theta) - y sin(theta),  y' = x sin(theta) + y cos(theta).
//!
//! Top: a letter F spins about the origin; the colored arrows are the rotated axes,
//! (cos, sin) and (-sin, cos), which are the columns of the rotation matrix. Drag P and
//! watch its image P' travel the dashed arc. Space pauses the spin.
//! Bottom: the same geometry tilts crowded month labels. Drag the slider to set the tilt;
//! labels that still collide with a neighbor turn red.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;
use std::f32::consts::PI;

/// Rotate p counter-clockwise about the origin (math axes: y points up).
fn rotate(p: Vec2, theta: f32) -> Vec2 {
    let (s, c) = theta.sin_cos();
    vec2(p.x * c - p.y * s, p.x * s + p.y * c)
}

/// Two equal labels `spacing` apart and tilted by `angle` overlap when, in their own
/// rotated frame, the offset (spacing cos a, spacing sin a) fits inside the box.
fn labels_collide(spacing: f32, width: f32, height: f32, angle: f32) -> bool {
    let (s, c) = angle.sin_cos();
    spacing * c < width && spacing * s < height
}

const F: [(f32, f32); 10] = [
    (-0.25, -0.5), (-0.05, -0.5), (-0.05, -0.05), (0.2, -0.05), (0.2, 0.12),
    (-0.05, 0.12), (-0.05, 0.32), (0.3, 0.32), (0.3, 0.5), (-0.25, 0.5),
];
/// The same F as three rectangles (x0, y0, x1, y1): stem, middle bar, top bar.
const F_PARTS: [(f32, f32, f32, f32); 3] =
    [(-0.25, -0.5, -0.05, 0.5), (-0.05, -0.05, 0.2, 0.12), (-0.05, 0.32, 0.3, 0.5)];
const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August",
    "September", "October", "November", "December",
];
const SALES: [f32; 12] = [0.42, 0.55, 0.61, 0.48, 0.72, 0.9, 0.84, 0.77, 0.63, 0.58, 0.69, 0.95];
const CENTER: Vec2 = vec2(500.0, 200.0);
const R: f32 = 140.0; // pixels per unit
const SLIDER: (f32, f32, f32) = (220.0, 760.0, 580.0); // x0, x1, y
const BLUE: Color = Color::new(0.35, 0.71, 0.90, 1.0);
const RED: Color = Color::new(0.90, 0.44, 0.49, 1.0);
const AMBER: Color = Color::new(0.94, 0.71, 0.35, 1.0);

fn to_screen(p: Vec2) -> Vec2 {
    vec2(CENTER.x + p.x * R, CENTER.y - p.y * R)
}

fn arrow(to: Vec2, width: f32, color: Color) {
    let (a, b) = (to_screen(Vec2::ZERO), to_screen(to));
    draw_line(a.x, a.y, b.x, b.y, width, color);
    let back = (a - b).normalize_or_zero() * 12.0;
    draw_triangle(b, b + Vec2::from_angle(0.4).rotate(back), b + Vec2::from_angle(-0.4).rotate(back), color);
}

fn dashed(points: &[Vec2], color: Color) {
    for (k, w) in points.windows(2).enumerate() {
        if k % 2 == 0 {
            draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 1.2, color);
        }
    }
}

fn draw_plane(theta: f32, point: Vec2) {
    let grid = Color::from_rgba(51, 67, 60, 255);
    draw_poly_lines(CENTER.x, CENTER.y, 96, R, 0.0, 1.0, grid);
    draw_line(CENTER.x - 1.15 * R, CENTER.y, CENTER.x + 1.15 * R, CENTER.y, 1.0, grid);
    draw_line(CENTER.x, CENTER.y - 1.15 * R, CENTER.x, CENTER.y + 1.15 * R, 1.0, grid);

    // The F is not convex, so fill it as three rotated rectangles (two triangles each).
    for (x0, y0, x1, y1) in F_PARTS {
        let c = [vec2(x0, y0), vec2(x1, y0), vec2(x1, y1), vec2(x0, y1)].map(|p| to_screen(rotate(p, theta)));
        draw_triangle(c[0], c[1], c[2], AMBER.with_alpha(0.25));
        draw_triangle(c[0], c[2], c[3], AMBER.with_alpha(0.25));
    }
    // Outlines: before rotation (gray) and after (amber).
    let n = F.len();
    for k in 0..n {
        let (a, b) = (vec2(F[k].0, F[k].1), vec2(F[(k + 1) % n].0, F[(k + 1) % n].1));
        let (a0, b0) = (to_screen(a), to_screen(b));
        let (a1, b1) = (to_screen(rotate(a, theta)), to_screen(rotate(b, theta)));
        draw_line(a0.x, a0.y, b0.x, b0.y, 1.0, GRAY);
        draw_line(a1.x, a1.y, b1.x, b1.y, 2.0, AMBER);
    }

    // The matrix columns are where (1, 0) and (0, 1) land.
    arrow(vec2(1.0, 0.0), 1.5, BLUE.with_alpha(0.3));
    arrow(vec2(0.0, 1.0), 1.5, RED.with_alpha(0.3));
    let (i, j) = (rotate(vec2(1.0, 0.0), theta), rotate(vec2(0.0, 1.0), theta));
    arrow(i, 3.0, BLUE);
    arrow(j, 3.0, RED);
    for (v, text, color) in [(i, "(cos, sin)", BLUE), (j, "(-sin, cos)", RED)] {
        let t = to_screen(v) + vec2(v.x * 56.0, -v.y * 18.0);
        let w = measure_text(text, None, 18, 1.0).width;
        draw_text(text, t.x - w / 2.0, t.y + 5.0, 18.0, color);
    }

    // One point, its image, and the arc between them.
    let q = rotate(point, theta);
    let arc: Vec<Vec2> = (0..=40).map(|k| to_screen(rotate(point, theta * k as f32 / 40.0))).collect();
    dashed(&arc, LIGHTGRAY);
    let (ps, qs) = (to_screen(point), to_screen(q));
    draw_circle(ps.x, ps.y, 8.0, Color::from_rgba(22, 32, 28, 255));
    draw_circle_lines(ps.x, ps.y, 8.0, 2.0, WHITE);
    draw_circle(qs.x, qs.y, 6.0, WHITE);
    draw_text("P", ps.x + 10.0, ps.y + 20.0, 20.0, WHITE);
    draw_text("P'", qs.x + 10.0, qs.y - 8.0, 20.0, WHITE);

    let (s, c) = theta.sin_cos();
    draw_text(&format!("theta = {:.0} deg", theta.to_degrees()), 20.0, 30.0, 20.0, WHITE);
    let xs = format!("x' = {:.2}*{c:.2} - {:.2}*{s:.2} = {:.2}", point.x, point.y, q.x);
    let ys = format!("y' = {:.2}*{s:.2} + {:.2}*{c:.2} = {:.2}", point.x, point.y, q.y);
    draw_text(&xs, 20.0, 52.0, 20.0, BLUE);
    draw_text(&ys, 20.0, 74.0, 20.0, RED);
}

fn draw_chart(angle: f32) {
    let (left, right, baseline) = (40.0, 760.0, 470.0);
    let slot = (right - left) / 12.0;
    let widths: Vec<f32> = MONTHS.iter().map(|m| measure_text(m, None, 18, 1.0).width).collect();
    let height = 13.0;
    let mut collisions = 0;
    for (k, m) in MONTHS.iter().enumerate() {
        let x = left + slot * (k as f32 + 0.5);
        let bar_h = SALES[k] * 60.0;
        draw_rectangle(x - slot * 0.3, baseline - bar_h, slot * 0.6, bar_h, AMBER.with_alpha(0.75));
        let hits_left = k > 0 && labels_collide(slot, widths[k], height, angle);
        let hits_right = k < 11 && labels_collide(slot, widths[k + 1], height, angle);
        collisions += hits_right as usize;
        // The label's right end sits under its bar; it runs along (cos a, -sin a) on screen,
        // and its baseline is one text height below the top edge.
        let end = vec2(if angle > 0.01 { x + 4.0 } else { x + widths[k] / 2.0 }, baseline + 6.0);
        let along = vec2(angle.cos(), -angle.sin());
        let down = vec2(angle.sin(), angle.cos()) * height;
        let start = end + down - along * widths[k];
        let color = if hits_left || hits_right { RED } else { WHITE };
        let params = TextParams { font_size: 18, rotation: -angle, color, ..Default::default() };
        draw_text_ex(m, start.x, start.y, params);
    }
    draw_line(left, baseline, right, baseline, 1.0, LIGHTGRAY);
    let note = match collisions {
        0 => "no labels collide".to_string(),
        n => format!("{n} label pairs collide: tilt them"),
    };
    let w = measure_text(&note, None, 18, 1.0).width;
    draw_text(&note, right - w, 398.0, 18.0, if collisions > 0 { RED } else { BLUE });

    // The tilt slider.
    let (x0, x1, y) = SLIDER;
    draw_rectangle(x0, y - 2.0, x1 - x0, 4.0, Color::from_rgba(51, 67, 60, 255));
    draw_circle(x0 + angle / (PI / 2.0) * (x1 - x0), y, 8.0, AMBER);
    draw_text(&format!("label tilt {:.0} deg", angle.to_degrees()), 40.0, y + 6.0, 20.0, GRAY);
}

#[macroquad::main("2D rotation")]
async fn main() {
    let mut theta = PI / 6.0;
    let mut spinning = true;
    let mut point = vec2(0.78, 0.32);
    let mut tilt: Option<f32> = None; // None: animate the tilt until the slider is used
    let mut drag_point = false;
    let mut drag_tilt = false;

    loop {
        let mouse = Vec2::from(mouse_position());
        let (x0, x1, sy) = SLIDER;
        if is_mouse_button_pressed(MouseButton::Left) {
            drag_point = mouse.distance(to_screen(point)) < 16.0;
            drag_tilt = !drag_point && (mouse.y - sy).abs() < 14.0 && mouse.x > x0 - 10.0;
        }
        if is_mouse_button_released(MouseButton::Left) {
            (drag_point, drag_tilt) = (false, false);
        }
        if drag_point {
            let p = vec2(mouse.x - CENTER.x, CENTER.y - mouse.y) / R;
            point = p.clamp_length_max(1.1); // stay inside radius 1.1
        }
        if drag_tilt {
            tilt = Some(((mouse.x - x0) / (x1 - x0)).clamp(0.0, 1.0) * PI / 2.0);
        }
        if is_key_pressed(KeyCode::Space) {
            spinning = !spinning;
        }
        if spinning {
            theta = (theta + 0.5 * get_frame_time() + PI).rem_euclid(2.0 * PI) - PI;
        }

        clear_background(Color::from_rgba(22, 32, 28, 255));
        draw_plane(theta, point);
        draw_line(20.0, 380.0, 780.0, 380.0, 1.0, Color::from_rgba(51, 67, 60, 255));
        let wave = 1.0 - (1.0 - (get_time() as f32 / 4.0).rem_euclid(2.0)).abs();
        draw_chart(tilt.unwrap_or(wave * PI / 2.0));
        next_frame().await;
    }
}
