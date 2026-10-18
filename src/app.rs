//! The app shell: a CAD-style workspace. Menu bar and toolbar on top, the
//! Browser on the left, the Inspector on the right, the command line and
//! status bar below, and the drafting canvas in the middle.

use egui::{Frame, Margin, Stroke};

use crate::cad::board::{Board, ViewportModel};
use crate::cad::camera::Camera;
use crate::cad::canvas::{Settings, Workspace};
use crate::cad::chrome::{self, Panels};
use crate::cad::inspector::{Inspector, Tab};
use crate::cad::sheet::UiState;
use crate::cad::theme::{self, Chrome};
use crate::content::Lang;
use crate::formulas::{self, ToolEnv};

/// Storage key for the saved workspace.
const KEY: &str = "board-v1";
/// Below this width the side panels start hidden.
const NARROW: f32 = 900.0;

#[derive(serde::Serialize, serde::Deserialize)]
struct Saved {
    viewports: Vec<ViewportModel>,
    camera: Camera,
    settings: Settings,
    dark: bool,
    browser: bool,
    inspector: bool,
    tab: Tab,
    lang: Lang,
    /// The board's shared data (sorting list, graph grid); absent in older saves.
    #[serde(default)]
    shared: Option<serde_json::Map<String, serde_json::Value>>,
}

pub struct FormulaApp {
    ws: Workspace,
    panels: Panels,
    inspector: Inspector,
    /// The theme last applied to egui's style.
    applied: Option<bool>,
    /// Fit the drawing on the first frame (once the canvas size is known).
    fit_pending: bool,
    /// A formula to show once the canvas exists (from the URL).
    focus_pending: Option<String>,
    started: bool,
}

impl FormulaApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_initial(cc, None)
    }

    /// Restore the last workspace, or open the demo sheet. `initial` (a tool
    /// id, e.g. from the URL) is brought into view, placed first if needed.
    pub fn with_initial(cc: &eframe::CreationContext<'_>, initial: Option<&str>) -> Self {
        let saved: Option<Saved> = cc.storage.and_then(|s| eframe::get_value(s, KEY));
        let initial = initial
            .filter(|id| formulas::tool(id).is_some())
            .map(str::to_owned);
        let mut app = match saved {
            Some(s) if !s.viewports.is_empty() => {
                let mut env = ToolEnv::new();
                if let Some(shared) = &s.shared {
                    env.load_state(shared);
                }
                let board = Board::from_model(&s.viewports, env);
                let mut panels = Panels::new(s.dark);
                panels.browser = s.browser;
                panels.inspector = s.inspector;
                Self {
                    ws: Workspace::new(board, s.camera, s.settings),
                    panels,
                    inspector: Inspector::new(s.tab, s.lang),
                    applied: None,
                    fit_pending: false,
                    focus_pending: initial.clone(),
                    started: false,
                }
            }
            _ => {
                // First launch (or an old single-page version's choice).
                let remembered = cc.storage.and_then(|s| s.get_string("formula"));
                let focus = initial.clone().or(remembered);
                Self {
                    ws: Workspace::new(
                        Board::demo(focus.as_deref()),
                        Camera::default(),
                        Settings::default(),
                    ),
                    panels: Panels::new(true),
                    inspector: Inspector::new(Tab::Properties, Lang::Python),
                    applied: None,
                    fit_pending: true,
                    focus_pending: initial,
                    started: false,
                }
            }
        };
        app.ws
            .say("Welcome. Pick a formula tool and click the canvas, or type help.");
        cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);
        app
    }

    /// Keep the workspace's copy of the panel state current, and apply the
    /// panel state of a sheet that was just opened.
    fn sync_ui_state(&mut self) {
        if self.ws.ui_changed {
            self.ws.ui_changed = false;
            let ui = self.ws.ui;
            self.panels.dark = ui.dark;
            self.panels.browser = ui.browser;
            self.panels.inspector = ui.inspector;
            self.inspector.tab = ui.tab;
            self.inspector.lang = ui.lang;
        }
        self.ws.ui = UiState {
            dark: self.panels.dark,
            browser: self.panels.browser,
            inspector: self.panels.inspector,
            tab: self.inspector.tab,
            lang: self.inspector.lang,
        };
    }

    fn first_frame(&mut self, width: f32) {
        if width < NARROW {
            self.panels.browser = false;
            self.panels.inspector = false;
        }
        if let Some(id) = self.focus_pending.take()
            && let Some(tool) = formulas::tool(&id)
        {
            let uid = match self.ws.board.viewports.iter().find(|v| v.tool.id == id) {
                Some(vp) => vp.uid,
                None => self.ws.place(tool, None, egui::Vec2::ZERO),
            };
            self.ws.board.select_only(uid);
            self.ws.focus(uid);
            self.fit_pending = false;
        }
        if self.fit_pending {
            self.ws.fit_all();
            self.fit_pending = false;
        }
    }
}

impl eframe::App for FormulaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.sync_ui_state();
        let dark = self.panels.dark;
        if self.applied != Some(dark) {
            theme::apply(ui.ctx(), dark);
            self.applied = Some(dark);
        }
        let chrome = Chrome::for_dark(dark);
        let palette = theme::palette(dark);
        let width = ui.available_width();

        let bar = |fill| {
            Frame::new()
                .fill(fill)
                .inner_margin(Margin::symmetric(6, 3))
        };
        egui::Panel::top("cad-menu")
            .frame(bar(chrome.panel))
            .show(ui, |ui| {
                chrome::menu_bar(ui, &mut self.ws, &mut self.panels)
            });
        egui::Panel::top("cad-toolbar")
            .frame(bar(chrome.panel).stroke(Stroke::new(1.0, chrome.border)))
            .show(ui, |ui| {
                chrome::toolbar(ui, &mut self.ws, &mut self.panels, &chrome)
            });
        egui::Panel::bottom("cad-status")
            .frame(bar(chrome.panel))
            .show(ui, |ui| {
                chrome::status_bar(ui, &mut self.ws, &mut self.panels, &chrome)
            });
        egui::Panel::bottom("cad-command")
            .frame(
                Frame::new()
                    .fill(chrome.well)
                    .stroke(Stroke::new(1.0, chrome.border)),
            )
            .show(ui, |ui| {
                chrome::command_line(ui, &mut self.ws, &mut self.panels, &chrome)
            });
        if self.panels.browser {
            egui::Panel::left("cad-browser")
                .resizable(true)
                .default_size(232.0)
                .size_range(180.0..=380.0)
                .frame(
                    Frame::new()
                        .fill(chrome.panel)
                        .stroke(Stroke::new(1.0, chrome.border)),
                )
                .show(ui, |ui| {
                    chrome::browser(ui, &mut self.ws, &mut self.panels, &chrome)
                });
        }
        if self.panels.inspector {
            // The library's hovered/picked tool is what the inspector shows
            // when nothing is selected.
            self.inspector.tool = self.panels.picked;
            // Maximized, the panel has its own id so the normal width is kept.
            let panel = if self.inspector.maximized {
                egui::Panel::right("cad-inspector-max")
                    .resizable(false)
                    .exact_size((width * 0.62).max(380.0))
            } else {
                egui::Panel::right("cad-inspector")
                    .resizable(true)
                    .default_size(380.0)
                    .size_range(300.0..=760.0)
            };
            panel
                .frame(
                    Frame::new()
                        .fill(chrome.panel)
                        .stroke(Stroke::new(1.0, chrome.border)),
                )
                .show(ui, |ui| {
                    self.inspector.show(ui, &mut self.ws, &chrome, &palette)
                });
        } else if self.ws.request.is_some() {
            self.panels.inspector = true;
        }
        egui::CentralPanel::default()
            .frame(Frame::new().fill(chrome.canvas))
            .show(ui, |ui| {
                if !self.started {
                    self.started = true;
                    self.ws.view = ui.max_rect();
                    self.first_frame(width);
                }
                self.ws.show(ui, &chrome, &palette);
            });
        chrome::help_window(ui.ctx(), &mut self.panels, &chrome);

        #[cfg(target_arch = "wasm32")]
        {
            let id = self
                .ws
                .board
                .primary()
                .and_then(|uid| self.ws.board.get(uid))
                .map(|vp| vp.tool.id);
            crate::web::show_in_url(id.unwrap_or(""));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let saved = Saved {
            viewports: self.ws.board.to_model(),
            camera: self.ws.camera,
            settings: self.ws.settings,
            dark: self.panels.dark,
            browser: self.panels.browser,
            inspector: self.panels.inspector,
            tab: self.inspector.tab,
            lang: self.inspector.lang,
            shared: Some(self.ws.board.env.save_state()),
        };
        eframe::set_value(storage, KEY, &saved);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Chrome::for_dark(self.panels.dark)
            .canvas
            .to_normalized_gamma_f32()
    }
}
