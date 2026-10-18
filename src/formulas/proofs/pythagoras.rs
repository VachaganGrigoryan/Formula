//! Two proofs of a² + b² = c²: four triangles rearranged inside an
//! (a + b)² square, and the altitude that splits a right triangle into two
//! similar copies of itself.

use egui::{Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::derive::{Derivation, Step, Substitution};
use crate::formula::{Formula, MathText, Palette, Tone};

/// A triangle as its right-angle corner, the end of its leg a, and the end of
/// its leg b.
pub type Tri = [[f64; 2]; 3];

/// The four triangles around the tilted c² square in the (a + b)² square
/// (corners at the origin and (a + b, a + b), y up).
pub fn tilted(a: f64, b: f64) -> [Tri; 4] {
    let s = a + b;
    [
        [[0.0, 0.0], [a, 0.0], [0.0, b]],
        [[s, 0.0], [s, a], [a, 0.0]],
        [[s, s], [b, s], [s, a]],
        [[0.0, s], [0.0, b], [b, s]],
    ]
}

/// How far each triangle slides to leave an a² and a b² square uncovered:
/// two triangles fill each a × b rectangle. Only translations are needed.
pub fn shifts(a: f64, b: f64) -> [[f64; 2]; 4] {
    [[0.0, a], [0.0, 0.0], [-b, 0.0], [a, -b]]
}

fn smoothstep(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// The triangles at time `t` from 0 (around c²) to 1 (around a² and b²). The
/// three that move take turns, each over a third of the time.
pub fn triangles(a: f64, b: f64, t: f64) -> [Tri; 4] {
    let mut tris = tilted(a, b);
    let moves = shifts(a, b);
    // Triangle 1 never moves. Triangle 3 goes first, clearing the corner that
    // triangle 0 slides into; triangle 2 joins it last.
    for (turn, i) in [3usize, 0, 2].into_iter().enumerate() {
        let u = smoothstep(t * 3.0 - turn as f64);
        for p in &mut tris[i] {
            p[0] += moves[i][0] * u;
            p[1] += moves[i][1] * u;
        }
    }
    tris
}

pub fn area(t: &Tri) -> f64 {
    let [p, q, r] = *t;
    0.5 * ((q[0] - p[0]) * (r[1] - p[1]) - (r[0] - p[0]) * (q[1] - p[1])).abs()
}

/// The altitude from the right angle splits the hypotenuse c into p (next to
/// leg a) and q (next to leg b), at height h: p = a²/c, q = b²/c, h = ab/c.
pub fn altitude(a: f64, b: f64) -> (f64, f64, f64, f64) {
    let c = a.hypot(b);
    (c, a * a / c, b * b / c, a * b / c)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Rearrange,
    Similar,
}

pub struct Pythagoras {
    a: f64,
    b: f64,
    t: f64,
    phase: f64,
    playing: bool,
    mode: Mode,
}

impl Default for Pythagoras {
    fn default() -> Self {
        Self {
            a: 3.0,
            b: 4.0,
            t: 0.0,
            phase: 0.0,
            playing: true,
            mode: Mode::Rearrange,
        }
    }
}

/// Seconds in one back-and-forth of the rearrangement.
const CYCLE: f64 = 10.0;

/// Time in the cycle → t: hold at 0, move to 1, hold, move back.
fn cycle(phase: f64) -> f64 {
    let u = phase.rem_euclid(CYCLE) / CYCLE;
    match u {
        u if u < 0.1 => 0.0,
        u if u < 0.45 => (u - 0.1) / 0.35,
        u if u < 0.6 => 1.0,
        u => 1.0 - (u - 0.6) / 0.4,
    }
}

impl Pythagoras {
    fn paint_rearrange(&self, painter: &egui::Painter, rect: Rect, palette: &Palette) {
        let s = self.a + self.b;
        let side = (rect.width() - 60.0).min(rect.height() - 70.0).max(40.0) as f64;
        let k = side / s;
        let origin = pos2(
            rect.center().x - (side / 2.0) as f32,
            rect.center().y + (side / 2.0) as f32 + 12.0,
        );
        let at = |p: [f64; 2]| origin + vec2((p[0] * k) as f32, -(p[1] * k) as f32);

        // What the triangles leave uncovered shows the square(s) underneath.
        let square = Rect::from_two_pos(at([0.0, 0.0]), at([s, s]));
        painter.rect_filled(square, 0.0, palette.y.gamma_multiply(0.22));
        painter.rect_stroke(
            square,
            0.0,
            Stroke::new(1.5, palette.ink),
            egui::StrokeKind::Middle,
        );
        for tri in triangles(self.a, self.b, self.t) {
            let pts: Vec<Pos2> = tri.iter().map(|&p| at(p)).collect();
            painter.add(Shape::convex_polygon(
                pts.clone(),
                palette.card,
                Stroke::new(1.5, palette.trace),
            ));
            // The legs in their colors: a in x, b in y.
            painter.line_segment([pts[0], pts[1]], Stroke::new(2.5, palette.x));
            painter.line_segment([pts[0], pts[2]], Stroke::new(2.5, palette.y));
        }
        let font = FontId::proportional(22.0);
        let (a, b) = (self.a, self.b);
        let fade = |w: f64| palette.ink.gamma_multiply(w.clamp(0.0, 1.0) as f32);
        painter.text(
            at([s / 2.0, s / 2.0]),
            Align2::CENTER_CENTER,
            "c²",
            font.clone(),
            fade(1.0 - 3.0 * self.t),
        );
        painter.text(
            at([a / 2.0, a / 2.0]),
            Align2::CENTER_CENTER,
            "a²",
            font.clone(),
            fade(3.0 * self.t - 2.0),
        );
        painter.text(
            at([a + b / 2.0, a + b / 2.0]),
            Align2::CENTER_CENTER,
            "b²",
            font,
            fade(3.0 * self.t - 2.0),
        );
        painter.text(
            rect.left_top() + vec2(12.0, 10.0),
            Align2::LEFT_TOP,
            format!(
                "(a + b)² = 4 triangles + {}",
                if self.t < 0.5 { "c²" } else { "a² + b²" }
            ),
            FontId::proportional(14.0),
            palette.ink,
        );
    }

    fn paint_similar(&self, painter: &egui::Painter, rect: Rect, palette: &Palette) {
        let (c, p, q, h) = altitude(self.a, self.b);
        let k =
            ((rect.width() - 80.0) as f64 / c).min((rect.height() - 110.0) as f64 / h.max(1e-9));
        let base = pos2(rect.center().x - (c * k / 2.0) as f32, rect.bottom() - 50.0);
        let at = |x: f64, y: f64| base + vec2((x * k) as f32, -(y * k) as f32);
        let (pa, pb, pc, ph) = (at(0.0, 0.0), at(c, 0.0), at(p, h), at(p, 0.0));
        let shade = |pts: Vec<Pos2>, color: Color32| {
            painter.add(Shape::convex_polygon(
                pts,
                color.gamma_multiply(0.25),
                Stroke::NONE,
            ));
        };
        shade(vec![pa, ph, pc], palette.x);
        shade(vec![ph, pb, pc], palette.y);
        painter.add(Shape::closed_line(
            vec![pa, pb, pc],
            Stroke::new(2.0, palette.trace),
        ));
        painter.extend(Shape::dashed_line(
            &[pc, ph],
            Stroke::new(1.5, palette.muted),
            5.0,
            4.0,
        ));
        let label = |pos: Pos2, text: String, color: Color32| {
            painter.text(
                pos,
                Align2::CENTER_CENTER,
                text,
                FontId::proportional(15.0),
                color,
            );
        };
        let mid = |u: Pos2, v: Pos2| u + (v - u) * 0.5;
        label(
            mid(pa, pc) + vec2(-14.0, -6.0),
            format!("a = {:.2}", self.a),
            palette.x,
        );
        label(
            mid(pb, pc) + vec2(16.0, -6.0),
            format!("b = {:.2}", self.b),
            palette.y,
        );
        label(
            mid(pa, ph) + vec2(0.0, 14.0),
            format!("p = {p:.2}"),
            palette.x,
        );
        label(
            mid(ph, pb) + vec2(0.0, 14.0),
            format!("q = {q:.2}"),
            palette.y,
        );
        label(
            mid(pa, pb) + vec2(0.0, 32.0),
            format!("c = p + q = {c:.2}"),
            palette.trace,
        );
        painter.text(
            rect.left_top() + vec2(12.0, 10.0),
            Align2::LEFT_TOP,
            format!(
                "a² = c·p = {:.2}    b² = c·q = {:.2}    a² + b² = c·(p + q) = c² = {:.2}",
                c * p,
                c * q,
                c * c
            ),
            FontId::proportional(13.0),
            palette.ink,
        );
    }
}

impl Formula for Pythagoras {
    fn id(&self) -> &'static str {
        "pythagorean-proof"
    }

    fn name(&self) -> &'static str {
        "Pythagorean theorem"
    }

    fn category(&self) -> &'static str {
        crate::formulas::PROOFS
    }

    fn summary(&self) -> &'static str {
        "Why a² + b² = c²: slide four copies of the triangle around an (a + b)² square, \
         or drop the altitude and find two smaller copies of the triangle inside it."
    }

    fn notes(&self) -> &'static str {
        "Every distance on a screen or a map, the length of a vector, the error of a \
         prediction and the magnitude of a complex number is the theorem in disguise."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("a")
                .tone(Tone::X)
                .sup("2")
                .tone(Tone::X)
                .t(" + ")
                .t("b")
                .tone(Tone::Y)
                .sup("2")
                .tone(Tone::Y)
                .t(" = ")
                .t("c")
                .tone(Tone::Trace)
                .sup("2")
                .tone(Tone::Trace),
        ]
    }

    fn params(&self) -> Vec<(String, f64)> {
        vec![
            ("a".into(), self.a),
            ("b".into(), self.b),
            ("c".into(), self.a.hypot(self.b)),
        ]
    }

    fn substitutions(&self) -> Vec<Substitution> {
        vec![Substitution::new(
            MathText::new().t("c").tone(Tone::Trace),
            "sqrt(a^2 + b^2)",
            &[("a", self.a), ("b", self.b)],
        )]
    }

    fn derivations(&self) -> Vec<Derivation> {
        let m = MathText::new;
        vec![
            Derivation::new(
                "Rearranging four triangles",
                vec![
                    Step::new(
                        m().t("(a + b)").sup("2"),
                        "Build a square with side a + b and place four copies of the triangle in its corners.",
                    ),
                    Step::new(
                        m().t("(a + b)")
                            .sup("2")
                            .t(" = ")
                            .t("c")
                            .tone(Tone::Trace)
                            .sup("2")
                            .t(" + 4 · ab / 2"),
                        "Their hypotenuses enclose a tilted square of side c: the corners of it are right angles because the two acute angles of the triangle add up to 90°.",
                    ),
                    Step::new(
                        m().t("(a + b)")
                            .sup("2")
                            .t(" = ")
                            .t("a")
                            .tone(Tone::X)
                            .sup("2")
                            .t(" + ")
                            .t("b")
                            .tone(Tone::Y)
                            .sup("2")
                            .t(" + 4 · ab / 2"),
                        "Slide the triangles into pairs that fill two a × b rectangles. What is left uncovered is a square of side a and one of side b.",
                    ),
                    Step::new(
                        m().t("c")
                            .tone(Tone::Trace)
                            .sup("2")
                            .t(" = ")
                            .t("a")
                            .tone(Tone::X)
                            .sup("2")
                            .t(" + ")
                            .t("b")
                            .tone(Tone::Y)
                            .sup("2"),
                        "The big square and the four triangles are the same both times, so the uncovered areas are equal.",
                    ),
                ],
            ),
            Derivation::new(
                "Similar triangles",
                vec![
                    Step::new(
                        m().t("c = p + q"),
                        "Drop the altitude from the right angle to the hypotenuse. It splits c into p, next to a, and q, next to b.",
                    ),
                    Step::new(
                        m().t("p / a = a / c"),
                        "The triangle with sides p and a has a right angle and shares the angle at the end of c with the whole triangle, so it is a scaled copy: corresponding sides are in proportion.",
                    ),
                    Step::new(
                        m().t("a")
                            .tone(Tone::X)
                            .sup("2")
                            .tone(Tone::X)
                            .t(" = c · p"),
                        "Cross-multiply.",
                    ),
                    Step::new(
                        m().t("b")
                            .tone(Tone::Y)
                            .sup("2")
                            .tone(Tone::Y)
                            .t(" = c · q"),
                        "The same for the triangle with sides q and b.",
                    ),
                    Step::new(
                        m().t("a")
                            .tone(Tone::X)
                            .sup("2")
                            .t(" + ")
                            .t("b")
                            .tone(Tone::Y)
                            .sup("2")
                            .t(" = c (p + q) = ")
                            .t("c")
                            .tone(Tone::Trace)
                            .sup("2"),
                        "Add the two and use c = p + q.",
                    ),
                ],
            ),
        ]
    }

    fn is_animating(&self) -> bool {
        self.playing && self.mode == Mode::Rearrange
    }

    fn min_size(&self) -> egui::Vec2 {
        egui::vec2(420.0, 340.0)
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.is_animating() {
            self.phase += dt as f64;
            self.t = cycle(self.phase);
        }
        let painter = painter.with_clip_rect(rect);
        match self.mode {
            Mode::Rearrange => self.paint_rearrange(&painter, rect, palette),
            Mode::Similar => self.paint_similar(&painter, rect, palette),
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.mode, Mode::Rearrange, "Rearrangement");
            ui.selectable_value(&mut self.mode, Mode::Similar, "Similar triangles");
        });
        ui.add_space(6.0);
        egui::Grid::new("pythagoras-controls")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("a");
                ui.add(
                    egui::Slider::new(&mut self.a, 0.5..=6.0).clamping(egui::SliderClamping::Edits),
                );
                ui.end_row();
                ui.label("b");
                ui.add(
                    egui::Slider::new(&mut self.b, 0.5..=6.0).clamping(egui::SliderClamping::Edits),
                );
                ui.end_row();
                if self.mode == Mode::Rearrange {
                    ui.label("t");
                    if ui
                        .add(
                            egui::Slider::new(&mut self.t, 0.0..=1.0)
                                .clamping(egui::SliderClamping::Edits),
                        )
                        .changed()
                    {
                        self.playing = false;
                    }
                    ui.end_row();
                }
            });
        if self.mode == Mode::Rearrange {
            ui.add_space(6.0);
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
            {
                self.playing = !self.playing;
                // Carry on from the slider's position: find it on the rising half of the cycle.
                self.phase = CYCLE * (0.1 + 0.35 * self.t);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dist(p: [f64; 2], q: [f64; 2]) -> f64 {
        (p[0] - q[0]).hypot(p[1] - q[1])
    }

    #[test]
    fn triangles_move_rigidly() {
        let (a, b) = (3.0, 4.0);
        let start = tilted(a, b);
        for t in [0.0, 0.2, 0.25, 0.5, 0.75, 0.9, 1.0] {
            for (tri, orig) in triangles(a, b, t).iter().zip(&start) {
                assert!((area(tri) - a * b / 2.0).abs() < 1e-9);
                assert!((dist(tri[0], tri[1]) - a).abs() < 1e-9, "leg a");
                assert!((dist(tri[0], tri[2]) - b).abs() < 1e-9, "leg b");
                assert!((dist(tri[1], tri[2]) - dist(orig[1], orig[2])).abs() < 1e-9);
                for p in tri {
                    assert!((-1e-9..=a + b + 1e-9).contains(&p[0]));
                    assert!((-1e-9..=a + b + 1e-9).contains(&p[1]));
                }
            }
        }
    }

    #[test]
    fn the_start_encloses_the_tilted_c_square() {
        let (a, b) = (2.0_f64, 5.0);
        let c = a.hypot(b);
        let corners = [[a, 0.0], [a + b, a], [b, a + b], [0.0, b]];
        for w in 0..4 {
            assert!((dist(corners[w], corners[(w + 1) % 4]) - c).abs() < 1e-9);
        }
        for tri in tilted(a, b) {
            // Each hypotenuse is a side of the tilted square.
            assert!(corners.contains(&tri[1]) && corners.contains(&tri[2]));
        }
    }

    #[test]
    fn the_end_fills_two_rectangles() {
        let (a, b) = (2.0, 5.0);
        let s = a + b;
        let inside = |tri: &Tri, x0: f64, y0: f64, x1: f64, y1: f64| {
            tri.iter().all(|p| {
                (x0 - 1e-9..=x1 + 1e-9).contains(&p[0]) && (y0 - 1e-9..=y1 + 1e-9).contains(&p[1])
            })
        };
        let end = triangles(a, b, 1.0);
        let right = end.iter().filter(|t| inside(t, a, 0.0, s, a)).count();
        let top = end.iter().filter(|t| inside(t, 0.0, a, a, s)).count();
        assert_eq!((right, top), (2, 2));
        // So what is uncovered is the a² square at the origin and the b² square opposite.
        assert!((s * s - 4.0 * a * b / 2.0 - (a * a + b * b)).abs() < 1e-9);
    }

    #[test]
    fn the_altitude_splits_c_into_similar_parts() {
        let (c, p, q, h) = altitude(3.0, 4.0);
        assert!((c - 5.0).abs() < 1e-12);
        assert!((p + q - c).abs() < 1e-12);
        assert!((c * p - 9.0).abs() < 1e-12 && (c * q - 16.0).abs() < 1e-12);
        assert!(
            (p.hypot(h) - 3.0).abs() < 1e-12,
            "the apex is a away from one end"
        );
        assert!(
            (q.hypot(h) - 4.0).abs() < 1e-12,
            "and b away from the other"
        );
    }

    #[test]
    fn the_cycle_holds_at_both_ends() {
        assert_eq!(cycle(0.0), 0.0);
        assert_eq!(cycle(0.5 * CYCLE), 1.0);
        assert!((cycle(0.275 * CYCLE) - 0.5).abs() < 1e-9);
        assert!((0.0..=1.0).contains(&cycle(0.8 * CYCLE)));
    }
}
