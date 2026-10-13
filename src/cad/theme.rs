//! The workspace look: a dark graphite theme and a light drafting-paper one.
//!
//! [`Chrome`] colors the workspace itself (canvas, grid, panels, selection,
//! dimensions); [`palette`] gives the formulas their colors to match.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Visuals};

use crate::formula::Palette;

#[derive(Clone, Copy, Debug)]
pub struct Chrome {
    pub dark: bool,
    /// The drafting area behind everything.
    pub canvas: Color32,
    pub grid_minor: Color32,
    pub grid_major: Color32,
    /// The world x axis (red) and y axis (green), as in most CAD packages.
    pub axis_x: Color32,
    pub axis_y: Color32,
    /// Side panels, toolbars and the status bar.
    pub panel: Color32,
    /// Recessed areas: rulers, the command line, input fields.
    pub well: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    /// Selection, active tool, focus.
    pub accent: Color32,
    /// Dimension annotations (sizes, coordinates while dragging).
    pub dimension: Color32,
    /// Smart alignment guides.
    pub guide: Color32,
    /// The viewport body behind a formula.
    pub viewport: Color32,
    /// Viewport title tabs.
    pub tab: Color32,
}

impl Chrome {
    pub fn graphite() -> Self {
        Self {
            dark: true,
            canvas: Color32::from_rgb(0x1A, 0x1E, 0x23),
            grid_minor: Color32::from_rgb(0x22, 0x27, 0x2D),
            grid_major: Color32::from_rgb(0x2C, 0x33, 0x3B),
            axis_x: Color32::from_rgb(0xC8, 0x52, 0x4E),
            axis_y: Color32::from_rgb(0x5C, 0xB8, 0x6A),
            panel: Color32::from_rgb(0x23, 0x28, 0x2F),
            well: Color32::from_rgb(0x17, 0x1B, 0x20),
            border: Color32::from_rgb(0x36, 0x3E, 0x48),
            text: Color32::from_rgb(0xD6, 0xDD, 0xE4),
            text_dim: Color32::from_rgb(0x87, 0x93, 0xA0),
            accent: Color32::from_rgb(0x3F, 0xB6, 0xE8),
            dimension: Color32::from_rgb(0xE8, 0xA3, 0x3F),
            guide: Color32::from_rgb(0xE0, 0x5A, 0xA8),
            viewport: Color32::from_rgb(0x14, 0x18, 0x1C),
            tab: Color32::from_rgb(0x2B, 0x31, 0x39),
        }
    }

    pub fn paper() -> Self {
        Self {
            dark: false,
            canvas: Color32::from_rgb(0xF2, 0xEF, 0xE6),
            grid_minor: Color32::from_rgb(0xE6, 0xE1, 0xD4),
            grid_major: Color32::from_rgb(0xD6, 0xCE, 0xBC),
            axis_x: Color32::from_rgb(0xB0, 0x3A, 0x2E),
            axis_y: Color32::from_rgb(0x2E, 0x7D, 0x3A),
            panel: Color32::from_rgb(0xE9, 0xE5, 0xDA),
            well: Color32::from_rgb(0xF7, 0xF5, 0xEF),
            border: Color32::from_rgb(0xCC, 0xC4, 0xB1),
            text: Color32::from_rgb(0x2A, 0x26, 0x20),
            text_dim: Color32::from_rgb(0x6E, 0x67, 0x5B),
            accent: Color32::from_rgb(0x1F, 0x6F, 0xB2),
            dimension: Color32::from_rgb(0xC0, 0x60, 0x1A),
            guide: Color32::from_rgb(0xB0, 0x30, 0x7A),
            viewport: Color32::from_rgb(0xFB, 0xFA, 0xF6),
            tab: Color32::from_rgb(0xDD, 0xD7, 0xC8),
        }
    }

    pub fn for_dark(dark: bool) -> Self {
        if dark {
            Self::graphite()
        } else {
            Self::paper()
        }
    }
}

/// Colors for the formulas, tuned to sit inside the workspace's viewports.
pub fn palette(dark: bool) -> Palette {
    let c = Chrome::for_dark(dark);
    if dark {
        Palette {
            paper: c.viewport,
            card: Color32::from_rgb(0x1C, 0x21, 0x27),
            ink: Color32::from_rgb(0xE4, 0xEA, 0xF0),
            muted: Color32::from_rgb(0x93, 0xA0, 0xAC),
            rule: Color32::from_rgb(0x2E, 0x36, 0x3F),
            ..Palette::dark()
        }
    } else {
        Palette {
            paper: c.viewport,
            card: Color32::from_rgb(0xF6, 0xF4, 0xEC),
            ..Palette::light()
        }
    }
}

/// Monospace text style for numeric readouts.
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Compact, squared-off widgets in the workspace colors. Call when the theme
/// changes, not every frame.
pub fn apply(ctx: &egui::Context, dark: bool) {
    let c = Chrome::for_dark(dark);
    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.panel_fill = c.panel;
    visuals.window_fill = c.panel;
    visuals.extreme_bg_color = c.well;
    visuals.faint_bg_color = c.well;
    visuals.window_stroke = Stroke::new(1.0, c.border);
    visuals.window_corner_radius = CornerRadius::same(4);
    visuals.menu_corner_radius = CornerRadius::same(3);
    visuals.selection.bg_fill = c.accent.gamma_multiply(if dark { 0.45 } else { 0.3 });
    visuals.selection.stroke = Stroke::new(1.0, c.accent);
    visuals.hyperlink_color = c.accent;
    visuals.override_text_color = Some(c.text);
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(2);
    }
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, c.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, c.text_dim);
    visuals.widgets.inactive.weak_bg_fill = c.tab;
    visuals.widgets.inactive.bg_fill = c.tab;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, c.accent.gamma_multiply(0.7));
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, c.accent);
    let theme = if dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    };
    ctx.set_theme(theme);
    ctx.set_visuals_of(theme, visuals);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(6.0, 5.0);
        style.spacing.button_padding = egui::vec2(6.0, 3.0);
        style.spacing.interact_size.y = 20.0;
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(13.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(13.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(11.0));
        style.text_styles.insert(TextStyle::Monospace, mono(12.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(18.0));
    });
}
