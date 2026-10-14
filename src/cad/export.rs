//! Getting exported code out of the app: copy to the clipboard, download a
//! file in the browser, or save it next to the app on the desktop.

/// Copy text to the system clipboard.
pub fn copy(ctx: &egui::Context, text: &str) {
    ctx.copy_text(text.to_owned());
}

/// Whether [`save`] downloads (web) rather than writes a file (desktop).
pub const SAVE_LABEL: &str = if cfg!(target_arch = "wasm32") {
    "Download"
} else {
    "Save file"
};

/// What to tell the user about the result of [`save`].
pub fn saved_message(result: Result<String, String>) -> String {
    match result {
        Ok(path) if cfg!(target_arch = "wasm32") => format!("Downloaded {path}"),
        Ok(path) => format!("Saved {path}"),
        Err(e) => format!("Could not save: {e}"),
    }
}

/// Save `text` as `file_name` for the tool `id`. On the web this downloads the
/// file; on the desktop it writes `exports/<id>/<file_name>` and returns the
/// path.
pub fn save(id: &str, file_name: &str, text: &str) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let name = format!("{id}-{file_name}");
        download(&name, text).map_err(|e| format!("{e:?}"))?;
        Ok(name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = std::path::Path::new("exports").join(id);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(file_name);
        std::fs::write(&path, text).map_err(|e| e.to_string())?;
        Ok(path.display().to_string())
    }
}

#[cfg(target_arch = "wasm32")]
fn download(name: &str, text: &str) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast as _;
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let parts = js_sys_array(text);
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/plain;charset=utf-8");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    let a: web_sys::HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    web_sys::Url::revoke_object_url(&url)?;
    Ok(())
}

/// A one-element JS array holding the string, as `Blob` wants.
#[cfg(target_arch = "wasm32")]
fn js_sys_array(text: &str) -> wasm_bindgen::JsValue {
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(text));
    array.into()
}
