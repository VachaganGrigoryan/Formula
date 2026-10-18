//! The right-hand panel: properties of the selected viewport, the full
//! reference documentation of its formula, and its exportable code.

use std::borrow::Cow;
use std::collections::HashMap;

use egui::{Color32, Frame, Margin, RichText, ScrollArea, Stroke, TextEdit};

use super::canvas::{Mode, Request, Workspace};
use super::highlight;
use super::icons::{self, Ui as Icon};
use super::markdown::{self, Block};
use super::theme::{Chrome, mono};
use super::{export, notebook};
use crate::content::{self, Lang};
use crate::derive::{Derivation, clamp_steps};
use crate::formula::{Formula, Palette};
use crate::formulas::{Tool, ToolEnv};
use crate::latex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Tab {
    Properties,
    Docs,
    Code,
    /// Appended last so saved values of the other variants keep their meaning.
    Derive,
}

pub struct Inspector {
    pub tab: Tab,
    pub lang: Lang,
    /// A tool picked in the library to read about while nothing is selected.
    pub tool: Option<&'static Tool>,
    docs: HashMap<&'static str, Vec<Block>>,
    /// One instance per tool, for its summary and equations before placing.
    previews: HashMap<&'static str, Box<dyn Formula>>,
    preview_env: ToolEnv,
    jump: Option<String>,
    /// Geometry or name before an edit started, for a single undo step.
    rect_before: Option<(u64, egui::Rect)>,
    title_before: Option<(u64, String)>,
    notice: Option<(String, f64)>,
    /// Widened for reading docs and code (not saved).
    pub maximized: bool,
    /// The Derive tab's place: which tool, which derivation, how many steps shown.
    derive_at: (Option<&'static str>, usize, usize),
}

impl Inspector {
    pub fn new(tab: Tab, lang: Lang) -> Self {
        Self {
            tab,
            lang,
            tool: None,
            docs: HashMap::new(),
            previews: HashMap::new(),
            preview_env: ToolEnv::new(),
            jump: None,
            rect_before: None,
            title_before: None,
            notice: None,
            maximized: false,
            derive_at: (None, 0, 0),
        }
    }

    fn preview(&mut self, tool: &'static Tool) -> &dyn Formula {
        let env = &self.preview_env;
        &**self
            .previews
            .entry(tool.id)
            .or_insert_with(|| (tool.make)(env))
    }

    /// The tool the panel is about: the selected viewport's, the one being
    /// placed, or the one picked in the library.
    pub fn context(&self, ws: &Workspace) -> Option<&'static Tool> {
        if let Some(uid) = ws.board.primary()
            && let Some(vp) = ws.board.get(uid)
        {
            return Some(vp.tool);
        }
        if let Mode::Place(t) = ws.mode {
            return Some(t);
        }
        self.tool
    }

    /// The formula instance the panel is about: the selected viewport's own
    /// (with its current parameters), or else a preview of the context tool.
    pub fn context_formula<'a>(&'a mut self, ws: &'a Workspace) -> Option<&'a dyn Formula> {
        if let Some(uid) = ws.board.primary()
            && let Some(vp) = ws.board.get(uid)
        {
            return Some(&*vp.formula);
        }
        let tool = self.context(ws)?;
        Some(self.preview(tool))
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        ws: &mut Workspace,
        chrome: &Chrome,
        palette: &Palette,
    ) {
        match ws.request.take() {
            Some(Request::Properties) => self.tab = Tab::Properties,
            Some(Request::Docs) => self.tab = Tab::Docs,
            Some(Request::Derive) => self.tab = Tab::Derive,
            Some(Request::Code) => self.tab = Tab::Code,
            None => {}
        }
        let tool = self.context(ws);
        let now = ui.input(|i| i.time);

        // Header.
        Frame::new()
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
                    match tool {
                        Some(t) => {
                            icons::tool_badge(ui.painter(), rect, t.id, chrome.accent, chrome.well)
                        }
                        None => {
                            ui.painter().rect_filled(rect, 3.0, chrome.well);
                            icons::ui(ui.painter(), rect.shrink(6.0), Icon::Layers, chrome.accent);
                        }
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        match tool {
                            Some(t) => {
                                ui.label(RichText::new(t.name).size(15.0).color(chrome.text));
                                ui.label(
                                    RichText::new(format!(
                                        "{}  ·  {}",
                                        t.category.to_uppercase(),
                                        t.alias
                                    ))
                                    .font(mono(9.5))
                                    .color(chrome.text_dim),
                                );
                            }
                            None => {
                                ui.label(RichText::new("Sheet").size(15.0).color(chrome.text));
                                ui.label(
                                    RichText::new("NOTHING SELECTED")
                                        .font(mono(9.5))
                                        .color(chrome.text_dim),
                                );
                            }
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        let (icon, tip) = if self.maximized {
                            (Icon::Restore, "Restore the Inspector's width")
                        } else {
                            (Icon::Maximize, "Widen the Inspector for reading")
                        };
                        if icons::button(ui, icon, self.maximized, tip).clicked() {
                            self.maximized = !self.maximized;
                        }
                    });
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    for (tab, icon, label) in [
                        (Tab::Properties, Icon::Properties, "Properties"),
                        (Tab::Docs, Icon::Docs, "Docs"),
                        (Tab::Derive, Icon::Derive, "Derive"),
                        (Tab::Code, Icon::Code, "Code"),
                    ] {
                        if tab_button(ui, icon, label, self.tab == tab, chrome).clicked() {
                            self.tab = tab;
                        }
                    }
                });
            });
        ui.painter().line_segment(
            [
                ui.cursor().left_top(),
                egui::pos2(ui.max_rect().right(), ui.cursor().top()),
            ],
            Stroke::new(1.0, chrome.border),
        );

        match self.tab {
            Tab::Properties => {
                // Both ways: a formula's controls may be wider than the
                // panel, and must scroll rather than push the panel wider.
                ScrollArea::both()
                    .id_salt("inspector-props")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        Frame::new()
                            .inner_margin(Margin::symmetric(10, 8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                self.properties(ui, ws, tool, chrome, palette)
                            });
                    });
            }
            Tab::Docs => match tool {
                Some(t) => self.docs(ui, t, chrome, palette),
                None => hint(
                    ui,
                    chrome,
                    "Select a viewport, or pick a tool in the library, to read its documentation.",
                ),
            },
            Tab::Derive => match tool {
                Some(t) => self.derive(ui, ws, t, chrome, palette),
                None => hint(
                    ui,
                    chrome,
                    "Select a viewport, or pick a tool in the library, to see how its formula is derived.",
                ),
            },
            Tab::Code => match tool {
                Some(t) => self.code(ui, ws, t, chrome, now),
                None => hint(
                    ui,
                    chrome,
                    "Select a viewport, or pick a tool in the library, to export its code.",
                ),
            },
        }
    }

    fn properties(
        &mut self,
        ui: &mut egui::Ui,
        ws: &mut Workspace,
        tool: Option<&'static Tool>,
        chrome: &Chrome,
        palette: &Palette,
    ) {
        let selection = ws.board.selection.clone();
        if selection.len() > 1 {
            section(ui, chrome, "SELECTION");
            ui.label(format!("{} viewports selected", selection.len()));
            ui.horizontal(|ui| {
                if ui.button("Duplicate").clicked() {
                    ws.duplicate_selection();
                }
                if ui.button("Delete").clicked() {
                    ws.delete_selection();
                }
                if ui.button("Focus").clicked() {
                    ws.focus_selection();
                }
            });
            return;
        }
        if let Some(uid) = ws.board.primary() {
            self.viewport(ui, ws, uid, chrome, palette);
            return;
        }
        match tool {
            Some(t) => {
                let placing = ws.mode == Mode::Place(t);
                section(ui, chrome, "TOOL");
                let f = self.preview(t);
                ui.label(RichText::new(f.summary()).color(chrome.text_dim));
                ui.add_space(6.0);
                for eq in f.equations() {
                    ui.label(eq.layout(palette, 15.0));
                }
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(if placing {
                                "Placing… click the canvas"
                            } else {
                                "Place on canvas"
                            })
                            .selected(placing),
                        )
                        .clicked()
                    {
                        ws.set_mode(Mode::Place(t));
                    }
                    if ui.button("Place beside drawing").clicked() {
                        ws.place(t, None, egui::Vec2::ZERO);
                    }
                });
            }
            None => self.sheet(ui, ws, chrome),
        }
    }

    fn viewport(
        &mut self,
        ui: &mut egui::Ui,
        ws: &mut Workspace,
        uid: u64,
        chrome: &Chrome,
        palette: &Palette,
    ) {
        section(ui, chrome, "VIEWPORT");
        let mut reshaped = false;
        let mut title_done = false;
        let mut actions: Vec<&str> = Vec::new();
        {
            let Some(vp) = ws.board.get_mut(uid) else {
                return;
            };
            let min = vp.min_size();
            egui::Grid::new(("vp-props", uid))
                .num_columns(4)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Name").color(chrome.text_dim));
                    let before = vp.title.clone();
                    let r = ui.add(TextEdit::singleline(&mut vp.title).desired_width(120.0));
                    if r.gained_focus() {
                        self.title_before = Some((uid, before));
                    }
                    if r.lost_focus() {
                        title_done = true;
                    }
                    ui.label(RichText::new("Tool").color(chrome.text_dim));
                    ui.label(
                        RichText::new(vp.tool.alias)
                            .font(mono(12.0))
                            .color(chrome.accent),
                    );
                    ui.end_row();

                    let start = vp.rect;
                    let (mut x, mut y, mut w, mut h) =
                        (start.left(), start.top(), start.width(), start.height());
                    let mut edits = Vec::new();
                    ui.label(RichText::new("X").color(chrome.axis_x));
                    edits.push(ui.add(egui::DragValue::new(&mut x).speed(1.0).max_decimals(0)));
                    ui.label(RichText::new("Y").color(chrome.axis_y));
                    edits.push(ui.add(egui::DragValue::new(&mut y).speed(1.0).max_decimals(0)));
                    ui.end_row();
                    ui.label(RichText::new("W").color(chrome.dimension));
                    edits.push(
                        ui.add(
                            egui::DragValue::new(&mut w)
                                .speed(1.0)
                                .range(min.x..=8000.0)
                                .max_decimals(0),
                        ),
                    );
                    ui.label(RichText::new("H").color(chrome.dimension));
                    edits.push(
                        ui.add(
                            egui::DragValue::new(&mut h)
                                .speed(1.0)
                                .range(min.y..=8000.0)
                                .max_decimals(0),
                        ),
                    );
                    ui.end_row();
                    if edits.iter().any(|r| r.drag_started() || r.gained_focus()) {
                        self.rect_before = Some((uid, start));
                    }
                    vp.rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h));
                    reshaped = edits.iter().any(|r| r.drag_stopped() || r.lost_focus());

                    ui.label(RichText::new("Lock").color(chrome.text_dim));
                    ui.checkbox(&mut vp.locked, "");
                    ui.label(RichText::new("Hide").color(chrome.text_dim));
                    ui.checkbox(&mut vp.hidden, "");
                    ui.end_row();
                });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for a in ["Focus", "Duplicate", "To front", "Delete"] {
                    if ui.button(a).clicked() {
                        actions.push(a);
                    }
                }
            });
        }
        if title_done
            && let Some((u, before)) = self.title_before.take()
            && u == uid
        {
            ws.rename(uid, before);
        }
        if reshaped
            && let Some((u, before)) = self.rect_before.take()
            && u == uid
        {
            ws.reshaped(uid, before);
        }
        for a in actions {
            match a {
                "Focus" => ws.focus(uid),
                "Duplicate" => ws.duplicate_selection(),
                "To front" => ws.board.bring_to_front(uid),
                _ => ws.delete_selection(),
            }
        }

        let Some(vp) = ws.board.get_mut(uid) else {
            return;
        };
        section(ui, chrome, "FORMULA");
        ui.label(RichText::new(vp.formula.summary()).color(chrome.text_dim));
        ui.add_space(6.0);
        Frame::new()
            .fill(chrome.well)
            .inner_margin(Margin::symmetric(10, 6))
            .corner_radius(3)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for eq in vp.formula.equations() {
                    ui.label(eq.layout(palette, 15.0));
                }
            });
        section(ui, chrome, "CONTROLS");
        ui.push_id(("controls", uid), |ui| {
            ui.spacing_mut().slider_width = (ui.available_width() - 170.0).max(80.0);
            vp.formula.controls(ui, palette);
        });
        ui.add_space(8.0);
        ui.push_id(("details", uid), |ui| {
            vp.formula.details(ui, palette);
        });
    }

    fn sheet(&mut self, ui: &mut egui::Ui, ws: &mut Workspace, chrome: &Chrome) {
        section(ui, chrome, "SHEET");
        let n = ws.board.viewports.len();
        ui.label(format!("{n} viewport{}", if n == 1 { "" } else { "s" }));
        ui.label(
            RichText::new(
                "Pick a formula tool from the toolbar or the library, then click on the \
                 canvas to place it, or drag out a rectangle. Type its alias in the \
                 command line (LOR, BEZ, VOR…) to place it beside the drawing.",
            )
            .color(chrome.text_dim),
        );
        section(ui, chrome, "DRAFTING SETTINGS");
        let s = &mut ws.settings;
        ui.checkbox(&mut s.grid, "Grid  (G)");
        ui.checkbox(&mut s.snap, "Snap to grid  (S)");
        ui.checkbox(&mut s.ortho, "Ortho: move along one axis  (O)");
        ui.checkbox(&mut s.rulers, "Rulers");
        ui.checkbox(&mut s.minimap, "Navigator");
        ui.checkbox(
            &mut s.animate_all,
            "Animate every viewport (off: only selected)",
        );
    }

    fn docs(&mut self, ui: &mut egui::Ui, tool: &'static Tool, chrome: &Chrome, palette: &Palette) {
        let Some(src) = content::doc(tool.id) else {
            hint(ui, chrome, "No documentation for this tool yet.");
            return;
        };
        let blocks = self
            .docs
            .entry(tool.id)
            .or_insert_with(|| markdown::parse(src));
        let jump = self.jump.take();
        Frame::new()
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                    for s in markdown::sections(blocks) {
                        if ui.small_button(s).clicked() {
                            self.jump = Some(s.to_owned());
                        }
                    }
                });
            });
        ScrollArea::both()
            .id_salt(("docs", tool.id))
            .auto_shrink(false)
            .show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        markdown::show(ui, blocks, chrome, palette, jump.as_deref());
                        ui.add_space(24.0);
                    });
            });
    }

    fn derive(
        &mut self,
        ui: &mut egui::Ui,
        ws: &Workspace,
        tool: &'static Tool,
        chrome: &Chrome,
        palette: &Palette,
    ) {
        let list: Vec<Derivation> = self
            .context_formula(ws)
            .map(|f| f.derivations())
            .unwrap_or_default();
        if list.is_empty() {
            hint(ui, chrome, "No derivation for this tool yet.");
            return;
        }
        // A different tool starts from its first step.
        if self.derive_at.0 != Some(tool.id) {
            self.derive_at = (Some(tool.id), 0, 0);
        }
        let lens: Vec<usize> = list.iter().map(|d| d.steps.len()).collect();
        let (mut pick, mut step) = clamp_steps(self.derive_at.1, self.derive_at.2, &lens);
        ScrollArea::vertical()
            .id_salt(("derive", tool.id))
            .auto_shrink(false)
            .show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if list.len() > 1 {
                            ui.horizontal_wrapped(|ui| {
                                for (i, d) in list.iter().enumerate() {
                                    if ui.selectable_label(i == pick, &d.title).clicked()
                                        && i != pick
                                    {
                                        (pick, step) = (i, 0);
                                    }
                                }
                            });
                            ui.add_space(4.0);
                        }
                        let d = &list[pick];
                        section(ui, chrome, &d.title.to_uppercase());
                        let n = d.steps.len();
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(step > 0, egui::Button::new("Prev"))
                                .clicked()
                            {
                                step -= 1;
                            }
                            if ui
                                .add_enabled(step + 1 < n, egui::Button::new("Next"))
                                .clicked()
                            {
                                step += 1;
                            }
                            if ui.button("Show all").clicked() {
                                step = n.saturating_sub(1);
                            }
                            if ui.button("Restart").clicked() {
                                step = 0;
                            }
                            ui.label(
                                RichText::new(format!("step {} of {n}", (step + 1).min(n)))
                                    .font(mono(10.5))
                                    .color(chrome.text_dim),
                            );
                        });
                        ui.add_space(6.0);
                        for (i, s) in d.steps.iter().enumerate().take(step + 1) {
                            let newest = i == step;
                            Frame::new()
                                .fill(if newest {
                                    chrome.well
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .inner_margin(Margin::symmetric(8, 5))
                                .corner_radius(3)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(s.math.layout(palette, 15.0));
                                    if !s.why.is_empty() {
                                        ui.label(RichText::new(&s.why).color(chrome.text_dim));
                                    }
                                });
                        }
                    });
            });
        self.derive_at = (Some(tool.id), pick, step);
    }

    fn code(
        &mut self,
        ui: &mut egui::Ui,
        ws: &Workspace,
        tool: &'static Tool,
        chrome: &Chrome,
        now: f64,
    ) {
        Frame::new()
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for lang in Lang::ALL {
                        if ui
                            .selectable_label(self.lang == lang, lang.label())
                            .clicked()
                        {
                            self.lang = lang;
                        }
                    }
                });
                let lang = self.lang;
                let Some(src) = code_source(self.context_formula(ws), tool.id, lang) else {
                    hint(ui, chrome, "No code export for this tool yet.");
                    return;
                };
                ui.add_space(4.0);
                ui.label(
                    RichText::new(self.lang.run_hint())
                        .font(mono(10.5))
                        .color(chrome.text_dim),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if icon_text_button(ui, Icon::Copy, "Copy", chrome).clicked() {
                        export::copy(ui.ctx(), &src);
                        self.notice = Some((
                            format!("Copied {} to the clipboard", self.lang.file_name()),
                            now,
                        ));
                    }
                    if icon_text_button(ui, Icon::Download, export::SAVE_LABEL, chrome).clicked() {
                        self.notice = Some((
                            export::saved_message(export::save(
                                tool.id,
                                self.lang.file_name(),
                                &src,
                            )),
                            now,
                        ));
                    }
                    let lines = src.lines().count();
                    ui.label(
                        RichText::new(format!("{lines} lines"))
                            .font(mono(10.5))
                            .color(chrome.text_dim),
                    );
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Also export")
                            .font(mono(10.5))
                            .color(chrome.text_dim),
                    );
                    if icon_text_button(ui, Icon::Copy, "LaTeX", chrome)
                        .on_hover_text(
                            "The equations with the current values, for a paper or notes",
                        )
                        .clicked()
                        && let Some(f) = self.context_formula(ws)
                    {
                        let tex = latex::latex_snippet(f);
                        export::copy(ui.ctx(), &tex);
                        self.notice = Some(("Copied the LaTeX to the clipboard".into(), now));
                    }
                    if icon_text_button(ui, Icon::Download, "Notebook", chrome)
                        .on_hover_text(
                            "A Jupyter notebook: overview, equations and the Python code",
                        )
                        .clicked()
                    {
                        let nb = self
                            .context_formula(ws)
                            .and_then(|f| notebook::for_formula(tool.id, f));
                        let msg = match nb {
                            Some(nb) => export::saved_message(export::save(
                                tool.id,
                                notebook::FILE_NAME,
                                &nb,
                            )),
                            None => "No notebook export for this tool".into(),
                        };
                        self.notice = Some((msg, now));
                    }
                });
                if let Some((msg, at)) = &self.notice {
                    let age = (now - at) as f32;
                    if age < 4.0 {
                        ui.label(
                            RichText::new(msg)
                                .small()
                                .color(chrome.accent.gamma_multiply((4.0 - age).min(1.0))),
                        );
                        ui.ctx().request_repaint();
                    } else {
                        self.notice = None;
                    }
                }
            });
        let lang = self.lang;
        let Some(src) = code_source(self.context_formula(ws), tool.id, lang) else {
            return;
        };
        let dark = chrome.dark;
        let plain = chrome.text;
        Frame::new()
            .fill(chrome.well)
            .stroke(Stroke::new(1.0, chrome.border))
            .inner_margin(Margin::same(6))
            .show(ui, |ui| {
                ScrollArea::both()
                    .id_salt(("code", tool.id, lang.label()))
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            let numbers: String = (1..=src.lines().count())
                                .map(|n| format!("{n:>4}\n"))
                                .collect();
                            ui.label(
                                RichText::new(numbers)
                                    .font(mono(12.0))
                                    .color(chrome.text_dim.gamma_multiply(0.6)),
                            );
                            let mut text: &str = &src;
                            let mut layouter =
                                |ui: &egui::Ui, buf: &dyn egui::TextBuffer, _wrap: f32| {
                                    let job =
                                        highlight::layout(buf.as_str(), lang, dark, plain, 12.0);
                                    ui.fonts_mut(|f| f.layout_job(job))
                                };
                            ui.add(
                                TextEdit::multiline(&mut text)
                                    .code_editor()
                                    .frame(Frame::new())
                                    .desired_width(f32::INFINITY)
                                    .layouter(&mut layouter),
                            );
                        });
                    });
            });
    }
}

fn section(ui: &mut egui::Ui, chrome: &Chrome, title: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(title).font(mono(10.0)).color(chrome.accent));
    let y = ui.cursor().top() - 1.0;
    ui.painter().line_segment(
        [
            egui::pos2(ui.max_rect().left(), y),
            egui::pos2(ui.max_rect().right(), y),
        ],
        Stroke::new(1.0, chrome.border),
    );
    ui.add_space(4.0);
}

fn hint(ui: &mut egui::Ui, chrome: &Chrome, text: &str) {
    Frame::new().inner_margin(Margin::same(14)).show(ui, |ui| {
        ui.label(RichText::new(text).color(chrome.text_dim));
    });
}

fn tab_button(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    on: bool,
    chrome: &Chrome,
) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(12.5),
        chrome.text,
    );
    let size = egui::vec2(galley.size().x + 32.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let fill = if on {
        chrome.well
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 3.0, fill);
    if !on {
        ui.painter()
            .rect_filled(rect, 3.0, chrome.tab.gamma_multiply(hover));
    }
    if on {
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(2.0, chrome.accent),
        );
    }
    let color = if on { chrome.accent } else { chrome.text_dim };
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(14.0, 0.0),
        egui::vec2(13.0, 13.0),
    );
    icons::ui(ui.painter(), icon_rect, icon, color);
    ui.painter().galley(
        egui::pos2(rect.left() + 25.0, rect.center().y - galley.size().y / 2.0),
        galley,
        if on { chrome.text } else { chrome.text_dim },
    );
    response
}

fn icon_text_button(ui: &mut egui::Ui, icon: Icon, label: &str, chrome: &Chrome) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(12.5),
        chrome.text,
    );
    let size = egui::vec2(galley.size().x + 30.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    ui.painter().rect_filled(rect, 3.0, chrome.tab);
    ui.painter().rect_stroke(
        rect,
        3.0,
        Stroke::new(1.0, chrome.accent.gamma_multiply(0.3 + 0.7 * hover)),
        egui::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(12.0, 0.0),
        egui::vec2(12.0, 12.0),
    );
    icons::ui(ui.painter(), icon_rect, icon, chrome.accent);
    ui.painter().galley(
        egui::pos2(rect.left() + 23.0, rect.center().y - galley.size().y / 2.0),
        galley,
        chrome.text,
    );
    response
}

/// The code the Code tab shows: what the formula generates for itself (the
/// expression plotter writes its rows into the file), else the static file.
pub fn code_source(
    formula: Option<&dyn Formula>,
    id: &str,
    lang: Lang,
) -> Option<Cow<'static, str>> {
    formula
        .and_then(|f| f.export_code(lang))
        .map(Cow::Owned)
        .or_else(|| content::code(id, lang).map(Cow::Borrowed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cad::board::Board;
    use crate::cad::camera::Camera;
    use crate::cad::canvas::Settings;
    use crate::formulas;

    fn workspace() -> Workspace {
        Workspace::new(Board::default(), Camera::default(), Settings::default())
    }

    #[test]
    fn context_formula_is_the_selected_viewports_instance() {
        let mut ws = workspace();
        let tool = formulas::tool("lissajous").unwrap();
        let uid = ws.place(tool, Some(egui::pos2(0.0, 0.0)), egui::Vec2::ZERO);
        ws.board.selection = vec![uid];
        let live: *const dyn Formula = &*ws.board.get(uid).unwrap().formula;
        let mut inspector = Inspector::new(Tab::Code, Lang::Python);
        let f = inspector.context_formula(&ws).unwrap();
        assert!(std::ptr::addr_eq(f, live));
    }

    #[test]
    fn context_formula_falls_back_to_a_preview() {
        let mut ws = workspace();
        let placed = formulas::tool("lissajous").unwrap();
        ws.place(placed, Some(egui::pos2(0.0, 0.0)), egui::Vec2::ZERO);
        ws.board.selection.clear();
        let mut inspector = Inspector::new(Tab::Code, Lang::Python);
        assert!(inspector.context_formula(&ws).is_none());
        inspector.tool = formulas::tool("mandelbrot");
        assert_eq!(inspector.context_formula(&ws).unwrap().id(), "mandelbrot");
    }

    #[test]
    fn code_source_prefers_generated_code() {
        use crate::formulas::expression::Expression;
        let plot = Expression::with_rows(&["y = q x^3"]);
        let generated = code_source(Some(&plot), "expression", Lang::Python).unwrap();
        assert!(generated.contains("return q * x ** 3"), "{generated}");
        let lissajous = formulas::registry()
            .into_iter()
            .find(|f| f.id() == "lissajous")
            .unwrap();
        assert_eq!(
            code_source(Some(&*lissajous), "lissajous", Lang::Rust).as_deref(),
            content::code("lissajous", Lang::Rust)
        );
        assert_eq!(
            code_source(None, "expression", Lang::TypeScript).as_deref(),
            content::code("expression", Lang::TypeScript)
        );
        assert!(code_source(None, "no-such-tool", Lang::Python).is_none());
    }

    #[test]
    fn saved_tabs_still_load() {
        for (json, tab) in [
            ("\"Properties\"", Tab::Properties),
            ("\"Docs\"", Tab::Docs),
            ("\"Code\"", Tab::Code),
            ("\"Derive\"", Tab::Derive),
        ] {
            assert_eq!(serde_json::from_str::<Tab>(json).unwrap(), tab);
            assert_eq!(serde_json::to_string(&tab).unwrap(), json);
        }
    }
}
