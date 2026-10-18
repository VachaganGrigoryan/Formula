//! Polar to Cartesian conversion: a radius and an angle become a pixel position.

use std::f32::consts::{PI, TAU};

use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::derive::{Derivation, Step, Substitution};
use crate::formula::{Formula, MathText, Palette, Tone};

/// `(r, θ)` → `(x, y)`, with y pointing up.
pub fn to_cartesian(r: f32, theta: f32) -> [f32; 2] {
    [r * theta.cos(), r * theta.sin()]
}

/// `(x, y)` → `(r, θ)` with `θ` in `[0, 2π)`.
pub fn to_polar(x: f32, y: f32) -> (f32, f32) {
    (x.hypot(y), y.atan2(x).rem_euclid(TAU))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Point,
    Rose,
}

pub struct Polar {
    mode: Mode,
    r: f32,
    theta: f32,
    petals: u32,
    playing: bool,
    speed: u32,
}

impl Default for Polar {
    fn default() -> Self {
        Self {
            mode: Mode::Point,
            r: 0.8,
            theta: 0.75,
            petals: 5,
            playing: true,
            speed: 3,
        }
    }
}

impl Polar {
    /// The radius actually plotted: free in point mode, `cos kθ` on the rose.
    fn radius(&self, theta: f32) -> f32 {
        match self.mode {
            Mode::Point => self.r,
            Mode::Rose => (self.petals as f32 * theta).cos(),
        }
    }
}

fn geometry(rect: Rect) -> (Pos2, f32) {
    (
        rect.center() - vec2(0.0, 10.0),
        rect.width().min(rect.height()) * 0.4,
    )
}

impl Formula for Polar {
    fn derivations(&self) -> Vec<Derivation> {
        let m = MathText::new;
        vec![Derivation::new(
            "The unit circle",
            vec![
                Step::new(
                    m().t("(cos θ, sin θ)"),
                    "By definition, cos θ and sin θ are the coordinates of the point at angle θ on the circle of radius 1.",
                ),
                Step::new(
                    m().t("(r cos θ, r sin θ)"),
                    "A point r times as far out in the same direction has both coordinates scaled by r.",
                ),
                Step::new(
                    m().t("x")
                        .tone(Tone::X)
                        .t(" = r cos θ,   ")
                        .t("y")
                        .tone(Tone::Y)
                        .t(" = r sin θ"),
                    "So these are its Cartesian coordinates.",
                ),
                Step::new(
                    m().t("r = √(x")
                        .sup("2")
                        .t(" + y")
                        .sup("2")
                        .t("),   θ = atan2(y, x)"),
                    "Going back: Pythagoras gives r, and the two-argument arctangent gives θ in the right quadrant.",
                ),
            ],
        )]
    }

    fn substitutions(&self) -> Vec<Substitution> {
        let values = [
            ("r", self.radius(self.theta) as f64),
            ("θ", self.theta as f64),
        ];
        vec![
            Substitution::new(MathText::new().t("x").tone(Tone::X), "r cos(θ)", &values),
            Substitution::new(MathText::new().t("y").tone(Tone::Y), "r sin(θ)", &values),
        ]
    }

    fn params(&self) -> Vec<(String, f64)> {
        vec![("r".into(), self.r as f64), ("θ".into(), self.theta as f64)]
    }

    fn id(&self) -> &'static str {
        "polar-to-cartesian"
    }

    fn name(&self) -> &'static str {
        "Polar to Cartesian"
    }

    fn category(&self) -> &'static str {
        super::COORDINATES
    }

    fn summary(&self) -> &'static str {
        "A distance and a direction become a position. Drag the point, or switch to \
         the rose curve to watch r = cos kθ draw petals one angle at a time."
    }

    fn notes(&self) -> &'static str {
        "Pie and donut charts, radar/spider charts, circular heatmaps, rose diagrams and \
         radial trees all compute positions from a radius r and an angle θ, then convert \
         to (x, y) pixels with exactly these two lines."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new().t("x").tone(Tone::X).t(" = r cos(θ)"),
            MathText::new().t("y").tone(Tone::Y).t(" = r sin(θ)"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        if !(response.dragged() || response.clicked()) {
            return;
        }
        let Some(at) = response.interact_pointer_pos() else {
            return;
        };
        let (center, scale) = geometry(rect);
        let (r, theta) = to_polar((at.x - center.x) / scale, (center.y - at.y) / scale);
        self.theta = theta;
        if self.mode == Mode::Point {
            self.r = r.min(1.0);
        }
        self.playing = false;
    }

    fn min_size(&self) -> egui::Vec2 {
        // The readout sits below the plot; narrower and it runs into the 270° label.
        egui::vec2(420.0, self.stage_height(420.0))
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.playing {
            self.theta = (self.theta + 0.15 * self.speed as f32 * dt).rem_euclid(TAU);
        }
        let (center, scale) = geometry(rect);
        let to_screen = |[x, y]: [f32; 2]| pos2(center.x + x * scale, center.y - y * scale);

        // Polar grid: rings of equal r, spokes of equal θ.
        let grid = Stroke::new(1.0, palette.rule);
        for ring in 1..=4 {
            painter.circle_stroke(center, scale * ring as f32 / 4.0, grid);
        }
        for spoke in 0..12 {
            let a = spoke as f32 * PI / 6.0;
            painter.line_segment([center, to_screen(to_cartesian(1.0, a))], grid);
            if spoke % 3 == 0 {
                let label = to_screen(to_cartesian(1.1, a));
                painter.text(
                    label,
                    Align2::CENTER_CENTER,
                    format!("{}°", spoke * 30),
                    FontId::proportional(12.0),
                    palette.muted,
                );
            }
        }
        let axis = Stroke::new(1.0, palette.muted.gamma_multiply(0.6));
        painter.line_segment([to_screen([-1.05, 0.0]), to_screen([1.05, 0.0])], axis);
        painter.line_segment([to_screen([0.0, -1.05]), to_screen([0.0, 1.05])], axis);

        if self.mode == Mode::Rose {
            let steps = 720;
            let full: Vec<Pos2> = (0..=steps)
                .map(|j| {
                    let a = TAU * j as f32 / steps as f32;
                    to_screen(to_cartesian(self.radius(a), a))
                })
                .collect();
            painter.extend(Shape::dashed_line(
                &full,
                Stroke::new(1.0, palette.muted.gamma_multiply(0.45)),
                3.0,
                4.0,
            ));
            let drawn = (self.theta / TAU * steps as f32).ceil() as usize;
            let trail: Vec<Pos2> = (0..=drawn)
                .map(|j| {
                    let a = (TAU * j as f32 / steps as f32).min(self.theta);
                    to_screen(to_cartesian(self.radius(a), a))
                })
                .collect();
            if trail.len() > 1 {
                painter.add(Shape::line(trail, Stroke::new(2.2, palette.trace)));
            }
        }

        let r = self.radius(self.theta);
        let [x, y] = to_cartesian(r, self.theta);
        let p = to_screen([x, y]);
        let foot_x = to_screen([x, 0.0]);
        let foot_y = to_screen([0.0, y]);

        // The angle, swept from the positive x axis.
        let arc_r = scale * 0.16;
        let arc: Vec<Pos2> = (0..=48)
            .map(|j| to_screen(to_cartesian(arc_r / scale, self.theta * j as f32 / 48.0)))
            .collect();
        painter.add(Shape::line(arc, Stroke::new(1.5, palette.muted)));
        painter.text(
            to_screen(to_cartesian(arc_r / scale + 0.08, self.theta / 2.0)),
            Align2::CENTER_CENTER,
            "θ",
            FontId::proportional(15.0),
            palette.muted,
        );

        // The two components the conversion produces.
        painter.line_segment([center, foot_x], Stroke::new(4.0, palette.x));
        painter.line_segment([center, foot_y], Stroke::new(4.0, palette.y));
        painter.extend(Shape::dashed_line(
            &[p, foot_x],
            Stroke::new(1.0, palette.x),
            4.0,
            4.0,
        ));
        painter.extend(Shape::dashed_line(
            &[p, foot_y],
            Stroke::new(1.0, palette.y),
            4.0,
            4.0,
        ));
        painter.line_segment([center, p], Stroke::new(2.0, palette.trace));
        painter.circle(p, 6.5, palette.trace, Stroke::new(2.0, palette.card));

        let font = FontId::monospace(13.0);
        let line = 18.0;
        let base = rect.left_bottom() + vec2(16.0, -12.0 - 2.0 * line);
        painter.text(
            base,
            Align2::LEFT_BOTTOM,
            format!("r = {r:+.2}   θ = {:.0}°", self.theta.to_degrees()),
            font.clone(),
            palette.ink,
        );
        painter.text(
            base + vec2(0.0, line),
            Align2::LEFT_BOTTOM,
            format!("x = r cos θ = {x:+.2}"),
            font.clone(),
            palette.x,
        );
        painter.text(
            base + vec2(0.0, 2.0 * line),
            Align2::LEFT_BOTTOM,
            format!("y = r sin θ = {y:+.2}"),
            font,
            palette.y,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.mode, Mode::Point, "Point");
            ui.selectable_value(&mut self.mode, Mode::Rose, "Rose curve r = cos kθ");
        });
        ui.add_space(6.0);
        egui::Grid::new("polar-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                if self.mode == Mode::Point {
                    ui.label("r");
                    ui.add(egui::Slider::new(&mut self.r, 0.0..=1.0));
                } else {
                    ui.label("k");
                    ui.add(egui::Slider::new(&mut self.petals, 1..=9));
                }
                ui.end_row();
                ui.label("θ");
                let mut deg = self.theta.to_degrees();
                if ui
                    .add(egui::Slider::new(&mut deg, 0.0..=359.9).suffix("°"))
                    .changed()
                {
                    self.theta = deg.to_radians();
                    self.playing = false;
                }
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
    fn quarter_turns() {
        let [x, y] = to_cartesian(2.0, PI / 2.0);
        assert!(x.abs() < 1e-6 && (y - 2.0).abs() < 1e-6);
        let [x, y] = to_cartesian(1.0, PI);
        assert!((x + 1.0).abs() < 1e-6 && y.abs() < 1e-6);
    }

    #[test]
    fn round_trip() {
        for &(r, t) in &[(0.5, 0.3), (1.0, 2.0), (0.8, 5.5)] {
            let [x, y] = to_cartesian(r, t);
            let (r2, t2) = to_polar(x, y);
            assert!((r - r2).abs() < 1e-5 && (t - t2).abs() < 1e-5);
        }
    }
}
