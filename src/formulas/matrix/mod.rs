//! A 2×2 matrix as a transformation of the plane: the grid warps, the basis
//! vectors land on the columns, and the eigenvectors are the lines that only
//! stretch. Up to three matrices can be chained and applied in order.

pub mod math;

use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::draw;
use crate::formula::{Formula, MathText, Palette, Tone};
use math::{Eigen, Mat2, Preset, Vec2, product, staged};

/// Matrix entries stay within ±`LIMIT`.
pub const LIMIT: f32 = 5.0;
/// How long each stage of the animation takes.
pub const ANIMATION_SECONDS: f32 = 1.6;
/// The most matrices a chain holds.
pub const MAX_STAGES: usize = 3;
const SNAP_STEP: f32 = 0.5;
/// Grid lines on each side of the origin, before and after the transform.
const GRID: i32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
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

/// The direction along the same line that points up (or right, if level),
/// so labels land on the visible upper half of an eigen line.
pub fn upward(v: Vec2) -> Vec2 {
    if v[1] < 0.0 || (v[1] == 0.0 && v[0] < 0.0) {
        [-v[0], -v[1]]
    } else {
        v
    }
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
            unit: (rect.width().min(rect.height()) / 5.0).max(20.0),
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
    /// The matrices, applied first to last; their product is the transform.
    chain: Vec<Mat2>,
    /// The stage the sliders and presets edit.
    selected: usize,
    /// Animation progress: stage `k` plays while `t` is in `k..k + 1`.
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
            chain: vec![Mat2::new(1.5, 0.5, 0.25, 1.0)],
            selected: 0,
            t: 1.0,
            animating: false,
            figure: Figure::House,
            dragging: None,
            snap: true,
        }
    }
}

impl MatrixTransform {
    /// Play the transformation from the identity, one stage at a time.
    fn animate(&mut self) {
        self.t = 0.0;
        self.animating = true;
    }

    fn stop(&mut self) {
        self.t = self.chain.len() as f32;
        self.animating = false;
    }

    fn advance(&mut self, dt: f32) {
        if self.animating {
            self.t += dt / ANIMATION_SECONDS;
            if self.t >= self.chain.len() as f32 {
                self.stop();
            }
        }
    }

    /// The matrix on screen: the product, or part of it while animating.
    fn shown(&self) -> Mat2 {
        staged(&self.chain, self.t)
    }

    /// Dragging the basis tips edits the matrix directly, which is only
    /// unambiguous while the chain holds a single matrix.
    fn tips_editable(&self) -> bool {
        self.chain.len() == 1
    }

    /// Append an identity stage and select it.
    fn add_stage(&mut self) {
        if self.chain.len() < MAX_STAGES {
            self.chain.push(Mat2::IDENTITY);
            self.selected = self.chain.len() - 1;
            self.dragging = None;
            self.stop();
        }
    }

    fn remove_stage(&mut self) {
        if self.chain.len() > 1 {
            self.chain.remove(self.selected);
            self.selected = self.selected.min(self.chain.len() - 1);
            self.dragging = None;
            self.stop();
        }
    }

    /// `M2·M1`: the chain written the way the product reads.
    fn chain_label(&self) -> String {
        let names: Vec<String> = (1..=self.chain.len())
            .rev()
            .map(|k| format!("M{k}"))
            .collect();
        names.join("·")
    }
}

/// What a sheet saves of a [`MatrixTransform`].
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct State {
    /// The matrices, first applied first, each as `[[a, b], [c, d]]`.
    chain: Vec<[[f32; 2]; 2]>,
    selected: usize,
    animating: bool,
    figure: Figure,
    snap: bool,
}

impl MatrixTransform {
    fn state(&self) -> State {
        State {
            chain: self
                .chain
                .iter()
                .map(|m| [[m.a, m.b], [m.c, m.d]])
                .collect(),
            selected: self.selected,
            animating: self.animating,
            figure: self.figure,
            snap: self.snap,
        }
    }

    fn apply(&mut self, s: State) {
        let chain: Vec<Mat2> = s
            .chain
            .iter()
            .take(MAX_STAGES)
            .map(|m| {
                let [[a, b], [c, d]] = m.map(|row| row.map(|v| v.clamp(-LIMIT, LIMIT)));
                Mat2::new(a, b, c, d)
            })
            .collect();
        if !chain.is_empty() {
            self.chain = chain;
        }
        self.selected = s.selected.min(self.chain.len() - 1);
        self.animating = s.animating;
        self.t = if self.animating {
            0.0
        } else {
            self.chain.len() as f32
        };
        self.figure = s.figure;
        self.snap = s.snap;
    }
}

impl Formula for MatrixTransform {
    crate::state_hooks!();

    fn params(&self) -> Vec<(String, f64)> {
        let Mat2 { a, b, c, d } = product(&self.chain);
        vec![
            ("a".into(), a as f64),
            ("b".into(), b as f64),
            ("c".into(), c as f64),
            ("d".into(), d as f64),
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
         lines are the eigenvectors: directions the matrix only stretches. Chain up to \
         three matrices to see that the order of composition matters."
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
        // A chain's tips are not draggable; leave body drags to the canvas.
        if self.tips_editable() {
            Sense::drag()
        } else {
            Sense::hover()
        }
    }

    fn is_animating(&self) -> bool {
        self.animating
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        if !self.tips_editable() {
            self.dragging = None;
            return;
        }
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
            self.chain[0] = set_column(self.chain[0], col, v);
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
            let square: Vec<Pos2> = Figure::Square
                .outline()
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
            let v = upward(v);
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
            if self.tips_editable() {
                draw::handle(&painter, tip, color, palette.card, self.dragging == Some(n));
            }
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
        let chain = if self.chain.len() > 1 {
            format!("A = {}", self.chain_label())
        } else {
            "A".to_owned()
        };
        let rows = [
            (chain, palette.muted),
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
        // On a card, so the grid lines do not run through the text.
        let galleys: Vec<_> = rows
            .into_iter()
            .map(|(text, color)| painter.layout_no_wrap(text, mono.clone(), color))
            .collect();
        let width = galleys.iter().map(|g| g.size().x).fold(0.0, f32::max);
        let card = Rect::from_min_size(
            base - vec2(8.0, 6.0),
            vec2(width + 16.0, 16.0 * galleys.len() as f32 + 10.0),
        );
        painter.rect(
            card,
            6.0,
            palette.card.gamma_multiply(0.92),
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Inside,
        );
        for (n, galley) in galleys.into_iter().enumerate() {
            painter.galley(base + vec2(0.0, 16.0 * n as f32), galley, palette.ink);
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            for k in 0..self.chain.len() {
                if ui
                    .selectable_label(self.selected == k, format!("M{}", k + 1))
                    .clicked()
                {
                    self.selected = k;
                }
            }
            if self.chain.len() < MAX_STAGES && ui.button("Then apply…").clicked() {
                self.add_stage();
            }
            if self.chain.len() > 1 && ui.button("Remove").clicked() {
                self.remove_stage();
            }
        });
        if self.chain.len() > 1 {
            ui.label(format!(
                "Applied M1 first; the result is A = {}. Edit each stage with its sliders.",
                self.chain_label()
            ));
        }
        let m = &mut self.chain[self.selected];
        let mut changed = false;
        egui::Grid::new("matrix-entries")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                for (name, v) in [
                    ("a", &mut m.a),
                    ("b", &mut m.b),
                    ("c", &mut m.c),
                    ("d", &mut m.d),
                ] {
                    ui.label(name);
                    changed |= ui.add(egui::Slider::new(v, -LIMIT..=LIMIT)).changed();
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
                    self.chain[self.selected] = p.matrix();
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
    fn a_chain_of_matrices_survives_a_save() {
        let m = MatrixTransform {
            chain: vec![
                Mat2::rotation(0.5),
                Mat2::new(2.0, 0.0, 0.0, 0.5),
                Mat2::new(1.0, 1.0, 0.0, 1.0),
            ],
            selected: 2,
            ..Default::default()
        };
        let saved = m.save_state().unwrap();
        let mut back = MatrixTransform::default();
        back.load_state(&saved);
        assert_eq!(back.chain, m.chain);
        assert_eq!(back.selected, 2);
        // Too many stages are cut, an empty chain is ignored.
        let five = serde_json::json!({"chain": vec![[[1, 0], [0, 1]]; 5], "selected": 9});
        back.load_state(&five);
        assert_eq!(
            (back.chain.len(), back.selected),
            (MAX_STAGES, MAX_STAGES - 1)
        );
        back.load_state(&serde_json::json!({"chain": []}));
        assert_eq!(back.chain.len(), MAX_STAGES);
    }

    #[test]
    fn dragging_a_basis_tip_sets_its_column() {
        let m = set_column(Mat2::IDENTITY, 0, [2.0, 1.0]);
        assert_eq!(m, Mat2::new(2.0, 0.0, 1.0, 1.0));
        let m = set_column(m, 1, [-1.0, 3.0]);
        assert_eq!(m, Mat2::new(2.0, -1.0, 1.0, 3.0));
    }

    #[test]
    fn eigen_labels_go_on_the_upper_end_of_their_line() {
        assert_eq!(upward([0.0, -1.0]), [0.0, 1.0]);
        assert_eq!(upward([-1.0, 0.0]), [1.0, 0.0]);
        assert_eq!(upward([-0.6, 0.8]), [-0.6, 0.8]);
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
            chain: vec![Preset::ShearX.matrix()],
            ..Default::default()
        };
        f.animate();
        assert_eq!(f.shown(), Mat2::IDENTITY);
        f.advance(ANIMATION_SECONDS / 2.0);
        let mid = f.shown();
        assert!(mid.b > 0.0 && mid.b < 1.0, "{mid:?}");
        f.advance(10.0);
        assert_eq!(f.shown(), f.chain[0]);
        assert!(!f.animating);
    }

    #[test]
    fn a_chain_animates_through_every_stage() {
        let mut f = MatrixTransform {
            chain: vec![Preset::Scale.matrix(), Preset::Rotate.matrix()],
            ..Default::default()
        };
        f.animate();
        f.advance(ANIMATION_SECONDS);
        assert_eq!(f.shown(), f.chain[0], "first stage done");
        assert!(f.animating);
        f.advance(ANIMATION_SECONDS);
        assert_eq!(f.shown(), math::product(&f.chain));
        assert!(!f.animating);
    }

    #[test]
    fn tips_are_draggable_only_for_a_single_matrix() {
        let mut f = MatrixTransform::default();
        assert!(f.tips_editable());
        f.add_stage();
        assert_eq!(f.chain.len(), 2);
        assert_eq!(f.selected, 1, "the new stage is selected");
        assert!(!f.tips_editable());
        f.add_stage();
        f.add_stage();
        assert_eq!(f.chain.len(), MAX_STAGES);
        f.remove_stage();
        assert_eq!(f.chain.len(), MAX_STAGES - 1);
        assert!(f.selected < f.chain.len());
    }

    /// Dragging the tip of `i` on the canvas moves the first column, through
    /// the workspace's real input routing.
    #[test]
    fn dragging_the_i_tip_on_the_canvas_edits_the_matrix() {
        use crate::cad::board::Board;
        use crate::cad::camera::Camera;
        use crate::cad::canvas::{Settings, Workspace};
        use crate::cad::theme::{Chrome, palette};
        use egui::{Event, PointerButton, RawInput};

        let mut ws = Workspace::new(Board::default(), Camera::default(), Settings::default());
        let tool = crate::formulas::tool("matrix-transform").unwrap();
        let uid = ws.place(tool, Some(pos2(0.0, 0.0)), egui::Vec2::ZERO);
        ws.board.selection = vec![uid];
        let rect = ws.board.get(uid).unwrap().rect;
        ws.camera.center = rect.center();
        let ctx = egui::Context::default();
        let (chrome, pal) = (Chrome::graphite(), palette(true));
        let run = |ws: &mut Workspace, events: Vec<Event>| {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 1000.0))),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().show(ui, |ui| ws.show(ui, &chrome, &pal));
            });
            out.textures_delta.clear();
        };
        run(&mut ws, vec![]);
        let to_screen = ws.camera.transform(ws.view, 1.0);
        let plane = Plane::new(rect);
        // The default matrix sends i to (1.5, 0.25); drag it to (2, 1).
        let from = to_screen.mul_pos(plane.to_screen([1.5, 0.25]));
        let to = to_screen.mul_pos(plane.to_screen([2.0, 1.0]));
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        run(&mut ws, vec![Event::PointerMoved(from)]);
        run(&mut ws, vec![button(from, true)]);
        for k in 1..=10 {
            run(
                &mut ws,
                vec![Event::PointerMoved(from.lerp(to, k as f32 / 10.0))],
            );
        }
        run(&mut ws, vec![button(to, false)]);
        let params = ws.board.get(uid).unwrap().formula.params();
        assert_eq!((params[0].1, params[2].1), (2.0, 1.0), "{params:?}");
    }

    /// Showing the sliders must not round a preset's exact entries.
    #[test]
    fn controls_leave_preset_values_exact() {
        let mut f = MatrixTransform {
            chain: vec![Preset::Rotate.matrix()],
            ..Default::default()
        };
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| f.controls(ui, &Palette::dark()));
        out.textures_delta.clear();
        assert_eq!(f.chain[0], Preset::Rotate.matrix());
    }

    /// With a chain the stage ignores drags, so it must not claim them:
    /// dragging the body should move the viewport instead.
    #[test]
    fn a_chain_leaves_body_drags_to_the_canvas() {
        let mut f = MatrixTransform::default();
        assert!(f.stage_sense().senses_drag());
        f.add_stage();
        assert!(!f.stage_sense().senses_drag());
    }

    #[test]
    fn changing_the_chain_ends_a_tip_drag() {
        let mut f = MatrixTransform {
            dragging: Some(0),
            ..Default::default()
        };
        f.add_stage();
        f.remove_stage();
        assert_eq!(f.dragging, None);
    }

    #[test]
    fn params_are_the_entries_of_the_product() {
        let f = MatrixTransform {
            chain: vec![Mat2::new(1.0, 2.0, 3.0, 4.0), Preset::Reflect.matrix()],
            ..Default::default()
        };
        let names: Vec<_> = f.params().into_iter().map(|p| p.0).collect();
        assert_eq!(names, ["a", "b", "c", "d"]);
        // Reflecting in the x axis negates the bottom row.
        let values: Vec<_> = f.params().iter().map(|p| p.1).collect();
        assert_eq!(values, [1.0, 2.0, -3.0, -4.0]);
    }
}
