//! Renders the README's pictures into `docs/media/`: a PNG of every tool, a
//! few animated GIFs, and the whole workbench (`hero.gif`, `inspector.png`).
//!
//! Run from the repository root (needs `ffmpeg` on the PATH for the GIFs):
//!
//! ```sh
//! cargo run --release --manifest-path tools/capture/Cargo.toml [-- <out_dir>]
//! ```
//!
//! Time advances a fixed `DT` per frame, so every run draws the same pictures.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use egui::{Pos2, Rect, Vec2, pos2, vec2};
use egui_kittest::Harness;
use formula::FormulaApp;
use formula::cad::board::Board;
use formula::cad::camera::Camera;
use formula::cad::canvas::{Settings, Workspace};
use formula::cad::inspector::Tab;
use formula::cad::sheet;
use formula::cad::theme::{self, Chrome};
use formula::formula::Palette;
use formula::formulas::{self, Tool};
use image::RgbaImage;

/// Seconds per frame. Formulas see it as `dt`.
const DT: f32 = 1.0 / 60.0;
/// Frames per second of the GIFs.
const GIF_FPS: usize = 12;
/// Tools shown as animations in the README, and each GIF's width.
const SHOWCASE: [(&str, u32); 6] = [
    ("fourier-epicycles", 420),
    ("lorenz", 420),
    ("sorting-race", 420),
    ("a-star", 420),
    ("game-of-life", 320),
    ("pythagorean-proof", 420),
];
/// The pathfinding tools search the maze from this example sheet.
const MAZE: &str = "Pathfinding lab";
/// The tools on the hero board, row by row.
const HERO: [&[&str]; 2] = [
    &["lorenz", "fourier-epicycles", "voronoi"],
    &["sorting-race", "a-star", "game-of-life"],
];
/// Canvas around a viewport: room for its title tab and the frame.
const PAD: Vec2 = vec2(16.0, 16.0);
const TAB: f32 = 26.0;
/// Extra canvas beyond the crop, so the title block lands outside it.
const SLACK: f32 = 260.0;

/// Seconds a tool runs before its still is taken.
fn warmup(id: &str) -> f32 {
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

/// A board whose shared grid is the maze from the `MAZE` example sheet.
fn board() -> Board {
    let mut board = Board::default();
    let (_, text) = sheet::examples()
        .into_iter()
        .find(|(title, _)| title == MAZE)
        .expect("the pathfinding example sheet");
    let (maze, _) = sheet::from_text(text).unwrap();
    board.env.load_state(&maze.shared);
    board
}

/// One viewport on an otherwise empty canvas.
struct Stage {
    ws: Workspace,
    chrome: Chrome,
    palette: Palette,
    themed: bool,
}

fn steps(seconds: f32) -> usize {
    (seconds / DT).round() as usize
}

/// A harness showing `tool` at its default size, and the part of the screen
/// to keep: the viewport with its title tab.
fn viewport(tool: &'static Tool) -> (Harness<'static, Stage>, Rect) {
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

fn crop(image: &RgbaImage, r: Rect) -> RgbaImage {
    let x = r.min.x.max(0.0) as u32;
    let y = r.min.y.max(0.0) as u32;
    let w = (r.width() as u32).min(image.width() - x);
    let h = (r.height() as u32).min(image.height() - y);
    image::imageops::crop_imm(image, x, y, w, h).to_image()
}

fn render<S>(h: &mut Harness<'_, S>) -> RgbaImage {
    h.render().unwrap_or_else(|e| panic!("render failed: {e}"))
}

/// Run `seconds`, keeping `GIF_FPS` frames a second, into a GIF `width` wide.
fn record<S>(h: &mut Harness<'_, S>, seconds: f32, area: Option<Rect>, out: &Path, width: u32) {
    let dir = out.with_extension("frames");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let every = steps(1.0 / GIF_FPS as f32).max(1);
    for (n, _) in (0..steps(seconds)).step_by(every).enumerate() {
        for _ in 0..every {
            h.step();
        }
        let mut frame = render(h);
        if let Some(r) = area {
            frame = crop(&frame, r);
        }
        frame.save(dir.join(format!("f{n:04}.png"))).unwrap();
    }
    let filter = format!(
        "scale={width}:-1:flags=lanczos,split[a][b];\
         [a]palettegen=max_colors=128:stats_mode=diff[p];\
         [b][p]paletteuse=dither=bayer:bayer_scale=4:diff_mode=rectangle"
    );
    let status = Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error", "-framerate"])
        .arg(GIF_FPS.to_string())
        .arg("-i")
        .arg(dir.join("f%04d.png"))
        .args(["-vf", &filter, "-loop", "0"])
        .arg(out)
        .status()
        .expect("ffmpeg must be on the PATH to make GIFs");
    assert!(status.success(), "ffmpeg failed for {}", out.display());
    fs::remove_dir_all(&dir).unwrap();
}

/// A still of every tool, and a GIF of the showcase ones.
fn tools(out: &Path) {
    for tool in formulas::TOOLS {
        let (mut h, area) = viewport(tool);
        h.run_steps(steps(warmup(tool.id)));
        let still = crop(&render(&mut h), area);
        still.save(out.join(format!("{}.png", tool.id))).unwrap();
        println!("{:<24} png {}x{}", tool.id, still.width(), still.height());
        if let Some(&(_, width)) = SHOWCASE.iter().find(|(id, _)| *id == tool.id) {
            let (mut h, area) = viewport(tool);
            h.run_steps(steps(warmup(tool.id) * 0.5));
            record(
                &mut h,
                3.0,
                Some(area),
                &out.join(format!("{}.gif", tool.id)),
                width,
            );
            println!("{:<24} gif", tool.id);
        }
    }
}

/// The hero board as sheet text, every tool at its default size. With
/// `inspector`, the Inspector is open on its Code tab.
fn hero_sheet(inspector: bool) -> String {
    let mut ws = Workspace::new(board(), Camera::default(), Settings::default());
    let mut y = 0.0;
    for row in HERO {
        let (mut x, mut bottom) = (0.0_f32, y);
        for id in row {
            let uid = ws.place(formulas::tool(id).unwrap(), Some(pos2(x, y)), Vec2::ZERO);
            let r = ws.board.get(uid).unwrap().rect;
            x = r.right() + 60.0;
            bottom = bottom.max(r.bottom());
        }
        y = bottom + 80.0;
    }
    ws.ui.dark = true;
    ws.ui.inspector = inspector;
    ws.ui.tab = if inspector {
        Tab::Code
    } else {
        Tab::Properties
    };
    sheet::to_text(&ws.sheet())
}

fn app(text: String, focus: Option<&'static str>) -> Harness<'static, FormulaApp> {
    Harness::builder()
        .with_size(vec2(1440.0, 820.0))
        .with_pixels_per_point(1.0)
        .with_step_dt(DT)
        .wgpu()
        .build_eframe(move |cc| {
            let mut app = FormulaApp::with_initial(cc, focus);
            assert!(app.open_sheet("hero", &text));
            app
        })
}

/// The whole workbench: an animated board, and the Inspector's Code tab.
fn workbench(out: &Path) {
    let mut h = app(hero_sheet(false), None);
    h.run_steps(steps(2.0));
    record(&mut h, 4.0, None, &out.join("hero.gif"), 960);
    println!("{:<24} gif", "hero");

    let mut h = app(hero_sheet(true), Some("lorenz"));
    h.run_steps(steps(3.0));
    render(&mut h).save(out.join("inspector.png")).unwrap();
    println!("{:<24} png", "inspector");
}

fn main() {
    let out: PathBuf = std::env::args_os()
        .nth(1)
        .map_or_else(|| "docs/media".into(), PathBuf::from);
    fs::create_dir_all(&out).unwrap();
    tools(&out);
    workbench(&out);
}
