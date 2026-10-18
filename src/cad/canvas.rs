//! The drafting canvas: viewports on a pannable, zoomable plane, and the
//! tools that place, select, move and resize them.
//!
//! Formulas are drawn at their natural size (one world unit = one point)
//! on their own layer, which egui scales and translates as a whole. All
//! workspace chrome is drawn on a second, untransformed layer above it.

use std::collections::HashMap;

use egui::emath::TSTransform;
use egui::{
    CursorIcon, Id, Key, LayerId, Modifiers, PointerButton, Pos2, Rect, Response, Sense, Stroke,
    UiBuilder, Vec2, pos2, vec2,
};

use super::board::{self, Board, Guide};
use super::camera::Camera;
use super::files::{self, Files};
use super::history::{Edit, History};
use super::icons;
use super::overlay;
use super::sheet::{self, Sheet, UiState};
use super::theme::Chrome;
use super::{export, notebook};
use crate::expr;
use crate::formula::Palette;
use crate::formulas::{self, Tool};
use crate::latex;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// Select, move and resize viewports; interact with the active one.
    Select,
    /// Drag anywhere to pan.
    Pan,
    /// Click to place a viewport of this tool, or drag out its rectangle.
    Place(&'static Tool),
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub grid: bool,
    pub snap: bool,
    pub ortho: bool,
    pub rulers: bool,
    pub minimap: bool,
    /// Animate every visible viewport, not only the selected ones.
    pub animate_all: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            grid: true,
            snap: true,
            ortho: false,
            rulers: true,
            minimap: true,
            animate_all: true,
        }
    }
}

/// Something the panels around the canvas should do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Request {
    Properties,
    Docs,
    Derive,
    Code,
}

/// A file dialog the chrome should open (commands cannot show dialogs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FileRequest {
    Open,
    SaveAs,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Handle {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Handle {
    const ALL: [Handle; 8] = [
        Handle::N,
        Handle::NE,
        Handle::E,
        Handle::SE,
        Handle::S,
        Handle::SW,
        Handle::W,
        Handle::NW,
    ];

    /// Which edges it moves: -1 the left/top, 1 the right/bottom, 0 neither.
    fn sides(self) -> (i8, i8) {
        match self {
            Handle::N => (0, -1),
            Handle::NE => (1, -1),
            Handle::E => (1, 0),
            Handle::SE => (1, 1),
            Handle::S => (0, 1),
            Handle::SW => (-1, 1),
            Handle::W => (-1, 0),
            Handle::NW => (-1, -1),
        }
    }

    fn at(self, r: Rect) -> Pos2 {
        let (sx, sy) = self.sides();
        let x = [r.left(), r.center().x, r.right()][(sx + 1) as usize];
        let y = [r.top(), r.center().y, r.bottom()][(sy + 1) as usize];
        pos2(x, y)
    }

    fn cursor(self) -> CursorIcon {
        match self {
            Handle::N | Handle::S => CursorIcon::ResizeVertical,
            Handle::E | Handle::W => CursorIcon::ResizeHorizontal,
            Handle::NE | Handle::SW => CursorIcon::ResizeNeSw,
            Handle::NW | Handle::SE => CursorIcon::ResizeNwSe,
        }
    }
}

enum Gesture {
    Move {
        start: Vec<(u64, Rect)>,
        grab: Pos2,
        guides: Vec<Guide>,
    },
    Resize {
        uid: u64,
        handle: Handle,
        start: Rect,
    },
    Place {
        tool: &'static Tool,
        from: Pos2,
    },
    /// Screen-space start of a selection rectangle.
    Marquee {
        from: Pos2,
        additive: bool,
    },
    /// Dragging in the navigator. The mapping is frozen at the start: the
    /// live one includes the visible area, which moves with the camera.
    Minimap {
        map: Rect,
        to_world: TSTransform,
    },
}

/// Responses from one frame, handled after everything is laid out.
struct Hit {
    uid: u64,
    body: Response,
    active: bool,
    /// The active formula uses drags itself, so the body cannot move it.
    formula_drags: bool,
}

pub struct Workspace {
    pub board: Board,
    pub camera: Camera,
    pub history: History,
    pub mode: Mode,
    pub settings: Settings,
    /// The pointer in world coordinates, while over the canvas.
    pub pointer: Option<Pos2>,
    /// Messages for the command line's log, oldest first.
    pub log: Vec<String>,
    pub request: Option<Request>,
    /// Text for the system clipboard, copied at the start of the next frame
    /// (commands run without access to egui).
    pub clipboard: Option<String>,
    /// The canvas rect last frame.
    pub view: Rect,
    /// The panels around the canvas, mirrored by the app each frame so a
    /// sheet can save them, and set by opening a sheet (`ui_changed`).
    pub ui: UiState,
    pub ui_changed: bool,
    /// The current sheet file, its saved state and the Recent list.
    pub files: Files,
    pub file_request: Option<FileRequest>,
    gesture: Option<Gesture>,
    panning: bool,
    now: f64,
    default_sizes: HashMap<&'static str, Vec2>,
}

impl Workspace {
    pub fn new(board: Board, camera: Camera, settings: Settings) -> Self {
        Self {
            board,
            camera,
            history: History::default(),
            mode: Mode::Select,
            settings,
            pointer: None,
            log: Vec::new(),
            request: None,
            clipboard: None,
            view: Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0)),
            ui: UiState::default(),
            ui_changed: false,
            files: Files::default(),
            file_request: None,
            gesture: None,
            panning: false,
            now: 0.0,
            default_sizes: HashMap::new(),
        }
    }

    pub fn say(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
        if self.log.len() > 100 {
            self.log.remove(0);
        }
    }

    /// The viewport whose formula receives input: the primary selection.
    pub fn active(&self) -> Option<u64> {
        match self.mode {
            Mode::Select => self.board.primary(),
            _ => None,
        }
    }

    fn default_size(&mut self, tool: &'static Tool) -> Vec2 {
        let env = &self.board.env;
        *self
            .default_sizes
            .entry(tool.id)
            .or_insert_with(|| board::default_size((tool.make)(env).as_ref()))
    }

    fn grid_step(&self) -> f32 {
        self.camera.grid_step()
    }

    fn snap(&self, p: Pos2) -> Pos2 {
        if self.settings.snap {
            Camera::snap(p, self.grid_step())
        } else {
            p.round()
        }
    }

    // ---- Sheets.

    /// Everything a sheet file saves.
    pub fn sheet(&self) -> Sheet {
        Sheet {
            title: None,
            camera: self.camera,
            settings: self.settings,
            ui: self.ui,
            shared: self.board.env.save_state(),
            viewports: self.board.to_model(),
        }
    }

    /// Replace the board with the sheet in `text`, as one undo step. A sheet
    /// that cannot be read changes nothing. `name` is for the messages.
    pub fn open_text(&mut self, name: &str, text: &str) -> bool {
        let (sheet, warnings) = match sheet::from_text(text) {
            Ok(read) => read,
            Err(e) => {
                self.say(format!("Could not open {name}: {e}"));
                return false;
            }
        };
        // The shared data changes in place: viewports brought back by undo
        // see the sheet's list and grid, not a stale copy.
        self.board.env.load_state(&sheet.shared);
        let mut edits = Vec::new();
        let uids: Vec<u64> = self.board.viewports.iter().map(|v| v.uid).collect();
        for uid in uids {
            if let Some((index, vp)) = self.board.remove(uid) {
                edits.push(Edit::Removed {
                    index,
                    vp: Box::new(vp),
                });
            }
        }
        for m in &sheet.viewports {
            if let Some(uid) = self.board.add_model(m, true, self.now) {
                edits.push(Edit::Added { uid });
            }
        }
        self.history.push(Edit::Group(edits));
        self.board.selection.clear();
        self.gesture = None;
        self.camera = Camera::default();
        self.camera.center = sheet.camera.center;
        self.camera.zoom = sheet.camera.zoom;
        self.settings = sheet.settings;
        self.ui = sheet.ui;
        self.ui_changed = true;
        match warnings.len() {
            0 => self.say(format!("Opened {name}")),
            n => {
                self.say(format!(
                    "Opened {name}, {n} warning{}:",
                    if n == 1 { "" } else { "s" }
                ));
                for w in warnings {
                    self.say(format!("  {w}"));
                }
            }
        }
        true
    }

    /// Open a sheet from its bytes (a dropped file, or one the browser
    /// handed over). On success it becomes the current file under `name`.
    pub fn open_bytes(&mut self, name: &str, bytes: &[u8]) -> bool {
        if bytes.len() as u64 > files::MAX_BYTES {
            self.say(format!("Could not open {name}: the file is too large"));
            return false;
        }
        let Ok(text) = std::str::from_utf8(bytes) else {
            self.say(format!(
                "Could not open {name}: {}",
                super::fwb::FwbError::NotASheet
            ));
            return false;
        };
        if !self.open_text(name, text) {
            return false;
        }
        self.files.path = Some(name.to_owned());
        self.mark_saved();
        true
    }

    /// Open the sheet at `path` and remember it under File > Recent. A file
    /// that is gone is dropped from the list.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_path(&mut self, path: &str) -> bool {
        let name = files::file_name(path);
        let read = std::fs::metadata(path).and_then(|m| {
            if m.len() > files::MAX_BYTES {
                Err(std::io::Error::other("the file is too large"))
            } else {
                std::fs::read(path)
            }
        });
        let bytes = match read {
            Ok(bytes) => bytes,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    files::remove_recent(&mut self.files.recent, path);
                    self.say(format!("Could not open {path}: the file is gone"));
                } else {
                    self.say(format!("Could not open {path}: {e}"));
                }
                return false;
            }
        };
        if !self.open_bytes(&name, &bytes) {
            return false;
        }
        self.files.path = Some(path.to_owned());
        files::push_recent(&mut self.files.recent, path);
        true
    }

    /// Save the sheet to `path` (on the web: download it under that name),
    /// which becomes the current file.
    pub fn save_to(&mut self, path: &str) -> bool {
        let path = files::with_extension(path);
        match export::save_sheet(&path, &sheet::to_text(&self.sheet())) {
            Ok(shown) => {
                self.say(export::saved_message(Ok(shown)));
                if !cfg!(target_arch = "wasm32") {
                    files::push_recent(&mut self.files.recent, &path);
                }
                self.files.path = Some(path);
                self.mark_saved();
                true
            }
            Err(e) => {
                self.say(export::saved_message(Err(e)));
                false
            }
        }
    }

    /// Save to the current file, or ask where when there is none (Ctrl+S).
    pub fn save(&mut self) {
        match self.files.path.clone() {
            Some(path) if !cfg!(target_arch = "wasm32") => {
                self.save_to(&path);
            }
            _ => self.file_request = Some(FileRequest::SaveAs),
        }
    }

    /// The board as it is now counts as saved.
    pub fn mark_saved(&mut self) {
        let key = files::change_key(&self.sheet());
        self.files.mark_saved(key.clone());
        self.files.check(self.now, &key);
    }

    /// Update the unsaved mark, at most twice a second.
    fn check_changes(&mut self) {
        if self.files.check_due(self.now) {
            let key = files::change_key(&self.sheet());
            self.files.check(self.now, &key);
        }
    }

    /// Open the first `.fwb` file dropped on the window.
    fn take_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        let Some(file) = dropped.first() else {
            return;
        };
        let path = file.path().display().to_string();
        let name = files::file_name(&path);
        if !name
            .to_lowercase()
            .ends_with(&format!(".{}", super::fwb::SHEET_EXT))
        {
            self.say(format!("{name} is not a .fwb sheet"));
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.open_path(&path);
        // Browsers only read files asynchronously; the contents arrive in the
        // inbox, which a later frame opens.
        #[cfg(target_arch = "wasm32")]
        {
            let (file, inbox) = (file.clone(), self.files.inbox.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let bytes = file.bytes_async().await;
                *inbox.borrow_mut() = Some((name, bytes));
            });
        }
    }

    // ---- Commands, shared by shortcuts, menus, panels and the command line.

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.gesture = None;
        if let Mode::Place(t) = mode {
            self.say(format!(
                "{}: click to place, or drag a rectangle. Esc cancels.",
                t.name
            ));
        }
    }

    /// Place a viewport. Without a position it goes to the right of the drawing.
    pub fn place(&mut self, tool: &'static Tool, at: Option<Pos2>, size: Vec2) -> u64 {
        let at = at.unwrap_or_else(|| self.snap(self.board.free_spot()));
        let uid = self.board.add(tool, at, size, self.now);
        self.history.push(Edit::Added { uid });
        self.board.select_only(uid);
        let vp = self.board.get(uid).unwrap();
        let (rect, label) = (vp.rect, vp.label());
        self.say(format!(
            "Placed {label} at {:.0}, {:.0}",
            rect.left(),
            rect.top()
        ));
        let seen = self.camera.rect_to_world(self.view, self.view);
        if !seen.contains_rect(rect) {
            self.camera.fly_to(
                self.view,
                rect.expand(40.0),
                self.camera.zoom.clamp(0.5, 1.0),
            );
        }
        uid
    }

    pub fn delete_selection(&mut self) {
        let uids: Vec<u64> = self.board.selection.clone();
        let mut edits = Vec::new();
        for uid in uids {
            if let Some((index, vp)) = self.board.remove(uid) {
                edits.push(Edit::Removed {
                    index,
                    vp: Box::new(vp),
                });
            }
        }
        if !edits.is_empty() {
            self.say(format!(
                "Deleted {} viewport{}",
                edits.len(),
                plural(edits.len())
            ));
            self.history.push(Edit::Group(edits));
        }
    }

    pub fn duplicate_selection(&mut self) {
        let uids: Vec<u64> = self.board.selection.clone();
        let mut new = Vec::new();
        let mut edits = Vec::new();
        for uid in uids {
            if let Some(d) = self.board.duplicate(uid, self.now) {
                edits.push(Edit::Added { uid: d });
                new.push(d);
            }
        }
        if !new.is_empty() {
            self.history.push(Edit::Group(edits));
            self.say(format!(
                "Duplicated {} viewport{}",
                new.len(),
                plural(new.len())
            ));
            self.board.selection = new;
        }
    }

    pub fn undo(&mut self) {
        match self.history.undo(&mut self.board) {
            Some(what) => self.say(format!("Undo: {what}")),
            None => self.say("Nothing to undo"),
        }
    }

    pub fn redo(&mut self) {
        match self.history.redo(&mut self.board) {
            Some(what) => self.say(format!("Redo: {what}")),
            None => self.say("Nothing to redo"),
        }
    }

    pub fn fit_all(&mut self) {
        if let Some(b) = self.board.bounds() {
            self.camera.fly_to(self.view, b.expand(30.0), 1.0);
        }
    }

    pub fn focus_selection(&mut self) {
        if let Some(b) = self.board.selection_bounds() {
            self.camera.fly_to(self.view, b.expand(30.0), 1.0);
        }
    }

    pub fn focus(&mut self, uid: u64) {
        if let Some(vp) = self.board.get(uid) {
            let r = vp.rect.expand(30.0);
            self.camera.fly_to(self.view, r, 1.0);
        }
    }

    pub fn zoom_by(&mut self, factor: f32) {
        self.camera.zoom_at(self.view, self.view.center(), factor);
    }

    pub fn select_all(&mut self) {
        self.board.selection = self
            .board
            .viewports
            .iter()
            .filter(|v| !v.hidden)
            .map(|v| v.uid)
            .collect();
    }

    pub fn nudge(&mut self, by: Vec2) {
        let mut changes = Vec::new();
        for uid in self.board.selection.clone() {
            if let Some(vp) = self.board.get_mut(uid)
                && !vp.locked
            {
                let before = vp.rect;
                vp.rect = vp.rect.translate(by);
                changes.push((uid, before, vp.rect));
            }
        }
        self.history.push(Edit::Reshaped(changes));
    }

    pub fn rename(&mut self, uid: u64, before: String) {
        if let Some(vp) = self.board.get(uid)
            && vp.title != before
        {
            let after = vp.title.clone();
            self.history.push(Edit::Renamed { uid, before, after });
        }
    }

    /// Record a geometry change made directly (e.g. from the inspector).
    pub fn reshaped(&mut self, uid: u64, before: Rect) {
        if let Some(vp) = self.board.get(uid) {
            self.history
                .push(Edit::Reshaped(vec![(uid, before, vp.rect)]));
        }
    }

    /// Run one command-line command. Returns false if it was not understood.
    pub fn run_command(&mut self, line: &str) -> bool {
        let line = line.trim();
        if line.is_empty() {
            return true;
        }
        self.say(format!("Command: {line}"));
        let mut words = line.split_whitespace();
        let cmd = words.next().unwrap_or("").to_lowercase();
        let arg: Vec<&str> = words.collect();
        match cmd.as_str() {
            "fit" | "zoomext" | "ze" => self.fit_all(),
            "focus" => self.focus_selection(),
            "zoom" | "z" => match arg
                .first()
                .and_then(|a| a.trim_end_matches('%').parse::<f32>().ok())
            {
                Some(p) => self.camera.zoom_to(p / 100.0),
                None => self.say("zoom <percent>, e.g. zoom 150"),
            },
            "del" | "delete" | "erase" | "e" => self.delete_selection(),
            "dup" | "copy" | "co" => self.duplicate_selection(),
            "undo" | "u" => self.undo(),
            "redo" => self.redo(),
            "all" => self.select_all(),
            "grid" => self.settings.grid = !self.settings.grid,
            "snap" => self.settings.snap = !self.settings.snap,
            "ortho" => self.settings.ortho = !self.settings.ortho,
            "select" | "v" => self.set_mode(Mode::Select),
            "pan" | "p" | "h" => self.set_mode(Mode::Pan),
            "docs" | "doc" => self.request = Some(Request::Docs),
            "derive" | "steps" => self.request = Some(Request::Derive),
            "code" => self.request = Some(Request::Code),
            "latex" | "tex" => self.copy_latex(),
            "ipynb" | "notebook" => self.save_notebook(),
            "plot" => {
                let src = line
                    .split_once(char::is_whitespace)
                    .map_or("", |(_, rest)| rest.trim());
                return self.plot(src);
            }
            "open" => {
                let path = line
                    .split_once(char::is_whitespace)
                    .map_or("", |(_, rest)| rest.trim());
                #[cfg(not(target_arch = "wasm32"))]
                if !path.is_empty() {
                    return self.open_path(path);
                }
                let _ = path;
                self.file_request = Some(FileRequest::Open);
            }
            "save" => {
                let name = line
                    .split_once(char::is_whitespace)
                    .map(|(_, rest)| rest.trim());
                let path = files::resolve_save_path(name, self.files.path.as_deref());
                return self.save_to(&path);
            }
            "clear" => {
                self.board.selection = self.board.viewports.iter().map(|v| v.uid).collect();
                self.delete_selection();
            }
            "help" | "?" => {
                self.say("Tools: type an alias (LOR, BEZ, MAN, VOR, LIF, BUB ...) or a tool name to place it.");
                self.say("Commands: fit, focus, zoom <pct>, del, dup, undo, redo, all, grid, snap, ortho, pan, select, docs, derive, code, latex, ipynb, plot <expr>, open [file], save [name], clear.");
            }
            _ => {
                // A tool: by alias, id or name. "place lorenz" works too.
                let query = if cmd == "place" || cmd == "add" {
                    arg.join(" ")
                } else {
                    line.to_owned()
                };
                match formulas::find_tool(&query) {
                    Some(tool) => {
                        self.place(tool, None, Vec2::ZERO);
                    }
                    None => {
                        self.say(format!("Unknown command \"{line}\". Type help."));
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Place an expression plotter showing `src`, e.g. `r = 1 + cos(3θ)`. A
    /// typo is reported with its column and nothing is placed.
    fn plot(&mut self, src: &str) -> bool {
        if src.is_empty() {
            self.say("plot <expression>, e.g. plot r = 1 + cos(3θ)");
            return true;
        }
        if let Err(e) = expr::parse(src) {
            let column = src[..e.span.start.min(src.len())].chars().count() + 1;
            self.say(format!("plot: {} at column {column}", e.message));
            return false;
        }
        let Some(tool) = formulas::tool("expression") else {
            return false;
        };
        let uid = self.place(tool, None, Vec2::ZERO);
        if let Some(vp) = self.board.get_mut(uid) {
            vp.formula.load_state(&serde_json::json!({ "rows": [src] }));
        }
        true
    }

    /// Put the selected viewport's equations, with its current values, on
    /// the clipboard as LaTeX.
    fn copy_latex(&mut self) {
        let Some(vp) = self.board.primary().and_then(|uid| self.board.get(uid)) else {
            self.say("latex: select a viewport first");
            return;
        };
        let text = latex::latex_snippet(&*vp.formula);
        let label = vp.label();
        self.clipboard = Some(text);
        self.say(format!("Copied the LaTeX for {label} to the clipboard"));
    }

    /// Export the selected viewport as a Jupyter notebook.
    fn save_notebook(&mut self) {
        let Some(vp) = self.board.primary().and_then(|uid| self.board.get(uid)) else {
            self.say("ipynb: select a viewport first");
            return;
        };
        let msg = match notebook::for_formula(vp.tool.id, &*vp.formula) {
            Some(nb) => export::saved_message(export::save(vp.tool.id, notebook::FILE_NAME, &nb)),
            None => format!("No notebook export for {}", vp.tool.name),
        };
        self.say(msg);
    }

    // ---- Per-frame.

    /// Keyboard shortcuts. The active formula gets first pick.
    fn keys(&mut self, ctx: &egui::Context) {
        // A text field has focus, or a menu is open (Esc should close it).
        if ctx.egui_wants_keyboard_input() || egui::Popup::is_any_open(ctx) {
            return;
        }
        if let Some(uid) = self.active()
            && let Some(vp) = self.board.get_mut(uid)
        {
            vp.formula.handle_keys(ctx);
        }
        let none = Modifiers::NONE;
        let cmd = Modifiers::COMMAND;
        let shift = Modifiers::SHIFT;
        let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
        let pressed = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));
        if pressed(none, Key::Escape) {
            if matches!(self.mode, Mode::Select) {
                self.board.selection.clear();
            }
            self.set_mode(Mode::Select);
        }
        if pressed(none, Key::Delete) || pressed(none, Key::Backspace) {
            self.delete_selection();
        }
        if pressed(cmd, Key::D) {
            self.duplicate_selection();
        }
        if pressed(cmd_shift, Key::Z) || pressed(cmd, Key::Y) {
            self.redo();
        }
        if pressed(cmd, Key::Z) {
            self.undo();
        }
        if pressed(cmd, Key::A) {
            self.select_all();
        }
        if pressed(shift, Key::F) {
            self.focus_selection();
        } else if pressed(none, Key::F) {
            self.fit_all();
        }
        if pressed(cmd, Key::Num0) {
            self.camera.zoom_to(1.0);
        }
        if pressed(none, Key::Plus) || pressed(none, Key::Equals) || pressed(shift, Key::Equals) {
            self.zoom_by(1.25);
        }
        if pressed(none, Key::Minus) {
            self.zoom_by(0.8);
        }
        if pressed(none, Key::V) {
            self.set_mode(Mode::Select);
        }
        if pressed(none, Key::H) {
            self.set_mode(Mode::Pan);
        }
        if pressed(none, Key::G) {
            self.settings.grid = !self.settings.grid;
        }
        if pressed(none, Key::S) {
            self.settings.snap = !self.settings.snap;
        }
        if pressed(none, Key::O) {
            self.settings.ortho = !self.settings.ortho;
        }
        for (key, d) in [
            (Key::ArrowLeft, vec2(-1.0, 0.0)),
            (Key::ArrowRight, vec2(1.0, 0.0)),
            (Key::ArrowUp, vec2(0.0, -1.0)),
            (Key::ArrowDown, vec2(0.0, 1.0)),
        ] {
            if !self.board.selection.is_empty() {
                if pressed(none, key) {
                    self.nudge(d * self.grid_step());
                } else if pressed(shift, key) {
                    self.nudge(d * self.grid_step() * 10.0);
                }
            }
        }
    }

    /// Draw the canvas and handle its input. `ui` is the central panel.
    pub fn show(&mut self, ui: &mut egui::Ui, chrome: &Chrome, palette: &Palette) {
        let ctx = ui.ctx().clone();
        let view = ui.max_rect();
        self.view = view;
        let (now, dt, ppp) = ctx.input(|i| (i.time, i.stable_dt.min(0.1), i.pixels_per_point));
        self.now = now;
        if let Some(text) = self.clipboard.take() {
            ctx.copy_text(text);
        }
        self.take_dropped_files(&ctx);
        #[cfg(target_arch = "wasm32")]
        {
            let picked = self.files.inbox.borrow_mut().take();
            match picked {
                Some((name, Ok(bytes))) => {
                    self.open_bytes(&name, &bytes);
                }
                Some((name, Err(e))) => self.say(format!("Could not read {name}: {e}")),
                None => {}
            }
        }
        self.check_changes();
        self.keys(&ctx);

        let (hover, shift, middle, delta, pinch, primary_down) = ctx.input(|i| {
            (
                i.pointer.hover_pos(),
                i.modifiers.shift,
                i.pointer.button_down(PointerButton::Middle),
                i.pointer.delta(),
                i.zoom_delta(),
                i.pointer.primary_down(),
            )
        });
        // The wheel zooms (the camera smooths it); Shift+wheel and sideways
        // trackpad scrolling pan. Ctrl+wheel already arrives as pinch zoom.
        let scroll = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::MouseWheel {
                        unit,
                        delta,
                        modifiers,
                        ..
                    } if !modifiers.ctrl && !modifiers.command => {
                        let k = match unit {
                            egui::MouseWheelUnit::Point => 1.0,
                            egui::MouseWheelUnit::Line => 50.0,
                            egui::MouseWheelUnit::Page => view.height(),
                        };
                        Some(if modifiers.shift {
                            vec2(delta.y + delta.x, 0.0) * k
                        } else {
                            *delta * k
                        })
                    }
                    _ => None,
                })
                .fold(Vec2::ZERO, |a, b| a + b)
        });
        // Over the canvas, and not over a menu or window floating above it.
        let hover = hover.filter(|p| {
            view.contains(*p)
                && ctx
                    .layer_id_at(*p)
                    .is_none_or(|l| l.order == ui.layer_id().order)
        });

        // The background: marquee, deselect, panning, placing, context menu.
        let bg = ui.interact(view, ui.id().with("cad-canvas"), Sense::click_and_drag());

        // ---- Camera.
        let t0 = self.camera.transform(view, ppp);
        let hover_world = hover.map(|p| t0.inverse().mul_pos(p));
        let over_scroll_user = hover_world.is_some_and(|w| {
            self.active()
                .and_then(|uid| self.board.get(uid))
                .is_some_and(|vp| vp.rect.contains(w) && vp.formula.wants_scroll())
        });
        if let Some(p) = hover
            && !over_scroll_user
        {
            if scroll.y != 0.0 {
                self.camera.zoom_at(view, p, (scroll.y * 0.0025).exp());
            }
            if scroll.x != 0.0 {
                self.camera.pan(vec2(scroll.x, 0.0));
            }
            if pinch != 1.0 {
                self.camera.zoom_at(view, p, pinch);
            }
        }
        if middle && (hover.is_some() || self.panning) {
            self.panning = true;
            self.camera.pan(delta);
        } else if !middle {
            self.panning = false;
        }
        if self.mode == Mode::Pan && bg.dragged_by(PointerButton::Primary) {
            self.camera.pan(bg.drag_delta());
        }
        let camera_moving = self.camera.update(view, dt);
        let t = self.camera.transform(view, ppp);
        let inv = t.inverse();
        let step = self.grid_step();
        let visible = inv.mul_rect(view);
        self.pointer = hover.map(|p| inv.mul_pos(p));

        overlay::paper(ui.painter(), view, t, step, chrome, self.settings.grid);

        // ---- The world layer: every viewport's formula at natural size.
        let world_layer = LayerId::new(ui.layer_id().order, ui.id().with("cad-world"));
        let overlay_layer = LayerId::new(ui.layer_id().order, ui.id().with("cad-overlay"));
        ctx.set_sublayer(ui.layer_id(), world_layer);
        ctx.set_sublayer(ui.layer_id(), overlay_layer);
        ctx.set_transform_layer(world_layer, t);
        let mut wui = ui.new_child(UiBuilder::new().layer_id(world_layer).max_rect(visible));
        wui.set_clip_rect(visible);

        let selecting = self.mode == Mode::Select && !self.panning;
        let active = if self.panning { None } else { self.active() };
        let selection = self.board.selection.clone();
        let mut hits: Vec<Hit> = Vec::new();
        let mut animating = false;
        for vp in &mut self.board.viewports {
            if vp.hidden || !vp.rect.intersects(visible) {
                continue;
            }
            let rect = vp.rect;
            let screen = t.mul_rect(rect);
            let painter = wui.painter_at(rect);
            painter.rect_filled(rect, 0.0, palette.paper);
            if screen.width() < 140.0 || screen.height() < 90.0 {
                // Too small to be useful: the overlay draws a placeholder card.
                continue;
            }
            let id = Id::new(("cad-vp", vp.uid));
            let is_active = active == Some(vp.uid);
            let stage = vp.formula.stage_sense();
            let body = if is_active {
                let extra = if stage.senses_drag() {
                    Sense::click()
                } else {
                    Sense::click_and_drag()
                };
                wui.interact(rect, id, stage | extra)
            } else if selecting {
                wui.interact(rect, id, Sense::click_and_drag())
            } else {
                wui.interact(rect, id, Sense::hover())
            };
            if is_active {
                vp.formula.interact(&body, rect);
            } else {
                // Hover readouts still work on inactive viewports.
                let probe = wui.interact(rect, id.with("probe"), Sense::hover());
                vp.formula.interact(&probe, rect);
            }
            let animate = self.settings.animate_all || selection.contains(&vp.uid);
            vp.formula
                .paint(&painter, rect, palette, if animate { dt } else { 0.0 });
            animating |= animate && vp.formula.is_animating();
            hits.push(Hit {
                uid: vp.uid,
                body,
                active: is_active,
                formula_drags: stage.senses_drag(),
            });
        }

        // ---- The overlay layer: frames, tabs, grips and the rest.
        let mut oui = ui.new_child(UiBuilder::new().layer_id(overlay_layer).max_rect(view));
        oui.set_clip_rect(view);
        let painter = oui.painter().clone();
        let hovered_vp = hover_world.and_then(|w| self.board.hit(w));
        let mut tabs: Vec<(u64, Response)> = Vec::new();
        let mut grips: Vec<(u64, Handle, Response)> = Vec::new();
        for vp in &self.board.viewports {
            if vp.hidden
                || !vp
                    .rect
                    .intersects(visible.expand(TAB_SLACK / self.camera.zoom))
            {
                continue;
            }
            let screen = t.mul_rect(vp.rect);
            let selected = selection.contains(&vp.uid);
            let hovered = hovered_vp == Some(vp.uid);
            if screen.width() < 140.0 || screen.height() < 90.0 {
                let badge = Rect::from_center_size(
                    screen.center(),
                    Vec2::splat(screen.size().min_elem().min(40.0) * 0.6),
                );
                icons::tool(&painter, badge, vp.tool.id, chrome.text_dim);
            }
            // Fade in after placement.
            let age = (now - vp.born) as f32;
            if age < 0.3 {
                let k = (age / 0.3).clamp(0.0, 1.0);
                painter.rect_filled(screen, 0.0, chrome.canvas.gamma_multiply(1.0 - k));
                painter.rect_stroke(
                    screen.expand(8.0 * (1.0 - k)),
                    0.0,
                    Stroke::new(1.5, chrome.accent.gamma_multiply(1.0 - k)),
                    egui::StrokeKind::Outside,
                );
            }
            let stroke = if selected {
                Stroke::new(1.5, chrome.accent)
            } else if hovered {
                Stroke::new(1.0, chrome.accent.gamma_multiply(0.55))
            } else {
                Stroke::new(1.0, chrome.border)
            };
            overlay::frame(&painter, screen, stroke);
            let label = vp.label();
            let tab = overlay::tab_rect(&painter, screen, &label);
            let response = if selecting {
                Some(oui.interact(tab, Id::new(("cad-tab", vp.uid)), Sense::click_and_drag()))
            } else {
                None
            };
            let tab_hovered = response.as_ref().is_some_and(|r| r.hovered());
            overlay::tab(
                &painter,
                tab,
                &label,
                selected,
                tab_hovered,
                vp.locked,
                chrome,
            );
            if let Some(r) = response {
                tabs.push((vp.uid, r));
            }
        }
        // Grips on a single, unlocked selection.
        if selecting
            && let [uid] = selection[..]
            && let Some(vp) = self.board.get(uid)
            && !vp.hidden
            && !vp.locked
        {
            let screen = t.mul_rect(vp.rect);
            for h in Handle::ALL {
                let at = h.at(screen);
                let r = oui.interact(
                    Rect::from_center_size(at, vec2(12.0, 12.0)),
                    Id::new(("cad-grip", uid, h as u8)),
                    Sense::drag(),
                );
                overlay::grip(&painter, at, r.hovered() || r.dragged(), chrome);
                grips.push((uid, h, r));
            }
        }

        // ---- Minimap.
        let minimap = if self.settings.minimap {
            self.board.bounds().map(|b| {
                let map = overlay::minimap_rect(view);
                let m = overlay::minimap_map(map, b.union(visible));
                let r = oui.interact(map, ui.id().with("cad-minimap"), Sense::click_and_drag());
                (map, m, r)
            })
        } else {
            None
        };
        // Its buttons: zoom out / in, fit, 100%.
        let mut strip_hovered = false;
        if let Some((map, _, _)) = &minimap {
            let strip = overlay::minimap_strip(*map);
            painter.rect_filled(strip, 3.0, chrome.panel.gamma_multiply(0.92));
            painter.rect_stroke(
                strip,
                3.0,
                Stroke::new(1.0, chrome.border),
                egui::StrokeKind::Inside,
            );
            // A child, not a scope: a scope would grow the overlay's own rect
            // over the whole canvas and hide the viewports from hit-testing.
            let mut bar = oui.new_child(
                UiBuilder::new()
                    .max_rect(strip.shrink(3.0))
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            bar.spacing_mut().item_spacing.x = 2.0;
            if icons::button(&mut bar, icons::Ui::ZoomOut, false, "Zoom out").clicked() {
                self.zoom_by(0.8);
            }
            if icons::button(&mut bar, icons::Ui::ZoomIn, false, "Zoom in").clicked() {
                self.zoom_by(1.25);
            }
            if icons::button(&mut bar, icons::Ui::Fit, false, "Fit all (F)").clicked() {
                self.fit_all();
            }
            if icons::text_button(&mut bar, "1:1", "Zoom to 100% (Ctrl+0)").clicked() {
                self.camera.zoom_to(1.0);
            }
            strip_hovered = hover.is_some_and(|p| strip.contains(p));
        }

        // ---- Handle this frame's input.
        let pointer_screen = ctx.input(|i| i.pointer.interact_pos()).or(hover);
        let pointer_world = pointer_screen.map(|p| inv.mul_pos(p));

        for hit in &hits {
            let b = &hit.body;
            if !hit.active {
                if b.clicked() {
                    self.click_select(hit.uid, shift);
                }
                if b.drag_started_by(PointerButton::Primary)
                    && let Some(w) = pointer_world
                {
                    self.start_move(hit.uid, w);
                }
                if b.secondary_clicked() {
                    self.board.select_only(hit.uid);
                }
                let uid = hit.uid;
                b.context_menu(|ui| self.viewport_menu(ui, uid));
            } else if !hit.formula_drags
                && b.drag_started_by(PointerButton::Primary)
                && let Some(w) = pointer_world
            {
                self.start_move(hit.uid, w);
            }
        }
        for (uid, r) in &tabs {
            let uid = *uid;
            if r.clicked() {
                self.click_select(uid, shift);
            }
            if r.double_clicked() {
                self.focus(uid);
            }
            if r.drag_started_by(PointerButton::Primary)
                && let Some(w) = pointer_world
            {
                self.start_move(uid, w);
            }
            if r.hovered() && self.gesture.is_none() {
                ctx.set_cursor_icon(CursorIcon::Grab);
            }
            if r.secondary_clicked() {
                self.board.select_only(uid);
            }
            r.context_menu(|ui| self.viewport_menu(ui, uid));
        }
        for (uid, h, r) in &grips {
            if r.hovered() || r.dragged() {
                ctx.set_cursor_icon(h.cursor());
            }
            if r.drag_started()
                && let Some(vp) = self.board.get(*uid)
            {
                self.gesture = Some(Gesture::Resize {
                    uid: *uid,
                    handle: *h,
                    start: vp.rect,
                });
            }
        }
        if let Some((map, m, r)) = &minimap {
            if r.drag_started() || r.clicked() {
                self.gesture = Some(Gesture::Minimap {
                    map: *map,
                    to_world: m.inverse(),
                });
            }
            if r.hovered() {
                ctx.set_cursor_icon(CursorIcon::PointingHand);
            }
        }

        if let Some(Gesture::Minimap { map, to_world }) = &self.gesture
            && let Some(p) = pointer_screen
        {
            self.camera.center = to_world.mul_pos(map.clamp(p));
        }

        match self.mode {
            Mode::Select if !self.panning => {
                if bg.clicked() && !shift {
                    self.board.selection.clear();
                }
                if bg.drag_started_by(PointerButton::Primary)
                    && let Some(p) = pointer_screen
                {
                    if !shift {
                        self.board.selection.clear();
                    }
                    self.gesture = Some(Gesture::Marquee {
                        from: p,
                        additive: shift,
                    });
                }
                bg.context_menu(|ui| self.canvas_menu(ui));
            }
            Mode::Place(tool) => {
                if bg.clicked()
                    && let Some(w) = pointer_world
                {
                    let at = self.snap(w);
                    self.finish_place(tool, at, Vec2::ZERO, shift);
                } else if bg.drag_started_by(PointerButton::Primary)
                    && let Some(w) = pointer_world
                {
                    self.gesture = Some(Gesture::Place {
                        tool,
                        from: self.snap(w),
                    });
                }
                if bg.secondary_clicked() {
                    self.set_mode(Mode::Select);
                }
            }
            _ => {}
        }

        // ---- Ongoing gesture.
        let mut tag: Option<String> = None;
        let mut guides: Vec<Guide> = Vec::new();
        let mut marquee: Option<(Rect, bool)> = None;
        let mut place_rect: Option<(Rect, &'static Tool)> = None;
        if let (Some(g), Some(w)) = (self.gesture.take(), pointer_world) {
            let done = !primary_down;
            let keep = match g {
                Gesture::Move { start, grab, .. } => {
                    let (rects, found) = self.move_to(&start, w - grab, shift);
                    guides = found.clone();
                    if let Some(&(uid, _)) = start.first()
                        && let Some(r) = rects.iter().find(|(u, _)| *u == uid)
                    {
                        tag = Some(format!("X {:.0}  Y {:.0}", r.1.left(), r.1.top()));
                    }
                    ctx.set_cursor_icon(CursorIcon::Grabbing);
                    if done {
                        let changes = start
                            .iter()
                            .filter_map(|&(uid, before)| {
                                self.board.get(uid).map(|v| (uid, before, v.rect))
                            })
                            .collect();
                        self.history.push(Edit::Reshaped(changes));
                        None
                    } else {
                        Some(Gesture::Move {
                            start,
                            grab,
                            guides: found,
                        })
                    }
                }
                Gesture::Resize { uid, handle, start } => {
                    let r = self.resize_to(uid, handle, start, w, shift);
                    tag = Some(format!("W {:.0} × H {:.0}", r.width(), r.height()));
                    ctx.set_cursor_icon(handle.cursor());
                    if done {
                        self.history.push(Edit::Reshaped(vec![(uid, start, r)]));
                        None
                    } else {
                        Some(Gesture::Resize { uid, handle, start })
                    }
                }
                Gesture::Place { tool, from } => {
                    let to = self.snap(w);
                    let r = Rect::from_two_pos(from, to);
                    tag = Some(format!("W {:.0} × H {:.0}", r.width(), r.height()));
                    if done {
                        // A tiny drag counts as a click: default size.
                        let size = if t.mul_rect(r).size().min_elem() < 12.0 {
                            Vec2::ZERO
                        } else {
                            r.size()
                        };
                        self.finish_place(
                            tool,
                            if size == Vec2::ZERO { from } else { r.min },
                            size,
                            shift,
                        );
                        None
                    } else {
                        place_rect = Some((r, tool));
                        Some(Gesture::Place { tool, from })
                    }
                }
                Gesture::Marquee { from, additive } => {
                    let to = pointer_screen.unwrap_or(from);
                    let r = Rect::from_two_pos(from, to);
                    let window = to.x >= from.x;
                    if done {
                        self.marquee_select(inv.mul_rect(r), window, additive);
                        None
                    } else {
                        marquee = Some((r, window));
                        Some(Gesture::Marquee { from, additive })
                    }
                }
                Gesture::Minimap { map, to_world } => {
                    (!done).then_some(Gesture::Minimap { map, to_world })
                }
            };
            self.gesture = keep;
        } else if !primary_down {
            self.gesture = None;
        }

        // ---- Overlay drawing that depends on the gesture.
        if let Some(Gesture::Move { guides: g, .. }) = &self.gesture {
            guides = g.clone();
        }
        overlay::guides(&painter, t, &guides, chrome);
        // Dimensions on a single selection, also while moving or resizing it.
        let other_gesture = matches!(
            self.gesture,
            Some(Gesture::Place { .. } | Gesture::Marquee { .. } | Gesture::Minimap { .. })
        );
        if !other_gesture
            && let [uid] = self.board.selection[..]
            && let Some(vp) = self.board.get(uid)
            && !vp.hidden
        {
            overlay::dimensions(&painter, t.mul_rect(vp.rect), vp.rect, chrome);
        }
        if let Some((r, window)) = marquee {
            overlay::marquee(&painter, r, window, chrome);
        }
        let over_chrome = tabs.iter().any(|(_, r)| r.hovered())
            || grips.iter().any(|(_, _, r)| r.hovered() || r.dragged())
            || minimap.as_ref().is_some_and(|(_, _, r)| r.hovered())
            || strip_hovered;
        let over_viewport = hovered_vp.is_some() || over_chrome;
        if let Mode::Place(tool) = self.mode {
            let (r, label) = match place_rect {
                Some((r, tool)) => (r, tool.name.to_owned()),
                None => {
                    let size = self.default_size(tool);
                    let at = pointer_world.map(|w| self.snap(w)).unwrap_or(Pos2::ZERO);
                    (
                        Rect::from_min_size(at, size),
                        format!("{}  {:.0} × {:.0}", tool.name, size.x, size.y),
                    )
                }
            };
            if hover.is_some() {
                overlay::ghost(&painter, t.mul_rect(r), &label, chrome);
            }
        }
        if let Some(p) = hover {
            let crosshair = match self.mode {
                Mode::Place(_) => true,
                Mode::Select => {
                    !over_viewport && self.gesture.is_none()
                        || matches!(self.gesture, Some(Gesture::Marquee { .. }))
                }
                Mode::Pan => false,
            };
            if crosshair && !self.panning {
                overlay::crosshair(&painter, view, p, chrome);
                ctx.set_cursor_icon(CursorIcon::None);
                if tag.is_none() && matches!(self.mode, Mode::Place(_)) {
                    let w = self.snap(inv.mul_pos(p));
                    tag = Some(format!("X {:.0}  Y {:.0}", w.x, w.y));
                }
            }
            if self.mode == Mode::Pan || self.panning {
                ctx.set_cursor_icon(if bg.dragged() || self.panning {
                    CursorIcon::Grabbing
                } else {
                    CursorIcon::Grab
                });
            }
            if let Some(text) = &tag {
                overlay::tag(&painter, p, text, chrome);
            }
        }
        if self.settings.rulers {
            overlay::rulers(
                &painter,
                view,
                t,
                step,
                chrome,
                hover,
                self.board.selection_bounds(),
            );
        }
        if let Some((map, m, _)) = minimap {
            let rects: Vec<(Rect, bool)> = self
                .board
                .viewports
                .iter()
                .filter(|v| !v.hidden)
                .map(|v| (v.rect, selection.contains(&v.uid)))
                .collect();
            overlay::minimap(&painter, map, m, &rects, visible, chrome);
        }
        let n = self.board.viewports.len();
        overlay::title_block(
            &painter,
            view,
            chrome,
            [
                "FORMULA WORKBENCH",
                &format!(
                    "SHEET 1 · {n} VIEWPORT{} · {}",
                    if n == 1 { "" } else { "S" },
                    overlay::today()
                ),
            ],
        );

        let fading = self.board.viewports.iter().any(|v| now - v.born < 0.3);
        if animating || camera_moving || self.gesture.is_some() || fading || self.panning {
            ctx.request_repaint();
        }
    }

    fn click_select(&mut self, uid: u64, shift: bool) {
        if shift {
            self.board.toggle_selected(uid);
        } else {
            self.board.select_only(uid);
        }
    }

    fn start_move(&mut self, uid: u64, grab: Pos2) {
        if !self.board.is_selected(uid) {
            self.board.select_only(uid);
        }
        let start: Vec<(u64, Rect)> = self
            .board
            .viewports
            .iter()
            .filter(|v| self.board.is_selected(v.uid) && !v.locked)
            .map(|v| (v.uid, v.rect))
            .collect();
        if !start.is_empty() {
            self.gesture = Some(Gesture::Move {
                start,
                grab,
                guides: Vec::new(),
            });
        }
    }

    /// Move the dragged viewports by `delta` from where they started, with
    /// ortho, grid snap and smart guides applied. Returns the new rects.
    fn move_to(
        &mut self,
        start: &[(u64, Rect)],
        mut delta: Vec2,
        shift: bool,
    ) -> (Vec<(u64, Rect)>, Vec<Guide>) {
        if self.settings.ortho != shift {
            if delta.x.abs() > delta.y.abs() {
                delta.y = 0.0;
            } else {
                delta.x = 0.0;
            }
        }
        let bounds = start
            .iter()
            .map(|(_, r)| *r)
            .reduce(|a, b| a.union(b))
            .unwrap_or(Rect::NOTHING);
        let moved = bounds.translate(delta);
        let snapped = self.snap(moved.min);
        let mut offset = snapped - moved.min;
        let others: Vec<Rect> = self
            .board
            .viewports
            .iter()
            .filter(|v| !v.hidden && !start.iter().any(|(u, _)| *u == v.uid))
            .map(|v| v.rect)
            .collect();
        let tolerance = 7.0 / self.camera.zoom;
        let (align, guides) = board::align(moved, &others, tolerance);
        if align.x != 0.0 || guides.iter().any(|g| g.vertical) {
            offset.x = align.x;
        }
        if align.y != 0.0 || guides.iter().any(|g| !g.vertical) {
            offset.y = align.y;
        }
        let delta = delta + offset;
        let rects: Vec<(u64, Rect)> = start
            .iter()
            .map(|&(u, r)| (u, r.translate(delta)))
            .collect();
        for &(uid, r) in &rects {
            if let Some(vp) = self.board.get_mut(uid) {
                vp.rect = r;
            }
        }
        (rects, guides)
    }

    /// Drag one grip of a viewport to the world point `to`.
    fn resize_to(
        &mut self,
        uid: u64,
        handle: Handle,
        start: Rect,
        to: Pos2,
        keep_aspect: bool,
    ) -> Rect {
        let to = self.snap(to);
        let Some(vp) = self.board.get_mut(uid) else {
            return start;
        };
        let min = vp.formula.min_size();
        let (sx, sy) = handle.sides();
        let mut r = start;
        if sx < 0 {
            r.min.x = to.x.min(r.max.x - min.x);
        } else if sx > 0 {
            r.max.x = to.x.max(r.min.x + min.x);
        }
        if sy < 0 {
            r.min.y = to.y.min(r.max.y - min.y);
        } else if sy > 0 {
            r.max.y = to.y.max(r.min.y + min.y);
        }
        if keep_aspect && sx != 0 && sy != 0 {
            let aspect = start.width() / start.height();
            let w = r.width().max(r.height() * aspect).max(min.x);
            let h = (w / aspect).max(min.y);
            let w = h * aspect;
            if sx < 0 {
                r.min.x = r.max.x - w;
            } else {
                r.max.x = r.min.x + w;
            }
            if sy < 0 {
                r.min.y = r.max.y - h;
            } else {
                r.max.y = r.min.y + h;
            }
        }
        vp.rect = r;
        r
    }

    fn finish_place(&mut self, tool: &'static Tool, at: Pos2, size: Vec2, keep_tool: bool) {
        self.place(tool, Some(at), size);
        if !keep_tool {
            self.mode = Mode::Select;
        }
    }

    fn marquee_select(&mut self, r: Rect, window: bool, additive: bool) {
        let picked: Vec<u64> = self
            .board
            .viewports
            .iter()
            .filter(|v| !v.hidden)
            .filter(|v| {
                if window {
                    r.contains_rect(v.rect)
                } else {
                    r.intersects(v.rect)
                }
            })
            .map(|v| v.uid)
            .collect();
        if !additive {
            self.board.selection.clear();
        }
        for uid in picked {
            if !self.board.is_selected(uid) {
                self.board.selection.push(uid);
            }
        }
    }

    fn viewport_menu(&mut self, ui: &mut egui::Ui, uid: u64) {
        let Some(vp) = self.board.get(uid) else {
            ui.close();
            return;
        };
        let (locked, label) = (vp.locked, vp.label());
        ui.label(egui::RichText::new(label).strong());
        ui.separator();
        if ui.button("Focus").clicked() {
            self.focus(uid);
            ui.close();
        }
        if ui.button("Properties").clicked() {
            self.board.select_only(uid);
            self.request = Some(Request::Properties);
            ui.close();
        }
        if ui.button("Documentation").clicked() {
            self.board.select_only(uid);
            self.request = Some(Request::Docs);
            ui.close();
        }
        if ui.button("Export code").clicked() {
            self.board.select_only(uid);
            self.request = Some(Request::Code);
            ui.close();
        }
        ui.separator();
        if ui.button("Duplicate        Ctrl+D").clicked() {
            self.board.select_only(uid);
            self.duplicate_selection();
            ui.close();
        }
        if ui.button("Bring to front").clicked() {
            self.board.bring_to_front(uid);
            ui.close();
        }
        if ui.button("Send to back").clicked() {
            self.board.send_to_back(uid);
            ui.close();
        }
        if ui
            .button(if locked { "Unlock" } else { "Lock position" })
            .clicked()
        {
            if let Some(vp) = self.board.get_mut(uid) {
                vp.locked = !locked;
            }
            ui.close();
        }
        if ui.button("Hide").clicked() {
            if let Some(vp) = self.board.get_mut(uid) {
                vp.hidden = true;
            }
            self.board.selection.retain(|&s| s != uid);
            ui.close();
        }
        ui.separator();
        if ui.button("Delete           Del").clicked() {
            self.board.select_only(uid);
            self.delete_selection();
            ui.close();
        }
    }

    fn canvas_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Place", |ui| {
            for (category, tools) in formulas::categories() {
                ui.menu_button(category, |ui| {
                    for tool in tools {
                        if ui
                            .button(format!("{}   {}", tool.name, tool.alias))
                            .clicked()
                        {
                            let at = self.pointer.map(|p| self.snap(p));
                            self.place(tool, at, Vec2::ZERO);
                            ui.close();
                        }
                    }
                });
            }
        });
        ui.separator();
        if ui.button("Zoom to fit        F").clicked() {
            self.fit_all();
            ui.close();
        }
        if ui.button("Zoom 100%     Ctrl+0").clicked() {
            self.camera.zoom_to(1.0);
            ui.close();
        }
        if ui.button("Select all     Ctrl+A").clicked() {
            self.select_all();
            ui.close();
        }
        ui.separator();
        ui.checkbox(&mut self.settings.grid, "Grid            G");
        ui.checkbox(&mut self.settings.snap, "Snap            S");
        ui.checkbox(&mut self.settings.ortho, "Ortho           O");
    }
}

/// How far outside the view a viewport may be and still show its tab.
const TAB_SLACK: f32 = 30.0;

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace() -> Workspace {
        Workspace::new(Board::default(), Camera::default(), Settings::default())
    }

    /// A fresh path in the system's temporary folder.
    fn temp_sheet(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("formula-test-{}", std::process::id()));
        dir.join(name).display().to_string()
    }

    #[test]
    fn saved_sheets_open_back_and_join_recent() {
        let mut ws = workspace();
        ws.run_command("lor");
        ws.run_command("bub");
        let path = temp_sheet("round-trip.fwb");
        assert!(ws.save_to(&path));
        assert_eq!(ws.files.recent[0], path);
        let saved = ws.board.to_model();
        let mut other = workspace();
        assert!(other.open_path(&path));
        let opened = other.board.to_model();
        assert_eq!(opened.len(), 2);
        for (a, b) in opened.iter().zip(&saved) {
            assert_eq!((&a.tool, &a.rect, &a.state), (&b.tool, &b.rect, &b.state));
        }
        assert_eq!(other.files.path.as_deref(), Some(path.as_str()));
        assert!(!other.files.dirty);
        // A recent file that has since been deleted drops off the list.
        std::fs::remove_file(&path).unwrap();
        assert!(!other.open_path(&path));
        assert!(other.files.recent.is_empty());
        assert_eq!(other.board.to_model(), opened);
        assert!(other.log.last().unwrap().contains("gone"));
    }

    #[test]
    fn the_unsaved_mark_follows_changes_but_not_the_camera() {
        let mut ws = workspace();
        ws.run_command("lerp");
        ws.mark_saved();
        let tick = |ws: &mut Workspace| {
            ws.now += 1.0;
            ws.check_changes();
            ws.files.dirty
        };
        assert!(!tick(&mut ws));
        ws.camera.center = pos2(500.0, 500.0);
        ws.ui.dark = false;
        assert!(!tick(&mut ws));
        ws.run_command("vor");
        assert!(tick(&mut ws));
        ws.undo();
        assert!(!tick(&mut ws));
        let uid = ws.board.viewports[0].uid;
        let f = &mut ws.board.get_mut(uid).unwrap().formula;
        f.load_state(&serde_json::json!({"v0": 1.0}));
        assert!(tick(&mut ws));
        assert!(ws.files.title().ends_with('\u{2022}'));
    }

    #[test]
    fn dropped_bytes_open_a_sheet_and_anything_else_is_refused() {
        let mut ws = workspace();
        ws.run_command("lor");
        let before = ws.board.to_model();
        assert!(!ws.open_bytes("photo.fwb", &[0xff, 0xd8, 0xff, 0x00]));
        assert!(!ws.open_bytes("notes.fwb", b"just some notes"));
        assert_eq!(ws.board.to_model(), before);
        assert!(ws.log.last().unwrap().contains("not a Formula sheet"));
        let text = "FWB 1\nSECTION VIEWPORT\n  tool voronoi\nENDSEC\n";
        assert!(ws.open_bytes("cells.fwb", text.as_bytes()));
        assert_eq!(ws.board.viewports[0].tool.id, "voronoi");
        assert_eq!(ws.files.title(), "cells.fwb");
    }

    #[test]
    fn open_without_a_file_asks_the_chrome_for_a_dialog() {
        let mut ws = workspace();
        assert!(ws.run_command("open"));
        assert_eq!(ws.file_request, Some(FileRequest::Open));
        ws.file_request = None;
        ws.save();
        assert_eq!(ws.file_request, Some(FileRequest::SaveAs));
        ws.run_command("help");
        assert!(ws.log.last().unwrap().contains("open [file], save [name]"));
    }

    #[test]
    fn opening_a_bad_sheet_changes_nothing() {
        let mut ws = workspace();
        ws.run_command("lor");
        ws.camera.zoom = 2.0;
        let before = (ws.board.to_model(), ws.camera, ws.history.can_redo());
        for bad in [
            "",
            "FWB 99\n",
            "FWB 1\nSECTION VIEWPORT\n  tool lerp\n",
            "\0\u{1}",
        ] {
            assert!(!ws.open_text("bad.fwb", bad));
            assert_eq!(
                (ws.board.to_model(), ws.camera, ws.history.can_redo()),
                before
            );
            assert!(
                ws.log
                    .last()
                    .unwrap()
                    .starts_with("Could not open bad.fwb: ")
            );
        }
        assert!(!ws.ui_changed);
    }

    #[test]
    fn opening_is_one_undo_step() {
        let mut ws = workspace();
        ws.run_command("lor");
        ws.run_command("vor");
        let old = ws.board.to_model();
        let text = "FWB 1\nSECTION UI\n  theme light\nENDSEC\nSECTION SHARED\n  sorting {\"n\":7}\nENDSEC\nSECTION VIEWPORT\n  uid 1\n  tool lerp\n  param v0 42\nENDSEC\n";
        assert!(ws.open_text("lesson.fwb", text));
        assert_eq!(ws.log.last().unwrap(), "Opened lesson.fwb");
        assert!(ws.ui_changed && !ws.ui.dark);
        let opened = ws.board.to_model();
        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0].state.as_ref().unwrap()["v0"], 42.0);
        // Fresh uids, so the old viewports can come back without clashing.
        assert!(old.iter().all(|m| m.uid != opened[0].uid));
        assert_eq!(ws.board.env.sorting.data.borrow().n, 7);
        ws.undo();
        assert_eq!(ws.board.to_model(), old);
        ws.redo();
        assert_eq!(ws.board.to_model(), opened);
        // The workspace's own sheet reads back to the same viewports.
        let (sheet, warnings) = sheet::from_text(&sheet::to_text(&ws.sheet())).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(sheet.viewports, opened);
    }

    /// Every tool, placed at its default size and at its minimum size, draws
    /// and handles input for a few frames without panicking.
    #[test]
    fn every_tool_runs_on_the_canvas() {
        let mut ws = workspace();
        let mut x = 0.0;
        for tool in formulas::TOOLS {
            let a = ws.place(tool, Some(pos2(x, 0.0)), Vec2::ZERO);
            let min = ws.board.get(a).unwrap().min_size();
            ws.place(tool, Some(pos2(x, 2000.0)), min);
            x += ws.board.get(a).unwrap().rect.width() + 40.0;
        }
        let ctx = egui::Context::default();
        let chrome = Chrome::graphite();
        let palette = super::super::theme::palette(true);
        let uids: Vec<u64> = ws.board.viewports.iter().map(|v| v.uid).collect();
        for (frame, uid) in uids.into_iter().enumerate() {
            // Centre each viewport at 100% and make it the active one.
            let rect = ws.board.get(uid).unwrap().rect;
            ws.camera = Camera::default();
            ws.camera.center = rect.center();
            ws.board.selection = vec![uid];
            let mut input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 1000.0))),
                ..Default::default()
            };
            input.events.push(egui::Event::PointerMoved(pos2(
                700.0 + (frame % 5) as f32 * 30.0,
                450.0,
            )));
            let mut out = ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().show(ui, |ui| ws.show(ui, &chrome, &palette));
            });
            out.textures_delta.clear();
        }
        assert_eq!(ws.board.viewports.len(), 2 * formulas::TOOLS.len());
    }

    #[test]
    fn commands_place_and_edit() {
        let mut ws = workspace();
        assert!(ws.run_command("lor"));
        assert!(ws.run_command("place Voronoi diagram"));
        assert_eq!(ws.board.viewports.len(), 2);
        assert_eq!(ws.board.viewports[0].tool.id, "lorenz");
        // Placed side by side, never on top of each other.
        assert!(
            !ws.board.viewports[0]
                .rect
                .intersects(ws.board.viewports[1].rect)
        );
        assert!(ws.run_command("dup"));
        assert_eq!(ws.board.viewports.len(), 3);
        assert!(ws.run_command("undo"));
        assert_eq!(ws.board.viewports.len(), 2);
        assert!(ws.run_command("all"));
        assert!(ws.run_command("del"));
        assert!(ws.board.viewports.is_empty());
        assert!(ws.run_command("u"));
        assert_eq!(ws.board.viewports.len(), 2);
        assert!(!ws.run_command("frobnicate"));
        assert!(ws.run_command("zoom 50"));
    }

    #[test]
    fn plot_command_places_an_expression() {
        let mut ws = workspace();
        let rows = |ws: &Workspace, i: usize| {
            ws.board.viewports[i].formula.save_state().unwrap()["rows"].clone()
        };
        assert!(ws.run_command("plot sin(x)"));
        assert_eq!(ws.board.viewports.len(), 1);
        assert_eq!(ws.board.viewports[0].tool.id, "expression");
        assert_eq!(rows(&ws, 0), serde_json::json!(["sin(x)"]));
        assert!(ws.run_command("PLOT r = 1 + cos(3θ)"));
        assert_eq!(rows(&ws, 1), serde_json::json!(["r = 1 + cos(3θ)"]));

        assert!(!ws.run_command("plot sin("));
        assert_eq!(ws.board.viewports.len(), 2, "a typo places nothing");
        let said = ws.log.last().unwrap();
        assert!(
            said.starts_with("plot: ") && said.contains("column 5"),
            "{said}"
        );
        assert!(ws.run_command("plot"));
        assert!(ws.log.last().unwrap().starts_with("plot <expression>"));
        assert_eq!(ws.board.viewports.len(), 2);

        assert!(ws.run_command("undo"));
        assert_eq!(ws.board.viewports.len(), 1);
        assert!(ws.run_command("redo"));
        assert_eq!(rows(&ws, 1), serde_json::json!(["r = 1 + cos(3θ)"]));
    }

    #[test]
    fn latex_command_copies_the_selected_viewports_snippet() {
        let mut ws = workspace();
        assert!(ws.run_command("latex"));
        assert!(ws.clipboard.is_none());
        assert!(ws.log.last().unwrap().contains("select a viewport"));
        ws.run_command("lissajous");
        assert!(ws.run_command("latex"));
        let copied = ws.clipboard.take().expect("snippet on the clipboard");
        assert!(copied.starts_with("\\begin{gathered}"), "{copied}");
        assert!(copied.contains("\\delta ="), "{copied}");
    }

    #[test]
    fn derive_command_opens_the_derive_tab() {
        let mut ws = workspace();
        assert!(ws.run_command("derive"));
        assert_eq!(ws.request, Some(Request::Derive));
    }

    #[test]
    fn ipynb_command_needs_a_selection() {
        let mut ws = workspace();
        assert!(ws.run_command("ipynb"));
        assert!(ws.log.last().unwrap().contains("select a viewport"));
    }

    #[test]
    fn move_snaps_to_grid_and_ortho() {
        let mut ws = workspace();
        let a = ws.place(
            formulas::tool("lerp").unwrap(),
            Some(pos2(0.0, 0.0)),
            Vec2::ZERO,
        );
        let start = vec![(a, ws.board.get(a).unwrap().rect)];
        let step = ws.grid_step();
        let (rects, _) = ws.move_to(&start, vec2(step * 3.0 + 3.0, step * 2.0 - 4.0), false);
        assert_eq!(rects[0].1.min, pos2(step * 3.0, step * 2.0));
        ws.settings.ortho = true;
        let (rects, _) = ws.move_to(&start, vec2(step * 5.0, step), false);
        assert_eq!(rects[0].1.min, pos2(step * 5.0, 0.0));
    }

    #[test]
    fn resize_never_goes_below_min_size() {
        let mut ws = workspace();
        let a = ws.place(
            formulas::tool("sorting-race").unwrap(),
            Some(pos2(0.0, 0.0)),
            Vec2::ZERO,
        );
        let start = ws.board.get(a).unwrap().rect;
        let min = ws.board.get(a).unwrap().min_size();
        let r = ws.resize_to(a, Handle::SE, start, start.min + vec2(10.0, 10.0), false);
        assert!(r.width() >= min.x && r.height() >= min.y);
        let r = ws.resize_to(a, Handle::W, start, pos2(start.min.x - 200.0, 0.0), false);
        assert_eq!(r.max.x, start.max.x);
        assert!(r.width() >= start.width() + 199.0);
    }

    /// Records the pointer positions of the drags it receives.
    struct Probe(std::rc::Rc<std::cell::RefCell<Vec<Pos2>>>);

    impl crate::formula::Formula for Probe {
        fn id(&self) -> &'static str {
            "probe"
        }
        fn name(&self) -> &'static str {
            "Probe"
        }
        fn category(&self) -> &'static str {
            "Test"
        }
        fn summary(&self) -> &'static str {
            ""
        }
        fn equations(&self) -> Vec<crate::formula::MathText> {
            Vec::new()
        }
        fn stage_sense(&self) -> Sense {
            Sense::drag()
        }
        fn interact(&mut self, response: &egui::Response, _rect: Rect) {
            if response.dragged()
                && let Some(p) = response.interact_pointer_pos()
            {
                self.0.borrow_mut().push(p);
            }
        }
        fn paint(&mut self, _: &egui::Painter, _: Rect, _: &Palette, _: f32) {}
        fn controls(&mut self, _: &mut egui::Ui, _: &Palette) {}
    }

    /// Runs the workspace frame by frame on a 1600 x 1000 screen.
    #[derive(Default)]
    struct Frames {
        ctx: egui::Context,
        time: f64,
    }

    impl Frames {
        fn run(&mut self, ws: &mut Workspace, events: Vec<egui::Event>) {
            self.time += 1.0 / 60.0;
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 1000.0))),
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let (chrome, palette) = (Chrome::graphite(), super::super::theme::palette(true));
            let mut out = self.ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().show(ui, |ui| ws.show(ui, &chrome, &palette));
            });
            out.textures_delta.clear();
        }
    }

    /// Drag with the primary button from `from` to `to` (screen points),
    /// one frame per event.
    fn drag_frames(from: Pos2, to: Pos2) -> Vec<Vec<egui::Event>> {
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        let mut frames = vec![
            vec![egui::Event::PointerMoved(from)],
            vec![button(from, true)],
        ];
        for k in 1..=8 {
            frames.push(vec![egui::Event::PointerMoved(
                from.lerp(to, k as f32 / 8.0),
            )]);
        }
        frames.push(vec![button(to, false)]);
        frames.push(vec![]);
        frames
    }

    /// Dragging inside the selected viewport goes to its formula, in world
    /// coordinates, and leaves the viewport where it is. Dragging an
    /// unselected viewport moves it. (Guards against anything on the overlay
    /// hiding the world layer from egui's hit test.)
    #[test]
    fn drags_reach_the_active_formula_or_move_the_viewport() {
        let mut ws = workspace();
        let lerp = formulas::tool("lerp").unwrap();
        let a = ws.place(lerp, Some(pos2(0.0, 0.0)), vec2(400.0, 300.0));
        let b = ws.place(lerp, Some(pos2(480.0, 0.0)), vec2(400.0, 300.0));
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        ws.board.get_mut(a).unwrap().formula = Box::new(Probe(seen.clone()));
        ws.board.selection = vec![a];
        ws.camera = Camera::default();
        // Off-center, so the world-to-screen translation is not zero.
        ws.camera.center = pos2(500.0, 200.0);
        let (rect_a, rect_b) = (ws.board.get(a).unwrap().rect, ws.board.get(b).unwrap().rect);

        let mut frames = Frames::default();
        frames.run(&mut ws, vec![]);
        let t = ws.camera.transform(ws.view, 1.0);

        for events in drag_frames(t.mul_pos(pos2(100.0, 150.0)), t.mul_pos(pos2(160.0, 190.0))) {
            frames.run(&mut ws, events);
        }
        let seen = seen.borrow();
        assert!(!seen.is_empty(), "the formula saw no drag");
        let last = *seen.last().unwrap();
        assert!(last.distance(pos2(160.0, 190.0)) < 1.0, "{last:?}");
        assert_eq!(ws.board.get(a).unwrap().rect, rect_a);

        for events in drag_frames(t.mul_pos(pos2(600.0, 150.0)), t.mul_pos(pos2(600.0, 350.0))) {
            frames.run(&mut ws, events);
        }
        let moved = ws.board.get(b).unwrap().rect;
        assert_eq!(moved.size(), rect_b.size());
        assert!(moved.top() > rect_b.top() + 150.0, "{moved:?}");
    }

    /// Dragging from the navigator out past its edge keeps the camera on the
    /// drawing (the live map includes the view, which moves with the camera).
    #[test]
    fn navigator_drag_stays_on_the_drawing() {
        let mut ws = workspace();
        let lerp = formulas::tool("lerp").unwrap();
        ws.place(lerp, Some(pos2(0.0, 0.0)), vec2(400.0, 300.0));
        ws.place(lerp, Some(pos2(480.0, 0.0)), vec2(400.0, 300.0));
        ws.board.selection.clear();
        ws.camera = Camera::default();
        ws.camera.center = pos2(440.0, 150.0);

        let mut frames = Frames::default();
        frames.run(&mut ws, vec![]);
        let map = overlay::minimap_rect(ws.view);
        let from = map.center();
        let to = map.left_top() - vec2(400.0, 300.0);
        let mut steps = drag_frames(from, to);
        // Hold the pointer out there for a while.
        let last = steps.pop().unwrap();
        let release = steps.pop().unwrap();
        steps.extend(std::iter::repeat_n(vec![], 60));
        steps.push(release);
        steps.push(last);
        for events in steps {
            frames.run(&mut ws, events);
        }
        let c = ws.camera.center;
        assert!(
            c.x.abs() < 3000.0 && c.y.abs() < 3000.0,
            "camera ran away to {c:?}"
        );
    }

    #[test]
    fn marquee_window_vs_crossing() {
        let mut ws = workspace();
        let lerp = formulas::tool("lerp").unwrap();
        let a = ws.place(lerp, Some(pos2(0.0, 0.0)), vec2(300.0, 300.0));
        let _b = ws.place(lerp, Some(pos2(400.0, 0.0)), vec2(300.0, 300.0));
        let r = Rect::from_min_max(pos2(-10.0, -10.0), pos2(500.0, 600.0));
        ws.marquee_select(r, true, false);
        assert_eq!(ws.board.selection, vec![a]);
        ws.marquee_select(r, false, false);
        assert_eq!(ws.board.selection.len(), 2);
    }
}
