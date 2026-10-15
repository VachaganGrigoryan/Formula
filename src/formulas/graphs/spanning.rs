//! The minimum-spanning-tree viewport shared by Kruskal's and Prim's
//! algorithms: one draggable graph per sheet, and each algorithm's recorded
//! run replayed step by step on it.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use super::Session;
use super::mst::{EdgeState, Graph, MstOp, MstState, Spanning, record};
use crate::draw;
use crate::formula::{Formula, MathText, Palette};
use crate::formulas::replay::{self, Player};

/// Height of the readout strip above the graph.
const HEADER: f32 = 30.0;
/// Margin between the graph and the edge of its area.
const MARGIN: f32 = 20.0;
/// Steps per second at the start.
const SPEED: f32 = 60.0;
/// How close (in points) a press must be to grab a vertex.
const GRAB: f32 = 14.0;
/// Vertex radius.
const DOT: f32 = 7.0;
/// Kruskal's side list needs a stage at least this wide.
const LIST_FROM: f32 = 560.0;
/// Width of Kruskal's side list.
const LIST_W: f32 = 150.0;
/// How many upcoming edges the side list shows.
const LIST_ROWS: usize = 10;

/// The square the unit square of points is drawn in, and Kruskal's list
/// column when there is room for it.
fn layout(rect: Rect, kruskal: bool) -> (Rect, Option<Rect>) {
    let mut area = Rect::from_min_max(rect.min + vec2(0.0, HEADER), rect.max);
    let mut list = None;
    if kruskal && rect.width() >= LIST_FROM {
        list = Some(Rect::from_min_max(
            pos2(area.max.x - LIST_W, area.min.y),
            area.max,
        ));
        area.max.x -= LIST_W;
    }
    let inner = area.shrink(MARGIN);
    let side = inner.width().min(inner.height()).max(1.0);
    (
        Rect::from_center_size(inner.center(), vec2(side, side)),
        list,
    )
}

fn to_screen(square: Rect, p: [f32; 2]) -> Pos2 {
    square.min + vec2(p[0], p[1]) * square.width()
}

fn to_unit(square: Rect, p: Pos2) -> [f32; 2] {
    let v = (p - square.min) / square.width();
    [v.x, v.y]
}

/// The root of `v` in the union-find forest `parent`.
fn root(parent: &[usize], mut v: usize) -> usize {
    while parent[v] != v {
        v = parent[v];
    }
    v
}

pub struct SpanningTree {
    alg: Spanning,
    session: Session,
    /// The graph generation last recorded.
    generation: u64,
    player: Player<MstState>,
    /// The point count the next "New points" uses.
    n: usize,
    seed: u64,
    /// The vertex being dragged.
    drag: Option<usize>,
}

impl SpanningTree {
    pub fn new(alg: Spanning, session: Session) -> Self {
        let g = session.graph.borrow().clone();
        let mut s = Self {
            alg,
            n: g.points().len(),
            player: Player::new(MstState::new(&g), record(alg, &g), SPEED),
            generation: g.generation(),
            session,
            seed: 7,
            drag: None,
        };
        s.sync();
        s
    }

    /// Re-record the run when the shared graph has changed. Ops recorded for
    /// an older triangulation must never be replayed against a newer graph.
    fn sync(&mut self) {
        let graph = self.session.graph.borrow();
        if graph.generation() != self.generation {
            self.player
                .reset(MstState::new(&graph), record(self.alg, &graph));
            self.generation = graph.generation();
        }
    }

    fn header(&self, graph: &Graph) -> String {
        let s = &self.player.state;
        format!(
            "{}  \u{b7}  tree weight {:.0}  \u{b7}  edges {} / {}",
            self.alg.name(),
            s.weight,
            s.accepted,
            graph.points().len().saturating_sub(1)
        )
    }

    fn paint_edges(&self, painter: &egui::Painter, graph: &Graph, square: Rect, palette: &Palette) {
        let state = &self.player.state;
        let at = |i: usize| to_screen(square, graph.points()[i]);
        // Quiet edges first so the interesting ones stay on top.
        for pass in [
            EdgeState::Unseen,
            EdgeState::Rejected,
            EdgeState::Accepted,
            EdgeState::Considering,
        ] {
            for (i, e) in graph.edges().iter().enumerate() {
                if state.edges[i] != pass {
                    continue;
                }
                let line = [at(e.a), at(e.b)];
                match pass {
                    EdgeState::Unseen => {
                        painter.line_segment(line, Stroke::new(1.0, palette.rule));
                    }
                    EdgeState::Considering => {
                        painter.line_segment(line, Stroke::new(3.0, palette.y));
                    }
                    EdgeState::Accepted => {
                        painter.line_segment(line, Stroke::new(3.0, palette.trace));
                    }
                    EdgeState::Rejected => {
                        painter.extend(Shape::dashed_line(
                            &line,
                            Stroke::new(1.5, palette.muted.gamma_multiply(0.4)),
                            5.0,
                            4.0,
                        ));
                    }
                }
            }
        }
    }

    fn paint_vertices(
        &self,
        painter: &egui::Painter,
        graph: &Graph,
        square: Rect,
        palette: &Palette,
    ) {
        let state = &self.player.state;
        let cycle: [Color32; 6] = [
            palette.x,
            palette.y,
            palette.trace,
            palette.ink,
            palette.muted,
            palette.x.gamma_multiply(0.6),
        ];
        for (i, p) in graph.points().iter().enumerate() {
            let fill = match self.alg {
                Spanning::Kruskal => cycle[root(&state.parent, i) % cycle.len()],
                Spanning::Prim if state.in_tree[i] => palette.trace,
                Spanning::Prim => palette.card,
            };
            let c = to_screen(square, *p);
            painter.circle_filled(c, DOT, fill);
            painter.circle_stroke(c, DOT, Stroke::new(1.5, palette.ink));
        }
        if let Some(MstOp::Reach(v)) = state.last
            && let Some(p) = graph.points().get(v)
        {
            painter.circle_stroke(
                to_screen(square, *p),
                DOT + 3.0,
                Stroke::new(1.5, palette.ink),
            );
        }
    }

    /// Kruskal's upcoming edges in sorted order, with their weights.
    fn paint_list(&self, painter: &egui::Painter, graph: &Graph, list: Rect, palette: &Palette) {
        let state = &self.player.state;
        let mut order: Vec<usize> = (0..graph.edges().len()).collect();
        order.sort_by(|&x, &y| {
            graph.edges()[x]
                .w
                .total_cmp(&graph.edges()[y].w)
                .then(x.cmp(&y))
        });
        painter.text(
            list.min + vec2(8.0, 10.0),
            Align2::LEFT_CENTER,
            "next edges by weight",
            FontId::proportional(11.0),
            palette.muted,
        );
        let upcoming = order
            .into_iter()
            .filter(|&e| state.edges[e] == EdgeState::Unseen)
            .take(LIST_ROWS);
        for (row, e) in upcoming.enumerate() {
            let edge = graph.edges()[e];
            painter.text(
                list.min + vec2(8.0, 30.0 + row as f32 * 18.0),
                Align2::LEFT_CENTER,
                format!("{} - {}   {:.1}", edge.a, edge.b, edge.w),
                FontId::monospace(11.0),
                palette.ink,
            );
        }
    }
}

impl Formula for SpanningTree {
    fn id(&self) -> &'static str {
        self.alg.id()
    }

    fn name(&self) -> &'static str {
        self.alg.name()
    }

    fn category(&self) -> &'static str {
        super::super::GRAPHS
    }

    fn summary(&self) -> &'static str {
        match self.alg {
            Spanning::Kruskal => {
                "sorts the edges by weight and keeps each one that joins two separate trees, \
                 merging a forest into one tree"
            }
            Spanning::Prim => {
                "grows a single tree from one vertex, always adding the lightest edge that \
                 leaves it"
            }
        }
    }

    fn notes(&self) -> &'static str {
        match self.alg {
            Spanning::Kruskal => {
                "Network design, single-linkage clustering, image segmentation and \
                 approximating the traveling salesman tour."
            }
            Spanning::Prim => "Cable and pipe layout, and maze generation (randomized Prim).",
        }
    }

    fn equations(&self) -> Vec<MathText> {
        let rule = match self.alg {
            Spanning::Kruskal => {
                "sort edges by weight; add each one that joins two different trees"
            }
            Spanning::Prim => "grow one tree; add the lightest edge leaving it",
        };
        vec![
            MathText::new().t(rule),
            MathText::new().t("minimize Σ w(e) over the n − 1 tree edges"),
        ]
    }

    fn params(&self) -> Vec<(&'static str, f64)> {
        let s = &self.player.state;
        vec![("weight", s.weight as f64), ("edges", s.accepted as f64)]
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * 0.7
    }

    fn min_size(&self) -> egui::Vec2 {
        vec2(420.0, self.stage_height(420.0) + 64.0)
    }

    fn stage_sense(&self) -> Sense {
        Sense::drag()
    }

    fn is_animating(&self) -> bool {
        self.player.playing
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let (square, _) = layout(rect, self.alg == Spanning::Kruskal);
        let pointer = response.interact_pointer_pos();
        if response.drag_started()
            && let Some(at) = pointer
        {
            let screen: Vec<Pos2> = self
                .session
                .graph
                .borrow()
                .points()
                .iter()
                .map(|p| to_screen(square, *p))
                .collect();
            self.drag = draw::pick(&screen, at, GRAB);
        }
        if response.dragged()
            && let (Some(i), Some(at)) = (self.drag, pointer)
        {
            self.session
                .graph
                .borrow_mut()
                .move_point(i, to_unit(square, at));
        }
        if response.drag_stopped() {
            self.drag = None;
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.sync();
        self.player.advance(dt);
        let painter = painter.with_clip_rect(rect);
        let (square, list) = layout(rect, self.alg == Spanning::Kruskal);
        let graph = self.session.graph.borrow();

        painter.text(
            rect.min + vec2(4.0, HEADER / 2.0),
            Align2::LEFT_CENTER,
            self.header(&graph),
            FontId::proportional(13.0),
            palette.ink,
        );
        self.paint_edges(&painter, &graph, square, palette);
        self.paint_vertices(&painter, &graph, square, palette);
        if let Some(list) = list {
            self.paint_list(&painter, &graph, list, palette);
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("New points").clicked() {
                self.seed += 1;
                self.session
                    .graph
                    .borrow_mut()
                    .regenerate(self.n, self.seed);
            }
            ui.add(egui::Slider::new(&mut self.n, 6..=24).text("points"));
        });
        self.sync();

        ui.add_space(10.0);
        replay::transport(ui, &mut self.player, 60.0);

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            let entries: [(&str, Color32, bool); 4] = [
                ("unseen", palette.rule, false),
                ("considering", palette.y, false),
                ("accepted", palette.trace, false),
                ("rejected", palette.muted.gamma_multiply(0.4), true),
            ];
            for (label, color, dashed) in entries {
                let (r, _) = ui.allocate_exact_size(vec2(18.0, 12.0), Sense::hover());
                let line = [r.left_center(), r.right_center()];
                if dashed {
                    ui.painter().extend(Shape::dashed_line(
                        &line,
                        Stroke::new(2.0, color),
                        4.0,
                        3.0,
                    ));
                } else {
                    ui.painter().line_segment(line, Stroke::new(3.0, color));
                }
                ui.label(egui::RichText::new(label).small().color(palette.muted));
                ui.add_space(8.0);
            }
            let note = match self.alg {
                Spanning::Kruskal => "vertex color: its tree",
                Spanning::Prim => "filled vertex: in the tree",
            };
            ui.label(egui::RichText::new(note).small().color(palette.muted));
        });
        ui.label(
            egui::RichText::new("Drag points to move them")
                .small()
                .color(palette.muted),
        );
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        replay::transport_keys(ctx, &mut self.player);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas::graphs::{Session, mst::Spanning};

    /// Review Focus 5 for graphs: moving a point mid-replay re-records every
    /// viewport sharing the graph.
    #[test]
    fn moving_a_point_rerecords_every_viewport() {
        let session = Session::new();
        let mut k = SpanningTree::new(Spanning::Kruskal, session.clone());
        let mut p = SpanningTree::new(Spanning::Prim, session.clone());
        k.player.seek(3);
        session.graph.borrow_mut().move_point(0, [0.5, 0.5]);
        k.sync();
        p.sync();
        assert_eq!(k.player.cursor, 0);
        assert_eq!(k.generation, session.graph.borrow().generation());
        assert_eq!(p.generation, k.generation);
    }
}
