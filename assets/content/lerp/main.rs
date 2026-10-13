//! Linear interpolation: v(t) = v0 + t * (v1 - v0).
//!
//! A point slides between two draggable endpoints, a number moves along a number line,
//! and a color bar shows the blend, all driven by one t that sweeps 0 -> 1 -> 0.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(lerp(a.r, b.r, t), lerp(a.g, b.g, t), lerp(a.b, b.b, t), 1.0)
}

/// Map any real s onto 0..1..0 at constant speed (a triangle wave).
fn ping_pong(s: f32) -> f32 {
    1.0 - (1.0 - s.rem_euclid(2.0)).abs()
}

#[macroquad::main("Linear interpolation")]
async fn main() {
    let mut ends = [vec2(120.0, 330.0), vec2(680.0, 90.0)];
    let (v0, v1) = (20.0, 80.0);
    let (c0, c1) = (Color::from_rgba(35, 120, 110, 255), Color::from_rgba(168, 112, 15, 255));
    let mut dragging: Option<usize> = None;

    loop {
        let t = ping_pong(get_time() as f32 / 2.0);
        clear_background(Color::from_rgba(22, 32, 28, 255));

        // Drag either endpoint with the mouse.
        let mouse = Vec2::from(mouse_position());
        if is_mouse_button_pressed(MouseButton::Left) {
            dragging = ends.iter().position(|e| e.distance(mouse) < 14.0);
        }
        if is_mouse_button_released(MouseButton::Left) {
            dragging = None;
        }
        if let Some(i) = dragging {
            ends[i] = vec2(mouse.x, mouse.y.min(400.0));
        }

        // The plane: segment, endpoints, and the interpolated point.
        draw_line(ends[0].x, ends[0].y, ends[1].x, ends[1].y, 1.0, GRAY);
        draw_circle(ends[0].x, ends[0].y, 9.0, c0);
        draw_circle(ends[1].x, ends[1].y, 9.0, c1);
        let p = vec2(lerp(ends[0].x, ends[1].x, t), lerp(ends[0].y, ends[1].y, t));
        draw_circle(p.x, p.y, 7.0, Color::from_rgba(157, 179, 255, 255));

        // The number line from 0 to 100.
        let x = |v: f32| 60.0 + v / 100.0 * 680.0;
        draw_line(x(0.0), 460.0, x(100.0), 460.0, 1.0, LIGHTGRAY);
        let v = lerp(v0, v1, t);
        draw_triangle(vec2(x(v), 452.0), vec2(x(v) - 7.0, 440.0), vec2(x(v) + 7.0, 440.0), SKYBLUE);

        // The color bar: the gradient, and a marker at t.
        for i in 0..680 {
            draw_rectangle(60.0 + i as f32, 500.0, 1.0, 40.0, lerp_color(c0, c1, i as f32 / 679.0));
        }
        draw_rectangle_lines(60.0 + t * 680.0 - 2.0, 496.0, 4.0, 48.0, 3.0, WHITE);

        let text = format!("t = {t:.2}   v = {v:.1}   p = ({:.0}, {:.0})", p.x, p.y);
        draw_text(&text, 60.0, 40.0, 22.0, WHITE);
        draw_text("drag the endpoints", 60.0, 580.0, 20.0, GRAY);
        next_frame().await;
    }
}
