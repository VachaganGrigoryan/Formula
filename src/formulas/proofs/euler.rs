//! Euler's formula e^(iθ) = cos θ + i sin θ, seen as the Taylor series of e^x
//! with x = iθ: each term turns a quarter turn from the last, and the partial
//! sums spiral in to the point at angle θ on the unit circle.

use std::f64::consts::{PI, TAU};

use egui::{Align2, FontId, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::derive::{Derivation, Step, Substitution};
use crate::formula::{Formula, MathText, Palette, Tone};

/// The partial sums S_0 … S_n of Σ (iθ)^k / k! as points (re, im). Each term
/// is the previous one times iθ / k, so no factorial is ever formed.
pub fn partial_sums(theta: f64, n: usize) -> Vec<[f64; 2]> {
    let mut term = [1.0, 0.0];
    let mut sum = term;
    let mut out = vec![sum];
    for k in 1..=n {
        // (re + i im) · iθ/k = (−im θ/k) + i (re θ/k)
        let s = theta / k as f64;
        term = [-term[1] * s, term[0] * s];
        sum = [sum[0] + term[0], sum[1] + term[1]];
        out.push(sum);
    }
    out
}

/// Where the origin goes and how many points one unit takes, so that the unit
/// circle and every point fit inside `rect` with a margin.
pub fn layout(points: &[[f64; 2]], rect: Rect) -> (Pos2, f64) {
    let (mut lo, mut hi) = ([-1.0f64, -1.0f64], [1.0f64, 1.0f64]);
    for p in points.iter().chain(std::iter::once(&[0.0, 0.0])) {
        for d in 0..2 {
            lo[d] = lo[d].min(p[d]);
            hi[d] = hi[d].max(p[d]);
        }
    }
    let margin = 30.0;
    let w = (rect.width() as f64 - 2.0 * margin).max(10.0);
    let h = (rect.height() as f64 - 2.0 * margin - 30.0).max(10.0);
    let scale = (w / (hi[0] - lo[0])).min(h / (hi[1] - lo[1]));
    let mid = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let center = rect.center() + vec2(0.0, 15.0);
    let origin = pos2(
        center.x - (mid[0] * scale) as f32,
        center.y + (mid[1] * scale) as f32,
    );
    (origin, scale)
}

pub struct Euler {
    theta: f64,
    terms: u32,
    playing: bool,
}

impl Default for Euler {
    fn default() -> Self {
        Self {
            theta: 2.0,
            terms: 8,
            playing: false,
        }
    }
}

fn arrow(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    painter.line_segment([from, to], stroke);
    let d = to - from;
    let len = d.length();
    if len < 4.0 {
        return;
    }
    let u = d / len;
    let n = vec2(-u.y, u.x);
    let head = 7.0f32.min(len * 0.4);
    painter.add(Shape::convex_polygon(
        vec![
            to,
            to - u * head + n * head * 0.45,
            to - u * head - n * head * 0.45,
        ],
        stroke.color,
        Stroke::NONE,
    ));
}

impl Formula for Euler {
    fn id(&self) -> &'static str {
        "euler-identity"
    }

    fn name(&self) -> &'static str {
        "Euler's formula"
    }

    fn category(&self) -> &'static str {
        crate::formulas::PROOFS
    }

    fn summary(&self) -> &'static str {
        "e to an imaginary power lands on the unit circle. Add up the series \
         1 + iθ + (iθ)²/2! + … term by term and watch the arrows spiral in to the \
         point at angle θ; at θ = π it is −1."
    }

    fn notes(&self) -> &'static str {
        "Every rotation and oscillation in signal processing, the Fourier transform, \
         AC circuit analysis and quantum mechanics is written as e^(iθ), because \
         multiplying by it turns a point by θ."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("e")
                .tone(Tone::Trace)
                .sup("iθ")
                .tone(Tone::Trace)
                .t(" = ")
                .t("cos θ")
                .tone(Tone::X)
                .t(" + i ")
                .t("sin θ")
                .tone(Tone::Y),
            MathText::new()
                .t("e")
                .sup("iθ")
                .t(" = Σ (iθ)")
                .sup("k")
                .t(" / k!"),
            MathText::new().t("e").sup("iπ").t(" + 1 = 0"),
        ]
    }

    fn params(&self) -> Vec<(String, f64)> {
        vec![("θ".into(), self.theta), ("n".into(), self.terms as f64)]
    }

    fn substitutions(&self) -> Vec<Substitution> {
        let values = [("θ", self.theta)];
        vec![
            Substitution::new(MathText::new().t("Re").tone(Tone::X), "cos(θ)", &values),
            Substitution::new(MathText::new().t("Im").tone(Tone::Y), "sin(θ)", &values),
        ]
    }

    fn derivations(&self) -> Vec<Derivation> {
        let m = MathText::new;
        vec![Derivation::new(
            "From the Taylor series",
            vec![
                Step::new(
                    m().t("e")
                        .sup("x")
                        .t(" = 1 + x + x")
                        .sup("2")
                        .t("/2! + x")
                        .sup("3")
                        .t("/3! + x")
                        .sup("4")
                        .t("/4! + …"),
                    "The Taylor series of e^x. It converges for every x, complex numbers included.",
                ),
                Step::new(
                    m().t("cos θ")
                        .tone(Tone::X)
                        .t(" = 1 − θ")
                        .sup("2")
                        .t("/2! + θ")
                        .sup("4")
                        .t("/4! − …"),
                    "The series for cosine: the even powers, with alternating signs.",
                ),
                Step::new(
                    m().t("sin θ")
                        .tone(Tone::Y)
                        .t(" = θ − θ")
                        .sup("3")
                        .t("/3! + θ")
                        .sup("5")
                        .t("/5! − …"),
                    "The series for sine: the odd powers, with alternating signs.",
                ),
                Step::new(
                    m().t("i")
                        .sup("2")
                        .t(" = −1,   i")
                        .sup("3")
                        .t(" = −i,   i")
                        .sup("4")
                        .t(" = 1"),
                    "Powers of i repeat every four: multiplying by i is a quarter turn.",
                ),
                Step::new(
                    m().t("e")
                        .sup("iθ")
                        .t(" = 1 + iθ − θ")
                        .sup("2")
                        .t("/2! − iθ")
                        .sup("3")
                        .t("/3! + θ")
                        .sup("4")
                        .t("/4! + …"),
                    "Put x = iθ into the series for e^x and simplify the powers of i.",
                ),
                Step::new(
                    m().t("e")
                        .sup("iθ")
                        .t(" = (1 − θ")
                        .sup("2")
                        .t("/2! + …)")
                        .tone(Tone::X)
                        .t(" + i (θ − θ")
                        .sup("3")
                        .t("/3! + …)")
                        .tone(Tone::Y),
                    "Group the real terms and the imaginary terms.",
                ),
                Step::new(
                    m().t("e")
                        .tone(Tone::Trace)
                        .sup("iθ")
                        .tone(Tone::Trace)
                        .t(" = ")
                        .t("cos θ")
                        .tone(Tone::X)
                        .t(" + i ")
                        .t("sin θ")
                        .tone(Tone::Y),
                    "The two groups are exactly the series for cos θ and sin θ.",
                ),
                Step::new(
                    m().t("e").sup("iπ").t(" + 1 = 0"),
                    "At θ = π, cos π = −1 and sin π = 0: Euler's identity.",
                ),
            ],
        )]
    }

    fn is_animating(&self) -> bool {
        self.playing
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.playing {
            self.theta = (self.theta + 0.3 * dt as f64).rem_euclid(TAU);
        }
        let painter = painter.with_clip_rect(rect);
        let sums = partial_sums(self.theta, self.terms as usize);
        let (origin, scale) = layout(&sums, rect);
        let at = |p: [f64; 2]| origin + vec2((p[0] * scale) as f32, -(p[1] * scale) as f32);

        let axis = Stroke::new(1.0, palette.rule);
        painter.line_segment(
            [pos2(rect.left(), origin.y), pos2(rect.right(), origin.y)],
            axis,
        );
        painter.line_segment(
            [pos2(origin.x, rect.top()), pos2(origin.x, rect.bottom())],
            axis,
        );
        painter.circle_stroke(origin, scale as f32, Stroke::new(1.2, palette.muted));

        // Term k points along 1, i, −1, −i, … : real terms in x, imaginary in y.
        let mut from = origin;
        for (k, &s) in sums.iter().enumerate() {
            let to = at(s);
            let color = if k % 2 == 0 { palette.x } else { palette.y };
            arrow(&painter, from, to, Stroke::new(2.0, color));
            from = to;
        }
        let target = at([self.theta.cos(), self.theta.sin()]);
        painter.line_segment(
            [origin, target],
            Stroke::new(1.0, palette.trace.gamma_multiply(0.5)),
        );
        painter.circle(target, 6.0, palette.trace, Stroke::new(2.0, palette.card));

        let last = *sums.last().unwrap_or(&[1.0, 0.0]);
        let gap = (last[0] - self.theta.cos()).hypot(last[1] - self.theta.sin());
        let font = FontId::monospace(12.0);
        painter.text(
            rect.left_top() + vec2(12.0, 10.0),
            Align2::LEFT_TOP,
            format!(
                "θ = {:.3}   sum of {} terms = ({:+.4}, {:+.4})",
                self.theta,
                self.terms + 1,
                last[0],
                last[1]
            ),
            font.clone(),
            palette.ink,
        );
        painter.text(
            rect.left_top() + vec2(12.0, 28.0),
            Align2::LEFT_TOP,
            format!(
                "cos θ + i sin θ = ({:+.4}, {:+.4})   distance {gap:.2e}",
                self.theta.cos(),
                self.theta.sin()
            ),
            font,
            palette.trace,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("euler-controls")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("θ");
                if ui
                    .add(
                        egui::Slider::new(&mut self.theta, 0.0..=TAU)
                            .clamping(egui::SliderClamping::Edits),
                    )
                    .changed()
                {
                    self.playing = false;
                }
                ui.end_row();
                ui.label("terms after 1");
                ui.add(egui::Slider::new(&mut self.terms, 0..=30));
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui
                .button(if self.playing { "Pause" } else { "Turn θ" })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui.button("θ = π").clicked() {
                self.theta = PI;
                self.playing = false;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_sums_are_one_and_one_plus_i_theta() {
        let s = partial_sums(0.7, 1);
        assert_eq!(s[0], [1.0, 0.0]);
        assert!((s[1][0] - 1.0).abs() < 1e-15 && (s[1][1] - 0.7).abs() < 1e-15);
    }

    #[test]
    fn thirty_terms_land_on_the_unit_circle() {
        for i in 0..=50 {
            let theta = TAU * i as f64 / 50.0;
            let s = *partial_sums(theta, 30).last().unwrap();
            assert!(
                (s[0] - theta.cos()).abs() < 1e-6 && (s[1] - theta.sin()).abs() < 1e-6,
                "θ = {theta}"
            );
        }
        let pi = *partial_sums(PI, 30).last().unwrap();
        assert!(
            (pi[0] + 1.0).abs() < 1e-6 && pi[1].abs() < 1e-6,
            "e^(iπ) = −1"
        );
    }

    #[test]
    fn the_spiral_fits_the_smallest_stage() {
        let size = Euler::default().min_size();
        let rect = Rect::from_min_size(pos2(100.0, 50.0), size);
        for n in 0..=30 {
            for theta in [0.5, PI, 5.0, TAU] {
                let sums = partial_sums(theta, n);
                let (origin, scale) = layout(&sums, rect);
                for p in sums
                    .iter()
                    .chain([[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]].iter())
                {
                    let q = origin + vec2((p[0] * scale) as f32, -(p[1] * scale) as f32);
                    assert!(rect.contains(q), "n {n} θ {theta}: {q:?} outside {rect:?}");
                }
            }
        }
    }
}
