//! Formula: an app for visualizing formulas.
//!
//! Each formula lives in [`formulas`] and implements [`formula::Formula`];
//! [`app::FormulaApp`] lists them and renders the selected one.

pub mod app;
pub mod audio;
pub mod cad;
pub mod content;
pub mod draw;
pub mod formula;
pub mod formulas;
pub mod rng;

pub use app::FormulaApp;

#[cfg(target_arch = "wasm32")]
mod web;
