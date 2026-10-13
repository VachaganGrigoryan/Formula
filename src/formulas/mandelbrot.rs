//! The Mandelbrot set: iterate z ← z² + c and color c by how fast z escapes.

use egui::{
    Align2, Color32, ColorImage, FontId, Pos2, Rect, Sense, Stroke, TextureHandle, TextureOptions,
    pos2, vec2,
};

use crate::draw::gradient;
use crate::formula::{Formula, MathText, Palette, Tone};

/// Escape radius squared. Larger than the minimal 4 so smooth coloring is smooth.
const BAILOUT: f64 = 256.0;

/// Points inside the main cardioid or the period-2 bulb never escape;
/// testing for them up front skips the most expensive pixels.
pub fn in_main_bulbs(x: f64, y: f64) -> bool {
    let q = (x - 0.25) * (x - 0.25) + y * y;
    q * (q + (x - 0.25)) <= 0.25 * y * y || (x + 1.0) * (x + 1.0) + y * y <= 0.0625
}

/// Iterate `z ← z² + c` from `z₀ = 0`. Returns the smooth (fractional) iteration
/// count at escape, or `None` if `z` stayed bounded for `max_iter` steps.
pub fn escape(cx: f64, cy: f64, max_iter: u32) -> Option<f64> {
    if in_main_bulbs(cx, cy) {
        return None;
    }
    let (mut x, mut y) = (0.0f64, 0.0f64);
    for n in 0..max_iter {
        let (x2, y2) = (x * x, y * y);
        if x2 + y2 > BAILOUT {
            // Continuous escape time: removes the banding of plain integer counts.
            let log_zn = (x2 + y2).ln() / 2.0;
            let nu = (log_zn / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
            return Some(n as f64 + 1.0 - nu);
        }
        y = 2.0 * x * y + cy;
        x = x2 - y2 + cx;
    }
    None
}

/// The first `steps` points of the orbit of `c`, stopping once it clearly escapes.
pub fn orbit(cx: f64, cy: f64, steps: usize) -> Vec<[f64; 2]> {
    let mut out = vec![[0.0, 0.0]];
    let (mut x, mut y) = (0.0f64, 0.0f64);
    for _ in 0..steps {
        (x, y) = (x * x - y * y + cx, 2.0 * x * y + cy);
        out.push([x, y]);
        if x * x + y * y > 16.0 {
            break;
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq)]
struct View {
    cx: f64,
    cy: f64,
    /// Height of the view in the complex plane.
    span: f64,
}

const HOME: View = View {
    cx: -0.6,
    cy: 0.0,
    span: 2.6,
};

const PRESETS: [(&str, View); 3] = [
    ("Whole set", HOME),
    (
        "Seahorse valley",
        View {
            cx: -0.7436,
            cy: 0.1318,
            span: 0.012,
        },
    ),
    (
        "Mini Mandelbrot",
        View {
            cx: -1.7549,
            cy: 0.0,
            span: 0.045,
        },
    ),
];

#[derive(Clone, Copy, PartialEq)]
struct RenderKey {
    view: View,
    size: [usize; 2],
    max_iter: u32,
    dark: bool,
}

pub struct Mandelbrot {
    view: View,
    max_iter: u32,
    texture: Option<TextureHandle>,
    rendered: Option<RenderKey>,
    /// Render at half resolution while the view is being dragged.
    coarse: bool,
    hover: Option<Pos2>,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Self {
            view: HOME,
            max_iter: 300,
            texture: None,
            rendered: None,
            coarse: false,
            hover: None,
        }
    }
}

impl Mandelbrot {
    fn pixel(&self, rect: Rect) -> f64 {
        self.view.span / rect.height() as f64
    }

    fn to_complex(&self, rect: Rect, p: Pos2) -> (f64, f64) {
        let px = self.pixel(rect);
        (
            self.view.cx + (p.x - rect.center().x) as f64 * px,
            self.view.cy - (p.y - rect.center().y) as f64 * px,
        )
    }

    fn to_screen(&self, rect: Rect, [x, y]: [f64; 2]) -> Pos2 {
        let px = self.pixel(rect);
        pos2(
            rect.center().x + ((x - self.view.cx) / px) as f32,
            rect.center().y - ((y - self.view.cy) / px) as f32,
        )
    }

    /// Zoom by `factor` (> 1 zooms in) keeping the point under `at` fixed.
    fn zoom_at(&mut self, rect: Rect, at: Pos2, factor: f64) {
        let (x, y) = self.to_complex(rect, at);
        self.view.cx = x + (self.view.cx - x) / factor;
        self.view.cy = y + (self.view.cy - y) / factor;
        self.view.span /= factor;
    }

    fn render(&self, size: [usize; 2], palette: &Palette, dark: bool) -> ColorImage {
        let [w, h] = size;
        let px = self.view.span / h as f64;
        let stops = [
            palette.paper,
            palette.trace,
            palette.x,
            palette.y,
            palette.paper,
        ];
        let inside = if dark {
            Color32::from_rgb(4, 8, 6)
        } else {
            palette.ink
        };
        let mut pixels = Vec::with_capacity(w * h);
        for j in 0..h {
            let cy = self.view.cy - (j as f64 + 0.5 - h as f64 / 2.0) * px;
            for i in 0..w {
                let cx = self.view.cx + (i as f64 + 0.5 - w as f64 / 2.0) * px;
                pixels.push(match escape(cx, cy, self.max_iter) {
                    None => inside,
                    // Points that escape in a step or two stay near the paper color;
                    // the bands only build up toward the boundary.
                    Some(mu) => {
                        gradient(&stops, ((mu - 1.0).max(0.0).sqrt() * 0.06).fract() as f32)
                    }
                });
            }
        }
        ColorImage::new(size, pixels)
    }
}

impl Formula for Mandelbrot {
    fn id(&self) -> &'static str {
        "mandelbrot"
    }

    fn name(&self) -> &'static str {
        "Mandelbrot set"
    }

    fn category(&self) -> &'static str {
        super::FRACTALS
    }

    fn summary(&self) -> &'static str {
        "Square a complex number, add c, repeat. Black points never escape; colors count \
         how fast the rest fly off. Hover to see an orbit, click to zoom in, right-click to \
         zoom out, drag to pan."
    }

    fn notes(&self) -> &'static str {
        "Escape-time iteration plus a color map is the core of fractal art, procedural \
         textures and terrain, and zoomable multi-scale visuals. The same idea, iterating \
         a function and coloring by how it behaves, drives Julia sets and Newton fractals."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("z")
                .tone(Tone::Trace)
                .sub("n+1")
                .tone(Tone::Trace)
                .t(" = z")
                .sub("n")
                .sup("2")
                .t(" + c,   z")
                .sub("0")
                .t(" = 0"),
            MathText::new()
                .t("c is in the set while |z")
                .sub("n")
                .t("| ≤ 2 for every n"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * if width < 520.0 { 1.0 } else { 0.68 }
    }

    fn is_animating(&self) -> bool {
        false
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        self.hover = response.hover_pos();
        if response.dragged() {
            let d = response.drag_delta();
            let px = self.pixel(rect);
            self.view.cx -= d.x as f64 * px;
            self.view.cy += d.y as f64 * px;
            self.coarse = true;
        } else {
            self.coarse = false;
        }
        if let Some(at) = response.interact_pointer_pos() {
            if response.clicked() {
                self.zoom_at(rect, at, 2.0);
            } else if response.secondary_clicked() {
                self.zoom_at(rect, at, 0.5);
            }
        }
        // Wheel and pinch zoom, only while this is the active viewport (an
        // inactive one is handed a hover-only response).
        if let Some(at) = self.hover
            && response.sense.senses_click()
        {
            let (pinch, wheel) = response
                .ctx
                .input(|i| (i.zoom_delta(), i.smooth_scroll_delta.y));
            let zoom = pinch as f64 * (wheel as f64 * 0.003).exp();
            if zoom != 1.0 {
                self.zoom_at(rect, at, zoom);
            }
        }
    }

    fn wants_scroll(&self) -> bool {
        true
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, _dt: f32) {
        let dark = palette.paper.r() < 128;
        let scale = if self.coarse { 0.5 } else { 1.0 };
        let size = [
            ((rect.width() * scale).round() as usize).clamp(1, 1400),
            ((rect.height() * scale).round() as usize).clamp(1, 1000),
        ];
        let key = RenderKey {
            view: self.view,
            size,
            max_iter: self.max_iter,
            dark,
        };
        if self.rendered != Some(key) {
            let image = self.render(size, palette, dark);
            match &mut self.texture {
                Some(t) => t.set(image, TextureOptions::LINEAR),
                None => {
                    self.texture = Some(painter.ctx().load_texture(
                        "mandelbrot",
                        image,
                        TextureOptions::LINEAR,
                    ))
                }
            }
            self.rendered = Some(key);
        }
        if let Some(t) = &self.texture {
            painter.image(
                t.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        let font = FontId::monospace(12.5);
        let label_bg = palette.card.gamma_multiply(0.85);
        let caption = |pos: Pos2, align: Align2, text: String, color: Color32| {
            let galley = painter.layout_no_wrap(text, font.clone(), color);
            let r = align.anchor_size(pos, galley.size()).expand(5.0);
            painter.rect_filled(r, 5.0, label_bg);
            painter.galley(r.min + vec2(5.0, 5.0), galley, color);
        };

        if let Some(at) = self.hover.filter(|p| rect.contains(*p)) {
            let (cx, cy) = self.to_complex(rect, at);
            let path: Vec<Pos2> = orbit(cx, cy, 60)
                .into_iter()
                .map(|z| self.to_screen(rect, z))
                .collect();
            painter.add(egui::Shape::line(
                path.clone(),
                Stroke::new(1.2, palette.ink.gamma_multiply(0.8)),
            ));
            for p in &path {
                painter.circle_filled(*p, 2.2, palette.ink);
            }
            painter.circle(
                at,
                5.0,
                Color32::TRANSPARENT,
                Stroke::new(2.0, palette.trace),
            );
            let fate = match escape(cx, cy, self.max_iter) {
                Some(mu) => format!("escapes after {:.0} iterations", mu.max(1.0)),
                None => format!("bounded for {} iterations", self.max_iter),
            };
            caption(
                rect.left_top() + vec2(14.0, 14.0),
                Align2::LEFT_TOP,
                format!(
                    "c = {cx:+.5} {} {:.5}i\n{fate}",
                    if cy < 0.0 { "−" } else { "+" },
                    cy.abs()
                ),
                palette.ink,
            );
        }
        caption(
            rect.right_bottom() - vec2(14.0, 14.0),
            Align2::RIGHT_BOTTOM,
            format!("zoom ×{:.0}", HOME.span / self.view.span),
            palette.trace,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("mandelbrot-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Max iterations");
                ui.add(egui::Slider::new(&mut self.max_iter, 30..=3000).logarithmic(true));
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for (name, view) in PRESETS {
                if ui.button(name).clicked() {
                    self.view = view;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_members() {
        for (x, y) in [(0.0, 0.0), (-1.0, 0.0), (-0.12, 0.75), (0.25, 0.0)] {
            assert_eq!(escape(x, y, 500), None, "({x}, {y}) should be bounded");
        }
    }

    #[test]
    fn known_escapees() {
        for (x, y) in [(1.0, 0.0), (0.5, 0.5), (-2.1, 0.0), (0.0, 1.1)] {
            assert!(escape(x, y, 500).is_some(), "({x}, {y}) should escape");
        }
    }

    #[test]
    fn bulb_shortcut_agrees_with_iteration() {
        // Sample a grid; wherever the shortcut claims membership, plain iteration agrees.
        for i in 0..60 {
            for j in 0..40 {
                let (x, y) = (-2.0 + i as f64 * 0.045, -1.0 + j as f64 * 0.05);
                if in_main_bulbs(x, y) {
                    let (mut zx, mut zy) = (0.0f64, 0.0f64);
                    for _ in 0..2000 {
                        (zx, zy) = (zx * zx - zy * zy + x, 2.0 * zx * zy + y);
                    }
                    assert!(zx * zx + zy * zy <= 4.0, "({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn smooth_count_matches_hand_computation() {
        // c = 1: z = 0, 1, 2, 5, 26 → |z|² = 676 passes 256 at step 4, so
        // μ = 4 + 1 − log₂(log₂ 26) ≈ 2.767.
        let mu = escape(1.0, 0.0, 100).unwrap();
        let expected = 5.0 - 26f64.log2().log2();
        assert!((mu - expected).abs() < 1e-9, "mu = {mu}");
    }

    #[test]
    fn orbit_of_minus_one_cycles() {
        let o = orbit(-1.0, 0.0, 6);
        assert_eq!(
            o,
            vec![
                [0.0, 0.0],
                [-1.0, 0.0],
                [0.0, 0.0],
                [-1.0, 0.0],
                [0.0, 0.0],
                [-1.0, 0.0],
                [0.0, 0.0]
            ]
        );
    }
}
