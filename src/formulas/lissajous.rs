//! Lissajous figures: two perpendicular oscillations at a frequency ratio.

use std::f64::consts::{PI, TAU};

use egui::{Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::formula::{Formula, MathText, Palette, Tone};

const SAMPLES: usize = 720;
/// Radians of `t` per second at speed 1.
const RADIANS_PER_SECOND: f64 = 0.25;

pub struct Lissajous {
    /// Amplitudes.
    amp_x: f64,
    amp_y: f64,
    a: u32,
    b: u32,
    /// Phase offset of x, in units of π.
    delta: f64,
    speed: u32,
    t: f64,
    playing: bool,
    /// Advance δ over time so the figure morphs and appears to spin.
    drift: bool,
    /// δ change per second, in units of π.
    drift_rate: f64,
}

impl Default for Lissajous {
    fn default() -> Self {
        Self {
            amp_x: 1.0,
            amp_y: 1.0,
            a: 3,
            b: 2,
            delta: 0.5,
            speed: 4,
            t: 0.0,
            playing: true,
            drift: true,
            drift_rate: 0.05,
        }
    }
}

impl Lissajous {
    fn point(&self, t: f64) -> [f64; 2] {
        [
            self.amp_x * (self.a as f64 * t + self.delta * PI).sin(),
            self.amp_y * (self.b as f64 * t).sin(),
        ]
    }
}

impl Formula for Lissajous {
    fn params(&self) -> Vec<(String, f64)> {
        vec![
            ("A".into(), self.amp_x),
            ("B".into(), self.amp_y),
            ("a".into(), self.a as f64),
            ("b".into(), self.b as f64),
            ("δ".into(), self.delta),
        ]
    }

    fn id(&self) -> &'static str {
        "lissajous"
    }

    fn name(&self) -> &'static str {
        "Lissajous figures"
    }

    fn category(&self) -> &'static str {
        super::CURVES
    }

    fn notes(&self) -> &'static str {
        "Oscilloscopes show the frequency ratio and phase difference of two signals \
         as a Lissajous figure; the same curves appear in audio phase meters, \
         vibration analysis and logo design."
    }

    fn summary(&self) -> &'static str {
        "Two sine waves, one driving each axis. Whole-number frequency ratios \
         close into knots; the phase shift twists them. With drift on, δ creeps \
         forward and the knot morphs and seems to turn in 3D."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("x(t)")
                .tone(Tone::X)
                .t(" = A sin(a t + δπ)"),
            MathText::new().t("y(t)").tone(Tone::Y).t(" = B sin(b t)"),
        ]
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * if width < 520.0 { 1.0 } else { 0.75 }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.playing {
            self.t += RADIANS_PER_SECOND * self.speed as f64 * dt as f64;
            if self.t >= TAU {
                self.t -= TAU;
            }
        }
        if self.drift {
            self.delta = (self.delta + self.drift_rate * dt as f64).rem_euclid(2.0);
        }

        // Plot area, with gutters on the top and left for the two oscillators.
        let gutter = 36.0;
        let margin = 24.0;
        let side = (rect.width() - gutter - 2.0 * margin)
            .min(rect.height() - gutter - 2.0 * margin - 40.0);
        let plot = Rect::from_min_size(
            pos2(
                rect.center().x - (side - gutter) / 2.0,
                rect.min.y + gutter + margin,
            ),
            vec2(side, side),
        );
        let to_screen = |p: [f64; 2]| {
            pos2(
                plot.center().x + p[0] as f32 * side * 0.45,
                plot.center().y - p[1] as f32 * side * 0.45,
            )
        };

        painter.rect_stroke(
            plot,
            8.0,
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Inside,
        );

        let full: Vec<Pos2> = (0..=SAMPLES)
            .map(|j| to_screen(self.point(TAU * j as f64 / SAMPLES as f64)))
            .collect();
        let faint = Stroke::new(1.0, palette.muted.gamma_multiply(0.45));
        painter.extend(Shape::dashed_line(&full, faint, 3.0, 4.0));

        let steps = ((self.t / TAU) * SAMPLES as f64).ceil() as usize;
        let trail: Vec<Pos2> = (0..=steps)
            .map(|j| to_screen(self.point((j as f64 / SAMPLES as f64 * TAU).min(self.t))))
            .collect();
        if trail.len() > 1 {
            painter.add(Shape::line(trail, Stroke::new(2.2, palette.trace)));
        }

        let tip = to_screen(self.point(self.t));
        let x_marker = pos2(tip.x, plot.top() - gutter / 2.0);
        let y_marker = pos2(plot.left() - gutter / 2.0, tip.y);
        painter.line_segment(
            [
                pos2(plot.left(), x_marker.y),
                pos2(plot.right(), x_marker.y),
            ],
            Stroke::new(1.0, palette.x.gamma_multiply(0.3)),
        );
        painter.line_segment(
            [
                pos2(y_marker.x, plot.top()),
                pos2(y_marker.x, plot.bottom()),
            ],
            Stroke::new(1.0, palette.y.gamma_multiply(0.3)),
        );
        painter.extend(Shape::dashed_line(
            &[x_marker, tip],
            Stroke::new(1.0, palette.x),
            5.0,
            5.0,
        ));
        painter.extend(Shape::dashed_line(
            &[y_marker, tip],
            Stroke::new(1.0, palette.y),
            5.0,
            5.0,
        ));
        painter.circle_filled(x_marker, 4.0, palette.x);
        painter.circle_filled(y_marker, 4.0, palette.y);
        painter.circle_filled(tip, 3.2, palette.ink);

        painter.text(
            rect.right_bottom() + vec2(-16.0, -12.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("{}:{}", self.a, self.b),
            egui::FontId::proportional((rect.width() * 0.06).clamp(20.0, 34.0)),
            palette.trace,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("lissajous-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("A");
                ui.add(
                    egui::Slider::new(&mut self.amp_x, 0.1..=1.0)
                        .step_by(0.05)
                        .clamping(egui::SliderClamping::Edits),
                );
                ui.end_row();
                ui.label("B");
                ui.add(
                    egui::Slider::new(&mut self.amp_y, 0.1..=1.0)
                        .step_by(0.05)
                        .clamping(egui::SliderClamping::Edits),
                );
                ui.end_row();
                ui.label("a");
                ui.add(egui::Slider::new(&mut self.a, 1..=9));
                ui.end_row();
                ui.label("b");
                ui.add(egui::Slider::new(&mut self.b, 1..=9));
                ui.end_row();
                ui.label("δ");
                ui.add(
                    egui::Slider::new(&mut self.delta, 0.0..=2.0)
                        .step_by(0.01)
                        .clamping(egui::SliderClamping::Edits)
                        .suffix(" π"),
                );
                ui.end_row();
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.speed, 1..=10));
                ui.end_row();
                ui.label("Drift");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.drift, "");
                    ui.add_enabled(
                        self.drift,
                        egui::Slider::new(&mut self.drift_rate, 0.01..=0.3)
                            .step_by(0.01)
                            .clamping(egui::SliderClamping::Edits)
                            .suffix(" π/s"),
                    );
                });
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
            if ui.button("Restart").clicked() {
                self.t = 0.0;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::latex::latex_snippet;

    /// With drift on, δ keeps moving while the controls are on screen.
    #[test]
    fn drift_advances_while_the_controls_are_shown() {
        let mut f = Lissajous::default();
        let start = f.delta;
        let ctx = egui::Context::default();
        let palette = Palette::dark();
        for _ in 0..120 {
            let mut out = ctx.run_ui(Default::default(), |ui| {
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 480.0));
                f.paint(ui.painter(), rect, &palette, 1.0 / 60.0);
                f.controls(ui, &palette);
            });
            out.textures_delta.clear();
        }
        // Two seconds at 0.05 π/s.
        assert!(
            (f.delta - start - 0.1).abs() < 0.01,
            "δ went {start} → {}",
            f.delta
        );
    }

    #[test]
    fn snippet_follows_the_sliders() {
        let mut f = Lissajous::default();
        let before = latex_snippet(&f);
        f.a = 5;
        f.delta = 0.25;
        let after = latex_snippet(&f);
        assert_ne!(before, after);
        assert!(after.contains(r"a = 5"), "{after}");
        assert!(after.contains(r"\delta = 0.25"), "{after}");
    }
}
