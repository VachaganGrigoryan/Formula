//! Cubic ease-out: f(t) = 1 - (1 - t)^3, with speed f'(t) = 3 (1 - t)^2.
//!
//! Top left: the curve against the linear diagonal, plus its speed (scaled by 1/3).
//! Top right: the same progress t drives a linear and an eased ball; the small dots mark
//! equal slices of time, evenly spaced for linear and bunched up at the end for ease-out.
//! Bottom: bars grow toward new heights with the eased progress, in a loop.
//! Click to replay; keys: E eases the bars or not, L loops, Up/Down change the duration.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

/// 1 - (1 - t)^3 for progress t, clamped to 0..1.
fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

/// The derivative 3 (1 - t)^2: three times linear speed at the start, zero at the end.
fn ease_out_cubic_speed(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    3.0 * u * u
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

const HOLD: f32 = 0.8; // seconds the finished state is held before looping
const BARS: usize = 7;
const GRAPH: Rect = Rect { x: 50.0, y: 30.0, w: 260.0, h: 260.0 };
const TRACKS: Rect = Rect { x: 370.0, y: 30.0, w: 390.0, h: 260.0 };
const CHART: Rect = Rect { x: 30.0, y: 350.0, w: 740.0, h: 205.0 };
const AMBER: Color = Color::new(0.94, 0.71, 0.35, 1.0);
const MUTED: Color = Color::new(0.58, 0.64, 0.61, 1.0);

/// A point of the unit square inside the graph (v grows upward).
fn graph_at(u: f32, v: f32) -> Vec2 {
    vec2(GRAPH.x + u * GRAPH.w, GRAPH.y + GRAPH.h - v * GRAPH.h)
}

fn plot(f: impl Fn(f32) -> f32, width: f32, color: Color) {
    for k in 0..64 {
        let (u0, u1) = (k as f32 / 64.0, (k + 1) as f32 / 64.0);
        let (a, b) = (graph_at(u0, f(u0)), graph_at(u1, f(u1)));
        draw_line(a.x, a.y, b.x, b.y, width, color);
    }
}

struct Bars {
    from: [f32; BARS],
    to: [f32; BARS],
}

impl Bars {
    /// Animate from where the bars are now (at blend `t`) toward fresh heights.
    fn retarget(&mut self, t: f32) {
        for k in 0..BARS {
            self.from[k] = lerp(self.from[k], self.to[k], t);
            self.to[k] = rand::gen_range(0.15, 1.0);
        }
    }
}

#[macroquad::main("Cubic ease-out")]
async fn main() {
    rand::srand(5);
    let mut bars = Bars { from: [0.0; BARS], to: [0.0; BARS] };
    bars.retarget(0.0);
    let (mut duration, mut elapsed) = (1.6, 0.0);
    let (mut looping, mut eased) = (true, true);

    loop {
        elapsed += get_frame_time().min(0.1);
        let t = (elapsed / duration).min(1.0);
        let bar_t = if eased { ease_out_cubic(t) } else { t };
        if is_mouse_button_pressed(MouseButton::Left) || (looping && elapsed > duration + HOLD) {
            bars.retarget(bar_t);
            elapsed = 0.0;
        }
        if is_key_pressed(KeyCode::E) {
            eased = !eased;
        }
        if is_key_pressed(KeyCode::L) {
            looping = !looping;
        }
        if is_key_pressed(KeyCode::Up) {
            duration = (duration + 0.1f32).min(4.0);
        }
        if is_key_pressed(KeyCode::Down) {
            duration = (duration - 0.1f32).max(0.3);
        }
        let t = (elapsed / duration).min(1.0);
        let e = ease_out_cubic(t);

        clear_background(Color::from_rgba(22, 32, 28, 255));

        // 1. The curve itself, with linear for reference and the speed f'(t) / 3.
        draw_rectangle_lines(GRAPH.x, GRAPH.y, GRAPH.w, GRAPH.h, 1.0, Color::from_rgba(51, 67, 60, 255));
        plot(|u| u, 1.0, MUTED);
        plot(|u| ease_out_cubic_speed(u) / 3.0, 1.2, Color::new(0.90, 0.44, 0.49, 0.7));
        plot(ease_out_cubic, 2.5, AMBER);
        let (base, lin, eas) = (graph_at(t, 0.0), graph_at(t, t), graph_at(t, e));
        draw_line(base.x, base.y, eas.x, eas.y, 1.0, Color::new(0.9, 0.92, 0.9, 0.4));
        draw_circle(lin.x, lin.y, 4.0, MUTED);
        draw_circle(eas.x, eas.y, 6.0, AMBER);
        draw_text("t = 0", GRAPH.x, GRAPH.y + GRAPH.h + 18.0, 18.0, MUTED);
        draw_text("1", GRAPH.x + GRAPH.w - 6.0, GRAPH.y + GRAPH.h + 18.0, 18.0, MUTED);
        let label = graph_at(0.5, 0.36);
        draw_text("speed f'(t) / 3", label.x, label.y, 18.0, Color::new(0.90, 0.44, 0.49, 1.0));

        // 2. The same progress driving position: linear vs eased.
        let lanes: [(&str, fn(f32) -> f32, Color); 2] =
            [("linear", |u| u, MUTED), ("ease-out", ease_out_cubic, AMBER)];
        for (i, (name, f, color)) in lanes.into_iter().enumerate() {
            let y = TRACKS.y + TRACKS.h / 2.0 * (i as f32 + 0.55);
            let (x0, x1) = (TRACKS.x, TRACKS.x + TRACKS.w);
            draw_text(name, x0, y - 22.0, 20.0, color);
            draw_line(x0, y, x1, y, 1.0, Color::from_rgba(51, 67, 60, 255));
            // Where the ball is after each tenth of the duration.
            for k in 0..=10 {
                draw_circle(lerp(x0, x1, f(k as f32 / 10.0)), y, 2.5, color);
            }
            draw_circle(lerp(x0, x1, f(t)), y, 9.0, color);
        }

        // 3. A bar chart animating to new values; the thin lines mark the targets.
        let bar_t = if eased { e } else { t };
        let slot = CHART.w / BARS as f32;
        for k in 0..BARS {
            let v = lerp(bars.from[k], bars.to[k], bar_t);
            let x = CHART.x + slot * k as f32;
            let bottom = CHART.y + CHART.h;
            let (x0, w) = (x + slot * 0.18, slot * 0.64);
            draw_rectangle(x0, bottom - v * CHART.h, w, v * CHART.h, if eased { AMBER } else { MUTED });
            draw_rectangle(x0, bottom - bars.to[k] * CHART.h - 1.0, w, 2.0, Color::new(0.9, 0.92, 0.9, 0.6));
        }
        draw_line(CHART.x, CHART.y + CHART.h, CHART.x + CHART.w, CHART.y + CHART.h, 1.0, MUTED);

        let status = format!("t = {t:.2}   f(t) = {e:.2}   duration {duration:.1} s");
        draw_text(&status, 30.0, 335.0, 20.0, WHITE);
        let on_off = if looping { "on" } else { "off" };
        let mode = if eased { "eased" } else { "linear" };
        let help = format!("click: replay   E: {mode} bars   L: loop {on_off}   Up/Down: duration");
        draw_text(&help, 30.0, 585.0, 20.0, MUTED);
        next_frame().await;
    }
}
