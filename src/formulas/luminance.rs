//! Relative luminance: how bright a color looks, used to pick readable text.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, pos2, vec2};

use crate::formula::{Formula, MathText, Palette, Tone, fit_text};

/// Channel weights: the eye is most sensitive to green, least to blue.
pub const WEIGHTS: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// Undo the sRGB transfer curve: encoded `[0, 1]` value → linear light.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// The three weighted terms of `Y`. With `linearize`, channels are converted to
/// linear light first, as WCAG 2 specifies; without, the weights are applied to
/// the encoded values directly (a common, cruder approximation).
pub fn contributions(rgb: [u8; 3], linearize: bool) -> [f32; 3] {
    std::array::from_fn(|i| {
        let c = rgb[i] as f32 / 255.0;
        WEIGHTS[i] * if linearize { srgb_to_linear(c) } else { c }
    })
}

/// `Y = 0.2126 R + 0.7152 G + 0.0722 B`
pub fn luminance(rgb: [u8; 3], linearize: bool) -> f32 {
    contributions(rgb, linearize).iter().sum()
}

/// WCAG contrast ratio between two luminances, from 1:1 up to 21:1.
pub fn contrast(a: f32, b: f32) -> f32 {
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Whether black text reads better than white on a background of luminance `y`.
pub fn prefers_black_text(y: f32) -> bool {
    contrast(y, 0.0) >= contrast(y, 1.0)
}

/// Hue/saturation/value (all `[0, 1]`) to 8-bit sRGB.
pub fn hsv_to_srgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor();
    let f = h - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let (r, g, b) = match i as u32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [r, g, b].map(|c| (c * 255.0).round().clamp(0.0, 255.0) as u8)
}

const COLS: usize = 12;
/// Saturation and value for each heatmap row, light to dark.
const ROWS: [(f32, f32); 5] = [
    (0.3, 1.0),
    (0.65, 1.0),
    (1.0, 0.95),
    (1.0, 0.7),
    (0.9, 0.42),
];
const PANEL_H: f32 = 150.0;

fn cell_color(col: usize, row: usize) -> [u8; 3] {
    let (s, v) = ROWS[row];
    hsv_to_srgb(col as f32 / COLS as f32, s, v)
}

fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

pub struct Luminance {
    selected: [u8; 3],
    linearize: bool,
    naive: bool,
    hovered: Option<(usize, usize)>,
}

impl Default for Luminance {
    fn default() -> Self {
        Self {
            selected: cell_color(2, 2),
            linearize: true,
            naive: false,
            hovered: None,
        }
    }
}

fn grid(rect: Rect) -> Rect {
    Rect::from_min_max(
        rect.min + vec2(18.0, 18.0),
        pos2(rect.max.x - 18.0, rect.max.y - PANEL_H - 12.0),
    )
}

fn cell_rect(grid: Rect, col: usize, row: usize) -> Rect {
    let w = grid.width() / COLS as f32;
    let h = grid.height() / ROWS.len() as f32;
    Rect::from_min_size(
        pos2(grid.left() + w * col as f32, grid.top() + h * row as f32),
        vec2(w, h),
    )
    .shrink(2.0)
}

fn cell_at(grid: Rect, p: Pos2) -> Option<(usize, usize)> {
    if !grid.contains(p) {
        return None;
    }
    let col = ((p.x - grid.left()) / grid.width() * COLS as f32) as usize;
    let row = ((p.y - grid.top()) / grid.height() * ROWS.len() as f32) as usize;
    Some((col.min(COLS - 1), row.min(ROWS.len() - 1)))
}

impl Formula for Luminance {
    fn params(&self) -> Vec<(&'static str, f64)> {
        let [r, g, b] = self.selected.map(|c| {
            let c = c as f32 / 255.0;
            (if self.linearize { srgb_to_linear(c) } else { c }) as f64
        });
        vec![("R", r), ("G", g), ("B", b)]
    }

    fn id(&self) -> &'static str {
        "relative-luminance"
    }

    fn name(&self) -> &'static str {
        "Relative luminance"
    }

    fn category(&self) -> &'static str {
        super::COLOR
    }

    fn summary(&self) -> &'static str {
        "Green looks far brighter than blue at the same strength, so the channels are \
         weighted. Each cell computes Y and picks black or white text, whichever contrasts \
         more. Click a cell to inspect it."
    }

    fn notes(&self) -> &'static str {
        "Labels on heatmap cells, bars and pie slices; badges and chips with data-driven \
         fill colors; checking a palette against WCAG contrast (4.5:1 for body text). The \
         weights are defined for linear light, so WCAG first undoes the sRGB curve; \
         applying them to raw values is a rougher shortcut."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new().t("Y").tone(Tone::Trace).t(" = 0.2126 R + 0.7152 G + 0.0722 B"),
            MathText::new().t("contrast = (Y").sub("light").t(" + 0.05) / (Y").sub("dark").t(" + 0.05)"),
            MathText::new().t("WCAG linearizes each channel first: c ≤ 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055)").sup("2.4"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::click()
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.42).max(220.0) + PANEL_H
    }

    fn min_size(&self) -> egui::Vec2 {
        // The cell labels and the Y = ... breakdown line need the width.
        egui::vec2(560.0, self.stage_height(560.0))
    }

    fn is_animating(&self) -> bool {
        false
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let grid = grid(rect);
        self.hovered = response.hover_pos().and_then(|p| cell_at(grid, p));
        if response.clicked()
            && let Some((c, r)) = response
                .interact_pointer_pos()
                .and_then(|p| cell_at(grid, p))
        {
            self.selected = cell_color(c, r);
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, _dt: f32) {
        let grid = grid(rect);
        let cell_font = FontId::proportional(13.0);
        for row in 0..ROWS.len() {
            for col in 0..COLS {
                let c = cell_color(col, row);
                let r = cell_rect(grid, col, row);
                let y = luminance(c, self.linearize);
                let text = if self.naive || !prefers_black_text(y) {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                };
                let outline = if c == self.selected {
                    Stroke::new(2.5, palette.ink)
                } else if self.hovered == Some((col, row)) {
                    Stroke::new(1.5, palette.muted)
                } else {
                    Stroke::NONE
                };
                painter.rect(r, 4.0, rgb(c), outline, egui::StrokeKind::Outside);
                painter.text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    format!("{y:.2}"),
                    cell_font.clone(),
                    text,
                );
            }
        }
        self.paint_breakdown(painter, rect, palette);
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Color");
            ui.color_edit_button_srgb(&mut self.selected);
            ui.checkbox(&mut self.linearize, "Linearize sRGB first (WCAG)");
            ui.checkbox(&mut self.naive, "Naive: always white text");
        });
    }
}

impl Luminance {
    fn paint_breakdown(&self, painter: &egui::Painter, rect: Rect, palette: &Palette) {
        let panel = Rect::from_min_max(
            pos2(rect.left() + 18.0, rect.bottom() - PANEL_H),
            pos2(rect.right() - 18.0, rect.bottom() - 16.0),
        );
        let c = self.selected;
        let parts = contributions(c, self.linearize);
        let y = parts.iter().sum::<f32>();

        // Swatch with both text options and their contrast ratios.
        let swatch = Rect::from_min_size(
            panel.min,
            vec2((panel.width() * 0.36).max(170.0), panel.height()),
        );
        painter.rect(
            swatch,
            8.0,
            rgb(c),
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Inside,
        );
        let black_wins = prefers_black_text(y);
        for (k, (label, color, ratio, wins)) in [
            ("Black text", Color32::BLACK, contrast(y, 0.0), black_wins),
            ("White text", Color32::WHITE, contrast(y, 1.0), !black_wins),
        ]
        .into_iter()
        .enumerate()
        {
            let font = FontId::proportional(if wins { 19.0 } else { 15.0 });
            let line = format!(
                "{label}  {ratio:.1}:1{}",
                if wins { "  (best)" } else { "" }
            );
            let galley = fit_text(painter, line, font, swatch.width() - 28.0, color);
            painter.galley(
                swatch.left_top() + vec2(14.0, 18.0 + 40.0 * k as f32),
                galley,
                color,
            );
        }

        // Stacked contributions on a 0..1 scale, with the black/white switch point.
        let bar_area = Rect::from_min_max(
            pos2(swatch.right() + 24.0, panel.top() + 26.0),
            pos2(panel.right(), panel.top() + 56.0),
        );
        let x_of = |v: f32| bar_area.left() + v * bar_area.width();
        painter.rect_filled(bar_area, 4.0, palette.rule.gamma_multiply(0.5));
        let channel_colors = [
            Color32::from_rgb(214, 69, 65),
            Color32::from_rgb(46, 160, 67),
            Color32::from_rgb(56, 104, 214),
        ];
        let mut start = 0.0;
        for (i, part) in parts.iter().enumerate() {
            let seg = Rect::from_min_max(
                pos2(x_of(start), bar_area.top()),
                pos2(x_of(start + part), bar_area.bottom()),
            );
            painter.rect_filled(seg, 0.0, channel_colors[i]);
            start += part;
        }
        let switch = 1.05f32.sqrt() * 0.05f32.sqrt() - 0.05;
        painter.line_segment(
            [
                pos2(x_of(switch), bar_area.top() - 6.0),
                pos2(x_of(switch), bar_area.bottom() + 6.0),
            ],
            Stroke::new(2.0, palette.ink),
        );
        painter.text(
            pos2(x_of(switch), bar_area.top() - 8.0),
            Align2::CENTER_BOTTOM,
            format!("text flips at Y = {switch:.3}"),
            FontId::proportional(12.0),
            palette.muted,
        );
        painter.line_segment(
            [
                pos2(x_of(y), bar_area.top()),
                pos2(x_of(y), bar_area.bottom() + 10.0),
            ],
            Stroke::new(2.0, palette.trace),
        );
        painter.text(
            pos2(bar_area.left(), bar_area.bottom() + 4.0),
            Align2::LEFT_TOP,
            "0",
            FontId::proportional(12.0),
            palette.muted,
        );
        painter.text(
            pos2(bar_area.right(), bar_area.bottom() + 4.0),
            Align2::RIGHT_TOP,
            "1",
            FontId::proportional(12.0),
            palette.muted,
        );

        let channel = |i: usize| {
            let v = c[i] as f32 / 255.0;
            if self.linearize { srgb_to_linear(v) } else { v }
        };
        let mono = FontId::monospace(12.5);
        let base = pos2(bar_area.left(), bar_area.bottom() + 24.0);
        let room = panel.right() - base.x;
        let rgb_line = format!(
            "RGB {} {} {}, channels {}",
            c[0],
            c[1],
            c[2],
            if self.linearize {
                "linearized"
            } else {
                "as is, /255"
            }
        );
        let y_line = format!(
            "Y = 0.2126·{:.2} + 0.7152·{:.2} + 0.0722·{:.2} = {y:.3}",
            channel(0),
            channel(1),
            channel(2)
        );
        for (k, (line, color)) in [(rgb_line, palette.muted), (y_line, palette.ink)]
            .into_iter()
            .enumerate()
        {
            let galley = fit_text(painter, line, mono.clone(), room, color);
            painter.galley(base + vec2(0.0, 18.0 * k as f32), galley, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extremes() {
        assert!((luminance([255, 255, 255], true) - 1.0).abs() < 1e-5);
        assert_eq!(luminance([0, 0, 0], true), 0.0);
        assert!((contrast(1.0, 0.0) - 21.0).abs() < 1e-5);
    }

    #[test]
    fn green_looks_brightest() {
        let (r, g, b) = (
            luminance([255, 0, 0], true),
            luminance([0, 255, 0], true),
            luminance([0, 0, 255], true),
        );
        assert!(g > r && r > b);
    }

    #[test]
    fn text_choice() {
        assert!(
            prefers_black_text(luminance([255, 230, 0], true)),
            "yellow wants black text"
        );
        assert!(
            !prefers_black_text(luminance([20, 30, 110], true)),
            "navy wants white text"
        );
        // The two contrasts are equal at Y = √(1.05 · 0.05) − 0.05 ≈ 0.179.
        let y = (1.05f32 * 0.05).sqrt() - 0.05;
        assert!((contrast(y, 0.0) - contrast(y, 1.0)).abs() < 1e-4);
    }

    #[test]
    fn linearization_darkens_midtones() {
        let gray = [128, 128, 128];
        assert!(luminance(gray, true) < luminance(gray, false));
        assert!((srgb_to_linear(0.5) - 0.214).abs() < 1e-3);
    }

    #[test]
    fn hsv_primaries() {
        assert_eq!(hsv_to_srgb(0.0, 1.0, 1.0), [255, 0, 0]);
        assert_eq!(hsv_to_srgb(1.0 / 3.0, 1.0, 1.0), [0, 255, 0]);
        assert_eq!(hsv_to_srgb(2.0 / 3.0, 1.0, 1.0), [0, 0, 255]);
        assert_eq!(hsv_to_srgb(0.5, 0.0, 0.5), [128, 128, 128]);
    }
}
