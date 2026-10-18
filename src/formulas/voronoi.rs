//! Voronoi diagrams: every point of the plane belongs to its nearest seed.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, Vec2, pos2, vec2};

use crate::draw::{handle, lerp_color, pick};
use crate::formula::{Formula, MathText, Palette, Tone};
use crate::formulas::luminance::hsv_to_srgb;
use crate::rng::Rng;

/// A corner of a cell, and which seed's bisector the edge leaving it lies on
/// (`None` for the outer border).
pub type Corner = (Pos2, Option<usize>);

/// The Voronoi cell of `seeds[i]` inside `bounds`: the bounding box clipped by
/// the half-plane "closer to i than to j" for every other seed j. Each
/// bisector is pushed `gap / 2` toward i, which leaves an even gap between
/// neighboring cells.
pub fn cell(seeds: &[Pos2], i: usize, bounds: Rect, gap: f32) -> Vec<Corner> {
    let b = bounds.shrink(gap / 2.0);
    let mut poly: Vec<Corner> = vec![
        (b.left_top(), None),
        (b.right_top(), None),
        (b.right_bottom(), None),
        (b.left_bottom(), None),
    ];
    let p = seeds[i];
    for (j, &q) in seeds.iter().enumerate() {
        if j == i || q == p || poly.is_empty() {
            continue;
        }
        // Keep x with (x − m) · n ≤ −gap/2, where m is the midpoint and n the
        // unit vector from p toward q.
        let n = (q - p).normalized();
        let m = p + (q - p) * 0.5;
        let side = |x: Pos2| (x - m).dot(n) + gap / 2.0;
        poly = clip(&poly, side, j);
    }
    poly
}

/// Sutherland–Hodgman against one half-plane (`side(x) ≤ 0` is kept); the
/// new edge along the clip line is labeled `label`.
fn clip(poly: &[Corner], side: impl Fn(Pos2) -> f32, label: usize) -> Vec<Corner> {
    let mut out = Vec::with_capacity(poly.len() + 1);
    for k in 0..poly.len() {
        let (a, edge) = poly[k];
        let (b, _) = poly[(k + 1) % poly.len()];
        let (sa, sb) = (side(a), side(b));
        if sa <= 0.0 {
            out.push((a, edge));
        }
        if (sa <= 0.0) != (sb <= 0.0) {
            let t = sa / (sa - sb);
            let x = a + (b - a) * t;
            // Leaving the kept side, the next edge runs along the clip line;
            // re-entering, it continues the original edge.
            out.push((x, if sa <= 0.0 { Some(label) } else { edge }));
        }
    }
    out
}

pub fn area(poly: &[Corner]) -> f32 {
    let n = poly.len();
    (0..n)
        .map(|k| {
            let (a, b) = (poly[k].0, poly[(k + 1) % n].0);
            a.x * b.y - b.x * a.y
        })
        .sum::<f32>()
        .abs()
        / 2.0
}

/// The center of mass of a polygon (shoelace formula).
pub fn centroid(poly: &[Corner]) -> Option<Pos2> {
    let n = poly.len();
    let (mut a2, mut c) = (0.0, Vec2::ZERO);
    for k in 0..n {
        let (p, q) = (poly[k].0, poly[(k + 1) % n].0);
        let cross = p.x * q.y - q.x * p.y;
        a2 += cross;
        c += (p.to_vec2() + q.to_vec2()) * cross;
    }
    (a2.abs() > 1e-6).then(|| (c / (3.0 * a2)).to_pos2())
}

/// Seeds whose cells share an edge with this one: the Delaunay neighbors.
pub fn neighbors(poly: &[Corner]) -> impl Iterator<Item = usize> + '_ {
    poly.iter().filter_map(|&(_, n)| n)
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Style {
    Colorful,
    Giraffe,
    Mud,
    Wings,
}

impl Style {
    const ALL: [Style; 4] = [Style::Colorful, Style::Giraffe, Style::Mud, Style::Wings];

    fn label(self) -> &'static str {
        match self {
            Style::Colorful => "Colorful",
            Style::Giraffe => "Giraffe",
            Style::Mud => "Cracked mud",
            Style::Wings => "Dragonfly wing",
        }
    }

    /// The gap between cells, in points.
    fn gap(self) -> f32 {
        match self {
            Style::Colorful => 0.0,
            Style::Giraffe => 7.0,
            Style::Mud => 4.0,
            Style::Wings => 0.0,
        }
    }
}

struct Seed {
    /// In the unit square, so the diagram survives resizing.
    at: Pos2,
    vel: Vec2,
    hue: f32,
}

pub struct Voronoi {
    seeds: Vec<Seed>,
    count: usize,
    rng: Rng,
    style: Style,
    drift: bool,
    relax: bool,
    delaunay: bool,
    show_seeds: bool,
    dragging: Option<usize>,
    hover: Option<Pos2>,
}

impl Default for Voronoi {
    fn default() -> Self {
        let mut v = Self {
            seeds: Vec::new(),
            count: 24,
            rng: Rng::new(11),
            style: Style::Colorful,
            drift: true,
            relax: false,
            delaunay: false,
            show_seeds: true,
            dragging: None,
            hover: None,
        };
        v.scatter();
        v
    }
}

impl Voronoi {
    fn new_seed(&mut self, at: Pos2) -> Seed {
        let angle = self.rng.range(0.0, std::f32::consts::TAU);
        Seed {
            at,
            vel: vec2(angle.cos(), angle.sin()) * self.rng.range(0.015, 0.04),
            hue: self.rng.f32(),
        }
    }

    fn scatter(&mut self) {
        self.seeds = (0..self.count)
            .map(|_| {
                let at = pos2(self.rng.range(0.03, 0.97), self.rng.range(0.03, 0.97));
                self.new_seed(at)
            })
            .collect();
    }

    fn to_screen(rect: Rect, p: Pos2) -> Pos2 {
        rect.min + vec2(p.x * rect.width(), p.y * rect.height())
    }

    fn to_unit(rect: Rect, p: Pos2) -> Pos2 {
        let u = (p - rect.min) / rect.size();
        pos2(u.x.clamp(0.0, 1.0), u.y.clamp(0.0, 1.0))
    }

    fn screen_seeds(&self, rect: Rect) -> Vec<Pos2> {
        self.seeds
            .iter()
            .map(|s| Self::to_screen(rect, s.at))
            .collect()
    }

    fn fill(&self, i: usize, palette: &Palette, dark: bool) -> Color32 {
        let h = self.seeds[i].hue;
        let hsv = |h: f32, s: f32, v: f32| {
            let [r, g, b] = hsv_to_srgb(h.rem_euclid(1.0), s, v);
            Color32::from_rgb(r, g, b)
        };
        match self.style {
            Style::Colorful => {
                if dark {
                    hsv(h, 0.45, 0.55)
                } else {
                    hsv(h, 0.32, 0.93)
                }
            }
            // Patches in tan to chestnut.
            Style::Giraffe => hsv(0.07 + h * 0.04, 0.6 + h * 0.25, 0.75 - h * 0.3),
            Style::Mud => hsv(
                0.08,
                0.35 + h * 0.15,
                if dark {
                    0.35 + h * 0.15
                } else {
                    0.62 + h * 0.15
                },
            ),
            Style::Wings => lerp_color(
                palette.card,
                hsv(0.5 + h * 0.15, 0.5, 0.85),
                if dark { 0.18 } else { 0.25 } + h * 0.15,
            ),
        }
    }

    fn background(&self, palette: &Palette) -> Color32 {
        match self.style {
            Style::Giraffe => Color32::from_rgb(0xF1, 0xE3, 0xC4),
            Style::Mud => Color32::from_rgb(0x3A, 0x2A, 0x1C),
            _ => palette.card,
        }
    }

    fn edge(&self, palette: &Palette) -> Stroke {
        match self.style {
            Style::Colorful => Stroke::new(1.2, palette.card),
            Style::Wings => Stroke::new(1.6, palette.ink.gamma_multiply(0.7)),
            _ => Stroke::NONE,
        }
    }

    fn animate(&mut self, rect: Rect, dt: f32) {
        if self.drift {
            for (k, s) in self.seeds.iter_mut().enumerate() {
                if self.dragging == Some(k) {
                    continue;
                }
                s.at += s.vel * dt;
                for axis in 0..2 {
                    if !(0.01..=0.99).contains(&s.at[axis]) {
                        s.vel[axis] = -s.vel[axis];
                        s.at[axis] = s.at[axis].clamp(0.01, 0.99);
                    }
                }
            }
        }
        if self.relax {
            // Lloyd's algorithm: each seed slides toward its cell's centroid.
            let pts = self.screen_seeds(rect);
            for k in 0..self.seeds.len() {
                if self.dragging == Some(k) {
                    continue;
                }
                if let Some(c) = centroid(&cell(&pts, k, rect, 0.0)) {
                    let target = Self::to_unit(rect, c);
                    let s = &mut self.seeds[k];
                    s.at += (target - s.at) * (dt * 2.0).min(1.0);
                }
            }
        }
    }
}

/// What a sheet saves of a [`Voronoi`].
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct State {
    /// Seed positions in the unit square.
    seeds: Vec<[f32; 2]>,
    count: usize,
    style: Style,
    drift: bool,
    relax: bool,
    delaunay: bool,
    show_seeds: bool,
}

impl Voronoi {
    fn state(&self) -> State {
        State {
            seeds: self.seeds.iter().map(|s| [s.at.x, s.at.y]).collect(),
            count: self.count,
            style: self.style,
            drift: self.drift,
            relax: self.relax,
            delaunay: self.delaunay,
            show_seeds: self.show_seeds,
        }
    }

    fn apply(&mut self, s: State) {
        self.count = s.count.clamp(2, 150);
        let at: Vec<Pos2> = s
            .seeds
            .iter()
            .take(150)
            .map(|p| pos2(p[0].clamp(0.0, 1.0), p[1].clamp(0.0, 1.0)))
            .collect();
        if at.len() >= 2 {
            self.seeds = at.into_iter().map(|p| self.new_seed(p)).collect();
        } else {
            self.scatter();
        }
        self.style = s.style;
        self.drift = s.drift;
        self.relax = s.relax;
        self.delaunay = s.delaunay;
        self.show_seeds = s.show_seeds;
    }
}

impl Formula for Voronoi {
    crate::state_hooks!();

    fn id(&self) -> &'static str {
        "voronoi"
    }

    fn name(&self) -> &'static str {
        "Voronoi diagram"
    }

    fn category(&self) -> &'static str {
        super::GEOMETRY
    }

    fn summary(&self) -> &'static str {
        "Each seed claims every point that is closer to it than to any other seed. The \
         borders are perpendicular bisectors, so every cell is a convex polygon. Drag \
         seeds, click to add one, right-click to remove; hover to compare distances."
    }

    fn notes(&self) -> &'static str {
        "Nearest-facility maps (closest hospital, cell tower or school), nearest-neighbor \
         lookups, procedural textures like stone, scales and cracked earth, and mesh \
         generation. The same pattern grows in nature: giraffe skin, dried mud, insect \
         wings and soap foam."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("V(p")
                .tone(Tone::Trace)
                .sub("i")
                .tone(Tone::Trace)
                .t(")")
                .tone(Tone::Trace)
                .t(" = { x | d(x, p")
                .sub("i")
                .t(") ≤ d(x, p")
                .sub("j")
                .t(") for all j ≠ i }"),
            MathText::new()
                .t("edges lie on d(x, p")
                .sub("i")
                .t(") = d(x, p")
                .sub("j")
                .t("), the perpendicular bisector"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * if width < 520.0 { 1.0 } else { 0.62 }
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        self.hover = response.hover_pos();
        let pts = self.screen_seeds(rect);
        if response.drag_started()
            && let Some(at) = response.interact_pointer_pos()
        {
            self.dragging = pick(&pts, at, 16.0);
        }
        if response.dragged()
            && let (Some(k), Some(at)) = (self.dragging, response.interact_pointer_pos())
        {
            self.seeds[k].at = Self::to_unit(rect, at);
        }
        if response.drag_stopped() {
            self.dragging = None;
        }
        if let Some(at) = response.interact_pointer_pos() {
            if response.clicked() && pick(&pts, at, 16.0).is_none() && self.seeds.len() < 200 {
                let s = self.new_seed(Self::to_unit(rect, at));
                self.seeds.push(s);
            } else if response.secondary_clicked()
                && let Some(k) = pick(&pts, at, 16.0)
                && self.seeds.len() > 2
            {
                self.seeds.remove(k);
            }
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.animate(rect, dt.min(0.1));
        let dark = palette.paper.r() < 128;
        let pts = self.screen_seeds(rect);
        let gap = self.style.gap();
        painter.rect_filled(rect, 0.0, self.background(palette));

        let cells: Vec<Vec<Corner>> = (0..pts.len()).map(|i| cell(&pts, i, rect, 0.0)).collect();
        let nearest = self.hover.filter(|p| rect.contains(*p)).map(|at| {
            let mut order: Vec<usize> = (0..pts.len()).collect();
            order.sort_by(|&a, &b| pts[a].distance(at).total_cmp(&pts[b].distance(at)));
            (at, order)
        });

        for (i, exact) in cells.iter().enumerate() {
            let poly = if gap > 0.0 {
                cell(&pts, i, rect, gap)
            } else {
                exact.clone()
            };
            if poly.len() < 3 {
                continue;
            }
            let mut fill = self.fill(i, palette, dark);
            if let Some((_, order)) = &nearest
                && order[0] == i
            {
                fill = lerp_color(fill, Color32::WHITE, if dark { 0.15 } else { 0.3 });
            }
            let corners: Vec<Pos2> = poly.iter().map(|c| c.0).collect();
            painter.add(Shape::convex_polygon(corners, fill, self.edge(palette)));
        }

        if self.delaunay {
            let stroke = Stroke::new(1.0, palette.ink.gamma_multiply(0.55));
            for (i, poly) in cells.iter().enumerate() {
                for j in neighbors(poly).filter(|&j| j > i) {
                    painter.extend(Shape::dashed_line(&[pts[i], pts[j]], stroke, 4.0, 3.0));
                }
            }
        }

        if self.show_seeds {
            let seed_color = match self.style {
                Style::Mud | Style::Giraffe => Color32::from_rgb(0x2A, 0x1C, 0x10),
                _ => palette.ink,
            };
            for (k, &p) in pts.iter().enumerate() {
                if self.dragging == Some(k) {
                    handle(painter, p, palette.trace, palette.card, true);
                } else {
                    painter.circle(p, 3.5, seed_color, Stroke::new(1.5, palette.card));
                }
            }
        }

        // Hover: the nearest seed wins; show the runner-up for comparison.
        if let Some((at, order)) = nearest
            && order.len() > 1
        {
            let (i, j) = (order[0], order[1]);
            let (di, dj) = (pts[i].distance(at), pts[j].distance(at));
            painter.line_segment([at, pts[j]], Stroke::new(1.5, palette.y));
            painter.line_segment([at, pts[i]], Stroke::new(2.5, palette.x));
            painter.circle_filled(at, 4.0, palette.ink);
            let label = |p: Pos2, text: String, color: Color32| {
                let mid = at + (p - at) * 0.5;
                let galley = painter.layout_no_wrap(text, FontId::proportional(12.0), color);
                let r = Align2::CENTER_CENTER.anchor_size(mid, galley.size());
                painter.rect_filled(r.expand(3.0), 4.0, palette.card);
                painter.galley(r.min, galley, color);
            };
            label(pts[j], format!("{dj:.0}"), palette.y);
            label(pts[i], format!("{di:.0} nearest"), palette.x);
        }

        painter.text(
            rect.right_bottom() + vec2(-10.0, -8.0),
            Align2::RIGHT_BOTTOM,
            format!("{} seeds", pts.len()),
            FontId::proportional(11.0),
            match self.style {
                Style::Mud => Color32::from_rgb(0xD8, 0xC2, 0xA0),
                Style::Giraffe => Color32::from_rgb(0x6A, 0x4A, 0x2A),
                _ => palette.muted,
            },
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("voronoi-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Seeds");
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.count, 2..=150));
                    if ui.button("Scatter").clicked() {
                        self.scatter();
                    }
                });
                ui.end_row();
                ui.label("Style");
                ui.horizontal_wrapped(|ui| {
                    for s in Style::ALL {
                        ui.selectable_value(&mut self.style, s, s.label());
                    }
                });
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.drift, "Drift");
            ui.checkbox(&mut self.relax, "Relax (Lloyd)")
                .on_hover_text("Move each seed toward the center of its cell; cells even out.");
            ui.checkbox(&mut self.delaunay, "Delaunay edges")
                .on_hover_text("Join seeds whose cells share an edge.");
            ui.checkbox(&mut self.show_seeds, "Seeds");
        });
    }

    fn details(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.label(
            egui::RichText::new(
                "Each cell is computed exactly: start from the whole stage and cut away the \
                 half on the far side of the bisector with every other seed. Relax repeats \
                 Lloyd's algorithm, moving seeds to their cell centroids, which turns a random \
                 scatter into an even, honeycomb-like mesh.",
            )
            .color(palette.muted),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 60.0))
    }

    fn sample_seeds() -> Vec<Pos2> {
        let mut rng = Rng::new(5);
        (0..30)
            .map(|_| pos2(rng.range(0.0, 100.0), rng.range(0.0, 60.0)))
            .collect()
    }

    #[test]
    fn two_seeds_split_in_half() {
        let seeds = [pos2(25.0, 30.0), pos2(75.0, 30.0)];
        let a = cell(&seeds, 0, bounds(), 0.0);
        assert!((area(&a) - 3000.0).abs() < 1e-2);
        assert_eq!(neighbors(&a).collect::<Vec<_>>(), vec![1]);
    }

    #[test]
    fn cells_tile_the_bounds() {
        let seeds = sample_seeds();
        let total: f32 = (0..seeds.len())
            .map(|i| area(&cell(&seeds, i, bounds(), 0.0)))
            .sum();
        assert!((total - 6000.0).abs() < 0.5, "{total}");
    }

    #[test]
    fn every_corner_is_nearest_to_its_seed() {
        let seeds = sample_seeds();
        for i in 0..seeds.len() {
            for (x, _) in cell(&seeds, i, bounds(), 0.0) {
                let di = x.distance(seeds[i]);
                assert!(seeds.iter().all(|s| x.distance(*s) >= di - 1e-3));
            }
        }
    }

    #[test]
    fn neighbors_are_mutual() {
        let seeds = sample_seeds();
        let cells: Vec<_> = (0..seeds.len())
            .map(|i| cell(&seeds, i, bounds(), 0.0))
            .collect();
        for (i, c) in cells.iter().enumerate() {
            for j in neighbors(c) {
                assert!(neighbors(&cells[j]).any(|k| k == i), "{i} -> {j} only");
            }
        }
    }

    #[test]
    fn gap_shrinks_cells() {
        let seeds = sample_seeds();
        let total: f32 = (0..seeds.len())
            .map(|i| area(&cell(&seeds, i, bounds(), 4.0)))
            .sum();
        assert!(total < 6000.0 * 0.85);
    }

    #[test]
    fn centroid_of_rectangle() {
        let c = cell(&[pos2(10.0, 10.0)], 0, bounds(), 0.0);
        let m = centroid(&c).unwrap();
        assert!((m.x - 50.0).abs() < 1e-3 && (m.y - 30.0).abs() < 1e-3);
    }
}
