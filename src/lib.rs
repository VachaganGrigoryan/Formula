//! Formula: an app for visualizing formulas.
//!
//! Each formula lives in [`formulas`] and implements [`formula::Formula`];
//! [`app::FormulaApp`] lists them and renders the selected one.

pub mod app;
pub mod audio;
pub mod cad;
pub mod content;
pub mod derive;
pub mod draw;
pub mod expr;
pub mod formula;
pub mod formulas;
pub mod latex;
pub mod rng;
pub mod state;

pub use app::FormulaApp;

#[cfg(target_arch = "wasm32")]
mod web;

/// The window, taskbar and dock icon (`assets/icon/formula.png`, drawn from the landing page logo).
#[cfg(not(target_arch = "wasm32"))]
pub fn app_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon/formula.png"))
        .expect("assets/icon/formula.png is a valid PNG")
}

#[cfg(test)]
mod tests {
    #[test]
    fn app_icon_is_a_square_rgba_image() {
        let icon = super::app_icon();
        assert!(icon.width >= 256 && icon.width == icon.height);
        assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
    }
}
