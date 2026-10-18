//! The drawing: viewports placed on the plane, in z-order, plus selection.

use egui::{Pos2, Rect, Vec2, pos2, vec2};

use crate::formula::Formula;
use crate::formulas::{self, Tool, ToolEnv};

/// Default viewport width in world units; the height comes from the formula.
pub const DEFAULT_WIDTH: f32 = 640.0;

/// A formula placed on the board.
pub struct Viewport {
    pub uid: u64,
    pub tool: &'static Tool,
    pub formula: Box<dyn Formula>,
    /// Position and size in world units (points at 100%).
    pub rect: Rect,
    /// Short name shown on the title tab, e.g. "VP-03".
    pub title: String,
    pub locked: bool,
    pub hidden: bool,
    /// When it was placed (egui time), for the fade-in.
    pub born: f64,
}

impl Viewport {
    pub fn label(&self) -> String {
        format!("{} · {}", self.title, self.tool.name)
    }

    pub fn min_size(&self) -> Vec2 {
        self.formula.min_size()
    }
}

/// The default size of a new viewport of this formula.
pub fn default_size(formula: &dyn Formula) -> Vec2 {
    let min = formula.min_size();
    vec2(
        DEFAULT_WIDTH.max(min.x),
        formula.stage_height(DEFAULT_WIDTH).max(min.y),
    )
    .round()
}

pub struct Board {
    pub env: ToolEnv,
    /// Bottom to top.
    pub viewports: Vec<Viewport>,
    /// Selected viewports; the last one is the primary (its properties show).
    pub selection: Vec<u64>,
    next_uid: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            env: ToolEnv::new(),
            viewports: Vec::new(),
            selection: Vec::new(),
            next_uid: 1,
        }
    }
}

impl Board {
    /// Place a new viewport of `tool` with its top-left corner at `at`. A
    /// zero `size` means the formula's default size.
    pub fn add(&mut self, tool: &'static Tool, at: Pos2, size: Vec2, now: f64) -> u64 {
        let formula = (tool.make)(&self.env);
        let min = formula.min_size();
        let size = if size == Vec2::ZERO {
            default_size(formula.as_ref())
        } else {
            size.max(min)
        };
        let uid = self.next_uid;
        self.next_uid += 1;
        self.viewports.push(Viewport {
            uid,
            tool,
            formula,
            rect: Rect::from_min_size(at, size),
            title: format!("VP-{uid:02}"),
            locked: false,
            hidden: false,
            born: now,
        });
        uid
    }

    /// Re-insert a viewport that was removed (undo), at its old z position.
    pub fn insert(&mut self, index: usize, vp: Viewport) {
        self.next_uid = self.next_uid.max(vp.uid + 1);
        let index = index.min(self.viewports.len());
        self.viewports.insert(index, vp);
    }

    /// Take a viewport off the board, returning it and its z position.
    pub fn remove(&mut self, uid: u64) -> Option<(usize, Viewport)> {
        let i = self.index(uid)?;
        self.selection.retain(|&s| s != uid);
        Some((i, self.viewports.remove(i)))
    }

    pub fn index(&self, uid: u64) -> Option<usize> {
        self.viewports.iter().position(|v| v.uid == uid)
    }

    pub fn get(&self, uid: u64) -> Option<&Viewport> {
        self.viewports.iter().find(|v| v.uid == uid)
    }

    pub fn get_mut(&mut self, uid: u64) -> Option<&mut Viewport> {
        self.viewports.iter_mut().find(|v| v.uid == uid)
    }

    /// A copy of a viewport with the same settings, at the same size, offset
    /// down-right.
    pub fn duplicate(&mut self, uid: u64, now: f64) -> Option<u64> {
        let vp = self.get(uid)?;
        let (tool, rect, state) = (vp.tool, vp.rect, vp.formula.save_state());
        let copy = self.add(tool, rect.min + vec2(40.0, 40.0), rect.size(), now);
        if let (Some(state), Some(vp)) = (state, self.get_mut(copy)) {
            vp.formula.load_state(&state);
        }
        Some(copy)
    }

    pub fn bring_to_front(&mut self, uid: u64) {
        if let Some(i) = self.index(uid) {
            let vp = self.viewports.remove(i);
            self.viewports.push(vp);
        }
    }

    pub fn send_to_back(&mut self, uid: u64) {
        if let Some(i) = self.index(uid) {
            let vp = self.viewports.remove(i);
            self.viewports.insert(0, vp);
        }
    }

    /// The topmost visible viewport containing the world point.
    pub fn hit(&self, p: Pos2) -> Option<u64> {
        self.viewports
            .iter()
            .rev()
            .find(|v| !v.hidden && v.rect.contains(p))
            .map(|v| v.uid)
    }

    pub fn primary(&self) -> Option<u64> {
        self.selection.last().copied()
    }

    pub fn is_selected(&self, uid: u64) -> bool {
        self.selection.contains(&uid)
    }

    pub fn select_only(&mut self, uid: u64) {
        self.selection = vec![uid];
    }

    pub fn toggle_selected(&mut self, uid: u64) {
        if let Some(i) = self.selection.iter().position(|&s| s == uid) {
            self.selection.remove(i);
        } else {
            self.selection.push(uid);
        }
    }

    /// The bounding box of all visible viewports.
    pub fn bounds(&self) -> Option<Rect> {
        self.viewports
            .iter()
            .filter(|v| !v.hidden)
            .map(|v| v.rect)
            .reduce(|a, b| a.union(b))
    }

    pub fn selection_bounds(&self) -> Option<Rect> {
        self.viewports
            .iter()
            .filter(|v| self.is_selected(v.uid))
            .map(|v| v.rect)
            .reduce(|a, b| a.union(b))
    }

    /// The next free spot to the right of everything, for placing from the
    /// command line or the library without a click.
    pub fn free_spot(&self) -> Pos2 {
        match self.bounds() {
            Some(b) => pos2(b.right() + 80.0, b.top()),
            None => pos2(-320.0, -240.0),
        }
    }

    pub fn to_model(&self) -> Vec<ViewportModel> {
        self.viewports
            .iter()
            .map(|v| ViewportModel {
                uid: v.uid,
                tool: v.tool.id.to_owned(),
                rect: [v.rect.min.x, v.rect.min.y, v.rect.width(), v.rect.height()],
                title: v.title.clone(),
                locked: v.locked,
                hidden: v.hidden,
                state: v.formula.save_state(),
            })
            .collect()
    }

    /// Rebuild a board from a saved layout; unknown tools are skipped. The
    /// shared data in `env` is what the formulas are built on.
    pub fn from_model(model: &[ViewportModel], env: ToolEnv) -> Self {
        let mut board = Board {
            env,
            ..Board::default()
        };
        for m in model {
            board.add_model(m, false, f64::NEG_INFINITY);
        }
        board
    }

    /// Place a saved viewport on top, with its formula's state restored.
    /// `fresh_uid` gives it a new uid instead of the saved one (when it joins
    /// a board that already has viewports, e.g. opening a sheet). `None` for
    /// an unknown tool.
    pub fn add_model(&mut self, m: &ViewportModel, fresh_uid: bool, now: f64) -> Option<u64> {
        let tool = formulas::tool(&m.tool)?;
        let mut formula = (tool.make)(&self.env);
        if let Some(state) = &m.state {
            formula.load_state(state);
        }
        let [x, y, w, h] = m.rect.map(|v| if v.is_finite() { v } else { 0.0 });
        let size = vec2(w, h).max(formula.min_size());
        let uid = if fresh_uid { self.next_uid } else { m.uid };
        self.viewports.push(Viewport {
            uid,
            tool,
            formula,
            rect: Rect::from_min_size(pos2(x, y), size),
            title: m.title.clone(),
            locked: m.locked,
            hidden: m.hidden,
            born: now,
        });
        self.next_uid = self.next_uid.max(uid + 1);
        Some(uid)
    }

    /// A first-launch sheet with a handful of formulas laid out in a grid.
    pub fn demo(focus: Option<&str>) -> Self {
        let mut board = Board::default();
        let mut ids = [
            "fourier-epicycles",
            "lorenz",
            "voronoi",
            "bubble-sort",
            "game-of-life",
            "cubic-bezier",
        ];
        if let Some(f) = focus
            && formulas::tool(f).is_some()
            && !ids.contains(&f)
        {
            ids[0] = f;
        }
        let (gap, mut x, mut y, mut row_h) = (80.0, 0.0, 0.0, 0.0f32);
        for (k, id) in ids.iter().enumerate() {
            let tool = formulas::tool(id).expect("demo tool exists");
            let uid = board.add(tool, pos2(x, y), Vec2::ZERO, f64::NEG_INFINITY);
            let size = board.get(uid).unwrap().rect.size();
            row_h = row_h.max(size.y);
            x += size.x + gap;
            if k % 3 == 2 {
                x = 0.0;
                y += row_h + gap;
                row_h = 0.0;
            }
        }
        board
    }
}

/// A viewport as saved between sessions and in sheet files.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ViewportModel {
    pub uid: u64,
    pub tool: String,
    /// x, y, width, height.
    pub rect: [f32; 4],
    pub title: String,
    pub locked: bool,
    pub hidden: bool,
    /// The formula's own settings ([`Formula::save_state`]); absent in saves
    /// from before formulas kept their state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<serde_json::Value>,
}

/// An alignment found while dragging: a line where an edge or center of the
/// moving rect lines up with one of another viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    /// true: a vertical line at x = `at`; false: horizontal at y = `at`.
    pub vertical: bool,
    pub at: f32,
    /// Extent along the line, covering both rects.
    pub from: f32,
    pub to: f32,
}

/// Nudge `moving` so its edges or center line up with any of `others` within
/// `tolerance` (world units). Returns the offset to apply and the guides.
pub fn align(moving: Rect, others: &[Rect], tolerance: f32) -> (Vec2, Vec<Guide>) {
    let xs = |r: Rect| [r.left(), r.center().x, r.right()];
    let ys = |r: Rect| [r.top(), r.center().y, r.bottom()];
    let mut best_x: Option<(f32, f32)> = None; // (delta, line)
    let mut best_y: Option<(f32, f32)> = None;
    for o in others {
        for a in xs(moving) {
            for b in xs(*o) {
                let d = b - a;
                if d.abs() <= tolerance && best_x.is_none_or(|(bd, _)| d.abs() < bd.abs()) {
                    best_x = Some((d, b));
                }
            }
        }
        for a in ys(moving) {
            for b in ys(*o) {
                let d = b - a;
                if d.abs() <= tolerance && best_y.is_none_or(|(bd, _)| d.abs() < bd.abs()) {
                    best_y = Some((d, b));
                }
            }
        }
    }
    let offset = vec2(
        best_x.map_or(0.0, |(d, _)| d),
        best_y.map_or(0.0, |(d, _)| d),
    );
    let moved = moving.translate(offset);
    let mut guides = Vec::new();
    // Every rect that touches the chosen line contributes to its extent.
    if let Some((_, x)) = best_x {
        let touching: Vec<Rect> = others
            .iter()
            .filter(|o| xs(**o).iter().any(|v| (v - x).abs() < 0.01))
            .copied()
            .collect();
        let span = touching.iter().fold(moved, |acc, r| acc.union(*r));
        guides.push(Guide {
            vertical: true,
            at: x,
            from: span.top(),
            to: span.bottom(),
        });
    }
    if let Some((_, y)) = best_y {
        let touching: Vec<Rect> = others
            .iter()
            .filter(|o| ys(**o).iter().any(|v| (v - y).abs() < 0.01))
            .copied()
            .collect();
        let span = touching.iter().fold(moved, |acc, r| acc.union(*r));
        guides.push(Guide {
            vertical: false,
            at: y,
            from: span.left(),
            to: span.right(),
        });
    }
    (offset, guides)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(id: &str) -> &'static Tool {
        formulas::tool(id).unwrap()
    }

    #[test]
    fn add_remove_insert_keep_order_and_uids() {
        let mut b = Board::default();
        let a = b.add(tool("lerp"), pos2(0.0, 0.0), Vec2::ZERO, 0.0);
        let c = b.add(tool("lissajous"), pos2(700.0, 0.0), Vec2::ZERO, 0.0);
        assert_ne!(a, c);
        assert_eq!(b.get(a).unwrap().title, "VP-01");
        b.select_only(a);
        let (i, vp) = b.remove(a).unwrap();
        assert!(b.selection.is_empty());
        assert_eq!(i, 0);
        b.insert(i, vp);
        assert_eq!(b.viewports[0].uid, a);
        let d = b.add(tool("lerp"), pos2(0.0, 0.0), Vec2::ZERO, 0.0);
        assert!(d > c);
    }

    #[test]
    fn default_size_respects_min_size() {
        let mut b = Board::default();
        let r = b.add(tool("sorting-race"), Pos2::ZERO, vec2(10.0, 10.0), 0.0);
        let vp = b.get(r).unwrap();
        assert!(vp.rect.width() >= vp.min_size().x && vp.rect.height() >= vp.min_size().y);
        let l = b.add(tool("lerp"), Pos2::ZERO, Vec2::ZERO, 0.0);
        assert_eq!(b.get(l).unwrap().rect.width(), DEFAULT_WIDTH);
    }

    #[test]
    fn hit_takes_the_topmost_and_z_order_changes() {
        let mut b = Board::default();
        let a = b.add(tool("lerp"), pos2(0.0, 0.0), vec2(400.0, 400.0), 0.0);
        let c = b.add(tool("lerp"), pos2(100.0, 100.0), vec2(400.0, 400.0), 0.0);
        assert_eq!(b.hit(pos2(150.0, 150.0)), Some(c));
        b.bring_to_front(a);
        assert_eq!(b.hit(pos2(150.0, 150.0)), Some(a));
        b.send_to_back(a);
        assert_eq!(b.hit(pos2(150.0, 150.0)), Some(c));
        b.get_mut(c).unwrap().hidden = true;
        assert_eq!(b.hit(pos2(150.0, 150.0)), Some(a));
        assert_eq!(b.hit(pos2(-5.0, 0.0)), None);
    }

    #[test]
    fn duplicate_copies_settings_nearby() {
        let mut b = Board::default();
        let a = b.add(tool("lissajous"), pos2(10.0, 20.0), vec2(500.0, 300.0), 0.0);
        b.get_mut(a)
            .unwrap()
            .formula
            .load_state(&serde_json::json!({"a": 7}));
        let d = b.duplicate(a, 1.0).unwrap();
        let (ra, rd) = (b.get(a).unwrap().rect, b.get(d).unwrap().rect);
        assert_eq!(ra.size(), rd.size());
        assert_eq!(rd.min, ra.min + vec2(40.0, 40.0));
        let copy = b.get(d).unwrap().formula.save_state().unwrap();
        assert_eq!(copy["a"], 7);
        assert_eq!(Some(copy), b.get(a).unwrap().formula.save_state());
    }

    #[test]
    fn model_round_trip() {
        let b = Board::demo(Some("mandelbrot"));
        assert_eq!(b.viewports[0].tool.id, "mandelbrot");
        let model = b.to_model();
        let json = serde_json::to_string(&model).unwrap();
        let back: Vec<ViewportModel> = serde_json::from_str(&json).unwrap();
        let b2 = Board::from_model(&back, ToolEnv::new());
        assert_eq!(b2.to_model(), model);
        // Demo viewports do not overlap.
        for (i, x) in b.viewports.iter().enumerate() {
            for y in &b.viewports[i + 1..] {
                assert!(
                    !x.rect.intersects(y.rect),
                    "{} overlaps {}",
                    x.title,
                    y.title
                );
            }
        }
    }

    #[test]
    fn saves_from_before_formula_state_still_load() {
        // A "board-v1" layout written before viewports carried `state`.
        let old = r#"[
            {"uid":1,"tool":"lorenz","rect":[0.0,0.0,640.0,480.0],"title":"VP-01","locked":false,"hidden":false},
            {"uid":4,"tool":"gone-tool","rect":[700.0,0.0,640.0,480.0],"title":"VP-04","locked":true,"hidden":false},
            {"uid":7,"tool":"bubble-sort","rect":[0.0,600.0,640.0,480.0],"title":"VP-07","locked":false,"hidden":true}
        ]"#;
        let model: Vec<ViewportModel> = serde_json::from_str(old).unwrap();
        assert!(model.iter().all(|m| m.state.is_none()));
        let b = Board::from_model(&model, ToolEnv::new());
        let ids: Vec<&str> = b.viewports.iter().map(|v| v.tool.id).collect();
        assert_eq!(ids, ["lorenz", "bubble-sort"]);
        assert!(b.viewports[1].hidden);
        // New viewports get uids after the largest saved one.
        let mut b = b;
        assert_eq!(b.add(tool("lerp"), Pos2::ZERO, Vec2::ZERO, 0.0), 8);
    }

    #[test]
    fn add_model_restores_state_and_can_take_a_fresh_uid() {
        let mut b = Board::default();
        let uid = b.add(tool("expression"), Pos2::ZERO, Vec2::ZERO, 0.0);
        b.get_mut(uid)
            .unwrap()
            .formula
            .load_state(&serde_json::json!({"rows": ["y = 3x"]}));
        let m = b.to_model().remove(0);
        assert!(m.state.is_some());
        let copy = b.add_model(&m, true, 0.0).unwrap();
        assert_ne!(copy, uid);
        assert_eq!(b.get(copy).unwrap().formula.save_state(), m.state);
        let mut bad = m.clone();
        bad.tool = "nope".into();
        assert_eq!(b.add_model(&bad, true, 0.0), None);
    }

    #[test]
    fn align_snaps_to_nearest_edge_or_center() {
        let other = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        // Left edge 3 units right of the other's right edge: snaps onto it.
        let moving = Rect::from_min_size(pos2(103.0, 250.0), vec2(50.0, 50.0));
        let (offset, guides) = align(moving, &[other], 6.0);
        assert_eq!(offset, vec2(-3.0, 0.0));
        assert_eq!(guides.len(), 1);
        assert!(guides[0].vertical && guides[0].at == 100.0);
        assert_eq!((guides[0].from, guides[0].to), (0.0, 300.0));
        // Center lines up vertically within tolerance.
        let moving = Rect::from_min_size(pos2(300.0, 27.0), vec2(40.0, 40.0));
        let (offset, _) = align(moving, &[other], 6.0);
        assert_eq!(offset, vec2(0.0, 3.0));
        // Too far: nothing.
        let (offset, guides) = align(moving.translate(vec2(0.0, 40.0)), &[other], 6.0);
        assert_eq!(offset, Vec2::ZERO);
        assert!(guides.is_empty());
    }
}
