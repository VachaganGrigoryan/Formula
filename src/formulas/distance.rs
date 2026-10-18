//! Euclidean distance: the straight-line length between two points, used for
//! nearest-point tooltips and for K-means clustering.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, pos2, vec2};

use crate::derive::{Derivation, Step, Substitution};
use crate::draw::lerp_color;
use crate::formula::{Formula, MathText, Palette, Tone};
use crate::rng::Rng;

pub type Point = [f32; 2];

/// `d = √((x₂ − x₁)² + (y₂ − y₁)²)`
pub fn distance(a: Point, b: Point) -> f32 {
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
}

/// Index of the point closest to `at`, and its distance.
pub fn nearest(points: &[Point], at: Point) -> Option<(usize, f32)> {
    points
        .iter()
        .enumerate()
        .map(|(i, &p)| (i, distance(p, at)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// One round of K-means: assign every point to its nearest centroid, then move
/// each centroid to the mean of its points. Returns the assignment and whether
/// any centroid moved.
pub fn kmeans_step(points: &[Point], centroids: &mut [Point]) -> (Vec<usize>, bool) {
    let assign: Vec<usize> = points
        .iter()
        .map(|&p| nearest(centroids, p).map_or(0, |(i, _)| i))
        .collect();
    let mut moved = false;
    for (c, centroid) in centroids.iter_mut().enumerate() {
        let members: Vec<Point> = points
            .iter()
            .zip(&assign)
            .filter(|&(_, &a)| a == c)
            .map(|(&p, _)| p)
            .collect();
        // An empty cluster keeps its centroid where it is.
        if members.is_empty() {
            continue;
        }
        let n = members.len() as f32;
        let mean = [
            members.iter().map(|p| p[0]).sum::<f32>() / n,
            members.iter().map(|p| p[1]).sum::<f32>() / n,
        ];
        if distance(mean, *centroid) > 1e-4 {
            moved = true;
        }
        *centroid = mean;
    }
    (assign, moved)
}

/// Data space is `0..W` by `0..H`, drawn with equal scale on both axes so that
/// distances on screen match distances in the data.
const W: f32 = 100.0;
const H: f32 = 60.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Nearest,
    KMeans,
}

pub struct Distance {
    mode: Mode,
    points: Vec<Point>,
    seed: u64,
    /// Where the nearest-point query is made (follows the mouse).
    probe: Point,
    k: usize,
    centroids: Vec<Point>,
    assign: Vec<usize>,
    converged: bool,
    auto: bool,
    /// Seconds until the next automatic K-means step.
    timer: f32,
    iteration: usize,
}

impl Default for Distance {
    fn default() -> Self {
        let mut s = Self {
            mode: Mode::Nearest,
            points: Vec::new(),
            seed: 2,
            probe: [62.0, 34.0],
            k: 4,
            centroids: Vec::new(),
            assign: Vec::new(),
            converged: false,
            auto: true,
            timer: 0.0,
            iteration: 0,
        };
        s.regenerate();
        s
    }
}

impl Distance {
    fn regenerate(&mut self) {
        let mut rng = Rng::new(self.seed);
        // One cluster per quadrant, jittered, so the groups stay distinguishable.
        let blobs: Vec<Point> = [[25.0, 18.0], [75.0, 18.0], [25.0, 42.0], [75.0, 42.0]]
            .iter()
            .map(|c| [c[0] + rng.range(-8.0, 8.0), c[1] + rng.range(-5.0, 5.0)])
            .collect();
        self.points = (0..90)
            .map(|i| {
                let c = blobs[i % blobs.len()];
                // The sum of three uniforms is a cheap bell curve.
                let mut g = || (rng.f32() + rng.f32() + rng.f32() - 1.5) * 9.0;
                [
                    (c[0] + g()).clamp(1.0, W - 1.0),
                    (c[1] + g()).clamp(1.0, H - 1.0),
                ]
            })
            .collect();
        self.restart_kmeans();
    }

    fn restart_kmeans(&mut self) {
        let mut rng = Rng::new(self.seed.wrapping_mul(7919).wrapping_add(self.k as u64));
        self.seed = self.seed.wrapping_add(100);
        self.centroids = (0..self.k)
            .map(|_| [rng.range(5.0, W - 5.0), rng.range(5.0, H - 5.0)])
            .collect();
        self.assign = vec![0; self.points.len()];
        for (i, &p) in self.points.iter().enumerate() {
            self.assign[i] = nearest(&self.centroids, p).map_or(0, |(c, _)| c);
        }
        self.converged = false;
        self.iteration = 0;
        self.timer = 0.8;
    }

    fn step(&mut self) {
        let (assign, moved) = kmeans_step(&self.points, &mut self.centroids);
        // Re-assign against the moved centroids so colors match what is drawn.
        self.assign = self
            .points
            .iter()
            .map(|&p| nearest(&self.centroids, p).map_or(0, |(c, _)| c))
            .collect();
        self.converged = !moved && assign == self.assign;
        self.iteration += 1;
    }
}

/// The plot area inside the stage and the scale from data units to points.
fn plot(rect: Rect) -> (Rect, f32) {
    let inner = rect.shrink2(vec2(24.0, 22.0)).with_max_y(rect.max.y - 44.0);
    let scale = (inner.width() / W).min(inner.height() / H);
    (
        Rect::from_center_size(inner.center(), vec2(W, H) * scale),
        scale,
    )
}

fn to_screen(plot: Rect, scale: f32, [x, y]: Point) -> Pos2 {
    pos2(plot.min.x + x * scale, plot.max.y - y * scale)
}

fn to_data(plot: Rect, scale: f32, p: Pos2) -> Point {
    [(p.x - plot.min.x) / scale, (plot.max.y - p.y) / scale]
}

fn cluster_colors(p: &Palette) -> [Color32; 6] {
    [
        p.trace,
        p.x,
        p.y,
        p.ink,
        lerp_color(p.trace, p.y, 0.5),
        p.muted,
    ]
}

impl Formula for Distance {
    fn derivations(&self) -> Vec<Derivation> {
        let m = MathText::new;
        vec![Derivation::new(
            "From Pythagoras",
            vec![
                Step::new(
                    m().t("Δx")
                        .tone(Tone::X)
                        .t(" = x")
                        .sub("2")
                        .t(" − x")
                        .sub("1"),
                    "The horizontal leg of the right triangle between the two points.",
                ),
                Step::new(
                    m().t("Δy")
                        .tone(Tone::Y)
                        .t(" = y")
                        .sub("2")
                        .t(" − y")
                        .sub("1"),
                    "The vertical leg.",
                ),
                Step::new(
                    m().t("a").sup("2").t(" + b").sup("2").t(" = c").sup("2"),
                    "Pythagoras: the squares on the legs of a right triangle add up to the square on the hypotenuse.",
                ),
                Step::new(
                    m().t("d")
                        .tone(Tone::Trace)
                        .sup("2")
                        .t(" = Δx")
                        .sup("2")
                        .t(" + Δy")
                        .sup("2"),
                    "Here the legs are Δx and Δy and the hypotenuse is the distance d.",
                ),
                Step::new(
                    m().t("d")
                        .tone(Tone::Trace)
                        .t(" = √(Δx")
                        .sup("2")
                        .t(" + Δy")
                        .sup("2")
                        .t(")"),
                    "Take the positive square root: a distance is never negative.",
                ),
                Step::new(
                    m().t("d")
                        .tone(Tone::Trace)
                        .t(" = √((x")
                        .sub("2")
                        .t(" − x")
                        .sub("1")
                        .t(")")
                        .sup("2")
                        .t(" + (y")
                        .sub("2")
                        .t(" − y")
                        .sub("1")
                        .t(")")
                        .sup("2")
                        .t(")"),
                    "Write the legs out in coordinates. In n dimensions the sum just gets more terms.",
                ),
            ],
        )]
    }

    fn substitutions(&self) -> Vec<Substitution> {
        let Some((i, _)) = nearest(&self.points, self.probe) else {
            return Vec::new();
        };
        let ([x1, y1], [x2, y2]) = (self.probe, self.points[i]);
        vec![Substitution::new(
            MathText::new().t("d").tone(Tone::Trace),
            "sqrt((x_2 - x_1)^2 + (y_2 - y_1)^2)",
            &[
                ("x_1", x1 as f64),
                ("y_1", y1 as f64),
                ("x_2", x2 as f64),
                ("y_2", y2 as f64),
            ],
        )]
    }

    fn id(&self) -> &'static str {
        "euclidean-distance"
    }

    fn name(&self) -> &'static str {
        "Euclidean distance"
    }

    fn category(&self) -> &'static str {
        super::GEOMETRY
    }

    fn summary(&self) -> &'static str {
        "Pythagoras in two dimensions. Move the mouse to find the closest point, as a \
         chart tooltip does, or switch to K-means and watch clusters form around centroids."
    }

    fn notes(&self) -> &'static str {
        "Scatter plot tooltips snap to the data point nearest the cursor; K-means \
         clustering assigns each point to its nearest centroid; network layouts check \
         node distances to detect overlap. Comparing d² instead of d gives the same \
         answer without the square root."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("d")
                .tone(Tone::Trace)
                .t(" = √( (x")
                .sub("2")
                .t(" − x")
                .sub("1")
                .t(")")
                .sup("2")
                .tone(Tone::X)
                .t(" + (y")
                .sub("2")
                .t(" − y")
                .sub("1")
                .t(")")
                .sup("2")
                .tone(Tone::Y)
                .t(" )"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::hover()
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.68).max(360.0)
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        if let Some(at) = response.hover_pos() {
            let (plot, scale) = plot(rect);
            if plot.contains(at) {
                self.probe = to_data(plot, scale, at);
            }
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.mode == Mode::KMeans && self.auto && !self.converged {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.step();
                self.timer = 0.7;
            }
        }

        let (plot, scale) = plot(rect);
        let screen = |p: Point| to_screen(plot, scale, p);
        painter.rect_stroke(
            plot,
            6.0,
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Outside,
        );
        for gx in (10..100).step_by(10) {
            let x = screen([gx as f32, 0.0]).x;
            painter.line_segment(
                [pos2(x, plot.top()), pos2(x, plot.bottom())],
                Stroke::new(1.0, palette.rule.gamma_multiply(0.5)),
            );
        }
        for gy in (10..60).step_by(10) {
            let y = screen([0.0, gy as f32]).y;
            painter.line_segment(
                [pos2(plot.left(), y), pos2(plot.right(), y)],
                Stroke::new(1.0, palette.rule.gamma_multiply(0.5)),
            );
        }
        let font = FontId::monospace(12.5);
        let caption = |text: String, color: Color32| {
            painter.text(
                rect.left_bottom() + vec2(16.0, -14.0),
                Align2::LEFT_BOTTOM,
                text,
                font.clone(),
                color,
            );
        };

        match self.mode {
            Mode::Nearest => {
                for &p in &self.points {
                    painter.circle_filled(screen(p), 3.5, palette.muted);
                }
                let Some((i, d)) = nearest(&self.points, self.probe) else {
                    return;
                };
                let a = self.probe;
                let b = self.points[i];
                let (pa, pb) = (screen(a), screen(b));
                let corner = screen([b[0], a[1]]);
                // Nothing lies inside this circle: that is what "nearest" means.
                painter.circle(
                    pa,
                    d * scale,
                    palette.fill,
                    Stroke::new(1.0, palette.trace.gamma_multiply(0.5)),
                );
                painter.line_segment([pa, corner], Stroke::new(2.0, palette.x));
                painter.line_segment([corner, pb], Stroke::new(2.0, palette.y));
                painter.line_segment([pa, pb], Stroke::new(2.5, palette.trace));
                painter.circle(pb, 6.5, palette.trace, Stroke::new(2.0, palette.card));
                painter.circle(pa, 4.0, palette.ink, Stroke::NONE);
                let small = FontId::proportional(12.0);
                painter.text(
                    pa.lerp(corner, 0.5) + vec2(0.0, 4.0),
                    Align2::CENTER_TOP,
                    format!("Δx {:.1}", (b[0] - a[0]).abs()),
                    small.clone(),
                    palette.x,
                );
                painter.text(
                    corner.lerp(pb, 0.5) + vec2(5.0, 0.0),
                    Align2::LEFT_CENTER,
                    format!("Δy {:.1}", (b[1] - a[1]).abs()),
                    small,
                    palette.y,
                );
                // A tooltip, as a scatter plot would show it.
                let tip = format!("point #{i}\n({:.1}, {:.1})\nd = {d:.2}", b[0], b[1]);
                let galley = painter.layout_no_wrap(tip, font.clone(), palette.ink);
                let at = pb + vec2(12.0, -12.0 - galley.size().y);
                let frame = Rect::from_min_size(at, galley.size()).expand(6.0);
                let frame = frame.translate(vec2(
                    (rect.right() - 8.0 - frame.right()).min(0.0),
                    (rect.top() + 8.0 - frame.top()).max(0.0),
                ));
                painter.rect(
                    frame,
                    6.0,
                    palette.card,
                    Stroke::new(1.0, palette.rule),
                    egui::StrokeKind::Inside,
                );
                painter.galley(frame.min + vec2(6.0, 6.0), galley, palette.ink);
                caption(
                    format!(
                        "checked {} distances, nearest d = {d:.2}",
                        self.points.len()
                    ),
                    palette.muted,
                );
            }
            Mode::KMeans => {
                let colors = cluster_colors(palette);
                for (&p, &c) in self.points.iter().zip(&self.assign) {
                    let col = colors[c % colors.len()];
                    if let Some(&centroid) = self.centroids.get(c) {
                        painter.line_segment(
                            [screen(p), screen(centroid)],
                            Stroke::new(1.0, col.gamma_multiply(0.2)),
                        );
                    }
                    painter.circle_filled(screen(p), 3.8, col);
                }
                for (c, &centroid) in self.centroids.iter().enumerate() {
                    let at = screen(centroid);
                    let col = colors[c % colors.len()];
                    painter.circle(at, 9.0, palette.card, Stroke::new(3.0, col));
                    painter.line_segment(
                        [at - vec2(5.0, 0.0), at + vec2(5.0, 0.0)],
                        Stroke::new(2.0, col),
                    );
                    painter.line_segment(
                        [at - vec2(0.0, 5.0), at + vec2(0.0, 5.0)],
                        Stroke::new(2.0, col),
                    );
                }
                let status = if self.converged {
                    "converged"
                } else {
                    "running"
                };
                caption(
                    format!("iteration {}  ({status})", self.iteration),
                    palette.muted,
                );
            }
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.mode, Mode::Nearest, "Nearest point");
            ui.selectable_value(&mut self.mode, Mode::KMeans, "K-means");
        });
        ui.add_space(6.0);
        if self.mode == Mode::KMeans {
            egui::Grid::new("kmeans-controls")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Clusters k");
                    if ui.add(egui::Slider::new(&mut self.k, 2..=6)).changed() {
                        self.restart_kmeans();
                    }
                    ui.end_row();
                });
            ui.add_space(6.0);
        }
        ui.horizontal_wrapped(|ui| {
            if self.mode == Mode::KMeans {
                if ui.button(if self.auto { "Pause" } else { "Run" }).clicked() {
                    self.auto = !self.auto;
                }
                if ui
                    .add_enabled(!self.converged, egui::Button::new("Step"))
                    .clicked()
                {
                    self.step();
                }
                if ui.button("New centroids").clicked() {
                    self.restart_kmeans();
                }
            }
            if ui.button("New data").clicked() {
                self.seed = self.seed.wrapping_add(1);
                self.regenerate();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_four_five() {
        assert_eq!(distance([0.0, 0.0], [3.0, 4.0]), 5.0);
        assert_eq!(distance([1.0, 1.0], [1.0, 1.0]), 0.0);
        assert_eq!(distance([3.0, 4.0], [0.0, 0.0]), 5.0);
    }

    #[test]
    fn nearest_picks_closest() {
        let pts = [[0.0, 0.0], [10.0, 0.0], [4.0, 4.0]];
        assert_eq!(nearest(&pts, [5.0, 3.0]).map(|n| n.0), Some(2));
        assert_eq!(nearest(&[], [0.0, 0.0]), None);
    }

    #[test]
    fn kmeans_finds_two_obvious_clusters() {
        let pts = [
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [10.0, 10.0],
            [11.0, 10.0],
            [10.0, 11.0],
        ];
        let mut c = [[0.0, 5.0], [5.0, 0.0]];
        let mut assign = Vec::new();
        for _ in 0..10 {
            let (a, moved) = kmeans_step(&pts, &mut c);
            assign = a;
            if !moved {
                break;
            }
        }
        assert_eq!(assign[0], assign[1]);
        assert_eq!(assign[0], assign[2]);
        assert_eq!(assign[3], assign[4]);
        assert_ne!(assign[0], assign[3]);
        let mut centers = c.to_vec();
        centers.sort_by(|a, b| a[0].total_cmp(&b[0]));
        assert!(distance(centers[0], [1.0 / 3.0, 1.0 / 3.0]) < 1e-5);
        assert!(distance(centers[1], [31.0 / 3.0, 31.0 / 3.0]) < 1e-5);
    }
}
