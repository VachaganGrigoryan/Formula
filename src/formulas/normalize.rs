//! Min-max normalization: squeeze any range of numbers into [0, 1], then into pixels.

use egui::{Align2, FontId, Rect, Shape, Stroke, pos2, vec2};

use crate::draw::lerp;
use crate::formula::{Formula, MathText, Palette, Tone};
use crate::rng::Rng;

/// `(x − min) / (max − min)` for every value. A flat dataset maps to all zeros.
pub fn min_max(values: &[f32]) -> Vec<f32> {
    let (lo, hi) = bounds(values);
    let span = hi - lo;
    values
        .iter()
        .map(|v| if span > 0.0 { (v - lo) / span } else { 0.0 })
        .collect()
}

pub fn bounds(values: &[f32]) -> (f32, f32) {
    values
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        })
}

const MONTHS: [&str; 12] = ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"];
/// The month whose value the outlier slider controls.
const OUTLIER: usize = 7;

pub struct Normalize {
    base: Vec<f32>,
    outlier: f32,
    seed: u64,
    hovered: Option<usize>,
}

impl Default for Normalize {
    fn default() -> Self {
        let mut s = Self {
            base: Vec::new(),
            outlier: 0.0,
            seed: 4,
            hovered: None,
        };
        s.regenerate();
        s
    }
}

impl Normalize {
    fn regenerate(&mut self) {
        let mut rng = Rng::new(self.seed);
        self.base = (0..12)
            .map(|i| 55.0 + 18.0 * (i as f32 * 0.55).sin() + rng.range(-12.0, 12.0))
            .collect();
        self.outlier = self.base[OUTLIER];
    }

    fn values(&self) -> Vec<f32> {
        let mut v = self.base.clone();
        v[OUTLIER] = self.outlier;
        v
    }
}

struct Layout {
    raw: Rect,
    unit: Rect,
    chart: Rect,
}

fn layout(rect: Rect) -> Layout {
    let inner = rect.shrink2(vec2(30.0, 18.0));
    let raw = Rect::from_min_size(inner.min + vec2(0.0, 26.0), vec2(inner.width(), 28.0));
    let unit = Rect::from_min_size(
        pos2(inner.min.x, raw.max.y + 54.0),
        vec2(inner.width(), 28.0),
    );
    let chart = Rect::from_min_max(
        pos2(inner.min.x, unit.max.y + 40.0),
        inner.max - vec2(0.0, 22.0),
    );
    Layout { raw, unit, chart }
}

fn bar_rect(chart: Rect, i: usize, n: usize) -> Rect {
    let w = chart.width() / n as f32;
    Rect::from_min_max(
        pos2(chart.left() + w * i as f32 + w * 0.15, chart.top()),
        pos2(chart.left() + w * (i as f32 + 0.85), chart.bottom()),
    )
}

impl Formula for Normalize {
    fn params(&self) -> Vec<(&'static str, f64)> {
        let (lo, hi) = bounds(&self.values());
        vec![("X_min", lo as f64), ("X_max", hi as f64)]
    }

    fn id(&self) -> &'static str {
        "min-max-normalization"
    }

    fn name(&self) -> &'static str {
        "Min-max normalization"
    }

    fn category(&self) -> &'static str {
        super::SCALING
    }

    fn summary(&self) -> &'static str {
        "Raw values on their own scale are mapped onto 0…1, then multiplied by the \
         chart height. Push the outlier up and watch every other bar get squashed."
    }

    fn notes(&self) -> &'static str {
        "Every chart axis does this before drawing: bar heights, scatter positions, \
         color intensity in heatmaps, bubble sizes. It is also a standard feature-scaling \
         step in machine learning. One extreme value compresses everything else, which is \
         why outliers are often clipped first."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("X")
                .tone(Tone::Trace)
                .sub("norm")
                .tone(Tone::Trace)
                .t(" = (X − X")
                .sub("min")
                .tone(Tone::X)
                .t(") / (X")
                .sub("max")
                .tone(Tone::Y)
                .t(" − X")
                .sub("min")
                .tone(Tone::X)
                .t(")"),
            MathText::new()
                .t("height = X")
                .sub("norm")
                .t(" · H")
                .sub("pixels"),
        ]
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.72).max(400.0)
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let chart = layout(rect).chart;
        self.hovered = response.hover_pos().and_then(|at| {
            (0..12).find(|&i| {
                let r = bar_rect(chart, i, 12);
                at.x >= r.left()
                    && at.x <= r.right()
                    && at.y >= chart.top() - 50.0
                    && at.y <= chart.bottom() + 22.0
            })
        });
    }

    fn is_animating(&self) -> bool {
        false
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, _dt: f32) {
        let values = self.values();
        let norm = min_max(&values);
        let (lo, hi) = bounds(&values);
        let Layout { raw, unit, chart } = layout(rect);
        let font = FontId::proportional(12.5);
        let mono = FontId::monospace(12.5);
        let n = values.len();

        // Raw axis: from 0 to a round number above the largest value.
        let axis_max = (hi / 50.0).ceil().max(1.0) * 50.0;
        let raw_x = |v: f32| lerp(raw.left(), raw.right(), v / axis_max);
        let ry = raw.center().y;
        painter.text(
            raw.left_top() - vec2(0.0, 22.0),
            Align2::LEFT_TOP,
            "raw values",
            font.clone(),
            palette.muted,
        );
        painter.line_segment(
            [pos2(raw.left(), ry), pos2(raw.right(), ry)],
            Stroke::new(1.0, palette.rule),
        );
        let step = if axis_max > 200.0 { 100.0 } else { 25.0 };
        let mut tick = 0.0;
        while tick <= axis_max {
            let x = raw_x(tick);
            painter.line_segment(
                [pos2(x, ry - 4.0), pos2(x, ry + 4.0)],
                Stroke::new(1.0, palette.rule),
            );
            painter.text(
                pos2(x, ry + 7.0),
                Align2::CENTER_TOP,
                format!("{tick:.0}"),
                font.clone(),
                palette.muted,
            );
            tick += step;
        }
        painter.line_segment(
            [pos2(raw_x(lo), ry), pos2(raw_x(hi), ry)],
            Stroke::new(3.0, palette.trace.gamma_multiply(0.4)),
        );

        // Unit axis: 0 at min, 1 at max.
        let uy = unit.center().y;
        let unit_x = |u: f32| lerp(unit.left(), unit.right(), u);
        painter.line_segment(
            [pos2(unit.left(), uy), pos2(unit.right(), uy)],
            Stroke::new(3.0, palette.trace.gamma_multiply(0.4)),
        );
        painter.text(
            pos2(unit.left(), uy + 7.0),
            Align2::LEFT_TOP,
            "0",
            font.clone(),
            palette.x,
        );
        painter.text(
            pos2(unit.right(), uy + 7.0),
            Align2::RIGHT_TOP,
            "1",
            font.clone(),
            palette.y,
        );
        painter.text(
            unit.left_top() - vec2(0.0, 4.0),
            Align2::LEFT_BOTTOM,
            "normalized",
            font.clone(),
            palette.muted,
        );

        for i in 0..n {
            let a = pos2(raw_x(values[i]), ry);
            let b = pos2(unit_x(norm[i]), uy);
            let active = self.hovered == Some(i);
            let color = if values[i] == lo {
                palette.x
            } else if values[i] == hi {
                palette.y
            } else {
                palette.trace
            };
            let link = if active {
                Stroke::new(1.5, palette.ink)
            } else {
                Stroke::new(1.0, color.gamma_multiply(0.35))
            };
            painter.line_segment([a, b], link);
            painter.circle_filled(a, if active { 5.5 } else { 4.0 }, color);
            painter.circle_filled(b, if active { 5.5 } else { 4.0 }, color);
        }
        painter.text(
            pos2(raw_x(lo), ry - 8.0),
            Align2::CENTER_BOTTOM,
            format!("min {lo:.0}"),
            font.clone(),
            palette.x,
        );
        painter.text(
            pos2(raw_x(hi), ry - 8.0),
            Align2::CENTER_BOTTOM,
            format!("max {hi:.0}"),
            font.clone(),
            palette.y,
        );

        // Bar chart: normalized value times the available pixel height.
        painter.line_segment(
            [chart.left_bottom(), chart.right_bottom()],
            Stroke::new(1.0, palette.rule),
        );
        painter.extend(Shape::dashed_line(
            &[chart.left_top(), chart.right_top()],
            Stroke::new(1.0, palette.rule),
            4.0,
            4.0,
        ));
        for i in 0..n {
            let slot = bar_rect(chart, i, n);
            let h = (norm[i] * chart.height()).max(2.0);
            let bar =
                Rect::from_min_max(pos2(slot.left(), chart.bottom() - h), slot.right_bottom());
            let active = self.hovered == Some(i);
            let fill = if active {
                palette.ink
            } else if i == OUTLIER {
                palette.y.gamma_multiply(0.8)
            } else {
                palette.trace
            };
            painter.rect_filled(bar, 3.0, fill);
            painter.text(
                pos2(slot.center().x, chart.bottom() + 4.0),
                Align2::CENTER_TOP,
                MONTHS[i],
                font.clone(),
                palette.muted,
            );
        }

        let note = match self.hovered {
            Some(i) => format!(
                "({:.0} − {lo:.0}) / ({hi:.0} − {lo:.0}) = {:.2}  ·  {:.0} px tall",
                values[i],
                norm[i],
                norm[i] * chart.height()
            ),
            None => "hover a bar".to_owned(),
        };
        painter.text(
            rect.right_top() + vec2(-16.0, 12.0),
            Align2::RIGHT_TOP,
            note,
            mono,
            palette.ink,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("normalize-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label(format!(
                    "Outlier ({})",
                    [
                        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct",
                        "Nov", "Dec"
                    ][OUTLIER]
                ));
                ui.add(egui::Slider::new(&mut self.outlier, 0.0..=500.0));
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("New data").clicked() {
                self.seed = self.seed.wrapping_add(1);
                self.regenerate();
            }
            if ui.button("Reset outlier").clicked() {
                self.outlier = self.base[OUTLIER];
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_to_unit_range() {
        assert_eq!(min_max(&[10.0, 20.0, 15.0]), vec![0.0, 1.0, 0.5]);
    }

    #[test]
    fn flat_data_does_not_divide_by_zero() {
        assert_eq!(min_max(&[3.0, 3.0]), vec![0.0, 0.0]);
    }

    #[test]
    fn outlier_compresses_the_rest() {
        let calm = min_max(&[10.0, 20.0, 30.0]);
        let spiked = min_max(&[10.0, 20.0, 30.0, 1000.0]);
        assert!(spiked[1] < calm[1] / 10.0);
    }
}
