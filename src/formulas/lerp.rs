//! Linear interpolation: one parameter t walks from a start value to an end value.

use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::derive::{Derivation, Step, Substitution};
use crate::draw::{self, lerp, lerp_color};
use crate::formula::{Formula, MathText, Palette, Tone};

/// Ping-pong `s` (any real) into `[0, 1]` with constant speed: a triangle wave.
pub fn ping_pong(s: f32) -> f32 {
    1.0 - (1.0 - s.rem_euclid(2.0)).abs()
}

pub struct Lerp {
    t: f32,
    phase: f32,
    playing: bool,
    speed: u32,
    v0: f32,
    v1: f32,
    /// Endpoints in the plane panel, as fractions of that panel.
    ends: [Pos2; 2],
    dragging: Option<usize>,
}

impl Default for Lerp {
    fn default() -> Self {
        Self {
            t: 0.0,
            phase: 0.0,
            playing: true,
            speed: 3,
            v0: 20.0,
            v1: 80.0,
            ends: [pos2(0.12, 0.75), pos2(0.88, 0.25)],
            dragging: None,
        }
    }
}

struct Layout {
    plane: Rect,
    line: Rect,
    bar: Rect,
}

fn layout(rect: Rect) -> Layout {
    let inner = rect.shrink2(vec2(28.0, 20.0));
    // The number line and color bar take a fixed 140 pt; the plane gets the rest.
    let plane = Rect::from_min_max(inner.min, pos2(inner.max.x, inner.max.y - 140.0));
    let line = Rect::from_min_max(
        pos2(inner.min.x, plane.max.y + 26.0),
        pos2(inner.max.x, plane.max.y + 56.0),
    );
    let bar = Rect::from_min_max(
        pos2(inner.min.x, line.max.y + 28.0),
        pos2(inner.max.x, line.max.y + 56.0),
    );
    Layout { plane, line, bar }
}

impl Lerp {
    fn ends_on_screen(&self, plane: Rect) -> [Pos2; 2] {
        self.ends.map(|p| plane.lerp_inside(p.to_vec2()))
    }
}

impl Formula for Lerp {
    fn derivations(&self) -> Vec<Derivation> {
        let m = MathText::new;
        let v = |i: &str| m().t("v").sub(i);
        vec![Derivation::new(
            "A weighted average",
            vec![
                Step::new(
                    m().t("v(t)")
                        .tone(Tone::Trace)
                        .t(" = (1 − t) ")
                        .append(v("0"))
                        .t(" + t ")
                        .append(v("1")),
                    "Mix the two values with weights 1 − t and t, which add up to 1.",
                ),
                Step::new(
                    m().t("v(t)")
                        .tone(Tone::Trace)
                        .t(" = ")
                        .append(v("0"))
                        .t(" − t ")
                        .append(v("0"))
                        .t(" + t ")
                        .append(v("1")),
                    "Multiply out the first term.",
                ),
                Step::new(
                    m().t("v(t)")
                        .tone(Tone::Trace)
                        .t(" = ")
                        .append(v("0"))
                        .t(" + t (")
                        .append(v("1"))
                        .t(" − ")
                        .append(v("0"))
                        .t(")"),
                    "Group the t terms: start at v_0 and move a fraction t of the way to v_1.",
                ),
                Step::new(
                    m().t("v(0) = ")
                        .append(v("0"))
                        .t(",   v(1) = ")
                        .append(v("1")),
                    "Check the ends: t = 0 gives the start and t = 1 the end.",
                ),
                Step::new(
                    m().t("v(1/2) = (")
                        .append(v("0"))
                        .t(" + ")
                        .append(v("1"))
                        .t(") / 2"),
                    "Halfway is the ordinary mean.",
                ),
            ],
        )]
    }

    fn substitutions(&self) -> Vec<Substitution> {
        vec![Substitution::new(
            MathText::new().t("v(t)").tone(Tone::Trace),
            "v_0 + t (v_1 - v_0)",
            &[
                ("t", self.t as f64),
                ("v_0", self.v0 as f64),
                ("v_1", self.v1 as f64),
            ],
        )]
    }

    fn params(&self) -> Vec<(String, f64)> {
        vec![
            ("t".into(), self.t as f64),
            ("v_0".into(), self.v0 as f64),
            ("v_1".into(), self.v1 as f64),
        ]
    }

    fn id(&self) -> &'static str {
        "lerp"
    }

    fn name(&self) -> &'static str {
        "Linear interpolation"
    }

    fn category(&self) -> &'static str {
        super::CURVES
    }

    fn summary(&self) -> &'static str {
        "One number t between 0 and 1 blends a start and an end: positions, plain \
         numbers and colors alike. Drag the endpoints; equal steps in t are equal steps along the way."
    }

    fn notes(&self) -> &'static str {
        "Color gradients and color scales, animated transitions between chart states, \
         tweening positions, and estimating values between two known samples. It is also \
         the building block of Bézier curves."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("v(t)")
                .tone(Tone::Trace)
                .t(" = v")
                .sub("0")
                .tone(Tone::X)
                .t(" + t (v")
                .sub("1")
                .tone(Tone::Y)
                .t(" − v")
                .sub("0")
                .tone(Tone::X)
                .t("),   0 ≤ t ≤ 1"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::drag()
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.62).max(380.0)
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let plane = layout(rect).plane;
        if response.drag_started()
            && let Some(at) = response.interact_pointer_pos()
        {
            self.dragging = draw::pick(&self.ends_on_screen(plane), at, 24.0);
        }
        if let (Some(i), Some(at)) = (self.dragging, response.interact_pointer_pos()) {
            let at = plane.clamp(at);
            self.ends[i] = pos2(
                (at.x - plane.min.x) / plane.width(),
                (at.y - plane.min.y) / plane.height(),
            );
        }
        if response.drag_stopped() {
            self.dragging = None;
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.playing {
            self.phase += 0.12 * self.speed as f32 * dt;
            self.t = ping_pong(self.phase);
        }
        let t = self.t;
        let Layout { plane, line, bar } = layout(rect);
        let font = FontId::proportional(13.0);

        // 1. Points in the plane.
        let [a, b] = self.ends_on_screen(plane);
        painter.extend(Shape::dashed_line(
            &[a, b],
            Stroke::new(1.0, palette.muted),
            4.0,
            4.0,
        ));
        for k in 1..10 {
            let u = k as f32 / 10.0;
            painter.circle_filled(a.lerp(b, u), 2.0, palette.muted.gamma_multiply(0.6));
        }
        painter.line_segment([a, a.lerp(b, t)], Stroke::new(3.0, palette.trace));
        draw::handle(
            painter,
            a,
            palette.x,
            palette.card,
            self.dragging == Some(0),
        );
        draw::handle(
            painter,
            b,
            palette.y,
            palette.card,
            self.dragging == Some(1),
        );
        let p = a.lerp(b, t);
        painter.circle(p, 6.0, palette.ink, Stroke::new(2.0, palette.card));
        painter.text(
            a + vec2(0.0, 14.0),
            Align2::CENTER_TOP,
            "P₀",
            font.clone(),
            palette.x,
        );
        painter.text(
            b + vec2(0.0, 14.0),
            Align2::CENTER_TOP,
            "P₁",
            font.clone(),
            palette.y,
        );
        painter.text(
            p + vec2(10.0, -10.0),
            Align2::LEFT_BOTTOM,
            format!("P(t), t = {t:.2}"),
            font.clone(),
            palette.ink,
        );

        // 2. A plain number on a number line.
        let v = lerp(self.v0, self.v1, t);
        let x_of = |value: f32| lerp(line.left(), line.right(), value / 100.0);
        let y = line.center().y;
        painter.line_segment(
            [pos2(line.left(), y), pos2(line.right(), y)],
            Stroke::new(1.0, palette.rule),
        );
        for tick in 0..=10 {
            let x = x_of(tick as f32 * 10.0);
            painter.line_segment(
                [pos2(x, y - 4.0), pos2(x, y + 4.0)],
                Stroke::new(1.0, palette.rule),
            );
        }
        painter.line_segment(
            [pos2(x_of(self.v0), y), pos2(x_of(v), y)],
            Stroke::new(3.0, palette.trace),
        );
        painter.circle_filled(pos2(x_of(self.v0), y), 5.0, palette.x);
        painter.circle_filled(pos2(x_of(self.v1), y), 5.0, palette.y);
        painter.circle(
            pos2(x_of(v), y),
            6.0,
            palette.ink,
            Stroke::new(2.0, palette.card),
        );
        painter.text(
            pos2(x_of(v), y - 10.0),
            Align2::CENTER_BOTTOM,
            format!("v = {v:.1}"),
            font.clone(),
            palette.ink,
        );
        painter.text(
            pos2(line.left(), y + 8.0),
            Align2::LEFT_TOP,
            "0",
            font.clone(),
            palette.muted,
        );
        painter.text(
            pos2(line.right(), y + 8.0),
            Align2::RIGHT_TOP,
            "100",
            font.clone(),
            palette.muted,
        );

        // 3. A color, channel by channel.
        let steps = 64;
        let mut mesh = egui::Mesh::default();
        for k in 0..steps {
            let u0 = k as f32 / steps as f32;
            let u1 = (k + 1) as f32 / steps as f32;
            let r = Rect::from_min_max(
                pos2(lerp(bar.left(), bar.right(), u0), bar.top()),
                pos2(lerp(bar.left(), bar.right(), u1) + 0.5, bar.bottom()),
            );
            mesh.add_colored_rect(r, lerp_color(palette.x, palette.y, (u0 + u1) / 2.0));
        }
        painter.add(mesh);
        let mx = lerp(bar.left(), bar.right(), t);
        painter.line_segment(
            [pos2(mx, bar.top() - 6.0), pos2(mx, bar.bottom() + 6.0)],
            Stroke::new(2.0, palette.ink),
        );
        let swatch = Rect::from_center_size(pos2(mx, bar.bottom() + 20.0), vec2(26.0, 18.0));
        painter.rect(
            swatch,
            4.0,
            lerp_color(palette.x, palette.y, t),
            Stroke::new(1.5, palette.ink),
            egui::StrokeKind::Inside,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("lerp-controls")
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
                ui.label("v₀");
                ui.add(egui::Slider::new(&mut self.v0, 0.0..=100.0));
                ui.end_row();
                ui.label("v₁");
                ui.add(egui::Slider::new(&mut self.v1, 0.0..=100.0));
                ui.end_row();
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.speed, 1..=10));
                ui.end_row();
            });
        ui.add_space(6.0);
        if ui
            .button(if self.playing { "Pause" } else { "Play" })
            .clicked()
        {
            self.playing = !self.playing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_endpoints_and_midpoint() {
        assert_eq!(lerp(20.0, 80.0, 0.0), 20.0);
        assert_eq!(lerp(20.0, 80.0, 1.0), 80.0);
        assert_eq!(lerp(20.0, 80.0, 0.5), 50.0);
    }

    #[test]
    fn ping_pong_is_a_triangle() {
        assert_eq!(ping_pong(0.0), 0.0);
        assert_eq!(ping_pong(0.25), 0.25);
        assert_eq!(ping_pong(1.0), 1.0);
        assert_eq!(ping_pong(1.25), 0.75);
        assert_eq!(ping_pong(2.0), 0.0);
    }
}
