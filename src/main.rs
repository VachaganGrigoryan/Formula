#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Formula")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([360.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Formula",
        options,
        Box::new(|cc| Ok(Box::new(formula::FormulaApp::new(cc)))),
    )
}

// On the web the entry point is `web::start` in the library.
#[cfg(target_arch = "wasm32")]
fn main() {}
