//! Where everything goes in a frame: pure geometry, no egui painting, so it
//! has tests that need no GPU.
//!
//! Top to bottom inside the content band: title and kicker, the equation
//! lines, the stage, the caption block, the footer. The band leaves the top
//! and bottom of a portrait frame free for the platforms' own buttons. The
//! caption block has a fixed height, so the stage never jumps between scenes.

use egui::{Rect, Vec2, pos2, vec2};

use crate::{PAD, TAB};

/// Longest caption the block reserves room for, in lines.
pub const CAPTION_LINES: usize = 3;
/// Most equation lines shown.
pub const EQUATION_LINES: usize = 3;
/// The stage never takes more than this share of the frame's height.
const STAGE_SHARE: f32 = 0.52;

/// Font sizes in points, scaled with the frame's width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fonts {
    pub unit: f32,
    pub title: f32,
    pub kicker: f32,
    pub equation: f32,
    pub heading: f32,
    pub caption: f32,
    pub footer: f32,
}

impl Fonts {
    /// A 540-point-wide layout (a Short at scale 2) gets the base sizes.
    pub fn for_width(width: f32) -> Self {
        let unit = (width / 540.0).clamp(0.75, 1.4);
        Self {
            unit,
            title: 30.0 * unit,
            kicker: 14.0 * unit,
            equation: 17.0 * unit,
            heading: 20.0 * unit,
            caption: 22.0 * unit,
            footer: 12.0 * unit,
        }
    }
}

/// Line height of a font size.
pub fn line(size: f32) -> f32 {
    (size * 1.3).round()
}

/// The frame's rectangles, in points.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub size: Vec2,
    /// Everything stays inside this band.
    pub content: Rect,
    pub title: Rect,
    pub kicker: Rect,
    /// Zero height when the storyboard shows no equations.
    pub equations: Rect,
    /// Where the formula paints (in chrome mode: where the viewport, its tab
    /// and padding go).
    pub stage: Rect,
    /// The stage is narrower than the tool's `min_size`: a very tall formula
    /// in a frame that cannot fit it at full width.
    pub cramped: bool,
    pub heading: Rect,
    pub caption: Rect,
    pub footer: Rect,
    pub fonts: Fonts,
}

/// What the layout needs to know about the tool.
pub struct Subject<'a> {
    /// `Formula::stage_height`.
    pub stage_height: &'a dyn Fn(f32) -> f32,
    /// `Formula::min_size`.
    pub min_size: Vec2,
    /// How many equation lines to make room for (0 to hide the block).
    pub equation_lines: usize,
    /// Reserve room for the viewport's title tab and frame.
    pub chrome: bool,
}

pub fn layout(size: Vec2, subject: &Subject) -> Frame {
    let portrait = size.y > size.x;
    let (top, bottom) = if portrait { (0.08, 0.18) } else { (0.05, 0.08) };
    let margin = (size.x * 0.06).round();
    let content = Rect::from_min_max(
        pos2(margin, (size.y * top).round()),
        pos2(size.x - margin, (size.y * (1.0 - bottom)).round()),
    );
    let fonts = Fonts::for_width(size.x);
    let gap = (10.0 * fonts.unit).round();
    let width = content.width();

    let mut y = content.top();
    let title = Rect::from_min_size(pos2(content.left(), y), vec2(width, line(fonts.title)));
    y = title.bottom();
    let kicker = Rect::from_min_size(pos2(content.left(), y), vec2(width, line(fonts.kicker)));
    y = kicker.bottom() + gap;

    let lines = subject.equation_lines.min(EQUATION_LINES);
    let eq_h = lines as f32 * (fonts.equation * 1.45).round();
    let equations = Rect::from_min_size(pos2(content.left(), y), vec2(width, eq_h));
    if lines > 0 {
        y = equations.bottom() + gap;
    }

    // The bottom blocks, measured up from the content's bottom edge.
    let footer = Rect::from_min_size(
        pos2(content.left(), content.bottom() - line(fonts.footer)),
        vec2(width, line(fonts.footer)),
    );
    let caption_h = CAPTION_LINES as f32 * line(fonts.caption);
    let caption = Rect::from_min_size(
        pos2(content.left(), footer.top() - gap - caption_h),
        vec2(width, caption_h),
    );
    let heading = Rect::from_min_size(
        pos2(content.left(), caption.top() - line(fonts.heading)),
        vec2(width, line(fonts.heading)),
    );

    // The stage takes what is left, up to its share of the frame, keeping the
    // formula's own aspect ratio.
    let band = Rect::from_min_max(
        pos2(content.left(), y),
        pos2(content.right(), heading.top() - gap),
    );
    let extra = if subject.chrome {
        2.0 * PAD.y + TAB
    } else {
        0.0
    };
    let avail = (band.height() - extra)
        .min(size.y * STAGE_SHARE - extra)
        .max(1.0);
    let mut w = width;
    let mut h = (subject.stage_height)(w);
    if h > avail {
        w = (w * avail / h).floor();
        h = (subject.stage_height)(w).min(avail);
    }
    let cramped = w < subject.min_size.x;
    let stage_size = vec2(w.round(), h.round() + extra);
    let stage = Rect::from_center_size(band.center(), stage_size);

    Frame {
        size,
        content,
        title,
        kicker,
        equations,
        stage,
        cramped,
        heading,
        caption,
        footer,
        fonts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn short(lines: usize, chrome: bool, k: f32) -> Frame {
        let stage_height = move |w: f32| w * k;
        layout(
            vec2(540.0, 960.0),
            &Subject {
                stage_height: &stage_height,
                min_size: vec2(360.0, 360.0 * k),
                equation_lines: lines,
                chrome,
            },
        )
    }

    fn stacked(f: &Frame) {
        let inside = |r: Rect| f.content.contains_rect(r.shrink(0.5));
        assert!(inside(f.title), "title {:?} in {:?}", f.title, f.content);
        assert!(inside(f.equations.shrink2(vec2(0.0, 0.0))) || f.equations.height() == 0.0);
        assert!(inside(f.stage), "stage {:?} in {:?}", f.stage, f.content);
        assert!(inside(f.caption) && inside(f.footer) && inside(f.heading));
        assert!(f.title.bottom() <= f.kicker.top() + 0.01);
        assert!(f.kicker.bottom() <= f.equations.top() + 0.01);
        assert!(f.equations.bottom() <= f.stage.top() + 0.01);
        assert!(f.stage.bottom() <= f.heading.top() + 0.01);
        assert!(f.heading.bottom() <= f.caption.top() + 0.01);
        assert!(f.caption.bottom() <= f.footer.top() + 0.01);
    }

    #[test]
    fn a_short_stacks_top_to_bottom_inside_the_safe_band() {
        let f = short(3, false, 0.75);
        stacked(&f);
        assert_eq!(f.content.top(), (960.0 * 0.08f32).round());
        assert_eq!(f.content.bottom(), (960.0 * 0.82f32).round());
        assert_eq!(f.content.left(), 32.0);
        assert_eq!(f.fonts.unit, 1.0);
        assert_eq!(f.fonts.title, 30.0);
        // Full width, at the formula's own aspect.
        assert_eq!(f.stage.width(), f.content.width());
        assert_eq!(f.stage.height(), (f.content.width() * 0.75).round());
        assert!((f.stage.center().x - 270.0).abs() < 0.01);
        assert_eq!(f.caption.height(), 3.0 * line(22.0));
    }

    #[test]
    fn a_tall_formula_is_capped_and_keeps_its_aspect() {
        let f = short(3, false, 2.0);
        stacked(&f);
        assert!(f.stage.height() <= 960.0 * STAGE_SHARE + 0.5);
        assert!(f.stage.width() < f.content.width());
        let aspect = f.stage.height() / f.stage.width();
        assert!((aspect - 2.0).abs() < 0.02, "{aspect}");
    }

    #[test]
    fn a_stage_below_the_tools_minimum_is_flagged() {
        assert!(!short(3, false, 0.75).cramped);
        let f = short(3, false, 1.6);
        stacked(&f);
        assert!(f.stage.width() < 360.0);
        assert!(f.cramped);
    }

    #[test]
    fn no_equations_moves_the_stage_up() {
        let with = short(3, false, 0.75);
        let without = short(0, false, 0.75);
        stacked(&without);
        assert_eq!(without.equations.height(), 0.0);
        assert!(without.stage.top() < with.stage.top());
    }

    #[test]
    fn chrome_reserves_the_tab_and_padding() {
        let plain = short(3, false, 0.75);
        let chrome = short(3, true, 0.75);
        stacked(&chrome);
        let inner = chrome.stage.height() - 2.0 * PAD.y - TAB;
        assert!((inner / chrome.stage.width() - 0.75).abs() < 0.02);
        assert!(chrome.stage.height() <= plain.stage.height() + 2.0 * PAD.y + TAB + 0.5);
    }

    #[test]
    fn landscape_and_square_fit_too() {
        for size in [vec2(1280.0, 720.0), vec2(540.0, 540.0), vec2(432.0, 768.0)] {
            let stage_height = |w: f32| w * 0.75;
            let f = layout(
                size,
                &Subject {
                    stage_height: &stage_height,
                    min_size: vec2(360.0, 270.0),
                    equation_lines: 2,
                    chrome: false,
                },
            );
            stacked(&f);
            assert!(f.stage.height() <= size.y * STAGE_SHARE + 0.5, "{size:?}");
        }
        assert_eq!(Fonts::for_width(1280.0).unit, 1.4);
        assert_eq!(Fonts::for_width(300.0).unit, 0.75);
    }
}
