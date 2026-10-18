//! The quadratic formula: a parabola with its roots, and the formula
//! derived by completing the square.

use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2};

use crate::derive::{Derivation, Step, Substitution};
use crate::formula::{Formula, MathText, Palette, Tone};
use crate::formulas::expression::plot::{View, sample_explicit};
use crate::formulas::expression::{paint_axes, pan_zoom};

/// The real solutions of a x² + b x + c = 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Roots {
    /// Two roots, smaller first.
    Two(f64, f64),
    /// A double root, where the parabola touches the axis.
    One(f64),
    /// No real roots: the discriminant is negative.
    None,
    /// a = 0, so b x + c = 0: one root, or none when b = 0 too.
    Linear(Option<f64>),
}

/// Below this size a coefficient or the discriminant counts as zero.
const EPS: f64 = 1e-12;

pub fn discriminant(a: f64, b: f64, c: f64) -> f64 {
    b * b - 4.0 * a * c
}

/// The roots, computed without cancellation: q = −(b + sign(b) √Δ) / 2,
/// then x = q / a and x = c / q.
pub fn roots(a: f64, b: f64, c: f64) -> Roots {
    if a.abs() < EPS {
        return Roots::Linear((b.abs() >= EPS).then(|| -c / b));
    }
    let d = discriminant(a, b, c);
    if d.abs() < EPS {
        return Roots::One(-b / (2.0 * a));
    }
    if d < 0.0 {
        return Roots::None;
    }
    let sign = if b < 0.0 { -1.0 } else { 1.0 };
    let q = -0.5 * (b + sign * d.sqrt());
    let (x1, x2) = (q / a, c / q);
    Roots::Two(x1.min(x2), x1.max(x2))
}

pub struct Quadratic {
    a: f64,
    b: f64,
    c: f64,
    view: View,
    hover: Option<Pos2>,
}

impl Default for Quadratic {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: -1.0,
            c: -2.0,
            view: View {
                cx: 0.5,
                cy: 0.0,
                scale: 0.018,
            },
            hover: None,
        }
    }
}

impl Quadratic {
    fn f(&self, x: f64) -> f64 {
        (self.a * x + self.b) * x + self.c
    }

    /// What the discriminant says, for the caption.
    fn caption(&self) -> String {
        let d = discriminant(self.a, self.b, self.c);
        match roots(self.a, self.b, self.c) {
            Roots::Two(x1, x2) => {
                format!("Δ = {d:.2} > 0: two real roots, x = {x1:.3} and x = {x2:.3}")
            }
            Roots::One(x) => format!("Δ = 0: one double root, x = {x:.3}"),
            Roots::None => format!("Δ = {d:.2} < 0: no real roots, the parabola misses the axis"),
            Roots::Linear(Some(x)) => format!("a = 0: the equation is linear, one root x = {x:.3}"),
            Roots::Linear(None) => "a = 0 and b = 0: no x to solve for".into(),
        }
    }
}

impl Formula for Quadratic {
    fn id(&self) -> &'static str {
        "quadratic-formula"
    }

    fn name(&self) -> &'static str {
        "Quadratic formula"
    }

    fn category(&self) -> &'static str {
        crate::formulas::PROOFS
    }

    fn summary(&self) -> &'static str {
        "Where does a parabola cross the x axis? Move a, b and c and watch the roots, \
         and open the Derive tab to see the formula come out of completing the square."
    }

    fn notes(&self) -> &'static str {
        "Projectile landing points, break-even quantities, the intersection of a ray \
         with a sphere in a ray tracer, and the time to collision of two moving objects \
         all come down to solving a quadratic."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new().t("a x").sup("2").t(" + b x + c = 0"),
            MathText::new()
                .t("x")
                .tone(Tone::X)
                .t(" = (−b ± √(b")
                .sup("2")
                .t(" − 4ac)) / 2a"),
            MathText::new()
                .t("Δ")
                .tone(Tone::Y)
                .t(" = b")
                .sup("2")
                .t(" − 4ac"),
        ]
    }

    fn params(&self) -> Vec<(String, f64)> {
        vec![
            ("a".into(), self.a),
            ("b".into(), self.b),
            ("c".into(), self.c),
        ]
    }

    fn derivations(&self) -> Vec<Derivation> {
        let m = MathText::new;
        let half = || m().t("(b / 2a)").sup("2");
        vec![Derivation::new(
            "Completing the square",
            vec![
                Step::new(
                    m().t("a x").sup("2").t(" + b x + c = 0"),
                    "The equation, with a not zero.",
                ),
                Step::new(
                    m().t("x").sup("2").t(" + (b / a) x + c / a = 0"),
                    "Divide by a.",
                ),
                Step::new(
                    m().t("x").sup("2").t(" + (b / a) x = −c / a"),
                    "Move the constant to the right.",
                ),
                Step::new(
                    m().t("x")
                        .sup("2")
                        .t(" + (b / a) x + ")
                        .append(half().tone(Tone::Y))
                        .t(" = ")
                        .append(half().tone(Tone::Y))
                        .t(" − c / a"),
                    "Add the square of half the x coefficient to both sides.",
                ),
                Step::new(
                    m().t("(x + b / 2a)")
                        .tone(Tone::Trace)
                        .sup("2")
                        .tone(Tone::Trace)
                        .t(" = (b")
                        .sup("2")
                        .t(" − 4ac) / 4a")
                        .sup("2"),
                    "The left side is now a perfect square. Put the right side over 4a².",
                ),
                Step::new(
                    m().t("x + b / 2a = ±√(b").sup("2").t(" − 4ac) / 2a"),
                    "Take square roots. Both signs give a solution, if b² − 4ac is not negative.",
                ),
                Step::new(
                    m().t("x")
                        .tone(Tone::X)
                        .t(" = (−b ± √(b")
                        .sup("2")
                        .t(" − 4ac)) / 2a"),
                    "Subtract b / 2a: the quadratic formula.",
                ),
            ],
        )]
    }

    fn substitutions(&self) -> Vec<Substitution> {
        let values = [("a", self.a), ("b", self.b), ("c", self.c)];
        vec![
            Substitution::new(MathText::new().t("Δ").tone(Tone::Y), "b^2 - 4a c", &values),
            Substitution::new(
                MathText::new().t("x").tone(Tone::X).sub("1"),
                "(-b - sqrt(b^2 - 4a c)) / (2a)",
                &values,
            ),
            Substitution::new(
                MathText::new().t("x").tone(Tone::X).sub("2"),
                "(-b + sqrt(b^2 - 4a c)) / (2a)",
                &values,
            ),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        self.hover = pan_zoom(&mut self.view, response, rect);
    }

    fn wants_scroll(&self) -> bool {
        true
    }

    fn is_animating(&self) -> bool {
        false
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, _dt: f32) {
        let painter = painter.with_clip_rect(rect);
        let view = self.view;
        let to_screen = |p: [f64; 2]| view.to_screen(rect, p);
        paint_axes(&painter, rect, &view, palette);

        // The axis of symmetry and the vertex.
        if self.a.abs() >= EPS {
            let vx = -self.b / (2.0 * self.a);
            let top = to_screen([vx, 0.0]);
            painter.extend(Shape::dashed_line(
                &[pos2(top.x, rect.top()), pos2(top.x, rect.bottom())],
                Stroke::new(1.0, palette.y),
                5.0,
                5.0,
            ));
            painter.circle_filled(to_screen([vx, self.f(vx)]), 4.5, palette.y);
        }

        let (xr, yr) = (view.x_range(rect), view.y_range(rect));
        for piece in sample_explicit(&|x| self.f(x), xr, yr, rect.width()) {
            let pts: Vec<Pos2> = piece.into_iter().map(to_screen).collect();
            painter.add(Shape::line(pts, Stroke::new(2.4, palette.trace)));
        }

        let marked: Vec<f64> = match roots(self.a, self.b, self.c) {
            Roots::Two(x1, x2) => vec![x1, x2],
            Roots::One(x) | Roots::Linear(Some(x)) => vec![x],
            Roots::None | Roots::Linear(None) => Vec::new(),
        };
        for x in marked {
            let p = to_screen([x, 0.0]);
            painter.circle(p, 6.0, palette.x, Stroke::new(2.0, palette.card));
            painter.text(
                p + egui::vec2(0.0, -12.0),
                Align2::CENTER_BOTTOM,
                format!("{x:.2}"),
                FontId::monospace(12.0),
                palette.x,
            );
        }

        painter.text(
            rect.left_top() + egui::vec2(12.0, 10.0),
            Align2::LEFT_TOP,
            self.caption(),
            FontId::proportional(14.0),
            palette.ink,
        );
        if let Some(at) = self.hover {
            let [x, _] = view.to_world(rect, at);
            painter.text(
                rect.left_bottom() + egui::vec2(12.0, -10.0),
                Align2::LEFT_BOTTOM,
                format!("x = {x:.3}   f(x) = {:.3}", self.f(x)),
                FontId::monospace(12.0),
                palette.muted,
            );
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("quadratic-controls")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                for (name, v) in [("a", &mut self.a), ("b", &mut self.b), ("c", &mut self.c)] {
                    ui.label(name);
                    ui.add(egui::Slider::new(v, -5.0..=5.0).clamping(egui::SliderClamping::Edits));
                    ui.end_row();
                }
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for (label, abc) in [
                ("Two roots", (1.0, -1.0, -2.0)),
                ("Double root", (1.0, -2.0, 1.0)),
                ("No real roots", (1.0, 0.0, 1.5)),
                ("Opens down", (-0.5, 1.0, 2.0)),
            ] {
                if ui.button(label).clicked() {
                    (self.a, self.b, self.c) = abc;
                }
            }
            if ui.button("Reset view").clicked() {
                self.view = Self::default().view;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr;
    use crate::rng::Rng;

    fn residual(a: f64, b: f64, c: f64, x: f64) -> f64 {
        ((a * x + b) * x + c).abs()
    }

    #[test]
    fn roots_satisfy_the_equation() {
        let mut rng = Rng::new(5);
        for _ in 0..200 {
            let mut coef = || (rng.f32() as f64 - 0.5) * 10.0;
            let (a, b, c) = (coef(), coef(), coef());
            match roots(a, b, c) {
                Roots::Two(x1, x2) => {
                    assert!(x1 < x2);
                    for x in [x1, x2] {
                        assert!(residual(a, b, c, x) < 1e-9 * (1.0 + x * x), "{a} {b} {c}");
                    }
                }
                Roots::One(x) => assert!(residual(a, b, c, x) < 1e-9),
                Roots::None => assert!(discriminant(a, b, c) < 0.0),
                Roots::Linear(_) => unreachable!("random a is not 0"),
            }
        }
    }

    #[test]
    fn roots_match_the_formula_evaluated_by_the_expression_engine() {
        for (a, b, c) in [(1.0, -1.0, -2.0), (2.0, 3.0, -4.5), (-0.5, 1.0, 2.0)] {
            let f = Quadratic {
                a,
                b,
                c,
                ..Default::default()
            };
            let subs = f.substitutions();
            let value = |n: &str| {
                [("a", a), ("b", b), ("c", c)]
                    .iter()
                    .find(|p| p.0 == n)
                    .map(|p| p.1)
            };
            let mut xs: Vec<f64> = subs[1..]
                .iter()
                .map(|s| expr::eval(&s.expr, &value))
                .collect();
            xs.sort_by(f64::total_cmp);
            let Roots::Two(x1, x2) = roots(a, b, c) else {
                panic!("two roots expected");
            };
            assert!((xs[0] - x1).abs() < 1e-9 && (xs[1] - x2).abs() < 1e-9);
        }
    }

    #[test]
    fn special_cases() {
        assert_eq!(roots(1.0, -2.0, 1.0), Roots::One(1.0));
        assert_eq!(roots(1.0, 0.0, 1.0), Roots::None);
        assert_eq!(roots(0.0, 2.0, -4.0), Roots::Linear(Some(2.0)));
        assert_eq!(roots(0.0, 0.0, 3.0), Roots::Linear(None));
        assert_eq!(roots(2.0, 0.0, 0.0), Roots::One(0.0));
        for (a, b, c) in [(0.0, 2.0, 1.0), (1.0, 0.0, 1.0), (0.0, 0.0, 1.0)] {
            let f = Quadratic {
                a,
                b,
                c,
                ..Default::default()
            };
            assert!(!f.caption().contains("NaN"), "{}", f.caption());
        }
    }

    #[test]
    fn the_vertex_is_on_the_axis_of_symmetry() {
        let f = Quadratic {
            a: 2.0,
            b: -6.0,
            c: 1.0,
            ..Default::default()
        };
        let vx = -f.b / (2.0 * f.a);
        assert!(f.f(vx) <= f.f(vx - 0.1) && f.f(vx) <= f.f(vx + 0.1));
        assert!((f.f(vx - 0.7) - f.f(vx + 0.7)).abs() < 1e-12);
    }
}
