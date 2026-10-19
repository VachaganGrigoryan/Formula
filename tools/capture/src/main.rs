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

use egui::{Rect, Vec2, pos2, vec2};
use egui_kittest::Harness;
use formula::FormulaApp;
use formula::cad::camera::Camera;
use formula::cad::canvas::{Settings, Workspace};
use formula::cad::inspector::Tab;
use formula::cad::sheet;
use formula::formulas;
use formula_capture::{DT, board, crop, render, viewport, warmup};

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
/// The tools on the hero board, row by row.
const HERO: [&[&str]; 2] = [
    &["lorenz", "fourier-epicycles", "voronoi"],
    &["sorting-race", "a-star", "game-of-life"],
];

fn steps(seconds: f32) -> usize {
    formula_capture::steps(seconds, DT)
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
