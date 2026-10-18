//! Sheet files on disk (or as browser downloads): the current file, whether
//! the board has changed since it was saved, and the Recent list.
//!
//! The pure parts (the Recent list, save paths, the change check) are here;
//! `Workspace` does the opening and saving with them.

use super::fwb::SHEET_EXT;
use super::sheet::{self, Sheet, UiState};
use crate::cad::camera::Camera;

/// How many files File > Recent keeps.
pub const RECENT_MAX: usize = 8;
/// Larger files are refused before they are read.
pub const MAX_BYTES: u64 = 8 * 1024 * 1024;
/// How often the change check runs, in seconds.
const CHECK_EVERY: f64 = 0.5;

/// Put `path` first in the Recent list, without duplicates, keeping at most
/// [`RECENT_MAX`].
pub fn push_recent(list: &mut Vec<String>, path: &str) {
    list.retain(|p| p != path);
    list.insert(0, path.to_owned());
    list.truncate(RECENT_MAX);
}

pub fn remove_recent(list: &mut Vec<String>, path: &str) {
    list.retain(|p| p != path);
}

/// The last part of a path: `lessons/intro.fwb` gives `intro.fwb`.
pub fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_owned()
}

/// `name` with the sheet extension, unless it already has it.
pub fn with_extension(name: &str) -> String {
    if name.to_lowercase().ends_with(&format!(".{SHEET_EXT}")) {
        name.to_owned()
    } else {
        format!("{name}.{SHEET_EXT}")
    }
}

/// The file name to download a sheet under, from what was typed in the web
/// Save As window: no folders, and the sheet extension. `None` when empty.
pub fn download_name(input: &str) -> Option<String> {
    let name = file_name(input.trim());
    let name = name.trim();
    (!name.is_empty()).then(|| with_extension(name))
}

/// Where `save [name]` writes. A bare name goes in `exports/sheets/`, like
/// the other exports; a path is used as given. Without a name: the current
/// file, else `exports/sheets/untitled.fwb`.
pub fn resolve_save_path(name: Option<&str>, current: Option<&str>) -> String {
    match name.map(str::trim).filter(|n| !n.is_empty()) {
        Some(n) if n.contains(['/', '\\']) => with_extension(n),
        Some(n) => format!("exports/sheets/{}", with_extension(n)),
        None => current.map_or_else(
            || format!("exports/sheets/untitled.{SHEET_EXT}"),
            str::to_owned,
        ),
    }
}

/// What counts as a change for the "unsaved" mark: the sheet without its
/// camera and panels, so panning around or switching tabs does not count.
pub fn change_key(sheet: &Sheet) -> String {
    sheet::to_text(&Sheet {
        camera: Camera::default(),
        ui: UiState::default(),
        ..sheet.clone()
    })
}

/// A file the browser handed over: its name and contents (or why they could
/// not be read).
#[cfg(target_arch = "wasm32")]
pub type Inbox = std::rc::Rc<std::cell::RefCell<Option<(String, Result<Vec<u8>, String>)>>>;

/// Ask for a sheet to open (desktop: blocks until the dialog closes).
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_open() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Open sheet")
        .add_filter("Formula sheet", &[SHEET_EXT])
        .pick_file()
        .map(|p| p.display().to_string())
}

/// Ask where to save the sheet, starting from the current file.
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_save(current: Option<&str>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Save sheet")
        .add_filter("Formula sheet", &[SHEET_EXT])
        .set_file_name(current.map_or_else(|| format!("untitled.{SHEET_EXT}"), file_name));
    if let Some(dir) = current.and_then(|p| std::path::Path::new(p).parent())
        && !dir.as_os_str().is_empty()
    {
        dialog = dialog.set_directory(dir);
    }
    dialog.save_file().map(|p| p.display().to_string())
}

/// Ask the browser for a sheet; its contents arrive in `inbox`.
#[cfg(target_arch = "wasm32")]
pub fn pick_open(inbox: Inbox) {
    wasm_bindgen_futures::spawn_local(async move {
        let picked = rfd::AsyncFileDialog::new()
            .set_title("Open sheet")
            .add_filter("Formula sheet", &[SHEET_EXT])
            .pick_file()
            .await;
        if let Some(handle) = picked {
            let bytes = handle.read().await;
            *inbox.borrow_mut() = Some((handle.file_name(), Ok(bytes)));
        }
    });
}

#[derive(Default)]
pub struct Files {
    /// The current sheet: a path on the desktop, a file name on the web.
    pub path: Option<String>,
    /// The change key when the sheet was last opened or saved.
    saved: Option<String>,
    /// The board changed since it was opened or saved.
    pub dirty: bool,
    /// When the change check last ran (egui time).
    checked: f64,
    /// Recently opened or saved files, most recent first (desktop only).
    pub recent: Vec<String>,
    /// Where an asynchronous Open dialog puts the chosen file.
    #[cfg(target_arch = "wasm32")]
    pub inbox: Inbox,
}

impl Files {
    /// Remember `key` as the saved state: the sheet is clean again.
    pub fn mark_saved(&mut self, key: String) {
        self.saved = Some(key);
        self.dirty = false;
    }

    /// Whether the change check is due at time `now`.
    pub fn check_due(&self, now: f64) -> bool {
        now - self.checked >= CHECK_EVERY || now < self.checked
    }

    /// Run the change check with the current change key.
    pub fn check(&mut self, now: f64, key: &str) {
        self.checked = now;
        self.dirty = self.saved.as_deref() != Some(key);
    }

    /// The name shown in the window title, with a dot while unsaved.
    pub fn title(&self) -> String {
        let name = self
            .path
            .as_deref()
            .map_or_else(|| format!("untitled.{SHEET_EXT}"), file_name);
        if self.dirty {
            format!("{name} \u{2022}")
        } else {
            name
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_moves_duplicates_up_and_keeps_eight() {
        let mut list = Vec::new();
        for i in 0..9 {
            push_recent(&mut list, &format!("{i}.fwb"));
        }
        assert_eq!(list.len(), RECENT_MAX);
        assert_eq!(list[0], "8.fwb");
        assert!(!list.contains(&"0.fwb".to_owned()));
        push_recent(&mut list, "5.fwb");
        assert_eq!(list[0], "5.fwb");
        assert_eq!(list.iter().filter(|p| *p == "5.fwb").count(), 1);
        remove_recent(&mut list, "5.fwb");
        assert!(!list.contains(&"5.fwb".to_owned()));
        assert_eq!(list.len(), RECENT_MAX - 1);
    }

    #[test]
    fn download_names() {
        assert_eq!(download_name("  lesson ").as_deref(), Some("lesson.fwb"));
        assert_eq!(download_name("a/b.fwb").as_deref(), Some("b.fwb"));
        assert_eq!(download_name("x.FWB").as_deref(), Some("x.FWB"));
        assert_eq!(download_name(""), None);
        assert_eq!(download_name("   "), None);
        assert_eq!(download_name("dir/"), None);
    }

    #[test]
    fn save_paths() {
        assert_eq!(resolve_save_path(None, None), "exports/sheets/untitled.fwb");
        assert_eq!(resolve_save_path(None, Some("/a/b.fwb")), "/a/b.fwb");
        assert_eq!(
            resolve_save_path(Some("lesson"), Some("/a/b.fwb")),
            "exports/sheets/lesson.fwb"
        );
        assert_eq!(
            resolve_save_path(Some("x.FWB"), None),
            "exports/sheets/x.FWB"
        );
        assert_eq!(resolve_save_path(Some("/tmp/y"), None), "/tmp/y.fwb");
        assert_eq!(
            resolve_save_path(Some("  "), None),
            "exports/sheets/untitled.fwb"
        );
        assert_eq!(file_name("/a/b/c.fwb"), "c.fwb");
        assert_eq!(file_name("C:\\x\\d.fwb"), "d.fwb");
        assert_eq!(file_name("e.fwb"), "e.fwb");
    }

    #[test]
    fn the_title_shows_the_file_and_a_dot_while_unsaved() {
        let mut files = Files::default();
        assert_eq!(files.title(), "untitled.fwb");
        files.path = Some("/x/lesson.fwb".into());
        files.mark_saved("a".into());
        files.check(1.0, "a");
        assert_eq!(files.title(), "lesson.fwb");
        assert!(!files.check_due(1.2) && files.check_due(1.5));
        files.check(2.0, "b");
        assert_eq!(files.title(), "lesson.fwb \u{2022}");
    }
}
