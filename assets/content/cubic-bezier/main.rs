//! Cubic Bezier curve: B(t) = (1-t)^3 P0 + 3(1-t)^2 t P1 + 3(1-t) t^2 P2 + t^3 P3.
//!
//! The curve is drawn up to the current t, which sweeps 0 -> 1 -> 0. The colored lines
//! are de Casteljau's construction (lerps of lerps) that lands on the same point, and
//! the bars below show the four Bernstein weights at t.
//! Drag the four points; Space plays or pauses, C toggles the construction.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

/// The four Bernstein weights (1-t)^3, 3(1-t)^2 t, 3(1-t) t^2, t^3. They sum to 1.
fn bernstein(t: f32) -> [f32; 4] {
    let u = 1.0 - t;
    [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t]
}

/// B(t) as the weighted sum of the control points.
fn bezier(p: &[Vec2; 4], t: f32) -> Vec2 {
    p.iter().zip(bernstein(t)).map(|(q, w)| *q * w).sum()
}

/// De Casteljau: four points -> three -> two -> one, by repeated lerps.
fn de_casteljau(p: &[Vec2; 4], t: f32) -> ([Vec2; 3], [Vec2; 2], Vec2) {
    let a = [p[0].lerp(p[1], t), p[1].lerp(p[2], t), p[2].lerp(p[3], t)];
    let b = [a[0].lerp(a[1], t), a[1].lerp(a[2], t)];
    (a, b, b[0].lerp(b[1], t))
}

/// Map any real s onto 0..1..0 at constant speed (a triangle wave).
fn ping_pong(s: f32) -> f32 {
    1.0 - (1.0 - s.rem_euclid(2.0)).abs()
}

fn polyline(points: &[Vec2], thickness: f32, color: Color) {
    for w in points.windows(2) {
        draw_line(w[0].x, w[0].y, w[1].x, w[1].y, thickness, color);
    }
}

fn dashed(a: Vec2, b: Vec2, color: Color) {
    let n = (a.distance(b) / 8.0).max(1.0) as usize;
    for i in 0..n {
        let (p, q) = (a.lerp(b, i as f32 / n as f32), a.lerp(b, (i as f32 + 0.5) / n as f32));
        draw_line(p.x, p.y, q.x, q.y, 1.0, color);
    }
}

#[macroquad::main("Cubic Bezier curve")]
async fn main() {
    let mut points = [vec2(70.0, 380.0), vec2(180.0, 60.0), vec2(560.0, 50.0), vec2(730.0, 330.0)];
    let mut dragging: Option<usize> = None;
    let (mut playing, mut construction, mut phase) = (true, true, 0.0_f32);
    let trace = Color::from_rgba(240, 180, 41, 255);
    let (red, green) = (Color::from_rgba(239, 107, 91, 255), Color::from_rgba(91, 200, 138, 255));
    let background = Color::from_rgba(22, 32, 28, 255);

    loop {
        // Drag any of the four points with the mouse.
        let mouse = Vec2::from(mouse_position());
        if is_mouse_button_pressed(MouseButton::Left) {
            dragging = points.iter().position(|p| p.distance(mouse) < 16.0);
        }
        if is_mouse_button_released(MouseButton::Left) {
            dragging = None;
        }
        if let Some(i) = dragging {
            points[i] = vec2(mouse.x, mouse.y.min(420.0));
        }
        if is_key_pressed(KeyCode::Space) {
            playing = !playing;
        }
        if is_key_pressed(KeyCode::C) {
            construction = !construction;
        }
        if playing {
            phase += 0.3 * get_frame_time();
        }
        let t = ping_pong(phase);
        clear_background(background);

        // Control polygon, the whole curve faintly, and the part up to t brightly.
        for w in points.windows(2) {
            dashed(w[0], w[1], DARKGRAY);
        }
        let steps = 120;
        let curve: Vec<Vec2> = (0..=steps).map(|j| bezier(&points, j as f32 / steps as f32)).collect();
        polyline(&curve, 1.5, Color::new(trace.r, trace.g, trace.b, 0.35));
        let mut drawn: Vec<Vec2> = curve[..=(t * steps as f32) as usize].to_vec();
        drawn.push(bezier(&points, t));
        polyline(&drawn, 3.0, trace);

        let (a, b, c) = de_casteljau(&points, t);
        if construction {
            polyline(&a, 1.5, red);
            polyline(&b, 1.5, green);
            a.iter().for_each(|q| draw_circle(q.x, q.y, 3.5, red));
            b.iter().for_each(|q| draw_circle(q.x, q.y, 3.5, green));
        }
        for (i, q) in points.iter().enumerate() {
            let fill = if i == 0 || i == 3 { WHITE } else { background };
            draw_circle(q.x, q.y, 8.0, fill);
            draw_circle_lines(q.x, q.y, 8.0, 2.0, LIGHTGRAY);
            draw_text(&format!("P{i}"), q.x - 8.0, q.y - 14.0, 18.0, LIGHTGRAY);
        }
        draw_circle(c.x, c.y, 6.5, trace);

        // The Bernstein weights: how much each point pulls on B(t) right now.
        let labels = ["(1-t)^3", "3(1-t)^2 t", "3(1-t) t^2", "t^3"];
        let (top, max_h, col_w) = (450.0, 90.0, screen_width() / 4.0);
        for (i, w) in bernstein(t).iter().enumerate() {
            let (x0, width) = (col_w * i as f32 + 30.0, col_w - 60.0);
            draw_rectangle(x0, top, width, max_h, Color::from_rgba(44, 58, 52, 255));
            draw_rectangle(x0, top + max_h - w * max_h, width, w * max_h, trace);
            draw_text(labels[i], x0, top + max_h + 20.0, 18.0, GRAY);
            draw_text(&format!("P{i}: {w:.2}"), x0, top + max_h + 40.0, 18.0, WHITE);
        }
        draw_text(&format!("t = {t:.2}"), screen_width() - 120.0, 30.0, 26.0, trace);
        draw_text("drag the points   Space: play/pause   C: construction", 16.0, 24.0, 18.0, GRAY);
        next_frame().await;
    }
}
