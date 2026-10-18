//! Force-directed graph layout: springs on edges (Hooke) and repulsion between
//! every pair of nodes (Coulomb), integrated until the graph settles.

use egui::{Align2, FontId, Pos2, Rect, Sense, Stroke, Vec2, pos2, vec2};

use crate::draw::{self, lerp_color};
use crate::formula::{Formula, MathText, Palette, Tone};
use crate::rng::Rng;

/// Hooke's law along an edge: negative pulls the nodes together.
pub fn spring(k: f32, d: f32, rest: f32) -> f32 {
    -k * (d - rest)
}

/// Coulomb-style repulsion between two nodes: always pushes apart.
pub fn repulsion(kr: f32, d: f32) -> f32 {
    kr / (d * d)
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub k: f32,
    pub rest: f32,
    pub kr: f32,
    /// Pull toward the origin so disconnected parts don't drift away.
    pub gravity: f32,
    /// Fraction of velocity kept each step.
    pub damping: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Node {
    pub pos: Vec2,
    pub vel: Vec2,
    pub cluster: usize,
}

/// Spring and repulsion forces on every node.
pub fn forces(nodes: &[Node], edges: &[(usize, usize)], p: &Params) -> (Vec<Vec2>, Vec<Vec2>) {
    let mut springs = vec![Vec2::ZERO; nodes.len()];
    let mut repel = vec![Vec2::ZERO; nodes.len()];
    for &(i, j) in edges {
        let delta = nodes[i].pos - nodes[j].pos;
        let d = delta.length().max(1e-3);
        let f = delta / d * spring(p.k, d, p.rest);
        springs[i] += f;
        springs[j] -= f;
    }
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            let delta = nodes[i].pos - nodes[j].pos;
            let d = delta.length().max(4.0);
            let f = delta / d * repulsion(p.kr, d);
            repel[i] += f;
            repel[j] -= f;
        }
    }
    (springs, repel)
}

/// One integration step. `pinned` stays where it is (the node being dragged).
/// Returns the total kinetic energy, which falls toward zero as the layout settles.
pub fn step(
    nodes: &mut [Node],
    edges: &[(usize, usize)],
    p: &Params,
    pinned: Option<usize>,
) -> f32 {
    let (springs, repel) = forces(nodes, edges, p);
    let mut kinetic = 0.0;
    for (i, n) in nodes.iter_mut().enumerate() {
        if Some(i) == pinned {
            n.vel = Vec2::ZERO;
            continue;
        }
        let f = springs[i] + repel[i] - n.pos * p.gravity;
        n.vel = (n.vel + f) * p.damping;
        // Cap the step so a stiff spring can't fling a node across the screen.
        let speed = n.vel.length();
        if speed > 12.0 {
            n.vel *= 12.0 / speed;
        }
        n.pos += n.vel;
        kinetic += 0.5 * n.vel.length_sq();
    }
    kinetic
}

/// A few clusters, each a random tree plus extra edges, loosely linked together.
pub fn random_graph(seed: u64) -> (Vec<Node>, Vec<(usize, usize)>) {
    let mut rng = Rng::new(seed);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let clusters = 4;
    let mut firsts = Vec::new();
    for c in 0..clusters {
        let start = nodes.len();
        firsts.push(start);
        let size = 5 + rng.index(5);
        for i in 0..size {
            nodes.push(Node {
                pos: vec2(rng.range(-150.0, 150.0), rng.range(-110.0, 110.0)),
                vel: Vec2::ZERO,
                cluster: c,
            });
            if i > 0 {
                edges.push((start + i, start + rng.index(i)));
            }
        }
        for _ in 0..size / 3 {
            let (a, b) = (start + rng.index(size), start + rng.index(size));
            if a != b && !edges.contains(&(a, b)) && !edges.contains(&(b, a)) {
                edges.push((a, b));
            }
        }
    }
    for c in 1..clusters {
        edges.push((firsts[c - 1], firsts[c]));
    }
    (nodes, edges)
}

pub struct ForceLayout {
    nodes: Vec<Node>,
    edges: Vec<(usize, usize)>,
    seed: u64,
    params: Params,
    running: bool,
    show_forces: bool,
    dragging: Option<usize>,
    hovered: Option<usize>,
    kinetic: f32,
}

impl Default for ForceLayout {
    fn default() -> Self {
        let (nodes, edges) = random_graph(1);
        Self {
            nodes,
            edges,
            seed: 1,
            params: Params {
                k: 0.04,
                rest: 60.0,
                kr: 3000.0,
                gravity: 0.004,
                damping: 0.85,
            },
            running: true,
            show_forces: true,
            dragging: None,
            hovered: None,
            kinetic: 0.0,
        }
    }
}

impl ForceLayout {
    fn screen(&self, rect: Rect) -> Vec<Pos2> {
        self.nodes.iter().map(|n| rect.center() + n.pos).collect()
    }

    fn shuffle(&mut self) {
        let mut rng = Rng::new(
            self.seed
                .wrapping_mul(31)
                .wrapping_add(self.nodes.len() as u64),
        );
        self.seed = self.seed.wrapping_add(1000);
        for n in &mut self.nodes {
            n.pos = vec2(rng.range(-200.0, 200.0), rng.range(-140.0, 140.0));
            n.vel = Vec2::ZERO;
        }
        self.running = true;
    }
}

/// What a sheet saves of a [`ForceLayout`].
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct State {
    /// The graph is generated from this seed.
    seed: u64,
    k: f32,
    rest: f32,
    kr: f32,
    running: bool,
    show_forces: bool,
}

impl ForceLayout {
    fn state(&self) -> State {
        State {
            seed: self.seed,
            k: self.params.k,
            rest: self.params.rest,
            kr: self.params.kr,
            running: self.running,
            show_forces: self.show_forces,
        }
    }

    fn apply(&mut self, s: State) {
        if s.seed != self.seed {
            self.seed = s.seed;
            (self.nodes, self.edges) = random_graph(self.seed);
        }
        self.params.k = s.k.clamp(0.005, 0.2);
        self.params.rest = s.rest.clamp(20.0, 140.0);
        self.params.kr = s.kr.clamp(200.0, 20000.0);
        self.running = s.running;
        self.show_forces = s.show_forces;
    }
}

impl Formula for ForceLayout {
    crate::state_hooks!();

    fn params(&self) -> Vec<(String, f64)> {
        vec![
            ("k".into(), self.params.k as f64),
            ("x_0".into(), self.params.rest as f64),
            ("k_r".into(), self.params.kr as f64),
        ]
    }

    fn id(&self) -> &'static str {
        "force-layout"
    }

    fn name(&self) -> &'static str {
        "Springs and repulsion"
    }

    fn category(&self) -> &'static str {
        super::PHYSICS
    }

    fn summary(&self) -> &'static str {
        "Edges are springs that pull linked nodes toward a rest length; every pair of \
         nodes pushes apart like charges. Drag a node and watch the graph rebalance."
    }

    fn notes(&self) -> &'static str {
        "Force-directed layouts arrange network graphs, org charts, knowledge graphs and \
         dependency maps: springs keep related items clustered while repulsion stops \
         nodes and labels from overlapping."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("F")
                .tone(Tone::X)
                .sub("s")
                .tone(Tone::X)
                .t(" = −k (d − x")
                .sub("0")
                .t(")"),
            MathText::new()
                .t("F")
                .tone(Tone::Y)
                .sub("r")
                .tone(Tone::Y)
                .t(" = k")
                .sub("r")
                .t(" / d")
                .sup("2"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let screen = self.screen(rect);
        self.hovered = response
            .hover_pos()
            .and_then(|at| draw::pick(&screen, at, 16.0));
        if response.drag_started()
            && let Some(at) = response.interact_pointer_pos()
        {
            self.dragging = draw::pick(&screen, at, 20.0);
        }
        if let (Some(i), Some(at)) = (self.dragging, response.interact_pointer_pos()) {
            self.nodes[i].pos = rect.shrink(12.0).clamp(at) - rect.center();
            self.running = true;
        }
        if response.drag_stopped() {
            self.dragging = None;
        }
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.75).max(420.0)
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, _dt: f32) {
        if self.running {
            for _ in 0..2 {
                self.kinetic = step(&mut self.nodes, &self.edges, &self.params, self.dragging);
            }
            let half = rect.size() / 2.0 - vec2(12.0, 12.0);
            for n in &mut self.nodes {
                n.pos = n.pos.clamp(-half, half);
            }
        }
        let screen = self.screen(rect);

        // Edge color shows the spring's state: x color stretched, y color squeezed.
        for &(i, j) in &self.edges {
            let d = screen[i].distance(screen[j]);
            let strain = ((d - self.params.rest) / self.params.rest).clamp(-1.0, 1.0);
            let color = if strain >= 0.0 {
                lerp_color(palette.muted, palette.x, strain)
            } else {
                lerp_color(palette.muted, palette.y, -strain)
            };
            painter.line_segment(
                [screen[i], screen[j]],
                Stroke::new(1.5 + 2.0 * strain.abs(), color),
            );
        }

        let cluster_colors = [palette.trace, palette.x, palette.y, palette.ink];
        for (i, p) in screen.iter().enumerate() {
            let active = Some(i) == self.dragging || Some(i) == self.hovered;
            let r = if active { 9.0 } else { 7.0 };
            painter.circle(
                *p,
                r,
                cluster_colors[self.nodes[i].cluster % 4],
                Stroke::new(2.0, palette.card),
            );
        }

        let focus = self.dragging.or(self.hovered);
        if let (true, Some(i)) = (self.show_forces, focus) {
            let (springs, repel) = forces(&self.nodes, &self.edges, &self.params);
            let scale = 12.0;
            let draw_arrow = |v: Vec2, color| {
                let v = v * scale;
                let v = if v.length() > 120.0 {
                    v * (120.0 / v.length())
                } else {
                    v
                };
                painter.arrow(screen[i], v, Stroke::new(2.5, color));
            };
            draw_arrow(springs[i], palette.x);
            draw_arrow(repel[i], palette.y);
        }

        let font = FontId::monospace(12.5);
        let status = if self.kinetic < 0.01 {
            "settled"
        } else {
            "moving"
        };
        painter.text(
            rect.left_bottom() + vec2(14.0, -12.0),
            Align2::LEFT_BOTTOM,
            format!("kinetic energy {:.3}  ({status})", self.kinetic),
            font.clone(),
            palette.muted,
        );
        if self.show_forces {
            let origin = rect.left_top() + vec2(14.0, 12.0);
            painter.text(
                origin,
                Align2::LEFT_TOP,
                "hover a node:",
                font.clone(),
                palette.muted,
            );
            for (row, (label, color)) in [("springs Fs", palette.x), ("repulsion Fr", palette.y)]
                .into_iter()
                .enumerate()
            {
                let y = origin.y + 24.0 + 16.0 * row as f32;
                painter.arrow(pos2(origin.x, y), vec2(18.0, 0.0), Stroke::new(2.0, color));
                painter.text(
                    pos2(origin.x + 24.0, y),
                    Align2::LEFT_CENTER,
                    label,
                    font.clone(),
                    color,
                );
            }
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        let before = (self.params.k, self.params.rest, self.params.kr);
        egui::Grid::new("force-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Stiffness k");
                ui.add(egui::Slider::new(&mut self.params.k, 0.005..=0.2).logarithmic(true));
                ui.end_row();
                ui.label("Rest length x₀");
                ui.add(egui::Slider::new(&mut self.params.rest, 20.0..=140.0));
                ui.end_row();
                ui.label("Repulsion kr");
                ui.add(egui::Slider::new(&mut self.params.kr, 200.0..=20000.0).logarithmic(true));
                ui.end_row();
            });
        if before != (self.params.k, self.params.rest, self.params.kr) {
            self.running = true;
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.running { "Pause" } else { "Run" })
                .clicked()
            {
                self.running = !self.running;
            }
            if ui.button("Shuffle").clicked() {
                self.shuffle();
            }
            if ui.button("New graph").clicked() {
                self.seed = self.seed.wrapping_add(1);
                (self.nodes, self.edges) = random_graph(self.seed);
                self.running = true;
            }
            if ui
                .button(if self.show_forces {
                    "Hide forces"
                } else {
                    "Show forces"
                })
                .clicked()
            {
                self.show_forces = !self.show_forces;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_direction() {
        assert!(spring(0.1, 80.0, 60.0) < 0.0, "stretched spring pulls in");
        assert!(spring(0.1, 40.0, 60.0) > 0.0, "squeezed spring pushes out");
        assert_eq!(spring(0.1, 60.0, 60.0), 0.0);
    }

    #[test]
    fn repulsion_inverse_square() {
        assert!((repulsion(100.0, 2.0) * 4.0 - repulsion(100.0, 1.0)).abs() < 1e-6);
    }

    #[test]
    fn pair_settles_where_forces_balance() {
        let p = Params {
            k: 0.05,
            rest: 60.0,
            kr: 3000.0,
            gravity: 0.0,
            damping: 0.8,
        };
        let mut nodes = vec![
            Node {
                pos: vec2(-10.0, 0.0),
                ..Default::default()
            },
            Node {
                pos: vec2(10.0, 0.0),
                ..Default::default()
            },
        ];
        let edges = [(0, 1)];
        for _ in 0..2000 {
            step(&mut nodes, &edges, &p, None);
        }
        let d = (nodes[0].pos - nodes[1].pos).length();
        // At rest the spring's pull equals the repulsion's push.
        assert!(
            (spring(p.k, d, p.rest) + repulsion(p.kr, d)).abs() < 1e-3,
            "d = {d}"
        );
        assert!(d > p.rest);
    }

    #[test]
    fn random_graph_is_connected_and_valid() {
        let (nodes, edges) = random_graph(5);
        assert!(
            edges
                .iter()
                .all(|&(a, b)| a < nodes.len() && b < nodes.len() && a != b)
        );
        // Union-find over the edges: everything should end up in one component.
        let mut parent: Vec<usize> = (0..nodes.len()).collect();
        fn root(p: &mut Vec<usize>, i: usize) -> usize {
            if p[i] != i {
                let r = root(p, p[i]);
                p[i] = r;
            }
            p[i]
        }
        for &(a, b) in &edges {
            let (ra, rb) = (root(&mut parent, a), root(&mut parent, b));
            parent[ra] = rb;
        }
        let r0 = root(&mut parent, 0);
        assert!((0..nodes.len()).all(|i| root(&mut parent, i) == r0));
    }
}
