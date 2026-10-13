//! Min-max normalization: x_norm = (x - min) / (max - min), height = x_norm * H.
//!
//! Twelve monthly values sit on a raw axis (0..500), get mapped onto a 0..1 axis, and
//! become bar heights. Drag the gold August dot along the raw axis: as it becomes the
//! maximum, every other bar is squashed. Hover a bar to see its calculation.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

/// Map values onto 0..1. A flat dataset (max == min) maps to all zeros.
fn min_max(values: &[f32]) -> Vec<f32> {
    let (lo, hi) = bounds(values);
    let span = hi - lo;
    values
        .iter()
        .map(|v| if span > 0.0 { (v - lo) / span } else { 0.0 })
        .collect()
}

fn bounds(values: &[f32]) -> (f32, f32) {
    values
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        })
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

const MONTHS: [&str; 12] = ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"];
const OUTLIER: usize = 7; // August
const RAW_MAX: f32 = 500.0;
const LEFT: f32 = 60.0;
const RIGHT: f32 = 740.0;
const RAW_Y: f32 = 100.0;
const UNIT_Y: f32 = 200.0;
const CHART_TOP: f32 = 260.0;
const CHART_BOTTOM: f32 = 500.0;

const LOW: Color = Color::new(0.16, 0.62, 0.56, 1.0);
const HIGH: Color = Color::new(0.91, 0.44, 0.32, 1.0);
const GOLD: Color = Color::new(0.91, 0.77, 0.42, 1.0);
const MID: Color = Color::new(0.49, 0.65, 0.88, 1.0);
const MUTED: Color = Color::new(0.54, 0.6, 0.57, 1.0);
const BAND: Color = Color::new(0.49, 0.65, 0.88, 0.4); // the min..max span
const RULE: Color = Color::new(0.29, 0.35, 0.32, 1.0);

fn raw_x(v: f32) -> f32 {
    lerp(LEFT, RIGHT, v / RAW_MAX)
}

fn unit_x(u: f32) -> f32 {
    lerp(LEFT, RIGHT, u)
}

fn centered(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let w = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, x - w / 2.0, y, size, color);
}

#[macroquad::main("Min-max normalization")]
async fn main() {
    let mut values: Vec<f32> = (0..12)
        .map(|i| 55.0 + 18.0 * (0.55 * i as f32).sin() + 10.0 * (2.7 * i as f32 + 1.0).sin())
        .collect();
    values[OUTLIER] = 260.0; // start with August far above the rest
    let slot = (RIGHT - LEFT) / 12.0;
    let h_px = CHART_BOTTOM - CHART_TOP;
    let mut dragging = false;

    loop {
        clear_background(Color::from_rgba(22, 32, 28, 255));
        let (mx, my) = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) && (my - RAW_Y).abs() < 20.0 {
            dragging = true;
        }
        if is_mouse_button_released(MouseButton::Left) {
            dragging = false;
        }
        if dragging {
            values[OUTLIER] = ((mx - LEFT) / (RIGHT - LEFT) * RAW_MAX).clamp(0.0, RAW_MAX);
        }
        let column = ((mx - LEFT) / slot).floor();
        let hovered =
            (my > CHART_TOP - 20.0 && my < CHART_BOTTOM + 25.0 && (0.0..12.0).contains(&column))
                .then_some(column as usize);

        let norm = min_max(&values);
        let (lo, hi) = bounds(&values);
        let color_of = |i: usize| {
            if values[i] == lo {
                LOW
            } else if values[i] == hi {
                HIGH
            } else {
                MID
            }
        };

        // Raw axis with ticks every 100.
        draw_text(
            "raw values (drag the gold August dot)",
            LEFT,
            RAW_Y - 40.0,
            18.0,
            MUTED,
        );
        draw_line(LEFT, RAW_Y, RIGHT, RAW_Y, 1.0, RULE);
        for k in 0..=5 {
            let x = raw_x(k as f32 * 100.0);
            draw_line(x, RAW_Y - 4.0, x, RAW_Y + 4.0, 1.0, RULE);
            centered(&format!("{}", k * 100), x, RAW_Y + 22.0, 16.0, MUTED);
        }
        draw_line(raw_x(lo), RAW_Y, raw_x(hi), RAW_Y, 4.0, BAND);

        // Normalized axis: min lands on 0, max on 1.
        draw_line(LEFT, UNIT_Y, RIGHT, UNIT_Y, 4.0, BAND);
        centered("0", unit_x(0.0), UNIT_Y + 24.0, 18.0, LOW);
        centered("1", unit_x(1.0), UNIT_Y + 24.0, 18.0, HIGH);
        centered("normalized", unit_x(0.5), UNIT_Y + 24.0, 18.0, MUTED);

        // Each value: a link from its raw position to its normalized position.
        for i in 0..12 {
            let active = hovered == Some(i) || (dragging && i == OUTLIER);
            let link = if active {
                WHITE
            } else {
                Color::new(0.63, 0.7, 0.78, 0.3)
            };
            draw_line(raw_x(values[i]), RAW_Y, unit_x(norm[i]), UNIT_Y, 1.0, link);
            let r = if active { 6.0 } else { 4.5 };
            let (r_raw, c_raw) = if i == OUTLIER {
                (7.0, GOLD)
            } else {
                (r, color_of(i))
            };
            draw_circle(raw_x(values[i]), RAW_Y, r_raw, c_raw);
            draw_circle(unit_x(norm[i]), UNIT_Y, r, color_of(i));
        }
        centered(&format!("min {lo:.0}"), raw_x(lo), RAW_Y - 12.0, 16.0, LOW);
        centered(&format!("max {hi:.0}"), raw_x(hi), RAW_Y - 12.0, 16.0, HIGH);

        // Bars: height = normalized value * available pixels.
        draw_line(LEFT, CHART_BOTTOM, RIGHT, CHART_BOTTOM, 1.0, RULE);
        draw_line(
            LEFT,
            CHART_TOP,
            RIGHT,
            CHART_TOP,
            1.0,
            Color::new(0.29, 0.35, 0.32, 0.5),
        );
        for i in 0..12 {
            let h = (norm[i] * h_px).max(2.0);
            let fill = if hovered == Some(i) {
                WHITE
            } else if i == OUTLIER {
                GOLD
            } else {
                MID
            };
            draw_rectangle(
                LEFT + slot * (i as f32 + 0.15),
                CHART_BOTTOM - h,
                slot * 0.7,
                h,
                fill,
            );
            centered(
                MONTHS[i],
                LEFT + slot * (i as f32 + 0.5),
                CHART_BOTTOM + 20.0,
                18.0,
                MUTED,
            );
        }

        let note = match hovered {
            Some(i) => format!(
                "({:.0} - {lo:.0}) / ({hi:.0} - {lo:.0}) = {:.2}  ->  {:.0} px of {h_px:.0}",
                values[i],
                norm[i],
                norm[i] * h_px
            ),
            None => "hover a bar".to_string(),
        };
        draw_text(&note, LEFT, 548.0, 20.0, WHITE);
        draw_text(
            "X_norm = (X - X_min) / (X_max - X_min)",
            LEFT,
            32.0,
            22.0,
            WHITE,
        );
        next_frame().await;
    }
}
