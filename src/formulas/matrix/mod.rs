//! A 2×2 matrix as a transformation of the plane: the grid warps, the basis
//! vectors land on the columns, and the eigenvectors are the lines that only
//! stretch.

pub mod math;

use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::draw;
use crate::formula::{Formula, MathText, Palette, Tone};
use math::{Eigen, Mat2, Preset, Vec2};

/// Matrix entries stay within ±`LIMIT`.
pub const LIMIT: f32 = 5.0;
/// How long the identity-to-matrix animation takes.
pub const ANIMATION_SECONDS: f32 = 1.6;
const SNAP_STEP: f32 = 0.5;
/// Grid lines on each side of the origin, before and after the transform.
const GRID: i32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Figure {
    Square,
    House,
    LetterF,
}

impl Figure {
    const ALL: [Figure; 3] = [Figure::House, Figure::LetterF, Figure::Square];

    fn label(self) -> &'static str {
        match self {
            Figure::Square => "Unit square",
            Figure::House => "House",
            Figure::LetterF => "Letter F",
        }
    }

    fn outline(self) -> &'static [Vec2] {
        match self {
            Figure::Square => &[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            Figure::House => &[
                [0.2, 0.0],
                [0.8, 0.0],
                [0.8, 0.6],
                [1.0, 0.6],
                [0.5, 1.05],
                [0.0, 0.6],
                [0.2, 0.6],
            ],
            // Asymmetric, so a reflection is easy to spot.
            Figure::LetterF => &[
                [0.25, 0.0],
                [0.45, 0.0],
                [0.45, 0.45],
                [0.7, 0.45],
                [0.7, 0.62],
                [0.45, 0.62],
                [0.45, 0.82],
                [0.8, 0.82],
                [0.8, 1.0],
                [0.25, 1.0],
            ],
        }
    }
}

/// `m` with column `col` (0: where `i` lands, 1: where `j` lands) set to `v`,
/// each entry clamped to ±[`LIMIT`].
pub fn set_column(m: Mat2, col: usize, v: Vec2) -> Mat2 {
    let [x, y] = v.map(|x| x.clamp(-LIMIT, LIMIT));
    if col == 0 {
        Mat2 { a: x, c: y, ..m }
    } else {
        Mat2 { b: x, d: y, ..m }
    }
}

/// Round each coordinate to the nearest multiple of `step`.
pub fn snap(v: Vec2, step: f32) -> Vec2 {
    v.map(|x| (x / step).round() * step)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Where the plane's origin sits on screen and how many points one unit is.
struct Plane {
    origin: Pos2,
    unit: f32,
}

impl Plane {
    fn new(rect: Rect) -> Self {
        Self {
            origin: rect.center() + vec2(0.0, rect.height() * 0.08),
            unit: (rect.width().min(rect.height()) / 7.0).max(20.0),
        }
    }

    fn to_screen(&self, [x, y]: Vec2) -> Pos2 {
        pos2(self.origin.x + x * self.unit, self.origin.y - y * self.unit)
    }

    fn to_world(&self, p: Pos2) -> Vec2 {
        [
            (p.x - self.origin.x) / self.unit,
            (self.origin.y - p.y) / self.unit,
        ]
    }
}

pub struct MatrixTransform {
    /// The matrix being studied.
    m: Mat2,
    /// Progress of the identity-to-`m` animation, 0..=1.
    t: f32,
    animating: bool,
    figure: Figure,
    /// The basis tip being dragged: 0 for `i`, 1 for `j`.
    dragging: Option<usize>,
    snap: bool,
}

impl Default for MatrixTransform {
    fn default() -> Self {
        Self {
            // Two distinct real eigenvalues, so both eigen lines show.
            m: Mat2::new(1.5, 0.5, 0.25, 1.0),
            t: 1.0,
            animating: false,
            figure: Figure::House,
            dragging: None,
            snap: true,
        }
    }
}

impl MatrixTransform {
    /// Play the transformation from the identity.
    fn animate(&mut self) {
        self.t = 0.0;
        self.animating = true;
    }

    fn stop(&mut self) {
        self.t = 1.0;
        self.animating = false;
    }

    fn advance(&mut self, dt: f32) {
        if self.animating {
            self.t += dt / ANIMATION_SECONDS;
            if self.t >= 1.0 {
                self.stop();
            }
        }
    }

    /// The matrix on screen: part way from the identity while animating.
    fn shown(&self) -> Mat2 {
        if self.t >= 1.0 {
            self.m
        } else {
            self.m.lerp(smoothstep(self.t))
        }
    }
}

impl Formula for MatrixTransform {
    fn params(&self) -> Vec<(&'static str, f64)> {
        let Mat2 { a, b, c, d } = self.m;
        vec![
            ("a", a as f64),
            ("b", b as f64),
            ("c", c as f64),
            ("d", d as f64),
        ]
    }

    fn id(&self) -> &'static str {
        "matrix-transform"
    }

    fn name(&self) -> &'static str {
        "Matrix transformation"
    }

    fn category(&self) -> &'static str {
        super::LINEAR_ALGEBRA
    }

    fn summary(&self) -> &'static str {
        "A 2×2 matrix moves every point of the plane. Its columns are where the basis \
         vectors i and j land; drag their tips and watch the whole grid follow. Dashed \
         lines are the eigenvectors: directions the matrix only stretches."
    }

    fn notes(&self) -> &'static str {
        "Every 2D graphics transform (scale, rotate, shear, mirror), image warping, \
         principal component analysis (the eigenvectors of a covariance matrix), Markov \
         chains and population models, and the stability of dynamical systems."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new().t("x'").tone(Tone::X).t(" = a x + b y"),
            MathText::new().t("y'").tone(Tone::Y).t(" = c x + d y"),
            MathText::new().t("det = ad − bc"),
            MathText::new().t("A v = λ v"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::drag()
    }

    fn is_animating(&self) -> bool {
        self.animating
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let plane = Plane::new(rect);
        let m = self.shown();
        let tips = [
            plane.to_screen(m.apply([1.0, 0.0])),
            plane.to_screen(m.apply([0.0, 1.0])),
        ];
        if response.drag_started()
            && let Some(at) = response.interact_pointer_pos()
        {
            self.dragging = draw::pick(&tips, at, 18.0);
            if self.dragging.is_some() {
                self.stop();
            }
        }
        if let Some(col) = self.dragging
            && let Some(at) = response.interact_pointer_pos()
        {
            let mut v = plane.to_world(at);
            if self.snap {
                v = snap(v, SNAP_STEP);
            }
            self.m = set_column(self.m, col, v);
        }
        if response.drag_stopped() {
            self.dragging = None;
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.advance(dt);
        let painter = painter.with_clip_rect(rect);
        let plane = Plane::new(rect);
        let screen = |v: Vec2| plane.to_screen(v);
        let m = self.shown();
        let k = GRID as f32;

        // The plane before (faint) and after the transform.
        let before = Stroke::new(1.0, palette.rule);
        let after = Stroke::new(1.0, palette.trace.gamma_multiply(0.35));
        for n in -GRID..=GRID {
            let x = n as f32;
            painter.line_segment([screen([x, -k]), screen([x, k])], before);
            painter.line_segment([screen([-k, x]), screen([k, x])], before);
        }
        for n in -GRID..=GRID {
            let x = n as f32;
            let stroke = if n == 0 {
                Stroke::new(1.5, palette.muted)
            } else {
                after
            };
            painter.line_segment([screen(m.apply([x, -k])), screen(m.apply([x, k]))], stroke);
            painter.line_segment([screen(m.apply([-k, x])), screen(m.apply([k, x]))], stroke);
        }

        // The unit square's image: its area is |det|, its color the sign.
        let det = m.det();
        let rank = m.rank();
        if rank == 2 {
            let square: Vec<Pos2> = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
                .iter()
                .map(|&p| screen(m.apply(p)))
                .collect();
            let fill = if det > 0.0 {
                palette.fill
            } else {
                palette.x.gamma_multiply(0.22)
            };
            painter.add(draw::fill_polygon(&square, fill, 1.0));
        }

        // The figure before (dashed) and after.
        let outline = self.figure.outline();
        let mut original: Vec<Pos2> = outline.iter().map(|&p| screen(p)).collect();
        original.push(original[0]);
        painter.extend(Shape::dashed_line(
            &original,
            Stroke::new(1.0, palette.muted.gamma_multiply(0.7)),
            4.0,
            4.0,
        ));
        let image: Vec<Pos2> = outline.iter().map(|&p| screen(m.apply(p))).collect();
        painter.add(Shape::closed_line(image, Stroke::new(2.0, palette.trace)));

        // Eigenvectors: lines through the origin that map onto themselves.
        let small = FontId::proportional(13.0);
        let eigen_line = |v: Vec2, l: f32| {
            let ends = [screen([-k * v[0], -k * v[1]]), screen([k * v[0], k * v[1]])];
            painter.extend(Shape::dashed_line(
                &ends,
                Stroke::new(1.5, palette.ink.gamma_multiply(0.55)),
                6.0,
                4.0,
            ));
            let r = 2.6;
            painter.text(
                screen([r * v[0], r * v[1]]) + vec2(6.0, -6.0),
                Align2::LEFT_BOTTOM,
                format!("λ = {l:.2}"),
                small.clone(),
                palette.ink,
            );
        };
        let eigen_caption = match m.eigen() {
            Eigen::Real { values, vectors } => {
                eigen_line(vectors[0], values[0]);
                eigen_line(vectors[1], values[1]);
                format!("λ = {:.2} and {:.2}", values[0], values[1])
            }
            Eigen::Repeated {
                value,
                vector: Some(v),
            } => {
                eigen_line(v, value);
                format!("λ = {value:.2} twice, one eigen direction")
            }
            Eigen::Repeated {
                value,
                vector: None,
            } => {
                format!("λ = {value:.2}: every direction is an eigenvector")
            }
            Eigen::Complex { re, im } => {
                format!("λ = {re:.2} ± {im:.2}i: no real eigenvectors, it turns")
            }
        };

        // Basis vectors and their draggable tips.
        let i = m.apply([1.0, 0.0]);
        let j = m.apply([0.0, 1.0]);
        let origin = screen([0.0, 0.0]);
        for (n, (v, color, name)) in [(i, palette.x, "i"), (j, palette.y, "j")]
            .into_iter()
            .enumerate()
        {
            let tip = screen(v);
            painter.arrow(origin, tip - origin, Stroke::new(3.0, color));
            draw::handle(&painter, tip, color, palette.card, self.dragging == Some(n));
            painter.text(
                tip + vec2(10.0, -10.0),
                Align2::LEFT_BOTTOM,
                name,
                FontId::proportional(15.0),
                color,
            );
        }

        // Readout.
        let mono = FontId::monospace(12.5);
        let base = rect.left_top() + vec2(14.0, 12.0);
        let rows = [
            (format!("[{:>6.2} {:>6.2} ]", m.a, m.b), palette.ink),
            (format!("[{:>6.2} {:>6.2} ]", m.c, m.d), palette.ink),
            (
                match rank {
                    2 if det < 0.0 => format!("det = {det:.2}: area × {:.2}, flipped", -det),
                    2 => format!("det = {det:.2}: area × {det:.2}"),
                    1 => "det = 0: the plane collapses onto a line".to_owned(),
                    _ => "det = 0: everything lands on the origin".to_owned(),
                },
                palette.muted,
            ),
            (eigen_caption, palette.muted),
        ];
        for (n, (text, color)) in rows.into_iter().enumerate() {
            painter.text(
                base + vec2(0.0, 16.0 * n as f32),
                Align2::LEFT_TOP,
                text,
                mono.clone(),
                color,
            );
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        let mut changed = false;
        egui::Grid::new("matrix-entries")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                for (name, v) in [
                    ("a", &mut self.m.a),
                    ("b", &mut self.m.b),
                    ("c", &mut self.m.c),
                    ("d", &mut self.m.d),
                ] {
                    ui.label(name);
                    changed |= ui
                        .add(egui::Slider::new(v, -LIMIT..=LIMIT).step_by(0.05))
                        .changed();
                    ui.end_row();
                }
            });
        if changed {
            self.stop();
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for p in Preset::ALL {
                if ui.button(p.label()).clicked() {
                    self.m = p.matrix();
                    self.animate();
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            for f in Figure::ALL {
                if ui.selectable_label(self.figure == f, f.label()).clicked() {
                    self.figure = f;
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Animate").clicked() {
                self.animate();
            }
            ui.checkbox(&mut self.snap, "Snap tips to 0.5");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::math::{Mat2, Preset};
    use super::*;

    #[test]
    fn dragging_a_basis_tip_sets_its_column() {
        let m = set_column(Mat2::IDENTITY, 0, [2.0, 1.0]);
        assert_eq!(m, Mat2::new(2.0, 0.0, 1.0, 1.0));
        let m = set_column(m, 1, [-1.0, 3.0]);
        assert_eq!(m, Mat2::new(2.0, -1.0, 1.0, 3.0));
    }

    #[test]
    fn snap_rounds_to_the_nearest_step() {
        assert_eq!(snap([0.74, -1.26], 0.5), [0.5, -1.5]);
        assert_eq!(snap([0.76, 0.0], 0.5), [1.0, 0.0]);
    }

    #[test]
    fn entries_are_clamped_to_the_editable_range() {
        let m = set_column(Mat2::IDENTITY, 0, [40.0, -40.0]);
        assert_eq!(m, Mat2::new(LIMIT, 0.0, -LIMIT, 1.0));
    }

    #[test]
    fn animation_runs_from_the_identity_to_the_matrix() {
        let mut f = MatrixTransform {
            m: Preset::ShearX.matrix(),
            ..Default::default()
        };
        f.animate();
        assert_eq!(f.shown(), Mat2::IDENTITY);
        f.advance(ANIMATION_SECONDS / 2.0);
        let mid = f.shown();
        assert!(mid.b > 0.0 && mid.b < 1.0, "{mid:?}");
        f.advance(10.0);
        assert_eq!(f.shown(), f.m);
        assert!(!f.animating);
    }

    #[test]
    fn params_are_the_four_entries() {
        let f = MatrixTransform {
            m: Mat2::new(1.0, 2.0, 3.0, 4.0),
            ..Default::default()
        };
        let names: Vec<_> = f.params().iter().map(|p| p.0).collect();
        assert_eq!(names, ["a", "b", "c", "d"]);
        assert_eq!(f.params()[3].1, 4.0);
    }
}
