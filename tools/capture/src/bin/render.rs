//! `formula-render`: draws a Short from a storyboard.
//!
//! ```sh
//! formula-render tools                                   # the catalogue as JSON
//! formula-render <storyboard.json> [--out DIR] [--contact-only]
//! ```
//!
//! Writes `video.mp4` (silent, H.264, needs `ffmpeg` on the PATH),
//! `contact.png` (the middle of every scene, side by side) and
//! `thumbnail.png` into `--out` (default: the storyboard's folder). Progress
//! goes to stderr; a one-line JSON summary goes to stdout.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use egui_kittest::Harness;
use formula_capture::shorts::frame::Show;
use formula_capture::shorts::{Storyboard, catalog};
use formula_capture::{render, steps};
use image::RgbaImage;

/// Width of each still on the contact sheet.
const CONTACT_WIDTH: u32 = 270;
const CONTACT_GAP: u32 = 8;

struct Args {
    storyboard: PathBuf,
    out: PathBuf,
    contact_only: bool,
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut it = std::env::args().skip(1);
    let first = it
        .next()
        .ok_or("usage: formula-render tools | <storyboard.json> [--out DIR] [--contact-only]")?;
    if first == "tools" {
        println!("{}", catalog::json());
        return Ok(None);
    }
    let storyboard = PathBuf::from(first);
    let mut out = None;
    let mut contact_only = false;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out" => out = Some(PathBuf::from(it.next().ok_or("--out needs a folder")?)),
            "--contact-only" => contact_only = true,
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    let out = out.unwrap_or_else(|| {
        storyboard
            .parent()
            .map_or_else(|| ".".into(), Path::to_path_buf)
    });
    Ok(Some(Args {
        storyboard,
        out,
        contact_only,
    }))
}

fn ffmpeg(board: &Storyboard, out: &Path) -> Result<Child, String> {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
        ])
        .arg(format!("{}x{}", board.width, board.height))
        .arg("-r")
        .arg(board.fps.to_string())
        .args([
            "-i", "pipe:0", "-c:v", "libx264", "-preset", "medium", "-crf", "18",
        ])
        .args(["-pix_fmt", "yuv420p", "-movflags", "+faststart"])
        .arg(out)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start ffmpeg (is it on the PATH?): {e}"))
}

/// The stills side by side, each `CONTACT_WIDTH` wide.
fn contact_sheet(stills: &[RgbaImage]) -> RgbaImage {
    let scaled: Vec<RgbaImage> = stills
        .iter()
        .map(|s| {
            let h = (s.height() as f32 * CONTACT_WIDTH as f32 / s.width() as f32).round() as u32;
            image::imageops::resize(
                s,
                CONTACT_WIDTH,
                h.max(1),
                image::imageops::FilterType::Triangle,
            )
        })
        .collect();
    let h = scaled.iter().map(RgbaImage::height).max().unwrap_or(1);
    let n = scaled.len() as u32;
    let w = n * CONTACT_WIDTH + (n + 1) * CONTACT_GAP;
    let mut sheet = RgbaImage::from_pixel(w, h + 2 * CONTACT_GAP, image::Rgba([0, 0, 0, 255]));
    for (i, s) in scaled.iter().enumerate() {
        let x = CONTACT_GAP + i as u32 * (CONTACT_WIDTH + CONTACT_GAP);
        image::imageops::overlay(&mut sheet, s, x as i64, CONTACT_GAP as i64);
    }
    sheet
}

fn run(args: Args) -> Result<(), String> {
    let text = std::fs::read_to_string(&args.storyboard)
        .map_err(|e| format!("could not read {}: {e}", args.storyboard.display()))?;
    let board = Storyboard::parse(&text)?;
    std::fs::create_dir_all(&args.out)
        .map_err(|e| format!("could not create {}: {e}", args.out.display()))?;

    let dt = board.dt();
    let fps = board.fps as f32;
    let frames = board.frames();
    let last = frames.saturating_sub(1);
    let at = |t: f32| ((t * fps).round() as usize).min(last);
    let mids: Vec<usize> = board.midpoints().into_iter().map(at).collect();
    let thumb = at(board.thumbnail_time());

    let show = Show::new(board.clone());
    let mut warnings = Vec::new();
    if show.frame().cramped {
        warnings.push(format!(
            "{} is narrower than its minimum size at this frame size; try a landscape or square size",
            board.tool
        ));
    }
    let mut h = Harness::builder()
        .with_size(board.logical_size())
        .with_pixels_per_point(board.scale)
        .with_step_dt(dt)
        .wgpu()
        .build_ui_state(|ui, s: &mut Show| s.paint(ui), show);

    // Warmup: the formula runs, nothing is drawn to disk.
    let warm = board.warmup_seconds();
    for i in 0..steps(warm, dt) {
        h.state_mut().t = -warm + i as f32 * dt;
        h.step();
    }

    let video = args.out.join("video.mp4");
    let mut encoder = if args.contact_only {
        None
    } else {
        Some(ffmpeg(&board, &video)?)
    };
    let mut stills = Vec::new();
    let mut thumbnail = None;
    eprintln!(
        "rendering {} frames ({:.1} s) of {} at {}x{}",
        frames,
        board.total_seconds(),
        board.tool,
        board.width,
        board.height
    );
    for n in 0..frames {
        h.state_mut().t = n as f32 / fps;
        h.step();
        let wanted = mids.contains(&n) || n == thumb;
        if encoder.is_none() && !wanted {
            continue;
        }
        let image = render(&mut h);
        if (image.width(), image.height()) != (board.width, board.height) {
            return Err(format!(
                "rendered {}x{} instead of {}x{}",
                image.width(),
                image.height(),
                board.width,
                board.height
            ));
        }
        if let Some(child) = &mut encoder {
            child
                .stdin
                .as_mut()
                .expect("piped stdin")
                .write_all(image.as_raw())
                .map_err(|e| format!("ffmpeg stopped reading: {e}"))?;
        }
        for _ in mids.iter().filter(|&&m| m == n) {
            stills.push(image.clone());
        }
        if n == thumb {
            thumbnail = Some(image);
        }
        if n > 0 && n % board.fps as usize == 0 {
            eprintln!(
                "  {:>4.0} s of {:.0}",
                n as f32 / fps,
                board.total_seconds()
            );
        }
    }
    if let Some(mut child) = encoder.take() {
        drop(child.stdin.take());
        let status = child.wait().map_err(|e| format!("ffmpeg: {e}"))?;
        if !status.success() {
            return Err(format!("ffmpeg failed with {status}"));
        }
    }
    contact_sheet(&stills)
        .save(args.out.join("contact.png"))
        .map_err(|e| format!("contact.png: {e}"))?;
    if let Some(t) = thumbnail {
        t.save(args.out.join("thumbnail.png"))
            .map_err(|e| format!("thumbnail.png: {e}"))?;
    }
    for i in &h.state().overflow {
        warnings.push(format!(
            "scene {}'s caption is too long for three lines; shorten it",
            i + 1
        ));
    }
    for w in &warnings {
        eprintln!("warning: {w}");
    }
    let summary = serde_json::json!({
        "video": if args.contact_only { None } else { Some(video.display().to_string()) },
        "contact": args.out.join("contact.png").display().to_string(),
        "thumbnail": args.out.join("thumbnail.png").display().to_string(),
        "frames": frames,
        "seconds": board.total_seconds(),
        "width": board.width,
        "height": board.height,
        "warnings": warnings,
    });
    println!("{summary}");
    Ok(())
}

fn main() {
    let result = parse_args().and_then(|a| a.map_or(Ok(()), run));
    if let Err(e) = result {
        eprintln!("formula-render: {e}");
        std::process::exit(1);
    }
}
