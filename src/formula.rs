//! The contract every visualized formula implements.
//!
//! The workspace owns layout (viewports on the canvas, and the Inspector with
//! the equations and controls); a formula only supplies its metadata, its
//! drawing and its own widgets. To add a new formula, implement [`Formula`]
//! and add it to [`crate::formulas::TOOLS`].

use std::sync::Arc;

use egui::text::LayoutJob;
use egui::{Align, Color32, FontId, TextFormat};

/// Colors shared by every formula, chosen for both light and dark themes.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub paper: Color32,
    pub card: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub rule: Color32,
    /// Accent for the x component / first series.
    pub x: Color32,
    /// Accent for the y component / second series.
    pub y: Color32,
    /// The traced result.
    pub trace: Color32,
    /// Translucent fill under the traced result.
    pub fill: Color32,
}

impl Palette {
    pub fn light() -> Self {
        Self {
            paper: Color32::from_rgb(0xE7, 0xEC, 0xE6),
            card: Color32::from_rgb(0xF3, 0xF5, 0xF1),
            ink: Color32::from_rgb(0x14, 0x24, 0x1F),
            muted: Color32::from_rgb(0x55, 0x66, 0x5F),
            rule: Color32::from_rgb(0xC6, 0xD0, 0xC8),
            x: Color32::from_rgb(0x23, 0x78, 0x6E),
            y: Color32::from_rgb(0xA8, 0x70, 0x0F),
            trace: Color32::from_rgb(0x23, 0x46, 0xA0),
            fill: Color32::from_rgba_unmultiplied(35, 70, 160, 26),
        }
    }

    pub fn dark() -> Self {
        Self {
            paper: Color32::from_rgb(0x0F, 0x17, 0x14),
            card: Color32::from_rgb(0x16, 0x20, 0x1C),
            ink: Color32::from_rgb(0xE4, 0xEB, 0xE6),
            muted: Color32::from_rgb(0x93, 0xA3, 0x9B),
            rule: Color32::from_rgb(0x2A, 0x38, 0x32),
            x: Color32::from_rgb(0x5C, 0xC2, 0xB2),
            y: Color32::from_rgb(0xE3, 0xA9, 0x3B),
            trace: Color32::from_rgb(0x9D, 0xB3, 0xFF),
            fill: Color32::from_rgba_unmultiplied(157, 179, 255, 31),
        }
    }

    pub fn for_dark_mode(dark: bool) -> Self {
        if dark { Self::dark() } else { Self::light() }
    }
}

/// Which palette accent a piece of an equation is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Muted,
    X,
    Y,
    Trace,
}

impl Tone {
    pub fn color(self, p: &Palette) -> Color32 {
        match self {
            Tone::Muted => p.muted,
            Tone::X => p.x,
            Tone::Y => p.y,
            Tone::Trace => p.trace,
        }
    }
}

/// One run of an equation's text.
#[derive(Clone, Debug)]
pub enum Piece {
    Text(String),
    Sub(String),
    Sup(String),
}

/// A small builder for typeset-looking equations: italic text with
/// subscripts and superscripts, optionally tinted.
#[derive(Clone, Debug, Default)]
pub struct MathText {
    pieces: Vec<(Piece, Option<Tone>)>,
}

impl MathText {
    pub fn new() -> Self {
        Self::default()
    }

    /// Plain (italic) text.
    pub fn t(mut self, s: &str) -> Self {
        self.pieces.push((Piece::Text(s.into()), None));
        self
    }

    /// Subscript.
    #[allow(clippy::should_implement_trait)]
    pub fn sub(mut self, s: &str) -> Self {
        self.pieces.push((Piece::Sub(s.into()), None));
        self
    }

    /// Superscript.
    pub fn sup(mut self, s: &str) -> Self {
        self.pieces.push((Piece::Sup(s.into()), None));
        self
    }

    /// Tint the most recently added piece.
    pub fn tone(mut self, tone: Tone) -> Self {
        if let Some(last) = self.pieces.last_mut() {
            last.1 = Some(tone);
        }
        self
    }

    /// The runs of text, in order, without their tints.
    pub fn pieces(&self) -> impl Iterator<Item = &Piece> {
        self.pieces.iter().map(|(p, _)| p)
    }

    /// The plain text of the equation, without formatting.
    pub fn text(&self) -> String {
        self.pieces
            .iter()
            .map(|(p, _)| match p {
                Piece::Text(s) | Piece::Sub(s) | Piece::Sup(s) => s.as_str(),
            })
            .collect()
    }

    pub fn layout(&self, palette: &Palette, size: f32) -> LayoutJob {
        let mut job = LayoutJob::default();
        for (piece, tone) in &self.pieces {
            let color = tone.map_or(palette.muted, |t| t.color(palette));
            let (text, font, valign) = match piece {
                Piece::Text(s) => (s, FontId::proportional(size), Align::Center),
                Piece::Sub(s) => (s, FontId::proportional(size * 0.68), Align::BOTTOM),
                Piece::Sup(s) => (s, FontId::proportional(size * 0.68), Align::TOP),
            };
            job.append(
                text,
                0.0,
                TextFormat {
                    font_id: font,
                    color,
                    italics: true,
                    valign,
                    ..Default::default()
                },
            );
        }
        job
    }
}

/// Lay out one line of `text`, shrinking the font (down to 9 points) when it
/// would be wider than `width`. Paint it with [`egui::Painter::galley`].
pub fn fit_text(
    painter: &egui::Painter,
    text: String,
    font: FontId,
    width: f32,
    color: Color32,
) -> Arc<egui::Galley> {
    let natural = painter.layout_no_wrap(text.clone(), font.clone(), color);
    if natural.size().x <= width {
        return natural;
    }
    let size = (font.size * width / natural.size().x).max(9.0);
    painter.layout_no_wrap(text, FontId::new(size, font.family), color)
}

/// Everything the shell needs from a formula.
pub trait Formula {
    /// Stable identifier (used for persistence and lookup).
    fn id(&self) -> &'static str;

    /// Display name, e.g. "Drawn by circles".
    fn name(&self) -> &'static str;

    /// Sidebar group, e.g. "Curves & Interpolation".
    fn category(&self) -> &'static str;

    /// One or two sentences shown under the title.
    fn summary(&self) -> &'static str;

    /// Where the formula is used in practice, shown under the equations.
    fn notes(&self) -> &'static str {
        ""
    }

    /// The equations being visualized, one per line.
    fn equations(&self) -> Vec<MathText>;

    /// The current values of the parameters in the equations, named as they
    /// appear there (`"θ"`, `"k_r"`; an underscore starts a subscript).
    fn params(&self) -> Vec<(&'static str, f64)> {
        Vec::new()
    }

    /// Height of the stage for a given width.
    fn stage_height(&self, width: f32) -> f32 {
        width * 0.75
    }

    /// What pointer input the stage listens to. Formulas that call
    /// [`Formula::interact`] for dragging or clicking ask for more.
    fn stage_sense(&self) -> egui::Sense {
        egui::Sense::hover()
    }

    /// Handle pointer input on the stage. Called just before [`Formula::paint`].
    fn interact(&mut self, _response: &egui::Response, _rect: egui::Rect) {}

    /// Paint the visualization into the stage. Called every frame;
    /// `dt` is the seconds elapsed since the previous frame.
    fn paint(&mut self, painter: &egui::Painter, rect: egui::Rect, palette: &Palette, dt: f32);

    /// Sliders and buttons shown under the stage.
    fn controls(&mut self, ui: &mut egui::Ui, palette: &Palette);

    /// Optional extra content under the equations (pickers, notes, credits).
    fn details(&mut self, _ui: &mut egui::Ui, _palette: &Palette) {}

    /// Whether the stage needs continuous repainting.
    fn is_animating(&self) -> bool {
        true
    }

    /// The smallest stage the formula lays out correctly in. The workspace
    /// will not let a viewport be resized below this. By default: 360 wide,
    /// and as tall as [`Formula::stage_height`] asks for at that width.
    fn min_size(&self) -> egui::Vec2 {
        egui::vec2(360.0, self.stage_height(360.0))
    }

    /// Keyboard shortcuts of the formula itself. Called once per frame, only
    /// for the selected viewport, before the workspace's own shortcuts, and
    /// never while a text field has keyboard focus. Consume the keys you use.
    fn handle_keys(&mut self, _ctx: &egui::Context) {}

    /// Whether the formula uses the mouse wheel and pinch itself while it is
    /// the active viewport (the workspace then leaves them alone over it).
    fn wants_scroll(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_text_shrinks_only_when_needed() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let painter = ui.painter();
            let font = FontId::monospace(12.0);
            let short = fit_text(painter, "abc".into(), font.clone(), 200.0, Color32::WHITE);
            assert!(short.size().x <= 200.0);
            assert_eq!(short.job.sections[0].format.font_id.size, 12.0);

            let long = "x".repeat(30);
            let fitted = fit_text(painter, long.clone(), font.clone(), 200.0, Color32::WHITE);
            assert!(fitted.size().x <= 201.0, "{}", fitted.size().x);
            assert!(fitted.job.sections[0].format.font_id.size < 12.0);

            // Never below 9 points, even if that still overflows.
            let tiny = fit_text(painter, long, font, 20.0, Color32::WHITE);
            assert_eq!(tiny.job.sections[0].format.font_id.size, 9.0);
        });
        out.textures_delta.clear();
    }
}
