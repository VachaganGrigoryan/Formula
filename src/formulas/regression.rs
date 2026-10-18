//! Simple linear regression by least squares: the line of best fit.

use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::derive::{Derivation, Step};
use crate::draw;
use crate::formula::{Formula, MathText, Palette, Tone, fit_text};
use crate::rng::Rng;

pub type Point = [f32; 2];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub slope: f32,
    pub intercept: f32,
    pub mean_x: f32,
    pub mean_y: f32,
    /// Coefficient of determination: the share of y's variance the line explains.
    pub r2: f32,
}

impl Fit {
    pub fn predict(&self, x: f32) -> f32 {
        self.slope * x + self.intercept
    }
}

/// Least-squares fit. `None` with fewer than two points or when every x is the
/// same (a vertical line has no slope).
pub fn fit(points: &[Point]) -> Option<Fit> {
    if points.len() < 2 {
        return None;
    }
    let n = points.len() as f32;
    let mean_x = points.iter().map(|p| p[0]).sum::<f32>() / n;
    let mean_y = points.iter().map(|p| p[1]).sum::<f32>() / n;
    let sxy: f32 = points
        .iter()
        .map(|p| (p[0] - mean_x) * (p[1] - mean_y))
        .sum();
    let sxx: f32 = points.iter().map(|p| (p[0] - mean_x).powi(2)).sum();
    if sxx <= f32::EPSILON {
        return None;
    }
    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    let ss_tot: f32 = points.iter().map(|p| (p[1] - mean_y).powi(2)).sum();
    let ss_res: f32 = points
        .iter()
        .map(|p| (p[1] - (slope * p[0] + intercept)).powi(2))
        .sum();
    let r2 = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };
    Some(Fit {
        slope,
        intercept,
        mean_x,
        mean_y,
        r2,
    })
}

/// Data space is `0..10` on both axes.
const SPAN: f32 = 10.0;

pub struct Regression {
    points: Vec<Point>,
    seed: u64,
    noise: f32,
    residuals: bool,
    squares: bool,
    dragging: Option<usize>,
}

impl Default for Regression {
    fn default() -> Self {
        let mut s = Self {
            points: Vec::new(),
            seed: 9,
            noise: 1.2,
            residuals: true,
            squares: false,
            dragging: None,
        };
        s.regenerate();
        s
    }
}

impl Regression {
    fn regenerate(&mut self) {
        let mut rng = Rng::new(self.seed);
        let slope = rng.range(-0.5, 0.9);
        let intercept = 5.0 - slope * 5.0;
        self.points = (0..18)
            .map(|_| {
                let x = rng.range(0.6, 9.4);
                let noise = (rng.f32() + rng.f32() + rng.f32() - 1.5) * 2.0 * self.noise;
                [x, (slope * x + intercept + noise).clamp(0.2, 9.8)]
            })
            .collect();
    }
}

fn plot(rect: Rect) -> Rect {
    let inner = rect.shrink(28.0).with_max_y(rect.max.y - 50.0);
    let side = inner.width().min(inner.height());
    Rect::from_center_size(inner.center() + vec2(10.0, 0.0), vec2(side, side))
}

fn to_screen(plot: Rect, [x, y]: Point) -> Pos2 {
    pos2(
        plot.left() + x / SPAN * plot.width(),
        plot.bottom() - y / SPAN * plot.height(),
    )
}

fn to_data(plot: Rect, p: Pos2) -> Point {
    [
        ((p.x - plot.left()) / plot.width() * SPAN).clamp(0.0, SPAN),
        ((plot.bottom() - p.y) / plot.height() * SPAN).clamp(0.0, SPAN),
    ]
}

impl Formula for Regression {
    fn derivations(&self) -> Vec<Derivation> {
        let m = MathText::new;
        vec![Derivation::new(
            "Least squares",
            vec![
                Step::new(
                    m().t("E(m, b) = Σ (y")
                        .sub("i")
                        .t(" − (m x")
                        .sub("i")
                        .t(" + b))")
                        .sup("2"),
                    "Add up the squared vertical misses of the line y = m x + b over all n points.",
                ),
                Step::new(
                    m().t("dE/db = −2 Σ (y")
                        .sub("i")
                        .t(" − m x")
                        .sub("i")
                        .t(" − b) = 0"),
                    "At the minimum, nudging b does not change E.",
                ),
                Step::new(
                    m().t("b")
                        .tone(Tone::Trace)
                        .t(" = μ")
                        .sub("y")
                        .t(" − m μ")
                        .sub("x"),
                    "Divide by n: the y values sum to n μ_y and the x values to n μ_x. The line passes through the mean point.",
                ),
                Step::new(
                    m().t("dE/dm = −2 Σ x")
                        .sub("i")
                        .t(" (y")
                        .sub("i")
                        .t(" − m x")
                        .sub("i")
                        .t(" − b) = 0"),
                    "Nudging the slope does not change E either.",
                ),
                Step::new(
                    m().t("Σ x")
                        .sub("i")
                        .t(" (y")
                        .sub("i")
                        .t(" − μ")
                        .sub("y")
                        .t(") = m Σ x")
                        .sub("i")
                        .t(" (x")
                        .sub("i")
                        .t(" − μ")
                        .sub("x")
                        .t(")"),
                    "Substitute b and collect the terms with m.",
                ),
                Step::new(
                    m().t("m")
                        .tone(Tone::Trace)
                        .t(" = Σ (x")
                        .sub("i")
                        .t(" − μ")
                        .sub("x")
                        .t(")(y")
                        .sub("i")
                        .t(" − μ")
                        .sub("y")
                        .t(") / Σ (x")
                        .sub("i")
                        .t(" − μ")
                        .sub("x")
                        .t(")")
                        .sup("2"),
                    "Replacing the leading x_i by x_i − μ_x changes neither sum, because the deviations from a mean add up to zero.",
                ),
            ],
        )]
    }

    fn id(&self) -> &'static str {
        "linear-regression"
    }

    fn name(&self) -> &'static str {
        "Line of best fit"
    }

    fn category(&self) -> &'static str {
        super::STATISTICS
    }

    fn summary(&self) -> &'static str {
        "Least squares picks the one line that makes the squared vertical misses as small \
         as possible. Drag points, click to add one, right-click to remove one."
    }

    fn notes(&self) -> &'static str {
        "Trend lines on scatter plots and time series, \"y rises 0.6 per unit of x\" \
         annotations, and quick forecasts. R² says how much of the scatter the line \
         explains; the line always passes through the mean point (μx, μy)."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("m")
                .tone(Tone::Trace)
                .t(" = Σ (x")
                .sub("i")
                .t(" − μ")
                .sub("x")
                .t(")(y")
                .sub("i")
                .t(" − μ")
                .sub("y")
                .t(") / Σ (x")
                .sub("i")
                .t(" − μ")
                .sub("x")
                .t(")")
                .sup("2"),
            MathText::new()
                .t("b")
                .tone(Tone::Trace)
                .t(" = μ")
                .sub("y")
                .t(" − m μ")
                .sub("x"),
            MathText::new()
                .t("μ")
                .tone(Tone::X)
                .sub("x")
                .tone(Tone::X)
                .t(", ")
                .t("μ")
                .tone(Tone::Y)
                .sub("y")
                .tone(Tone::Y)
                .t(" are the means of x and y (often written with a bar)"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.72).max(380.0)
    }

    fn min_size(&self) -> egui::Vec2 {
        // The chart's axis labels and the fit line under it need the width.
        egui::vec2(440.0, self.stage_height(440.0))
    }

    fn is_animating(&self) -> bool {
        false
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let plot = plot(rect);
        let screen: Vec<Pos2> = self.points.iter().map(|&p| to_screen(plot, p)).collect();
        if response.drag_started()
            && let Some(at) = response.interact_pointer_pos()
        {
            self.dragging = draw::pick(&screen, at, 14.0);
        }
        if let (Some(i), Some(at)) = (self.dragging, response.interact_pointer_pos()) {
            self.points[i] = to_data(plot, at);
        }
        if response.drag_stopped() {
            self.dragging = None;
        }
        if let Some(at) = response.interact_pointer_pos() {
            if response.clicked() && plot.contains(at) && draw::pick(&screen, at, 14.0).is_none() {
                self.points.push(to_data(plot, at));
            } else if response.secondary_clicked()
                && let Some(i) = draw::pick(&screen, at, 18.0)
            {
                self.points.remove(i);
            }
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, _dt: f32) {
        let plot = plot(rect);
        let screen = |p: Point| to_screen(plot, p);
        let font = FontId::proportional(12.0);

        painter.rect_stroke(
            plot,
            4.0,
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Outside,
        );
        for k in 0..=10 {
            let v = k as f32;
            let (x, y) = (screen([v, 0.0]).x, screen([0.0, v]).y);
            let faint = Stroke::new(1.0, palette.rule.gamma_multiply(0.45));
            painter.line_segment([pos2(x, plot.top()), pos2(x, plot.bottom())], faint);
            painter.line_segment([pos2(plot.left(), y), pos2(plot.right(), y)], faint);
            if k % 2 == 0 {
                painter.text(
                    pos2(x, plot.bottom() + 4.0),
                    Align2::CENTER_TOP,
                    k.to_string(),
                    font.clone(),
                    palette.muted,
                );
                painter.text(
                    pos2(plot.left() - 6.0, y),
                    Align2::RIGHT_CENTER,
                    k.to_string(),
                    font.clone(),
                    palette.muted,
                );
            }
        }

        let result = fit(&self.points);
        if let Some(f) = result {
            if self.squares || self.residuals {
                for &p in &self.points {
                    let on_line = [p[0], f.predict(p[0])];
                    let (a, b) = (screen(p), screen(on_line));
                    if self.squares {
                        // The square of each residual, drawn as an actual square.
                        let side = b.y - a.y;
                        let dir = if f.slope >= 0.0 { -1.0 } else { 1.0 };
                        let sq = Rect::from_two_pos(a, a + vec2(dir * side.abs(), side));
                        painter.rect(
                            sq,
                            0.0,
                            palette.y.gamma_multiply(0.12),
                            Stroke::new(1.0, palette.y.gamma_multiply(0.5)),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if self.residuals {
                        painter.line_segment([a, b], Stroke::new(1.5, palette.y));
                    }
                }
            }
            // Mean point: the fitted line always passes through it.
            let mean = screen([f.mean_x, f.mean_y]);
            painter.extend(Shape::dashed_line(
                &[pos2(mean.x, plot.top()), pos2(mean.x, plot.bottom())],
                Stroke::new(1.0, palette.x.gamma_multiply(0.6)),
                4.0,
                4.0,
            ));
            painter.extend(Shape::dashed_line(
                &[pos2(plot.left(), mean.y), pos2(plot.right(), mean.y)],
                Stroke::new(1.0, palette.y.gamma_multiply(0.6)),
                4.0,
                4.0,
            ));

            let clipped = painter.with_clip_rect(plot);
            clipped.line_segment(
                [
                    screen([0.0, f.predict(0.0)]),
                    screen([SPAN, f.predict(SPAN)]),
                ],
                Stroke::new(3.0, palette.trace),
            );
            painter.circle(mean, 6.0, palette.card, Stroke::new(2.5, palette.trace));
            painter.text(
                mean + vec2(8.0, -8.0),
                Align2::LEFT_BOTTOM,
                "(μx, μy)",
                font.clone(),
                palette.trace,
            );
        }

        for (i, &p) in self.points.iter().enumerate() {
            let active = self.dragging == Some(i);
            painter.circle(
                screen(p),
                if active { 7.0 } else { 5.0 },
                palette.ink,
                Stroke::new(1.5, palette.card),
            );
        }

        let mono = FontId::monospace(12.5);
        let text = match result {
            Some(f) => {
                let sse: f32 = self
                    .points
                    .iter()
                    .map(|p| (p[1] - f.predict(p[0])).powi(2))
                    .sum();
                format!(
                    "y = {:.2}x {} {:.2}    R² = {:.2}    Σ residual² = {sse:.1}    n = {}",
                    f.slope,
                    if f.intercept < 0.0 { "−" } else { "+" },
                    f.intercept.abs(),
                    f.r2,
                    self.points.len()
                )
            }
            None => "add points with different x to fit a line".to_owned(),
        };
        let galley = fit_text(painter, text, mono, rect.width() - 24.0, palette.ink);
        let at = rect.center_bottom() - vec2(galley.size().x / 2.0, 12.0 + galley.size().y);
        painter.galley(at, galley, palette.ink);
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("regression-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Noise");
                if ui
                    .add(egui::Slider::new(&mut self.noise, 0.0..=3.0))
                    .changed()
                {
                    self.regenerate();
                }
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("New data").clicked() {
                self.seed = self.seed.wrapping_add(1);
                self.regenerate();
            }
            ui.checkbox(&mut self.residuals, "Residuals");
            ui.checkbox(&mut self.squares, "Squared errors");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_line() {
        let pts = [[0.0, 1.0], [1.0, 3.0], [2.0, 5.0], [3.0, 7.0]];
        let f = fit(&pts).unwrap();
        assert!((f.slope - 2.0).abs() < 1e-5);
        assert!((f.intercept - 1.0).abs() < 1e-5);
        assert!((f.r2 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn passes_through_means() {
        let pts = [[1.0, 2.0], [2.0, 1.0], [4.0, 5.0], [6.0, 4.0], [7.0, 8.0]];
        let f = fit(&pts).unwrap();
        assert!((f.predict(f.mean_x) - f.mean_y).abs() < 1e-5);
        assert!(f.r2 > 0.0 && f.r2 < 1.0);
    }

    #[test]
    fn residuals_sum_to_zero() {
        let pts = [[1.0, 2.0], [2.0, 1.0], [4.0, 5.0], [6.0, 4.0], [7.0, 8.0]];
        let f = fit(&pts).unwrap();
        let sum: f32 = pts.iter().map(|p| p[1] - f.predict(p[0])).sum();
        assert!(sum.abs() < 1e-4);
    }

    #[test]
    fn degenerate_inputs() {
        assert!(fit(&[[1.0, 1.0]]).is_none());
        assert!(fit(&[[2.0, 1.0], [2.0, 5.0]]).is_none());
    }
}
