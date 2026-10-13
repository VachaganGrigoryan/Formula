//! Browser entry point: `wasm-bindgen` calls [`start`] when the module loads.

use wasm_bindgen::JsCast as _;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        // `#mandelbrot` opens that formula directly.
        let initial = window
            .location()
            .hash()
            .ok()
            .map(|h| h.trim_start_matches('#').to_owned())
            .filter(|h| !h.is_empty());
        let canvas = document
            .get_element_by_id("formula_canvas")
            .expect("missing #formula_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#formula_canvas is not a canvas");
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| {
                    Ok(Box::new(crate::FormulaApp::with_initial(
                        cc,
                        initial.as_deref(),
                    )))
                }),
            )
            .await;
        if let Err(e) = result {
            web_sys::console::error_1(&e);
        }
    });
}

/// Keep the address bar's `#id` in step with the selected formula, so links can be shared.
pub fn show_in_url(id: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let wanted = format!("#{id}");
    if window.location().hash().ok().as_deref() == Some(wanted.as_str()) {
        return;
    }
    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&wanted));
    }
}
