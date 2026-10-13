//! Cubic ease-out: start fast, glide to a stop.

use egui::{Align2, FontId, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::draw::lerp;
use crate::formula::{Formula, MathText, Palette, Tone};
use crate::rng::Rng;

/// `f(t) = 1 − (1 − t)³` for progress `t` in `[0, 1]`.
pub fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

/// Speed of the eased motion, `f'(t) = 3(1 − t)²`: 3× linear at the start, 0 at the end.
pub fn ease_out_cubic_speed(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    3.0 * u * u
}

/// Seconds the finished state is held before a looping animation restarts.
const HOLD: f32 = 0.8;
const BARS: usize = 7;

pub struct Easing {
    duration: f32,
    /// Seconds since the animation started.
    elapsed: f32,
    looping: bool,
    eased: bool,
    from: [f32; BARS],
    to: [f32; BARS],
    seed: u64,
}

impl Default for Easing {
    fn default() -> Self {
        let mut s = Self {
            duration: 1.6,
            elapsed: 0.0,
            looping: true,
            eased: true,
            from: [0.0; BARS],
            to: [0.0; BARS],
            seed: 5,
        };
        s.new_targets();
        s.from = [0.0; BARS];
        s
    }
}

impl Easing {
    fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Animate the bars from where they are now to fresh values.
    fn new_targets(&mut self) {
        let t = self.bar_t();
        for k in 0..BARS {
            self.from[k] = lerp(self.from[k], self.to[k], t);
        }
        let mut rng = Rng::new(self.seed);
        self.seed = self.seed.wrapping_add(1);
        self.to = std::array::from_fn(|_| rng.range(0.15, 1.0));
        self.elapsed = 0.0;
    }

    fn bar_t(&self) -> f32 {
        let t = self.progress();
        if self.eased { ease_out_cubic(t) } else { t }
    }
}

struct Layout {
    graph: Rect,
    tracks: Rect,
    bars: Rect,
}

fn layout(rect: Rect) -> Layout {
    let inner = rect.shrink(22.0);
    let top_h = (inner.height() * 0.5).min(inner.width() * 0.42);
    let graph = Rect::from_min_size(inner.min + vec2(18.0, 0.0), vec2(top_h, top_h));
    let tracks = Rect::from_min_max(
        pos2(graph.right() + 36.0, inner.top()),
        pos2(inner.right(), graph.bottom()),
    );
    let bars = Rect::from_min_max(pos2(inner.left(), graph.bottom() + 40.0), inner.max);
    Layout {
        graph,
        tracks,
        bars,
    }
}

impl Formula for Easing {
    fn id(&self) -> &'static str {
        "ease-out-cubic"
    }

    fn name(&self) -> &'static str {
        "Cubic ease-out"
    }

    fn category(&self) -> &'static str {
        super::ANIMATION
    }

    fn summary(&self) -> &'static str {
        "Feed animation progress t through f(t) = 1 − (1 − t)³ and motion starts quick and \
         settles gently. The dots are equal slices of time: evenly spaced for linear, \
         bunched up at the end for ease-out."
    }

    fn notes(&self) -> &'static str {
        "Bars growing into place, points sliding to new positions after a filter change, \
         panels opening, counters ticking up. Ease-out reads as \"arriving\"; ease-in \
         (t³) reads as \"leaving\". The 1-to-N grow animation in the Drawn by circles viewport \
         uses exactly this curve."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("f(t)")
                .tone(Tone::Trace)
                .t(" = 1 − (1 − t)")
                .sup("3")
                .t(",   0 ≤ t ≤ 1"),
            MathText::new()
                .t("f'(t)")
                .tone(Tone::Y)
                .t(" = 3(1 − t)")
                .sup("2")
                .t("   speed: 3 at the start, 0 at the end"),
        ]
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.72).max(420.0)
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.elapsed += dt;
        if self.looping && self.elapsed > self.duration + HOLD {
            self.new_targets();
        }
        let t = self.progress();
        let e = ease_out_cubic(t);
        let Layout {
            graph,
            tracks,
            bars,
        } = layout(rect);
        let font = FontId::proportional(12.5);

        // 1. The curve itself, with linear for reference.
        let at = |u: f32, v: f32| {
            pos2(
                lerp(graph.left(), graph.right(), u),
                lerp(graph.bottom(), graph.top(), v),
            )
        };
        painter.rect_stroke(
            graph,
            4.0,
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Outside,
        );
        painter.extend(Shape::dashed_line(
            &[at(0.0, 0.0), at(1.0, 1.0)],
            Stroke::new(1.0, palette.muted),
            4.0,
            4.0,
        ));
        let curve: Vec<Pos2> = (0..=64)
            .map(|k| k as f32 / 64.0)
            .map(|u| at(u, ease_out_cubic(u)))
            .collect();
        painter.add(Shape::line(curve, Stroke::new(2.5, palette.trace)));
        let speed: Vec<Pos2> = (0..=64)
            .map(|k| k as f32 / 64.0)
            .map(|u| at(u, ease_out_cubic_speed(u) / 3.0))
            .collect();
        painter.add(Shape::line(
            speed,
            Stroke::new(1.2, palette.y.gamma_multiply(0.7)),
        ));
        painter.line_segment(
            [at(t, 0.0), at(t, e)],
            Stroke::new(1.0, palette.ink.gamma_multiply(0.4)),
        );
        painter.circle(at(t, t), 4.0, palette.muted, Stroke::NONE);
        painter.circle(at(t, e), 6.0, palette.trace, Stroke::new(2.0, palette.card));
        painter.text(
            graph.left_bottom() + vec2(0.0, 4.0),
            Align2::LEFT_TOP,
            "t = 0",
            font.clone(),
            palette.muted,
        );
        painter.text(
            graph.right_bottom() + vec2(0.0, 4.0),
            Align2::RIGHT_TOP,
            "1",
            font.clone(),
            palette.muted,
        );
        painter.text(
            graph.left_top() + vec2(-6.0, 0.0),
            Align2::RIGHT_TOP,
            "f",
            font.clone(),
            palette.trace,
        );
        painter.text(
            at(0.02, 0.98),
            Align2::LEFT_TOP,
            "speed f'(t) / 3",
            font.clone(),
            palette.y,
        );

        // 2. The same progress driving position: linear vs eased.
        let linear: fn(f32) -> f32 = |u| u;
        let lanes = [
            ("linear", t, palette.muted, linear),
            ("ease-out", e, palette.trace, ease_out_cubic),
        ];
        let lane_h = tracks.height() / 2.0;
        for (i, (label, value, color, f)) in lanes.into_iter().enumerate() {
            let y = tracks.top() + lane_h * (i as f32 + 0.55);
            let (x0, x1) = (tracks.left() + 10.0, tracks.right() - 10.0);
            painter.text(
                pos2(x0, y - 22.0),
                Align2::LEFT_BOTTOM,
                label,
                font.clone(),
                color,
            );
            painter.line_segment([pos2(x0, y), pos2(x1, y)], Stroke::new(1.0, palette.rule));
            // Where the ball is at each tenth of the duration.
            for k in 0..=10 {
                let x = lerp(x0, x1, f(k as f32 / 10.0));
                painter.circle_filled(pos2(x, y), 2.5, color.gamma_multiply(0.45));
            }
            painter.circle(
                pos2(lerp(x0, x1, value), y),
                9.0,
                color,
                Stroke::new(2.0, palette.card),
            );
        }

        // 3. A bar chart animating to new values.
        let bt = self.bar_t();
        let slot = bars.width() / BARS as f32;
        painter.line_segment(
            [bars.left_bottom(), bars.right_bottom()],
            Stroke::new(1.0, palette.muted),
        );
        for k in 0..BARS {
            let v = lerp(self.from[k], self.to[k], bt);
            let x = bars.left() + slot * k as f32;
            let r = Rect::from_min_max(
                pos2(x + slot * 0.18, bars.bottom() - v * bars.height()),
                pos2(x + slot * 0.82, bars.bottom()),
            );
            painter.rect_filled(
                r,
                3.0,
                if self.eased {
                    palette.trace
                } else {
                    palette.muted
                },
            );
            let target_y = bars.bottom() - self.to[k] * bars.height();
            painter.line_segment(
                [pos2(r.left(), target_y), pos2(r.right(), target_y)],
                Stroke::new(1.5, palette.ink.gamma_multiply(0.5)),
            );
        }
        painter.text(
            bars.right_top(),
            Align2::RIGHT_TOP,
            format!("t = {t:.2}   f(t) = {e:.2}"),
            FontId::monospace(12.5),
            palette.ink,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("easing-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Duration");
                ui.add(egui::Slider::new(&mut self.duration, 0.3..=4.0).suffix(" s"));
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Replay").clicked() {
                self.new_targets();
            }
            ui.checkbox(&mut self.looping, "Loop");
            ui.checkbox(&mut self.eased, "Ease the bars");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        assert_eq!(ease_out_cubic(0.5), 0.875);
    }

    #[test]
    fn monotonic_and_ahead_of_linear() {
        let mut prev = 0.0;
        for k in 1..100 {
            let t = k as f32 / 100.0;
            let v = ease_out_cubic(t);
            assert!(v > prev && v > t);
            prev = v;
        }
    }

    #[test]
    fn speed_is_the_derivative() {
        for k in 1..10 {
            let t = k as f32 / 10.0;
            let h = 1e-3;
            let numeric = (ease_out_cubic(t + h) - ease_out_cubic(t - h)) / (2.0 * h);
            assert!((numeric - ease_out_cubic_speed(t)).abs() < 1e-2);
        }
        assert_eq!(ease_out_cubic_speed(1.0), 0.0);
        assert_eq!(ease_out_cubic_speed(0.0), 3.0);
    }
}
