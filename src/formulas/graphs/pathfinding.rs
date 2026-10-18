//! The interactive pathfinding board shared by BFS, DFS, Dijkstra and A*: one
//! grid per sheet that you draw on, and a search replayed step by step on it.

use egui::{Align2, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

use super::Session;
use super::grid::{GRID_H, GRID_W, MUD};
use super::path::{PathOp, PathState, Search, path_cost, record};
use crate::formula::{Formula, MathText, Palette};
use crate::formulas::replay::{self, Player};

/// Height of the readout strip above the board.
const HEADER: f32 = 30.0;
/// Steps per second at the start.
const SPEED: f32 = 60.0;
/// Below this cell size the distance numbers would not fit.
const NUMBERS_FROM: f32 = 18.0;

/// The largest square-celled board centered in `rect`, below the header strip.
pub fn board_rect(rect: Rect, w: usize, h: usize) -> Rect {
    let area = Rect::from_min_max(rect.min + vec2(0.0, HEADER), rect.max);
    let cell = (area.width() / w as f32)
        .min(area.height() / h as f32)
        .max(1.0);
    Rect::from_center_size(area.center(), vec2(cell * w as f32, cell * h as f32))
}

/// The cell under `p`, or `None` off the board.
pub fn cell_at(rect: Rect, w: usize, h: usize, p: Pos2) -> Option<usize> {
    let board = board_rect(rect, w, h);
    if !board.contains(p) {
        return None;
    }
    let cell = board.width() / w as f32;
    let x = (((p.x - board.min.x) / cell) as usize).min(w - 1);
    let y = (((p.y - board.min.y) / cell) as usize).min(h - 1);
    Some(y * w + x)
}

/// What a drag does to every cell it passes over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Brush {
    /// Painting walls on (`true`) or off.
    Walls(bool),
    MoveStart,
    MoveGoal,
    /// Painting mud on (`true`) or off.
    Mud(bool),
}

pub struct Pathfinding {
    search: Search,
    session: Session,
    /// The grid generation last recorded.
    generation: u64,
    player: Player<PathState>,
    /// Brush mode: paint mud instead of walls.
    mud: bool,
    stroke: Option<Brush>,
}

impl Pathfinding {
    pub fn new(search: Search, session: Session) -> Self {
        let mut p = Self {
            search,
            session,
            generation: u64::MAX,
            player: Player::new(PathState::new(0), Vec::new(), SPEED),
            mud: false,
            stroke: None,
        };
        p.sync();
        p
    }

    /// Re-record the search when the shared grid has changed.
    fn sync(&mut self) {
        let grid = self.session.grid.borrow();
        if grid.generation() != self.generation {
            self.player
                .reset(PathState::new(grid.len()), record(self.search, &grid));
            self.generation = grid.generation();
        }
    }

    fn path_steps(&self) -> usize {
        self.player.state.path.len().saturating_sub(1)
    }

    fn choose_stroke(&self, cell: usize) -> Brush {
        let grid = self.session.grid.borrow();
        if cell == grid.start() {
            Brush::MoveStart
        } else if cell == grid.goal() {
            Brush::MoveGoal
        } else if self.mud {
            Brush::Mud(grid.weight(cell) != MUD)
        } else {
            Brush::Walls(!grid.is_wall(cell))
        }
    }

    /// Choose the brush where the press began, not where the pointer is once
    /// the drag threshold has passed, so a quick flick off a marker still
    /// moves the marker.
    fn begin_stroke(&mut self, rect: Rect, origin: Option<Pos2>, pointer_cell: Option<usize>) {
        let cell = origin
            .and_then(|p| cell_at(rect, GRID_W, GRID_H, p))
            .or(pointer_cell);
        if let Some(cell) = cell {
            self.stroke = Some(self.choose_stroke(cell));
        }
    }

    fn apply_stroke(&self, stroke: Brush, cell: usize) {
        let mut grid = self.session.grid.borrow_mut();
        match stroke {
            Brush::Walls(on) => grid.set_wall(cell, on),
            Brush::Mud(on) => grid.set_weight(cell, if on { MUD } else { 1 }),
            Brush::MoveStart => {
                grid.move_start(cell);
            }
            Brush::MoveGoal => {
                grid.move_goal(cell);
            }
        }
    }

    fn header(&self) -> String {
        let s = &self.player.state;
        if s.no_path {
            return "No path: the goal is walled off".to_owned();
        }
        let mut text = format!("{}  ·  visited {}", self.search.name(), s.visits);
        if !s.path.is_empty() {
            text.push_str(&format!("  ·  path {} steps", self.path_steps()));
            if self.search.weighted() {
                let cost = path_cost(&self.session.grid.borrow(), self.search, &s.path);
                text.push_str(&format!("  ·  cost {cost}"));
            }
        }
        text
    }
}

impl Formula for Pathfinding {
    fn id(&self) -> &'static str {
        self.search.id()
    }

    fn name(&self) -> &'static str {
        self.search.name()
    }

    fn category(&self) -> &'static str {
        super::super::GRAPHS
    }

    fn summary(&self) -> &'static str {
        match self.search {
            Search::Bfs => {
                "Breadth-first search explores in rings of equal distance, so the first path it \
                 finds is the shortest in steps."
            }
            Search::Dfs => {
                "Depth-first search dives down one corridor before backing up. It finds a path, \
                 but rarely the shortest."
            }
            Search::Dijkstra => {
                "Dijkstra's algorithm settles cells in order of total cost, so mud (weight 5) is \
                 avoided when a detour is cheaper."
            }
            Search::AStar => {
                "A* is Dijkstra's algorithm steered by a straight-line guess of the remaining \
                 distance, so it visits fewer cells."
            }
        }
    }

    fn notes(&self) -> &'static str {
        "Maze solving, game AI and navigation meshes, GPS routing, network routing and \
         flood fill."
    }

    fn equations(&self) -> Vec<MathText> {
        match self.search {
            Search::Bfs => vec![
                MathText::new().t("dist(n) = dist(u) + 1"),
                MathText::new().t("visit cells in order of dist"),
            ],
            Search::Dfs => {
                vec![MathText::new().t("follow the newest frontier cell first (a stack)")]
            }
            Search::Dijkstra => vec![MathText::new().t("dist(n) = min(dist(n), dist(u) + w(n))")],
            Search::AStar => vec![
                MathText::new().t("f(n) = g(n) + h(n)"),
                MathText::new()
                    .t("h(n) = |x − x")
                    .sub("goal")
                    .t("| + |y − y")
                    .sub("goal")
                    .t("|"),
            ],
        }
    }

    fn params(&self) -> Vec<(String, f64)> {
        vec![
            ("visited".into(), self.player.state.visits as f64),
            ("path".into(), self.path_steps() as f64),
        ]
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * GRID_H as f32 / GRID_W as f32 + 64.0
    }

    fn min_size(&self) -> egui::Vec2 {
        vec2(420.0, self.stage_height(420.0))
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn is_animating(&self) -> bool {
        self.player.playing
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        let cell = response
            .interact_pointer_pos()
            .and_then(|p| cell_at(rect, GRID_W, GRID_H, p));
        if response.drag_started() || response.clicked() {
            let origin = response.ctx.input(|i| i.pointer.press_origin());
            self.begin_stroke(rect, origin, cell);
        }
        if let (Some(stroke), Some(cell)) = (self.stroke, cell)
            && (response.dragged() || response.clicked() || response.drag_started())
        {
            self.apply_stroke(stroke, cell);
        }
        if response.drag_stopped() || response.clicked() {
            self.stroke = None;
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.sync();
        self.player.advance(dt);
        let painter = painter.with_clip_rect(rect);
        let board = board_rect(rect, GRID_W, GRID_H);
        let cell = board.width() / GRID_W as f32;
        let grid = self.session.grid.borrow();
        let state = &self.player.state;

        painter.text(
            rect.min + vec2(4.0, HEADER / 2.0),
            Align2::LEFT_CENTER,
            self.header(),
            FontId::proportional(13.0),
            if state.no_path {
                palette.x
            } else {
                palette.ink
            },
        );

        let on_path: Vec<bool> = {
            let mut v = vec![false; grid.len()];
            for &c in &state.path {
                v[c] = true;
            }
            v
        };
        let numbers = cell >= NUMBERS_FROM;
        for (i, &path_cell) in on_path.iter().enumerate() {
            let (x, y) = grid.xy(i);
            let r = Rect::from_min_size(
                board.min + vec2(x as f32 * cell, y as f32 * cell),
                vec2(cell, cell),
            );
            let wall = grid.is_wall(i);
            let mud = !wall && grid.weight(i) == MUD;
            // Mud is a tint under the search colors; the path sits on top.
            painter.rect_filled(r, 0.0, if wall { palette.ink } else { palette.paper });
            if mud {
                painter.rect_filled(r, 0.0, palette.y.gamma_multiply(0.35));
            }
            if !wall {
                if path_cell {
                    painter.rect_filled(r, 0.0, palette.trace);
                } else if state.frontier[i] {
                    painter.rect_filled(r, 0.0, palette.x.gamma_multiply(0.55));
                } else if state.visited[i] {
                    painter.rect_filled(r, 0.0, palette.fill.gamma_multiply(0.7));
                }
            }
            if mud {
                // Keeps mud readable under every search color.
                painter.rect_stroke(
                    r.shrink(cell * 0.2),
                    0.0,
                    Stroke::new(1.0, palette.y),
                    StrokeKind::Inside,
                );
            }
            if numbers
                && state.visited[i]
                && let Some(d) = state.dist[i]
            {
                painter.text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    d.to_string(),
                    FontId::proportional(9.0),
                    palette.ink,
                );
            }
        }

        let line = Stroke::new(1.0, palette.rule);
        for k in 0..=GRID_W {
            let x = board.min.x + k as f32 * cell;
            painter.line_segment([pos2(x, board.min.y), pos2(x, board.max.y)], line);
        }
        for k in 0..=GRID_H {
            let y = board.min.y + k as f32 * cell;
            painter.line_segment([pos2(board.min.x, y), pos2(board.max.x, y)], line);
        }

        let center = |i: usize| {
            let (x, y) = grid.xy(i);
            board.min + vec2((x as f32 + 0.5) * cell, (y as f32 + 0.5) * cell)
        };
        painter.circle_filled(center(grid.start()), cell * 0.32, palette.x);
        painter.circle_stroke(
            center(grid.goal()),
            cell * 0.32,
            Stroke::new(2.0, palette.y),
        );

        if let Some(op) = state.last {
            let i = match op {
                PathOp::Frontier { cell, .. } => Some(cell),
                PathOp::Visit(cell) | PathOp::Path(cell) => Some(cell),
                PathOp::NoPath => None,
            };
            if let Some(i) = i {
                let (x, y) = grid.xy(i);
                let r = Rect::from_min_size(
                    board.min + vec2(x as f32 * cell, y as f32 * cell),
                    vec2(cell, cell),
                );
                painter.rect_stroke(r, 0.0, Stroke::new(2.0, palette.ink), StrokeKind::Inside);
            }
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            let mut grid = self.session.grid.borrow_mut();
            if ui.button("Empty").clicked() {
                grid.clear();
            }
            if ui.button("Maze").clicked() {
                grid.maze_next();
            }
            if ui.button("Random walls").clicked() {
                grid.scatter_next(0.28);
            }
        });
        ui.checkbox(&mut self.mud, "Paint mud (weight 5)");
        self.sync();

        ui.add_space(10.0);
        replay::transport(ui, &mut self.player, 2000.0);

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            for (label, color) in [
                ("wall", palette.ink),
                ("mud", palette.y.gamma_multiply(0.35)),
                ("frontier", palette.x.gamma_multiply(0.55)),
                ("visited", palette.fill),
                ("path", palette.trace),
            ] {
                let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                ui.painter().rect_filled(r, 3.0, color);
                ui.painter().rect_stroke(
                    r,
                    3.0,
                    Stroke::new(1.0, palette.rule),
                    StrokeKind::Inside,
                );
                ui.label(egui::RichText::new(label).small().color(palette.muted));
                ui.add_space(8.0);
            }
        });
        ui.label(
            egui::RichText::new(
                "Drag to draw walls or mud; drag the dot (start) or ring (goal) to move it; \
                 Space, Left and Right step",
            )
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
    use crate::formulas::graphs::{Session, path::Search};
    use egui::{Rect, pos2, vec2};

    #[test]
    fn cells_map_to_and_from_the_board() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(620.0, 410.0));
        let board = board_rect(rect, 31, 19);
        let cell = board.width() / 31.0;
        assert!((board.height() / 19.0 - cell).abs() < 1e-3, "square cells");
        let p = board.min + vec2(cell * 2.5, cell * 3.5);
        assert_eq!(cell_at(rect, 31, 19, p), Some(3 * 31 + 2));
        assert_eq!(cell_at(rect, 31, 19, board.min - vec2(1.0, 1.0)), None);
        assert_eq!(cell_at(rect, 31, 19, board.max + vec2(1.0, 1.0)), None);
    }

    #[test]
    fn presets_differ_between_viewports_sharing_a_board() {
        let session = Session::new();
        let _a = Pathfinding::new(Search::Bfs, session.clone());
        let _b = Pathfinding::new(Search::Dfs, session.clone());
        let walls = || {
            let g = session.grid.borrow();
            (0..g.len()).map(|i| g.is_wall(i)).collect::<Vec<_>>()
        };
        session.grid.borrow_mut().maze_next();
        let first = walls();
        session.grid.borrow_mut().maze_next();
        let second = walls();
        assert_ne!(first, second);
        session.grid.borrow_mut().scatter_next(0.28);
        let third = walls();
        assert_ne!(second, third);
        session.grid.borrow_mut().scatter_next(0.28);
        assert_ne!(third, walls());
    }

    #[test]
    fn the_brush_is_chosen_where_the_press_began() {
        let session = Session::new();
        let mut v = Pathfinding::new(Search::Bfs, session.clone());
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(620.0, 440.0));
        let board = board_rect(rect, GRID_W, GRID_H);
        let cell = board.width() / GRID_W as f32;
        let centre = |i: usize| {
            let (x, y) = session.grid.borrow().xy(i);
            board.min + vec2((x as f32 + 0.5) * cell, (y as f32 + 0.5) * cell)
        };
        let (s, g) = (session.grid.borrow().start(), session.grid.borrow().goal());
        // The pointer is already on an empty cell, but the press was on S.
        let away = cell_at(rect, GRID_W, GRID_H, centre(s) + vec2(cell * 3.0, 0.0));
        v.begin_stroke(rect, Some(centre(s)), away);
        assert_eq!(v.stroke, Some(Brush::MoveStart));
        v.begin_stroke(rect, Some(centre(g)), away);
        assert_eq!(v.stroke, Some(Brush::MoveGoal));
        v.begin_stroke(rect, None, away);
        assert_eq!(v.stroke, Some(Brush::Walls(true)));
    }

    #[test]
    fn the_header_shows_a_path_only_once_there_is_one() {
        let session = Session::new();
        let mut v = Pathfinding::new(Search::Bfs, session.clone());
        assert!(!v.header().contains("path"), "{}", v.header());
        v.player.seek(v.player.ops.len());
        assert!(v.header().contains("path"), "{}", v.header());
        let mut d = Pathfinding::new(Search::Dijkstra, session);
        d.player.seek(d.player.ops.len());
        assert!(d.header().contains("cost"), "{}", d.header());
    }

    /// Review Focus 5: when the shared grid changes mid-replay, every viewport
    /// re-records and its cursor restarts inside the new log.
    #[test]
    fn viewports_share_the_grid_and_rerecord_when_it_changes() {
        let session = Session::new();
        let mut bfs = Pathfinding::new(Search::Bfs, session.clone());
        let mut dfs = Pathfinding::new(Search::Dfs, session.clone());
        bfs.player.seek(bfs.player.ops.len());
        session.grid.borrow_mut().maze(3);
        bfs.sync();
        dfs.sync();
        assert_eq!(bfs.player.cursor, 0);
        assert!(bfs.player.cursor <= bfs.player.ops.len());
        assert_eq!(bfs.generation, session.grid.borrow().generation());
        assert_eq!(dfs.generation, bfs.generation);
    }
}
