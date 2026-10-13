//! Browser entry point: `wasm-bindgen` calls [`start`] when the module loads.

use wasm_bindgen::JsCast as _;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("no document");
        let canvas = document
            .get_element_by_id("formula_canvas")
            .expect("missing #formula_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#formula_canvas is not a canvas");
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(crate::FormulaApp::new(cc)))),
            )
            .await;
        if let Err(e) = result {
            web_sys::console::error_1(&e);
        }
    });
}
