//! Painting one frame of a Short: the typeset text around the formula, and
//! the formula itself, either bare on a card (clean look) or inside the
//! workbench viewport with its title tab (chrome look).

use std::sync::Arc;

use egui::text::LayoutJob;
use egui::{
    Align, Color32, CornerRadius, FontId, Galley, Pos2, Rect, Sense, Stroke, Vec2, pos2, vec2,
};
use formula::cad::camera::Camera;
use formula::cad::canvas::Workspace;
use formula::cad::theme::{self, Chrome};
use formula::formula::{Formula, Palette};
use formula::formulas::{self, Tool, ToolEnv};

use super::layout::{self, CAPTION_LINES, EQUATION_LINES, Frame, Subject};
use super::storyboard::{Cue, Storyboard};
use crate::{PAD, TAB, workspace_with};

/// Seconds a caption takes to fade in and out.
const FADE: f32 = 0.25;

/// What draws the formula.
enum Body {
    Clean {
        formula: Box<dyn Formula>,
        /// Keeps the shared sorting list and grid alive.
        _env: ToolEnv,
    },
    Chrome {
        ws: Box<Workspace>,
        uid: u64,
        chrome: Chrome,
    },
}

/// A storyboard being played: the formula, the clock, and the layout.
pub struct Show {
    pub board: Storyboard,
    body: Body,
    palette: Palette,
    frame: Frame,
    /// Seconds since the first frame (negative during the warmup).
    pub t: f32,
    /// The scene whose `state` was applied last.
    applied: Option<usize>,
    themed: bool,
    tool: &'static Tool,
    name: String,
    category: String,
    /// Captions too long for the block, reported once each.
    pub overflow: Vec<usize>,
}

impl Show {
    pub fn new(board: Storyboard) -> Self {
        let tool = formulas::tool(&board.tool).expect("validated tool");
        let dark = board.theme.is_dark();
        let palette = theme::palette(dark);
        let (body, probe): (Body, Box<dyn Formula>) = if board.chrome {
            let (ws, uid, _) = workspace_with(tool);
            let probe = (tool.make)(&ToolEnv::new());
            (
                Body::Chrome {
                    ws: Box::new(ws),
                    uid,
                    chrome: Chrome::for_dark(dark),
                },
                probe,
            )
        } else {
            let env = ToolEnv::new();
            let formula = (tool.make)(&env);
            let probe = (tool.make)(&env);
            (Body::Clean { formula, _env: env }, probe)
        };
        let shows_equations = board.scenes.iter().any(|s| s.equations);
        let equation_lines = if shows_equations {
            probe.equations().len().min(EQUATION_LINES)
        } else {
            0
        };
        let stage_height = |w: f32| probe.stage_height(w);
        let frame = layout::layout(
            board.logical_size(),
            &Subject {
                stage_height: &stage_height,
                min_size: probe.min_size(),
                equation_lines,
                chrome: board.chrome,
            },
        );
        let mut show = Self {
            body,
            palette,
            frame,
            t: -board.warmup_seconds(),
            applied: None,
            themed: false,
            tool,
            name: tool.name.to_string(),
            category: tool.category.to_string(),
            overflow: Vec::new(),
            board,
        };
        show.fit_chrome_viewport();
        show
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    /// In the chrome look the viewport is resized to fill the stage band at
    /// zoom 1, so the formula draws at its natural size.
    fn fit_chrome_viewport(&mut self) {
        let stage = self.frame.stage;
        if let Body::Chrome { ws, uid, .. } = &mut self.body {
            let inner = vec2(
                stage.width() - 2.0 * PAD.x,
                stage.height() - 2.0 * PAD.y - TAB,
            );
            let vp = ws.board.get_mut(*uid).expect("placed viewport");
            vp.rect = Rect::from_min_size(Pos2::ZERO, inner.max(Vec2::splat(1.0)));
            // A fresh camera: `place` may have started a glide toward the
            // viewport's old rect. The view's center lands on the viewport,
            // shifted up by half the tab so the tab is inside the band too.
            let center = vp.rect.center() - vec2(0.0, TAB / 2.0);
            ws.camera = Camera::default();
            ws.camera.center = center;
            ws.camera.zoom = 1.0;
            ws.title_block = false;
        }
    }

    fn formula_mut(&mut self) -> &mut dyn Formula {
        match &mut self.body {
            Body::Clean { formula, .. } => formula.as_mut(),
            Body::Chrome { ws, uid, .. } => ws
                .board
                .get_mut(*uid)
                .expect("placed viewport")
                .formula
                .as_mut(),
        }
    }

    /// Apply the `state` of the scene showing at `t`, once per scene.
    fn apply_scene(&mut self) {
        let index = self.board.active_scene(self.t.max(0.0));
        if self.applied == Some(index) {
            return;
        }
        self.applied = Some(index);
        let scene = &self.board.scenes[index];
        // The first scene starts with the fresh instance already.
        if scene.restart && index > 0 {
            let make = self.tool.make;
            match &mut self.body {
                Body::Clean { formula, _env } => *formula = make(_env),
                Body::Chrome { ws, uid, .. } => {
                    let fresh = make(&ws.board.env);
                    ws.board.get_mut(*uid).expect("placed viewport").formula = fresh;
                }
            }
        }
        if let Some(state) = self.board.scenes[index].state.clone() {
            self.formula_mut().load_state(&state);
        }
    }

    /// Draw the frame at `self.t` into the whole screen.
    pub fn paint(&mut self, ui: &mut egui::Ui) {
        if !self.themed {
            theme::apply(ui.ctx(), self.board.theme.is_dark());
            self.themed = true;
        }
        self.apply_scene();
        let screen = Rect::from_min_size(Pos2::ZERO, self.frame.size);
        let p = self.palette;
        let painter = ui.painter_at(screen);
        painter.rect_filled(screen, 0.0, p.paper);

        let f = self.frame.clone();
        let cue = self.board.cue(self.t);

        // Title and kicker. The chrome look shows the name on the viewport's
        // tab as well, so it keeps the title; both looks read the same.
        let title = self
            .board
            .title
            .clone()
            .unwrap_or_else(|| self.name.clone());
        let kicker = self
            .board
            .kicker
            .clone()
            .unwrap_or_else(|| self.category.clone());
        let g = fitted(
            &painter,
            &title,
            FontId::proportional(f.fonts.title),
            f.title.width(),
            p.ink,
        );
        bold(&painter, centered(f.title, &g), g, p.ink, f.fonts.unit);
        let g = fitted(
            &painter,
            &kicker.to_uppercase(),
            FontId::proportional(f.fonts.kicker),
            f.kicker.width(),
            p.x,
        );
        painter.galley(centered(f.kicker, &g), g, p.x);

        // Equations, while the current scene wants them.
        let scene = self.board.active_scene(self.t.max(0.0));
        if self.board.scenes[scene].equations && f.equations.height() > 0.0 {
            let lines = self.formula_mut().equations();
            let n = lines.len().min(EQUATION_LINES);
            let row = f.equations.height() / n.max(1) as f32;
            for (i, eq) in lines.iter().take(n).enumerate() {
                let mut size = f.fonts.equation;
                let mut g = painter.layout_job(eq.layout(&p, size));
                if g.size().x > f.equations.width() {
                    size = (size * f.equations.width() / g.size().x).max(9.0);
                    g = painter.layout_job(eq.layout(&p, size));
                }
                let slot = Rect::from_min_size(
                    pos2(f.equations.left(), f.equations.top() + i as f32 * row),
                    vec2(f.equations.width(), row),
                );
                painter.galley(centered(slot, &g), g, p.muted);
            }
        }

        // The formula.
        let dt = self.board.dt();
        match &mut self.body {
            Body::Clean { formula, .. } => {
                let stage = f.stage;
                painter.rect_filled(stage.expand(1.0), CornerRadius::same(6), p.rule);
                let sp = ui.painter_at(stage);
                sp.rect_filled(stage, CornerRadius::same(5), p.paper);
                let probe = ui.interact(stage, ui.id().with("short-stage"), Sense::hover());
                formula.interact(&probe, stage);
                formula.paint(&sp, stage, &p, dt);
            }
            Body::Chrome { ws, chrome, .. } => {
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(f.stage));
                child.set_clip_rect(f.stage);
                ws.show(&mut child, chrome, &p);
                painter.rect_stroke(
                    f.stage,
                    CornerRadius::same(4),
                    Stroke::new(1.0, p.rule),
                    egui::StrokeKind::Outside,
                );
            }
        }

        // The caption of the scene playing now.
        if let Some(Cue {
            index,
            local,
            remaining,
        }) = cue
        {
            let a = (local / FADE).min(remaining / FADE).clamp(0.0, 1.0);
            let s = &self.board.scenes[index];
            if !s.heading.is_empty() {
                let c = p.y.gamma_multiply(a);
                let g = fitted(
                    &painter,
                    &s.heading,
                    FontId::proportional(f.fonts.heading),
                    f.heading.width(),
                    c,
                );
                bold(&painter, centered(f.heading, &g), g, c, f.fonts.unit);
            }
            let c = p.ink.gamma_multiply(a);
            let (g, fits) = wrapped(
                &painter,
                &s.caption,
                f.fonts.caption,
                f.caption.width(),
                f.caption.height(),
                c,
            );
            if !fits && !self.overflow.contains(&index) {
                self.overflow.push(index);
            }
            let top = Rect::from_min_size(f.caption.min, vec2(f.caption.width(), g.size().y));
            painter.galley(centered(top, &g), g, c);
        }

        // Footer and a progress line over the scenes.
        let g = fitted(
            &painter,
            &self.board.footer,
            FontId::proportional(f.fonts.footer),
            f.footer.width(),
            p.muted,
        );
        painter.galley(centered(f.footer, &g), g, p.muted);
        let total = self.board.total_seconds();
        let bar = Rect::from_min_size(
            pos2(f.content.left(), f.footer.top() - 6.0 * f.fonts.unit),
            vec2(f.content.width(), 2.0 * f.fonts.unit),
        );
        painter.rect_filled(bar, CornerRadius::same(1), p.rule);
        let done = (self.t / total).clamp(0.0, 1.0);
        painter.rect_filled(
            Rect::from_min_size(bar.min, vec2(bar.width() * done, bar.height())),
            CornerRadius::same(1),
            p.x,
        );
    }
}

/// Where to put `g` so it is centered in `slot`.
fn centered(slot: Rect, g: &Galley) -> Pos2 {
    slot.center() - g.rect.center().to_vec2()
}

/// One line of `text`, shrunk (down to 60%) to fit `width`.
fn fitted(
    painter: &egui::Painter,
    text: &str,
    font: FontId,
    width: f32,
    color: Color32,
) -> Arc<Galley> {
    let g = painter.layout_no_wrap(text.to_string(), font.clone(), color);
    if g.size().x <= width {
        return g;
    }
    let size = (font.size * width / g.size().x).max(font.size * 0.6);
    painter.layout_no_wrap(text.to_string(), FontId::new(size, font.family), color)
}

/// `text` wrapped to `width` and centered line by line, shrunk until it fits
/// `height` (down to 70%). Also says whether it fit.
fn wrapped(
    painter: &egui::Painter,
    text: &str,
    size: f32,
    width: f32,
    height: f32,
    color: Color32,
) -> (Arc<Galley>, bool) {
    let make = |size: f32| {
        let mut job = LayoutJob::simple(text.to_string(), FontId::proportional(size), color, width);
        job.halign = Align::Center;
        job.wrap.max_rows = CAPTION_LINES + 2;
        painter.layout_job(job)
    };
    let mut s = size;
    loop {
        let g = make(s);
        let fits = g.size().y <= height + 0.5 && !g.elided;
        if fits || s <= size * 0.7 {
            return (g, fits);
        }
        s *= 0.92;
    }
}

/// The default font has no bold face; draw twice, a hair apart.
fn bold(painter: &egui::Painter, pos: Pos2, g: Arc<Galley>, color: Color32, unit: f32) {
    painter.galley(pos, g.clone(), color);
    painter.galley(pos + vec2(0.6 * unit, 0.0), g, color);
}
