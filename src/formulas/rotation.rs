//! The 2D rotation matrix: turn any point about the origin by an angle θ.

use std::f32::consts::PI;

use egui::epaint::TextShape;
use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::derive::{Derivation, Step, Substitution};
use crate::draw;
use crate::formula::{Formula, MathText, Palette, Tone};

pub type Point = [f32; 2];

/// `x' = x cos θ − y sin θ`, `y' = x sin θ + y cos θ` (counter-clockwise, y up).
pub fn rotate([x, y]: Point, theta: f32) -> Point {
    let (s, c) = theta.sin_cos();
    [x * c - y * s, x * s + y * c]
}

/// Do two equal-angle text labels collide? They are `spacing` apart horizontally,
/// the right one is `width` long, both are `height` tall and tilted by `angle`.
///
/// In the labels' own rotated frame the offset between them is
/// `(spacing cos a, spacing sin a)`; congruent parallel boxes overlap exactly
/// when that offset is shorter than the box along both axes.
pub fn labels_collide(spacing: f32, width: f32, height: f32, angle: f32) -> bool {
    let (s, c) = angle.abs().sin_cos();
    spacing * c < width && spacing * s < height
}

/// A capital F: asymmetric, so its orientation is always readable.
const SHAPE: [Point; 10] = [
    [-0.25, -0.5],
    [-0.05, -0.5],
    [-0.05, -0.05],
    [0.2, -0.05],
    [0.2, 0.12],
    [-0.05, 0.12],
    [-0.05, 0.32],
    [0.3, 0.32],
    [0.3, 0.5],
    [-0.25, 0.5],
];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const SALES: [f32; 12] = [
    0.42, 0.55, 0.61, 0.48, 0.72, 0.9, 0.84, 0.77, 0.63, 0.58, 0.69, 0.95,
];
const CHART_H: f32 = 170.0;

pub struct Rotation {
    theta: f32,
    spinning: bool,
    /// The point being rotated, in plane units.
    point: Point,
    dragging: bool,
    /// Tilt of the axis labels in the chart below, in radians.
    label_angle: f32,
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            theta: PI / 6.0,
            spinning: true,
            point: [0.78, 0.32],
            dragging: false,
            label_angle: 0.0,
        }
    }
}

fn plane(rect: Rect) -> (Pos2, f32) {
    let top = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.max.y - CHART_H));
    (
        top.center() + vec2(0.0, 6.0),
        (top.width().min(top.height()) * 0.42).max(40.0),
    )
}

/// What a sheet saves of a [`Rotation`].
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct State {
    theta: f32,
    spinning: bool,
    /// The point being rotated, in plane units.
    point: [f32; 2],
    /// Tilt of the chart's labels, in radians.
    label_angle: f32,
}

impl Rotation {
    fn state(&self) -> State {
        State {
            theta: self.theta,
            spinning: self.spinning,
            point: self.point,
            label_angle: self.label_angle,
        }
    }

    fn apply(&mut self, s: State) {
        self.theta = if (-PI..=PI).contains(&s.theta) {
            s.theta
        } else {
            (s.theta + PI).rem_euclid(2.0 * PI) - PI
        };
        self.spinning = s.spinning;
        let [x, y] = s.point;
        let len = x.hypot(y);
        self.point = if len > 1.1 {
            [x / len * 1.1, y / len * 1.1]
        } else {
            [x, y]
        };
        self.label_angle = s.label_angle.clamp(0.0, PI / 2.0);
    }
}

impl Formula for Rotation {
    crate::state_hooks!();

    fn derivations(&self) -> Vec<Derivation> {
        let m = MathText::new;
        vec![Derivation::new(
            "From the angle-addition identities",
            vec![
                Step::new(
                    m().t("x = r cos α,   y = r sin α"),
                    "Write the point in polar form: distance r from the origin at angle α.",
                ),
                Step::new(
                    m().t("x'")
                        .tone(Tone::X)
                        .t(" = r cos(α + θ),   ")
                        .t("y'")
                        .tone(Tone::Y)
                        .t(" = r sin(α + θ)"),
                    "Rotating about the origin keeps r and adds θ to the angle.",
                ),
                Step::new(
                    m().t("cos(α + θ) = cos α cos θ − sin α sin θ"),
                    "The angle-addition identity for cosine.",
                ),
                Step::new(
                    m().t("sin(α + θ) = sin α cos θ + cos α sin θ"),
                    "And the one for sine.",
                ),
                Step::new(
                    m().t("x'")
                        .tone(Tone::X)
                        .t(" = (r cos α) cos θ − (r sin α) sin θ"),
                    "Expand x' with the cosine identity.",
                ),
                Step::new(
                    m().t("x'").tone(Tone::X).t(" = x cos θ − y sin θ"),
                    "r cos α is x and r sin α is y.",
                ),
                Step::new(
                    m().t("y'").tone(Tone::Y).t(" = x sin θ + y cos θ"),
                    "The same with the sine identity. The four coefficients are the rotation matrix.",
                ),
            ],
        )]
    }

    fn substitutions(&self) -> Vec<Substitution> {
        let values = [
            ("θ", self.theta as f64),
            ("x", self.point[0] as f64),
            ("y", self.point[1] as f64),
        ];
        vec![
            Substitution::new(
                MathText::new().t("x'").tone(Tone::X),
                "x cos(θ) - y sin(θ)",
                &values,
            ),
            Substitution::new(
                MathText::new().t("y'").tone(Tone::Y),
                "x sin(θ) + y cos(θ)",
                &values,
            ),
        ]
    }

    fn params(&self) -> Vec<(String, f64)> {
        vec![
            ("θ".into(), self.theta as f64),
            ("x".into(), self.point[0] as f64),
            ("y".into(), self.point[1] as f64),
        ]
    }

    fn id(&self) -> &'static str {
        "rotation-matrix"
    }

    fn name(&self) -> &'static str {
        "2D rotation"
    }

    fn category(&self) -> &'static str {
        super::GEOMETRY
    }

    fn summary(&self) -> &'static str {
        "Two lines of arithmetic turn any point about the origin. The colored arrows are \
         the rotated axes. Below, tilt the month labels until they stop colliding."
    }

    fn notes(&self) -> &'static str {
        "Rotated axis labels that would otherwise overlap, tilted text on radar-chart \
         spokes, compass needles and gauges, spinning dashboard widgets, and every 2D \
         sprite or icon rotation. In 3D the same idea becomes a 3×3 matrix per axis."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("x'")
                .tone(Tone::X)
                .t(" = x cos(θ) − y sin(θ)"),
            MathText::new()
                .t("y'")
                .tone(Tone::Y)
                .t(" = x sin(θ) + y cos(θ)"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::drag()
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.6).max(300.0) + CHART_H
    }

    fn min_size(&self) -> egui::Vec2 {
        // Room for the readout beside the circle.
        egui::vec2(480.0, self.stage_height(480.0))
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let (center, scale) = plane(rect);
        let to_screen = |[x, y]: Point| pos2(center.x + x * scale, center.y - y * scale);
        if response.drag_started()
            && let Some(at) = response.interact_pointer_pos()
        {
            self.dragging = draw::pick(&[to_screen(self.point)], at, 24.0).is_some();
        }
        if self.dragging
            && let Some(at) = response.interact_pointer_pos()
        {
            let p = [(at.x - center.x) / scale, (center.y - at.y) / scale];
            let len = (p[0] * p[0] + p[1] * p[1]).sqrt();
            self.point = if len > 1.1 {
                [p[0] * 1.1 / len, p[1] * 1.1 / len]
            } else {
                p
            };
        }
        if response.drag_stopped() {
            self.dragging = false;
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.spinning {
            self.theta = (self.theta + 0.5 * dt + PI).rem_euclid(2.0 * PI) - PI;
        }
        let theta = self.theta;
        let (center, scale) = plane(rect);
        let to_screen = |[x, y]: Point| pos2(center.x + x * scale, center.y - y * scale);

        let grid = Stroke::new(1.0, palette.rule);
        painter.circle_stroke(center, scale, grid);
        painter.line_segment([to_screen([-1.15, 0.0]), to_screen([1.15, 0.0])], grid);
        painter.line_segment([to_screen([0.0, -1.15]), to_screen([0.0, 1.15])], grid);

        // The shape before (dashed) and after rotation (filled).
        let mut before: Vec<Pos2> = SHAPE.iter().map(|&p| to_screen(p)).collect();
        before.push(before[0]);
        painter.extend(Shape::dashed_line(
            &before,
            Stroke::new(1.0, palette.muted.gamma_multiply(0.7)),
            4.0,
            4.0,
        ));
        let after: Vec<Pos2> = SHAPE.iter().map(|&p| to_screen(rotate(p, theta))).collect();
        painter.add(draw::fill_polygon(
            &after,
            palette.trace.gamma_multiply(0.25),
            1.0,
        ));
        painter.add(Shape::closed_line(after, Stroke::new(2.0, palette.trace)));

        // Basis vectors: the matrix's columns are where (1, 0) and (0, 1) land.
        let faint = 0.3;
        painter.arrow(
            center,
            to_screen([1.0, 0.0]) - center,
            Stroke::new(1.5, palette.x.gamma_multiply(faint)),
        );
        painter.arrow(
            center,
            to_screen([0.0, 1.0]) - center,
            Stroke::new(1.5, palette.y.gamma_multiply(faint)),
        );
        let i = rotate([1.0, 0.0], theta);
        let j = rotate([0.0, 1.0], theta);
        painter.arrow(center, to_screen(i) - center, Stroke::new(3.0, palette.x));
        painter.arrow(center, to_screen(j) - center, Stroke::new(3.0, palette.y));
        let small = FontId::proportional(13.0);
        painter.text(
            to_screen([i[0] * 1.12, i[1] * 1.12]),
            Align2::CENTER_CENTER,
            "(cos θ, sin θ)",
            small.clone(),
            palette.x,
        );
        painter.text(
            to_screen([j[0] * 1.12, j[1] * 1.12]),
            Align2::CENTER_CENTER,
            "(−sin θ, cos θ)",
            small.clone(),
            palette.y,
        );

        // One point and its image, joined by the arc it travels along.
        let p = self.point;
        let q = rotate(p, theta);
        let steps = 40;
        let arc: Vec<Pos2> = (0..=steps)
            .map(|k| to_screen(rotate(p, theta * k as f32 / steps as f32)))
            .collect();
        painter.extend(Shape::dashed_line(
            &arc,
            Stroke::new(1.2, palette.muted),
            3.0,
            3.0,
        ));
        draw::handle(
            painter,
            to_screen(p),
            palette.card,
            palette.muted,
            self.dragging,
        );
        painter.circle(
            to_screen(q),
            6.5,
            palette.ink,
            Stroke::new(2.0, palette.card),
        );
        painter.text(
            to_screen(p) + vec2(10.0, 8.0),
            Align2::LEFT_TOP,
            "P",
            small.clone(),
            palette.muted,
        );
        painter.text(
            to_screen(q) + vec2(10.0, -8.0),
            Align2::LEFT_BOTTOM,
            "P'",
            small,
            palette.ink,
        );

        let mono = FontId::monospace(12.5);
        let (s, c) = theta.sin_cos();
        let base = pos2(rect.left() + 16.0, rect.bottom() - CHART_H - 50.0);
        painter.text(
            base,
            Align2::LEFT_TOP,
            format!("θ = {:.0}°", theta.to_degrees()),
            mono.clone(),
            palette.ink,
        );
        painter.text(
            base + vec2(0.0, 16.0),
            Align2::LEFT_TOP,
            format!("x' = {:.2}·{c:.2} − {:.2}·{s:.2} = {:.2}", p[0], p[1], q[0]),
            mono.clone(),
            palette.x,
        );
        painter.text(
            base + vec2(0.0, 32.0),
            Align2::LEFT_TOP,
            format!("y' = {:.2}·{s:.2} + {:.2}·{c:.2} = {:.2}", p[0], p[1], q[1]),
            mono.clone(),
            palette.y,
        );

        self.paint_labels(painter, rect, palette);
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("rotation-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("θ");
                let mut deg = self.theta.to_degrees();
                if ui
                    .add(egui::Slider::new(&mut deg, -180.0..=180.0).suffix("°"))
                    .changed()
                {
                    self.theta = deg.to_radians();
                    self.spinning = false;
                }
                ui.end_row();
                ui.label("Label tilt");
                let mut tilt = self.label_angle.to_degrees();
                if ui
                    .add(egui::Slider::new(&mut tilt, 0.0..=90.0).suffix("°"))
                    .changed()
                {
                    self.label_angle = tilt.to_radians();
                }
                ui.end_row();
            });
        ui.add_space(6.0);
        if ui
            .button(if self.spinning { "Stop" } else { "Spin" })
            .clicked()
        {
            self.spinning = !self.spinning;
        }
    }
}

impl Rotation {
    /// A bar chart whose month labels are rotated by `label_angle`.
    fn paint_labels(&self, painter: &egui::Painter, rect: Rect, palette: &Palette) {
        let area = Rect::from_min_max(
            pos2(rect.left() + 24.0, rect.bottom() - CHART_H + 10.0),
            pos2(rect.right() - 24.0, rect.bottom() - 12.0),
        );
        painter.line_segment(
            [
                pos2(rect.left() + 12.0, area.top() - 10.0),
                pos2(rect.right() - 12.0, area.top() - 10.0),
            ],
            Stroke::new(1.0, palette.rule),
        );
        let bars_h = 52.0;
        let baseline = area.top() + bars_h;
        let slot = area.width() / 12.0;
        let font = FontId::proportional(12.5);
        let angle = self.label_angle;

        let galleys: Vec<_> = MONTHS
            .iter()
            .map(|m| painter.layout_no_wrap((*m).to_owned(), font.clone(), palette.ink))
            .collect();
        let mut collisions = 0;
        for (k, galley) in galleys.iter().enumerate() {
            let x = area.left() + slot * (k as f32 + 0.5);
            let bar = Rect::from_min_max(
                pos2(x - slot * 0.3, baseline - SALES[k] * bars_h),
                pos2(x + slot * 0.3, baseline),
            );
            painter.rect_filled(bar, 2.0, palette.trace.gamma_multiply(0.7));

            let size = galley.size();
            let hits_left = k > 0 && labels_collide(slot, size.x, size.y, angle);
            let hits_right = k + 1 < galleys.len()
                && labels_collide(slot, galleys[k + 1].size().x, size.y, angle);
            let color = if hits_left || hits_right {
                palette.y
            } else {
                palette.ink
            };
            if hits_right {
                collisions += 1;
            }
            // Anchor the label's right end under its bar and tilt it upward.
            let anchor = pos2(
                x + if angle > 0.01 { 4.0 } else { size.x / 2.0 },
                baseline + 6.0,
            );
            let shape = TextShape::new(anchor - vec2(size.x, 0.0), galley.clone(), color)
                .with_override_text_color(color)
                .with_angle_and_anchor(-angle, Align2::RIGHT_TOP);
            painter.add(shape);
        }
        painter.line_segment(
            [pos2(area.left(), baseline), pos2(area.right(), baseline)],
            Stroke::new(1.0, palette.muted),
        );
        let note = if collisions == 0 {
            "no labels collide".to_owned()
        } else {
            format!("{collisions} label pairs collide: tilt them")
        };
        painter.text(
            pos2(area.right(), area.top() - 4.0),
            Align2::RIGHT_TOP,
            note,
            FontId::monospace(12.0),
            if collisions == 0 {
                palette.x
            } else {
                palette.y
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Point, b: Point) -> bool {
        (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5
    }

    #[test]
    fn quarter_turn() {
        assert!(close(rotate([1.0, 0.0], PI / 2.0), [0.0, 1.0]));
        assert!(close(rotate([0.0, 1.0], PI / 2.0), [-1.0, 0.0]));
    }

    #[test]
    fn preserves_length_and_inverts() {
        let p = [0.3, -1.7];
        let q = rotate(p, 1.234);
        assert!(((p[0].hypot(p[1])) - q[0].hypot(q[1])).abs() < 1e-5);
        assert!(close(rotate(q, -1.234), p));
    }

    #[test]
    fn label_collision() {
        // Flat labels wider than their spacing collide; tilting them fixes it.
        assert!(labels_collide(40.0, 60.0, 14.0, 0.0));
        assert!(!labels_collide(40.0, 60.0, 14.0, PI / 4.0));
        assert!(!labels_collide(40.0, 30.0, 14.0, 0.0));
        // Vertical labels only collide if they are taller than the spacing.
        assert!(!labels_collide(40.0, 60.0, 14.0, PI / 2.0));
    }
}
