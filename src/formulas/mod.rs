//! Every formula the app can visualize.
//!
//! Adding one: create a module here, implement [`Formula`] for its state,
//! and append it to [`registry`]. Nothing else in the app needs to change.

pub mod bezier;
pub mod distance;
pub mod easing;
pub mod force;
pub mod fourier;
pub mod lerp;
pub mod lissajous;
pub mod luminance;
pub mod mandelbrot;
pub mod normalize;
pub mod polar;
pub mod projection;
pub mod regression;
pub mod rotation;

use crate::formula::Formula;

// Sidebar groups. Categories appear in the order their first formula is registered.
pub const COORDINATES: &str = "Coordinate Systems & Projections";
pub const CURVES: &str = "Curves & Interpolation";
pub const PHYSICS: &str = "Physics & Network Layouts";
pub const SCALING: &str = "Data Scaling & Normalization";
pub const FRACTALS: &str = "Fractals & Generative Patterns";
pub const GEOMETRY: &str = "Geometry & Spatial Analysis";
pub const STATISTICS: &str = "Statistics & Trend Analysis";
pub const ANIMATION: &str = "Animation & Easing";
pub const COLOR: &str = "Color & Accessibility";

/// Shown on first launch, before a choice has been remembered.
pub const DEFAULT_ID: &str = "fourier-epicycles";

/// The catalogue, in the order shown in the sidebar.
pub fn registry() -> Vec<Box<dyn Formula>> {
    vec![
        Box::new(polar::Polar::default()),
        Box::new(projection::Projection::default()),
        Box::new(lerp::Lerp::default()),
        Box::new(bezier::Bezier::default()),
        Box::new(fourier::Epicycles::default()),
        Box::new(lissajous::Lissajous::default()),
        Box::new(force::ForceLayout::default()),
        Box::new(normalize::Normalize::default()),
        Box::new(mandelbrot::Mandelbrot::default()),
        Box::new(distance::Distance::default()),
        Box::new(rotation::Rotation::default()),
        Box::new(regression::Regression::default()),
        Box::new(easing::Easing::default()),
        Box::new(luminance::Luminance::default()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let all = registry();
        let mut ids: Vec<_> = all.iter().map(|f| f.id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
        assert!(ids.contains(&DEFAULT_ID));
    }

    #[test]
    fn categories_are_contiguous() {
        // The sidebar groups by first appearance; a category split across the
        // list would still group correctly, but keep the registry tidy anyway.
        let cats: Vec<_> = registry().iter().map(|f| f.category()).collect();
        let mut seen = Vec::new();
        for w in cats.windows(2) {
            if w[0] != w[1] {
                seen.push(w[0]);
                assert!(!seen.contains(&w[1]), "{} is split", w[1]);
            }
        }
    }

    /// Every character a formula shows in its text must exist in egui's bundled
    /// fonts, or it renders as an empty box.
    #[test]
    fn all_text_has_glyphs() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
        let font = egui::FontId::proportional(14.0);
        for f in registry() {
            let mut text = format!("{}{}{}{}", f.name(), f.category(), f.summary(), f.notes());
            for eq in f.equations() {
                text.push_str(&eq.text());
            }
            let missing: String = ctx.fonts_mut(|fonts| {
                text.chars()
                    .filter(|&c| !c.is_whitespace() && !fonts.has_glyph(&font, c))
                    .collect()
            });
            assert!(missing.is_empty(), "{}: no glyph for {missing:?}", f.id());
        }
    }
}
