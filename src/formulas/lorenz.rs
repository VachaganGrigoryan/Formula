//! The Lorenz attractor: three coupled differential equations whose solutions
//! loop forever around two lobes without ever repeating.

use std::collections::VecDeque;

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::formula::{Formula, MathText, Palette, Tone};

pub type V3 = [f64; 3];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub sigma: f64,
    pub rho: f64,
    pub beta: f64,
}

/// Lorenz's own values from 1963.
pub const CLASSIC: Params = Params {
    sigma: 10.0,
    rho: 28.0,
    beta: 8.0 / 3.0,
};

/// The right-hand side: how fast each coordinate is changing at `s`.
pub fn derivative(p: &Params, [x, y, z]: V3) -> V3 {
    [p.sigma * (y - x), x * (p.rho - z) - y, x * y - p.beta * z]
}

/// One fourth-order Runge–Kutta step of size `h`.
pub fn rk4(p: &Params, s: V3, h: f64) -> V3 {
    let add = |a: V3, b: V3, k: f64| [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k];
    let k1 = derivative(p, s);
    let k2 = derivative(p, add(s, k1, h / 2.0));
    let k3 = derivative(p, add(s, k2, h / 2.0));
    let k4 = derivative(p, add(s, k3, h));
    let mut out = s;
    for i in 0..3 {
        out[i] += h / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
    }
    out
}

/// Points where every derivative is zero: the origin, plus the centers of the
/// two lobes, C±, once ρ > 1.
pub fn fixed_points(p: &Params) -> Vec<V3> {
    let mut out = vec![[0.0; 3]];
    if p.rho > 1.0 {
        let r = (p.beta * (p.rho - 1.0)).sqrt();
        out.push([r, r, p.rho - 1.0]);
        out.push([-r, -r, p.rho - 1.0]);
    }
    out
}

/// Above this ρ the lobe centers stop attracting and the motion turns
/// chaotic: ρ_H = σ(σ + β + 3) / (σ − β − 1). Only defined for σ > β + 1.
pub fn hopf_rho(p: &Params) -> Option<f64> {
    let d = p.sigma - p.beta - 1.0;
    (d > 0.0).then(|| p.sigma * (p.sigma + p.beta + 3.0) / d)
}

pub fn distance(a: V3, b: V3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Integration step, in model time units.
const H: f64 = 0.004;
/// How far apart the extra trajectories start.
const NUDGE: f64 = 1e-5;
/// A point near the attractor, so the path starts flipping between lobes at once.
const START: V3 = [-8.0, 8.0, 27.0];
/// Longest the separation plot runs, in model time units.
const PLOT_SPAN: f64 = 40.0;

struct Trajectory {
    state: V3,
    trail: VecDeque<V3>,
}

impl Trajectory {
    fn new(state: V3) -> Self {
        Self {
            state,
            trail: VecDeque::from([state]),
        }
    }
}

pub struct Lorenz {
    params: Params,
    paths: Vec<Trajectory>,
    /// How many trajectories run side by side (1 to 3).
    count: usize,
    time: f64,
    /// Model time units per second.
    speed: f64,
    /// Trail length, in integration steps.
    trail: usize,
    /// (time, log10 of the distance between the first two trajectories).
    separation: Vec<(f64, f64)>,
    yaw: f32,
    pitch: f32,
    spinning: bool,
    playing: bool,
    carry: f64,
}

impl Default for Lorenz {
    fn default() -> Self {
        let mut s = Self {
            params: CLASSIC,
            paths: Vec::new(),
            count: 2,
            time: 0.0,
            speed: 1.0,
            trail: 2500,
            separation: Vec::new(),
            yaw: 0.0,
            pitch: 0.25,
            spinning: true,
            playing: true,
            carry: 0.0,
        };
        s.restart();
        s
    }
}

impl Lorenz {
    fn restart(&mut self) {
        self.paths = (0..self.count)
            .map(|k| {
                let nudge = [0.0, NUDGE, -NUDGE][k];
                Trajectory::new([START[0] + nudge, START[1], START[2]])
            })
            .collect();
        self.time = 0.0;
        self.carry = 0.0;
        self.separation.clear();
    }

    fn step(&mut self, steps: usize) {
        for _ in 0..steps {
            for path in &mut self.paths {
                path.state = rk4(&self.params, path.state, H);
                if !path.state.iter().all(|v| v.is_finite()) {
                    path.state = START;
                }
                path.trail.push_back(path.state);
                while path.trail.len() > self.trail {
                    path.trail.pop_front();
                }
            }
            self.time += H;
            if self.paths.len() > 1 {
                let d = distance(self.paths[0].state, self.paths[1].state);
                let due = self
                    .separation
                    .last()
                    .is_none_or(|&(t, _)| self.time - t >= 0.05);
                if due && self.time <= PLOT_SPAN {
                    self.separation.push((self.time, d.max(1e-12).log10()));
                }
            }
        }
    }

    /// Orthographic view: spin about the vertical z axis, then tilt. The
    /// attractor grows with ρ, so the view is scaled to match.
    fn project(&self, rect: Rect, [x, y, z]: V3) -> Pos2 {
        let size = self.params.rho.max(10.0) as f32;
        let (x, y, z) = (x as f32, y as f32, z as f32 - 0.85 * size);
        let (sy, cy) = self.yaw.sin_cos();
        let (x1, depth) = (x * cy - y * sy, x * sy + y * cy);
        let (sp, cp) = self.pitch.sin_cos();
        let up = z * cp - depth * sp;
        let scale = rect.height().min(rect.width() * 0.9) / (1.85 * size);
        pos2(rect.center().x + x1 * scale, rect.center().y - up * scale)
    }

    fn colors(&self, palette: &Palette) -> [Color32; 3] {
        [palette.trace, palette.y, palette.x]
    }

    fn paint_trail(&self, painter: &egui::Painter, rect: Rect, path: &Trajectory, color: Color32) {
        let pts: Vec<Pos2> = path.trail.iter().map(|&p| self.project(rect, p)).collect();
        if pts.len() < 2 {
            return;
        }
        // Draw in chunks that fade toward the tail; one shape per chunk keeps
        // thousands of points cheap.
        let chunks = 40.min(pts.len() - 1);
        let per = (pts.len() - 1).div_ceil(chunks);
        let mut k = 0;
        while k < pts.len() - 1 {
            let end = (k + per).min(pts.len() - 1);
            let age = end as f32 / (pts.len() - 1) as f32;
            let piece = pts[k..=end].to_vec();
            painter.add(Shape::line(
                piece.clone(),
                Stroke::new(5.0, color.gamma_multiply(0.10 * age)),
            ));
            painter.add(Shape::line(
                piece,
                Stroke::new(1.3, color.gamma_multiply(0.15 + 0.85 * age)),
            ));
            k = end;
        }
        let head = *pts.last().unwrap();
        painter.circle_filled(head, 7.0, color.gamma_multiply(0.25));
        painter.circle_filled(head, 3.2, color);
    }

    fn paint_separation(&self, painter: &egui::Painter, rect: Rect, palette: &Palette) {
        let size = vec2((rect.width() * 0.32).clamp(150.0, 240.0), 70.0);
        let area = Rect::from_min_size(rect.left_bottom() + vec2(14.0, -size.y - 26.0), size);
        painter.rect_filled(area.expand(6.0), 6.0, palette.card.gamma_multiply(0.85));
        painter.rect_stroke(
            area.expand(6.0),
            6.0,
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Inside,
        );
        // log10 distance from 1e-6 up to 1e2.
        let (lo, hi) = (-6.0, 2.0);
        let at = |t: f64, l: f64| {
            pos2(
                area.left() + (t / PLOT_SPAN) as f32 * area.width(),
                area.bottom() - ((l - lo) / (hi - lo)).clamp(0.0, 1.0) as f32 * area.height(),
            )
        };
        let one = at(0.0, 0.0).y;
        painter.line_segment(
            [pos2(area.left(), one), pos2(area.right(), one)],
            Stroke::new(1.0, palette.rule),
        );
        let line: Vec<Pos2> = self.separation.iter().map(|&(t, l)| at(t, l)).collect();
        if line.len() > 1 {
            painter.add(Shape::line(line, Stroke::new(1.6, palette.y)));
        }
        painter.text(
            area.left_bottom() + vec2(0.0, 6.0),
            Align2::LEFT_TOP,
            "distance between the two paths (log scale)",
            FontId::proportional(10.5),
            palette.muted,
        );
        painter.text(
            pos2(area.right(), one - 2.0),
            Align2::RIGHT_BOTTOM,
            "1",
            FontId::proportional(10.5),
            palette.muted,
        );
    }
}

impl Formula for Lorenz {
    fn params(&self) -> Vec<(&'static str, f64)> {
        let p = &self.params;
        vec![("σ", p.sigma), ("ρ", p.rho), ("β", p.beta)]
    }

    fn id(&self) -> &'static str {
        "lorenz"
    }

    fn name(&self) -> &'static str {
        "Lorenz attractor"
    }

    fn category(&self) -> &'static str {
        super::CHAOS
    }

    fn summary(&self) -> &'static str {
        "Three coupled equations, stepped forward a few thousandths of a time unit at a \
         time. The path circles one lobe, flips to the other at unpredictable moments, and \
         never crosses itself. Two paths that start 0.00001 apart soon go separate ways. \
         Drag to orbit."
    }

    fn notes(&self) -> &'static str {
        "Lorenz found this in 1963 while simplifying a model of convection in the \
         atmosphere, and it is why long-range weather forecasts are hopeless: tiny \
         measurement errors grow exponentially. The same numerical integration (Euler, \
         Runge-Kutta) drives physics engines, orbit simulators and particle effects."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new().t("dx/dt").tone(Tone::X).t(" = σ(y − x)"),
            MathText::new()
                .t("dy/dt")
                .tone(Tone::Y)
                .t(" = x(ρ − z) − y"),
            MathText::new().t("dz/dt").tone(Tone::Trace).t(" = xy − βz"),
            MathText::new().t("classic values: σ = 10, ρ = 28, β = 8/3"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::drag()
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * if width < 520.0 { 1.0 } else { 0.72 }
    }

    fn interact(&mut self, response: &egui::Response, _rect: Rect) {
        if response.dragged() {
            let d = response.drag_delta();
            self.yaw += d.x * 0.01;
            self.pitch = (self.pitch + d.y * 0.01).clamp(-1.5, 1.5);
            self.spinning = false;
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.playing {
            self.carry += self.speed * dt.min(0.1) as f64 / H;
            let steps = self.carry.floor();
            self.carry -= steps;
            self.step((steps as usize).min(4000));
        }
        if self.spinning {
            self.yaw += 0.12 * dt;
        }

        // The fixed points: the origin and the centers of the two lobes.
        for (k, &p) in fixed_points(&self.params).iter().enumerate() {
            let at = self.project(rect, p);
            let s = 4.0;
            let st = Stroke::new(1.2, palette.muted);
            painter.line_segment([at - vec2(s, s), at + vec2(s, s)], st);
            painter.line_segment([at - vec2(s, -s), at + vec2(s, -s)], st);
            painter.text(
                at + vec2(7.0, -7.0),
                Align2::LEFT_BOTTOM,
                ["0", "C+", "C−"][k],
                FontId::proportional(11.0),
                palette.muted,
            );
        }

        let colors = self.colors(palette);
        for (k, path) in self.paths.iter().enumerate().rev() {
            self.paint_trail(painter, rect, path, colors[k]);
        }

        // Live readout for the first path.
        let s = self.paths[0].state;
        let d = derivative(&self.params, s);
        let font = FontId::monospace(12.0);
        let rows = [
            (
                format!("x = {:7.2}   dx/dt = {:8.1}", s[0], d[0]),
                palette.x,
            ),
            (
                format!("y = {:7.2}   dy/dt = {:8.1}", s[1], d[1]),
                palette.y,
            ),
            (
                format!("z = {:7.2}   dz/dt = {:8.1}", s[2], d[2]),
                palette.trace,
            ),
            (format!("t = {:7.2}", self.time), palette.muted),
        ];
        for (k, (text, color)) in rows.into_iter().enumerate() {
            painter.text(
                rect.left_top() + vec2(14.0, 12.0 + 16.0 * k as f32),
                Align2::LEFT_TOP,
                text,
                font.clone(),
                color,
            );
        }

        if self.paths.len() > 1 {
            self.paint_separation(painter, rect, palette);
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        let before = self.params;
        egui::Grid::new("lorenz-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("σ");
                ui.add(egui::Slider::new(&mut self.params.sigma, 0.5..=20.0).step_by(0.1));
                ui.end_row();
                ui.label("ρ");
                ui.add(egui::Slider::new(&mut self.params.rho, 0.5..=60.0).step_by(0.1));
                ui.end_row();
                ui.label("β");
                ui.add(egui::Slider::new(&mut self.params.beta, 0.1..=6.0).step_by(0.01));
                ui.end_row();
                ui.label("Speed");
                ui.add(
                    egui::Slider::new(&mut self.speed, 0.05..=3.0)
                        .step_by(0.05)
                        .suffix(" t/s"),
                );
                ui.end_row();
                ui.label("Trail");
                ui.add(egui::Slider::new(&mut self.trail, 200..=12000).suffix(" steps"));
                ui.end_row();
                ui.label("Paths");
                ui.horizontal(|ui| {
                    for n in 1..=3 {
                        if ui
                            .selectable_label(self.count == n, n.to_string())
                            .clicked()
                            && self.count != n
                        {
                            self.count = n;
                            self.restart();
                        }
                    }
                });
                ui.end_row();
            });
        if self.params != before {
            // A new system: start the separation plot over.
            self.separation.clear();
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui.button("Restart").clicked() {
                self.restart();
            }
            if ui.button("Classic values").clicked() {
                self.params = CLASSIC;
                self.restart();
            }
            ui.checkbox(&mut self.spinning, "Spin");
        });
    }

    fn details(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let p = &self.params;
        let regime = match hopf_rho(p) {
            _ if p.rho < 1.0 => "ρ < 1: every path falls into the origin.".to_string(),
            Some(h) if p.rho < h => format!(
                "1 < ρ < {h:.2}: paths spiral into one of the lobe centers C+ or C−. \
                 Raise ρ past {h:.2} for chaos."
            ),
            Some(h) => format!(
                "ρ > {h:.2} = σ(σ + β + 3) / (σ − β − 1): the lobe centers repel and the \
                 motion is chaotic."
            ),
            None => "With σ ≤ β + 1 the lobe centers stay stable for every ρ.".to_string(),
        };
        ui.label(egui::RichText::new(regime).color(palette.muted));
        if self.paths.len() > 1 {
            let d = distance(self.paths[0].state, self.paths[1].state);
            ui.label(
                egui::RichText::new(format!(
                    "The paths started {NUDGE:e} apart and are now {d:.3e} apart. On the \
                     log plot that growth is a straight ramp: the gap multiplies by about e \
                     every 1.1 time units (the Lyapunov exponent, about 0.9) until it is as \
                     wide as the attractor."
                ))
                .color(palette.muted),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_points_do_not_move() {
        for p in fixed_points(&CLASSIC) {
            let d = derivative(&CLASSIC, p);
            assert!(d.iter().all(|v| v.abs() < 1e-9), "{p:?} moves: {d:?}");
        }
        assert_eq!(
            fixed_points(&Params {
                rho: 0.5,
                ..CLASSIC
            })
            .len(),
            1
        );
    }

    #[test]
    fn classic_hopf_threshold() {
        assert!((hopf_rho(&CLASSIC).unwrap() - 24.7368).abs() < 1e-3);
        assert!(
            hopf_rho(&Params {
                sigma: 2.0,
                ..CLASSIC
            })
            .is_none()
        );
    }

    #[test]
    fn rk4_matches_exact_decay() {
        // With x = y = 0 only z moves: dz/dt = −βz, so z(t) = z0·e^(−βt).
        let mut s = [0.0, 0.0, 10.0];
        for _ in 0..250 {
            s = rk4(&CLASSIC, s, 0.004);
        }
        let exact = 10.0 * (-CLASSIC.beta * 1.0).exp();
        assert!((s[2] - exact).abs() < 1e-8);
    }

    #[test]
    fn stays_on_the_attractor() {
        let mut s = START;
        for _ in 0..20_000 {
            s = rk4(&CLASSIC, s, H);
            assert!(s[0].abs() < 30.0 && s[1].abs() < 35.0 && (-1.0..60.0).contains(&s[2]));
        }
    }

    #[test]
    fn nearby_paths_diverge() {
        let (mut a, mut b) = (START, [START[0] + NUDGE, START[1], START[2]]);
        for _ in 0..(30.0 / H) as usize {
            a = rk4(&CLASSIC, a, H);
            b = rk4(&CLASSIC, b, H);
        }
        assert!(distance(a, b) > 1.0, "still {}", distance(a, b));
    }

    #[test]
    fn low_rho_settles_on_a_lobe_center() {
        let p = Params {
            rho: 14.0,
            ..CLASSIC
        };
        let mut s = START;
        for _ in 0..(60.0 / H) as usize {
            s = rk4(&p, s, H);
        }
        let near = fixed_points(&p)[1..]
            .iter()
            .map(|&c| distance(s, c))
            .fold(f64::MAX, f64::min);
        assert!(near < 1e-3, "{s:?}");
    }

    #[test]
    fn trail_is_capped() {
        let mut l = Lorenz {
            trail: 300,
            ..Lorenz::default()
        };
        l.step(1000);
        assert!(l.paths.iter().all(|p| p.trail.len() == 300));
        assert_eq!(l.paths.len(), 2);
        assert!(!l.separation.is_empty());
    }
}
