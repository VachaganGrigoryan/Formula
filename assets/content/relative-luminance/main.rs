//! Relative luminance (WCAG): Y = 0.2126 R + 0.7152 G + 0.0722 B on linearized sRGB,
//! and contrast = (Y_light + 0.05) / (Y_dark + 0.05).
//!
//! Every heatmap cell shows its Y, written in black or white, whichever contrasts more.
//! Click a cell to inspect it below: the swatch with both text colors and their contrast
//! ratios, and a stacked bar of the red, green and blue contributions to Y with the
//! switch point (Y = 0.179) marked. Keys: L toggles linearization, N the naive
//! "always white" labels.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

/// Channel weights: the eye is most sensitive to green, least to blue.
const WEIGHTS: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// Undo the sRGB transfer curve: encoded 0..1 value -> linear light.
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// The three weighted terms of Y for an 8-bit color.
fn contributions(rgb: [u8; 3], linearize: bool) -> [f32; 3] {
    std::array::from_fn(|i| {
        let c = rgb[i] as f32 / 255.0;
        WEIGHTS[i] * if linearize { srgb_to_linear(c) } else { c }
    })
}

/// WCAG contrast ratio between two luminances, from 1:1 up to 21:1.
fn contrast(a: f32, b: f32) -> f32 {
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn prefers_black_text(y: f32) -> bool {
    contrast(y, 0.0) >= contrast(y, 1.0)
}

/// Hue, saturation, value (all 0..1) to 8-bit sRGB.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let f = |n: f32| {
        let k = (n + h * 6.0) % 6.0;
        (255.0 * (v - v * s * k.min(4.0 - k).clamp(0.0, 1.0))).round() as u8
    };
    [f(5.0), f(3.0), f(1.0)]
}

const COLS: usize = 12;
/// Saturation and value for each heatmap row, light to dark.
const ROWS: [(f32, f32); 5] = [(0.3, 1.0), (0.65, 1.0), (1.0, 0.95), (1.0, 0.7), (0.9, 0.42)];
const GRID: Rect = Rect { x: 18.0, y: 18.0, w: 764.0, h: 360.0 };

fn cell_color(col: usize, row: usize) -> [u8; 3] {
    hsv_to_rgb(col as f32 / COLS as f32, ROWS[row].0, ROWS[row].1)
}

fn to_color(c: [u8; 3]) -> Color {
    Color::from_rgba(c[0], c[1], c[2], 255)
}

fn cell_at(p: Vec2) -> Option<(usize, usize)> {
    GRID.contains(p).then(|| {
        let col = ((p.x - GRID.x) / GRID.w * COLS as f32) as usize;
        let row = ((p.y - GRID.y) / GRID.h * ROWS.len() as f32) as usize;
        (col.min(COLS - 1), row.min(ROWS.len() - 1))
    })
}

/// Draw text centered on a point.
fn centered(text: &str, at: Vec2, size: u16, color: Color) {
    let d = measure_text(text, None, size, 1.0);
    draw_text(text, at.x - d.width / 2.0, at.y + d.offset_y / 2.0, size as f32, color);
}

fn draw_grid(selected: (usize, usize), hovered: Option<(usize, usize)>, linearize: bool, naive: bool) {
    let (w, h) = (GRID.w / COLS as f32, GRID.h / ROWS.len() as f32);
    for row in 0..ROWS.len() {
        for col in 0..COLS {
            let rgb = cell_color(col, row);
            let y: f32 = contributions(rgb, linearize).iter().sum();
            let (x0, y0) = (GRID.x + w * col as f32, GRID.y + h * row as f32);
            draw_rectangle(x0 + 2.0, y0 + 2.0, w - 4.0, h - 4.0, to_color(rgb));
            if selected == (col, row) {
                draw_rectangle_lines(x0, y0, w, h, 3.0, WHITE);
            } else if hovered == Some((col, row)) {
                draw_rectangle_lines(x0, y0, w, h, 2.0, GRAY);
            }
            let text = if naive || !prefers_black_text(y) { WHITE } else { BLACK };
            centered(&format!("{y:.2}"), vec2(x0 + w / 2.0, y0 + h / 2.0), 20, text);
        }
    }
}

fn draw_panel(rgb: [u8; 3], linearize: bool, naive: bool) {
    let parts = contributions(rgb, linearize);
    let y: f32 = parts.iter().sum();
    let top = 400.0;

    // Swatch with both text options and their contrast ratios.
    draw_rectangle(18.0, top, 260.0, 180.0, to_color(rgb));
    let black = prefers_black_text(y);
    let options = [
        ("Black text", BLACK, contrast(y, 0.0), black),
        ("White text", WHITE, contrast(y, 1.0), !black),
    ];
    for (k, (label, color, ratio, wins)) in options.into_iter().enumerate() {
        let text = format!("{label} {ratio:.1}:1{}", if wins { " (best)" } else { "" });
        draw_text(&text, 28.0, top + 60.0 + 70.0 * k as f32, if wins { 22.0 } else { 18.0 }, color);
    }

    // Stacked contributions on a 0..1 scale, with the black/white switch point.
    let bar = Rect::new(310.0, top + 30.0, 470.0, 32.0);
    let x_of = |v: f32| bar.x + v * bar.w;
    draw_rectangle(bar.x, bar.y, bar.w, bar.h, Color::from_rgba(51, 67, 60, 255));
    let channel_colors = [(214, 69, 65), (46, 160, 67), (56, 104, 214)];
    let mut start = 0.0;
    for (part, (r, g, b)) in parts.iter().zip(channel_colors) {
        draw_rectangle(x_of(start), bar.y, part * bar.w, bar.h, Color::from_rgba(r, g, b, 255));
        start += part;
    }
    let switch = (1.05f32 * 0.05).sqrt() - 0.05;
    draw_rectangle(x_of(switch) - 1.0, bar.y - 6.0, 2.0, bar.h + 12.0, WHITE);
    draw_rectangle(x_of(y) - 1.5, bar.y, 3.0, bar.h + 12.0, ORANGE);
    draw_text(&format!("text flips at Y = {switch:.3}"), x_of(switch) - 60.0, bar.y - 10.0, 18.0, GRAY);
    draw_text("0", bar.x, bar.y + bar.h + 18.0, 18.0, GRAY);
    draw_text("1", bar.x + bar.w - 8.0, bar.y + bar.h + 18.0, 18.0, GRAY);

    let ch = rgb.map(|v| if linearize { srgb_to_linear(v as f32 / 255.0) } else { v as f32 / 255.0 });
    let mode = if linearize { "linearized" } else { "as is, /255" };
    let rgb_text = format!("RGB {} {} {}, channels {mode} (L)", rgb[0], rgb[1], rgb[2]);
    draw_text(&rgb_text, bar.x, top + 112.0, 20.0, GRAY);
    let formula = format!("Y = 0.2126*{:.2} + 0.7152*{:.2} + 0.0722*{:.2}", ch[0], ch[1], ch[2]);
    draw_text(&formula, bar.x, top + 136.0, 20.0, WHITE);
    draw_text(&format!("  = {y:.3}"), bar.x, top + 156.0, 20.0, WHITE);
    let labels = if naive { "always white" } else { "best contrast" };
    draw_text(&format!("labels: {labels} (N)   click a cell"), bar.x, top + 178.0, 20.0, GRAY);
}

#[macroquad::main("Relative luminance")]
async fn main() {
    let mut selected = (2, 2); // column, row
    let (mut linearize, mut naive) = (true, false);

    loop {
        let hovered = cell_at(Vec2::from(mouse_position()));
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(c) = hovered {
                selected = c;
            }
        }
        if is_key_pressed(KeyCode::L) {
            linearize = !linearize;
        }
        if is_key_pressed(KeyCode::N) {
            naive = !naive;
        }

        clear_background(Color::from_rgba(22, 32, 28, 255));
        draw_grid(selected, hovered, linearize, naive);
        draw_panel(cell_color(selected.0, selected.1), linearize, naive);
        next_frame().await;
    }
}
