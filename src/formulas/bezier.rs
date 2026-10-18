//! Cubic Bézier curves: four points, three rounds of linear interpolation.

use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use super::lerp::ping_pong;
use crate::draw;
use crate::formula::{Formula, MathText, Palette, Tone};

/// The four Bernstein weights `(1−t)³, 3(1−t)²t, 3(1−t)t², t³`. They always sum to 1.
pub fn bernstein(t: f32) -> [f32; 4] {
    let u = 1.0 - t;
    [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t]
}

/// `B(t)` as the weighted sum of the control points.
pub fn point(p: &[Pos2; 4], t: f32) -> Pos2 {
    let w = bernstein(t);
    let mut out = Pos2::ZERO;
    for (q, w) in p.iter().zip(w) {
        out += q.to_vec2() * w;
    }
    out
}

/// De Casteljau's construction: the 3, 2 and 1 points produced by repeated lerps.
pub fn de_casteljau(p: &[Pos2; 4], t: f32) -> ([Pos2; 3], [Pos2; 2], Pos2) {
    let a = [p[0].lerp(p[1], t), p[1].lerp(p[2], t), p[2].lerp(p[3], t)];
    let b = [a[0].lerp(a[1], t), a[1].lerp(a[2], t)];
    (a, b, b[0].lerp(b[1], t))
}

pub struct Bezier {
    /// Control points as fractions of the curve panel.
    points: [Pos2; 4],
    t: f32,
    phase: f32,
    playing: bool,
    construction: bool,
    speed: u32,
    dragging: Option<usize>,
}

impl Default for Bezier {
    fn default() -> Self {
        Self {
            points: [
                pos2(0.08, 0.85),
                pos2(0.22, 0.08),
                pos2(0.7, 0.05),
                pos2(0.92, 0.75),
            ],
            t: 0.0,
            phase: 0.0,
            playing: true,
            construction: true,
            speed: 3,
            dragging: None,
        }
    }
}

fn panels(rect: Rect) -> (Rect, Rect) {
    let inner = rect.shrink2(vec2(24.0, 20.0));
    let bars_h = 86.0;
    let curve = Rect::from_min_max(inner.min, pos2(inner.max.x, inner.max.y - bars_h - 16.0));
    let bars = Rect::from_min_max(pos2(inner.min.x, inner.max.y - bars_h), inner.max);
    (curve, bars)
}

impl Bezier {
    fn on_screen(&self, panel: Rect) -> [Pos2; 4] {
        self.points.map(|p| panel.lerp_inside(p.to_vec2()))
    }
}

impl Formula for Bezier {
    fn params(&self) -> Vec<(String, f64)> {
        vec![("t".into(), self.t as f64)]
    }

    fn id(&self) -> &'static str {
        "cubic-bezier"
    }

    fn name(&self) -> &'static str {
        "Cubic Bézier curve"
    }

    fn category(&self) -> &'static str {
        super::CURVES
    }

    fn summary(&self) -> &'static str {
        "Two endpoints and two control points shape one smooth curve. Drag the points; \
         the colored lines show the same curve built by lerping lerps."
    }

    fn notes(&self) -> &'static str {
        "Smooth line charts, curved edges in node-link and Sankey diagrams, SVG and font \
         outlines, and CSS easing functions are all cubic Béziers. They give clean curves \
         at any zoom, with no jagged pixel steps."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("B(t)")
                .tone(Tone::Trace)
                .t(" = (1−t)")
                .sup("3")
                .t(" P")
                .sub("0")
                .t(" + 3(1−t)")
                .sup("2")
                .t("t P")
                .sub("1")
                .t(" + 3(1−t)t")
                .sup("2")
                .t(" P")
                .sub("2")
                .t(" + t")
                .sup("3")
                .t(" P")
                .sub("3"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::drag()
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.75).max(420.0)
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let (panel, _) = panels(rect);
        if response.drag_started()
            && let Some(at) = response.interact_pointer_pos()
        {
            self.dragging = draw::pick(&self.on_screen(panel), at, 24.0);
        }
        if let (Some(i), Some(at)) = (self.dragging, response.interact_pointer_pos()) {
            let at = panel.clamp(at);
            self.points[i] = pos2(
                (at.x - panel.min.x) / panel.width(),
                (at.y - panel.min.y) / panel.height(),
            );
        }
        if response.drag_stopped() {
            self.dragging = None;
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.playing {
            self.phase += 0.1 * self.speed as f32 * dt;
            self.t = ping_pong(self.phase);
        }
        let t = self.t;
        let (panel, bars) = panels(rect);
        let p = self.on_screen(panel);
        let font = FontId::proportional(13.0);

        painter.extend(Shape::dashed_line(
            &p,
            Stroke::new(1.0, palette.muted.gamma_multiply(0.7)),
            4.0,
            4.0,
        ));

        let steps = 120;
        let curve: Vec<Pos2> = (0..=steps)
            .map(|j| point(&p, j as f32 / steps as f32))
            .collect();
        painter.add(Shape::line(
            curve,
            Stroke::new(1.5, palette.trace.gamma_multiply(0.45)),
        ));
        let done = (t * steps as f32).ceil() as usize;
        let drawn: Vec<Pos2> = (0..=done)
            .map(|j| point(&p, (j as f32 / steps as f32).min(t)))
            .collect();
        if drawn.len() > 1 {
            painter.add(Shape::line(drawn, Stroke::new(3.0, palette.trace)));
        }

        let (a, b, c) = de_casteljau(&p, t);
        if self.construction {
            painter.add(Shape::line(a.to_vec(), Stroke::new(1.5, palette.x)));
            painter.line_segment([b[0], b[1]], Stroke::new(1.5, palette.y));
            for q in a {
                painter.circle_filled(q, 3.5, palette.x);
            }
            for q in b {
                painter.circle_filled(q, 3.5, palette.y);
            }
        }

        for (i, q) in p.iter().enumerate() {
            let endpoint = i == 0 || i == 3;
            let fill = if endpoint { palette.ink } else { palette.card };
            draw::handle(painter, *q, fill, palette.muted, self.dragging == Some(i));
            painter.text(
                *q + vec2(0.0, -12.0),
                Align2::CENTER_BOTTOM,
                format!("P{i}"),
                font.clone(),
                palette.muted,
            );
        }
        painter.circle(c, 6.5, palette.trace, Stroke::new(2.0, palette.card));

        // The Bernstein weights: how much each point pulls on B(t) right now.
        let labels = ["(1−t)³", "3(1−t)²t", "3(1−t)t²", "t³"];
        let w = bernstein(t);
        let col_w = bars.width() / 4.0;
        let max_h = bars.height() - 34.0;
        for i in 0..4 {
            let x0 = bars.left() + col_w * i as f32 + 10.0;
            let x1 = x0 + col_w - 20.0;
            let base = bars.top() + max_h;
            let track = Rect::from_min_max(pos2(x0, bars.top()), pos2(x1, base));
            painter.rect_filled(track, 4.0, palette.rule.gamma_multiply(0.5));
            let bar = Rect::from_min_max(pos2(x0, base - w[i] * max_h), pos2(x1, base));
            painter.rect_filled(bar, 4.0, palette.trace);
            painter.text(
                pos2((x0 + x1) / 2.0, base + 4.0),
                Align2::CENTER_TOP,
                labels[i],
                font.clone(),
                palette.muted,
            );
            painter.text(
                pos2((x0 + x1) / 2.0, base + 19.0),
                Align2::CENTER_TOP,
                format!("P{i}: {:.2}", w[i]),
                font.clone(),
                palette.ink,
            );
        }
        painter.text(
            panel.right_top(),
            Align2::RIGHT_TOP,
            format!("t = {t:.2}"),
            FontId::proportional(18.0),
            palette.trace,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("bezier-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("t");
                if ui
                    .add(
                        egui::Slider::new(&mut self.t, 0.0..=1.0)
                            .custom_formatter(|v, _| format!("{v:.2}")),
                    )
                    .changed()
                {
                    self.playing = false;
                    self.phase = self.t;
                }
                ui.end_row();
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.speed, 1..=10));
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui
                .button(if self.construction {
                    "Hide construction"
                } else {
                    "Show construction"
                })
                .clicked()
            {
                self.construction = !self.construction;
            }
            if ui.button("Reset points").clicked() {
                self.points = Self::default().points;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: [Pos2; 4] = [
        pos2(0.0, 0.0),
        pos2(1.0, 3.0),
        pos2(4.0, 3.0),
        pos2(5.0, 0.0),
    ];

    #[test]
    fn weights_sum_to_one() {
        for i in 0..=10 {
            let s: f32 = bernstein(i as f32 / 10.0).iter().sum();
            assert!((s - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn passes_through_endpoints() {
        assert_eq!(point(&P, 0.0), P[0]);
        assert_eq!(point(&P, 1.0), P[3]);
    }

    #[test]
    fn de_casteljau_matches_bernstein() {
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            let (_, _, c) = de_casteljau(&P, t);
            assert!(c.distance(point(&P, t)) < 1e-4);
        }
    }
}
