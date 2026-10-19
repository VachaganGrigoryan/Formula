//! Headless rendering of the workbench and its tools: the README capture
//! (`formula-capture`, `src/main.rs`) uses it, and so can any other renderer
//! that depends on this crate. Keep its `pub` items stable.
//!
//! Everything here draws with `egui_kittest`'s wgpu renderer. Time advances a
//! fixed step per frame, so every run draws the same pictures.

use egui::{Pos2, Rect, Vec2, vec2};
use egui_kittest::Harness;
use formula::cad::board::Board;
use formula::cad::camera::Camera;
use formula::cad::canvas::{Settings, Workspace};
use formula::cad::sheet;
use formula::cad::theme::{self, Chrome};
use formula::formula::Palette;
use formula::formulas::Tool;
use image::RgbaImage;

/// Seconds per frame of the README capture. Formulas see it as `dt`.
pub const DT: f32 = 1.0 / 60.0;
/// The pathfinding tools search the maze from this example sheet.
pub const MAZE: &str = "Pathfinding lab";
/// Canvas around a viewport: room for its title tab and the frame.
pub const PAD: Vec2 = vec2(16.0, 16.0);
pub const TAB: f32 = 26.0;
/// Extra canvas beyond the crop, so the title block lands outside it.
const SLACK: f32 = 260.0;

/// Seconds a tool runs before it looks like itself: trails have grown,
/// searches have spread, sorts are half way.
pub fn warmup(id: &str) -> f32 {
    match id {
        "fourier-epicycles" => 9.0,
        "lorenz" | "flow-field" => 6.0,
        "game-of-life" => 4.0,
        id if id.ends_with("-sort") || id.ends_with("sort-3way") => 3.0,
        "bfs" | "dfs" | "dijkstra" | "a-star" => 5.0,
        "sorting-race" | "kruskal" | "prim" => 3.0,
        "binary-search-tree" => 3.0,
        _ => 1.5,
    }
}

/// A board whose shared grid is the maze from the [`MAZE`] example sheet.
pub fn board() -> Board {
    let mut board = Board::default();
    let (_, text) = sheet::examples()
        .into_iter()
        .find(|(title, _)| title == MAZE)
        .expect("the pathfinding example sheet");
    let (maze, _) = sheet::from_text(text).unwrap();
    board.env.load_state(&maze.shared);
    board
}

/// Number of `dt`-second frames in `seconds`.
pub fn steps(seconds: f32, dt: f32) -> usize {
    (seconds / dt).round() as usize
}

/// One viewport on an otherwise empty canvas.
pub struct Stage {
    pub ws: Workspace,
    pub chrome: Chrome,
    pub palette: Palette,
    pub themed: bool,
}

/// A workspace with `tool` placed at the origin at its default size, nothing
/// selected, rulers and minimap off, and the camera centered on it. Returns
/// the workspace, the viewport's uid and its world rect.
pub fn workspace_with(tool: &'static Tool) -> (Workspace, u64, Rect) {
    let settings = Settings {
        rulers: false,
        minimap: false,
        ..Settings::default()
    };
    let mut ws = Workspace::new(board(), Camera::default(), settings);
    let uid = ws.place(tool, Some(Pos2::ZERO), Vec2::ZERO);
    ws.board.selection.clear();
    let rect = ws.board.get(uid).unwrap().rect;
    ws.camera.center = rect.center();
    (ws, uid, rect)
}

/// A harness showing `tool` at its default size, and the part of the screen
/// to keep: the viewport with its title tab.
pub fn viewport(tool: &'static Tool) -> (Harness<'static, Stage>, Rect) {
    let (ws, _, rect) = workspace_with(tool);
    let size = rect.size() + 2.0 * PAD + vec2(SLACK, SLACK + TAB);
    let screen = Rect::from_min_size(Pos2::ZERO, size);
    let shown = Rect::from_center_size(screen.center(), rect.size());
    let crop = Rect::from_min_max(shown.min - PAD - vec2(0.0, TAB), shown.max + PAD);
    let stage = Stage {
        ws,
        chrome: Chrome::graphite(),
        palette: theme::palette(true),
        themed: false,
    };
    let harness = Harness::builder()
        .with_size(size)
        .with_pixels_per_point(1.0)
        .with_step_dt(DT)
        .wgpu()
        .build_ui_state(
            |ui, s: &mut Stage| {
                if !s.themed {
                    theme::apply(ui.ctx(), true);
                    s.themed = true;
                }
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ui, |ui| s.ws.show(ui, &s.chrome, &s.palette));
            },
            stage,
        );
    (harness, crop)
}

/// The part of `image` inside `r` (clamped to the image).
pub fn crop(image: &RgbaImage, r: Rect) -> RgbaImage {
    let x = r.min.x.max(0.0) as u32;
    let y = r.min.y.max(0.0) as u32;
    let w = (r.width() as u32).min(image.width() - x);
    let h = (r.height() as u32).min(image.height() - y);
    image::imageops::crop_imm(image, x, y, w, h).to_image()
}

/// The current frame as an image, or a panic naming the renderer error.
pub fn render<S>(h: &mut Harness<'_, S>) -> RgbaImage {
    h.render().unwrap_or_else(|e| panic!("render failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_round_to_whole_frames() {
        assert_eq!(steps(1.0, DT), 60);
        assert_eq!(steps(0.5, 1.0 / 30.0), 15);
        assert_eq!(steps(0.0, DT), 0);
    }

    #[test]
    fn warmup_knows_the_slow_starters() {
        assert!(warmup("fourier-epicycles") > warmup("lerp"));
        assert_eq!(warmup("quick-sort-3way"), warmup("bubble-sort"));
    }
}
