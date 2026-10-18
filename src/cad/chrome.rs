//! The panels around the canvas: menu bar, toolbar, the Browser (tool
//! library and viewport list), command line and status bar.

use egui::{Color32, Frame, Key, Margin, RichText, ScrollArea, Stroke, TextEdit};

use super::canvas::{FileRequest, Mode, Workspace};
use super::icons::{self, Ui as Icon};
use super::theme::{Chrome, mono};
use crate::formulas::{self, Tool};

/// State of the panels that is not part of the drawing.
pub struct Panels {
    pub dark: bool,
    pub browser: bool,
    pub inspector: bool,
    pub help: bool,
    pub command: String,
    pub history: Vec<String>,
    history_pos: Option<usize>,
    /// Tool the user is reading about in the library (shown by the inspector).
    pub picked: Option<&'static Tool>,
    /// Frames per second, smoothed.
    fps: f32,
    focus_command: bool,
}

impl Panels {
    pub fn new(dark: bool) -> Self {
        Self {
            dark,
            browser: true,
            inspector: true,
            help: false,
            command: String::new(),
            history: Vec::new(),
            history_pos: None,
            picked: None,
            fps: 60.0,
            focus_command: false,
        }
    }
}

/// Short labels for the toolbar's category buttons.
fn short(category: &str) -> &'static str {
    match category {
        formulas::COORDINATES => "Coordinates",
        formulas::CURVES => "Curves",
        formulas::PHYSICS => "Physics",
        formulas::SCALING => "Scaling",
        formulas::FRACTALS => "Generative",
        formulas::CHAOS => "Chaos",
        formulas::GEOMETRY => "Geometry",
        formulas::LINEAR_ALGEBRA => "Matrices",
        formulas::STATISTICS => "Statistics",
        formulas::ANIMATION => "Easing",
        formulas::COLOR => "Color",
        formulas::SORTING => "Sorting",
        formulas::GRAPHS => "Graphs",
        formulas::PROOFS => "Proofs",
        _ => "Other",
    }
}

pub fn menu_bar(ui: &mut egui::Ui, ws: &mut Workspace, p: &mut Panels) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New sheet").clicked() {
                ws.new_sheet();
                ui.close();
            }
            if ui
                .add(egui::Button::new("Open...").shortcut_text("Ctrl+O"))
                .clicked()
            {
                ws.file_request = Some(FileRequest::Open);
                ui.close();
            }
            if ui
                .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                .clicked()
            {
                ws.save();
                ui.close();
            }
            if ui
                .add(egui::Button::new("Save As...").shortcut_text("Ctrl+Shift+S"))
                .clicked()
            {
                ws.file_request = Some(FileRequest::SaveAs);
                ui.close();
            }
            #[cfg(not(target_arch = "wasm32"))]
            ui.menu_button("Recent", |ui| {
                if ws.files.recent.is_empty() {
                    ui.weak("No recent sheets");
                }
                for path in ws.files.recent.clone() {
                    if ui
                        .button(super::files::file_name(&path))
                        .on_hover_text(&path)
                        .clicked()
                    {
                        ws.open_path(&path);
                        ui.close();
                    }
                }
                if !ws.files.recent.is_empty() {
                    ui.separator();
                    if ui.button("Clear recent").clicked() {
                        ws.files.recent.clear();
                        ui.close();
                    }
                }
            });
            ui.separator();
            if ui.button("Demo sheet").clicked() {
                ws.run_command("clear");
                let demo = super::board::Board::demo(None);
                for vp in demo.to_model() {
                    if let Some(t) = formulas::tool(&vp.tool) {
                        let [x, y, w, h] = vp.rect;
                        ws.place(t, Some(egui::pos2(x, y)), egui::vec2(w, h));
                    }
                }
                ws.board.selection.clear();
                ws.fit_all();
                ui.close();
            }
        });
        ui.menu_button("Edit", |ui| {
            if ui
                .add_enabled(
                    ws.history.can_undo(),
                    egui::Button::new("Undo").shortcut_text("Ctrl+Z"),
                )
                .clicked()
            {
                ws.undo();
                ui.close();
            }
            if ui
                .add_enabled(
                    ws.history.can_redo(),
                    egui::Button::new("Redo").shortcut_text("Ctrl+Y"),
                )
                .clicked()
            {
                ws.redo();
                ui.close();
            }
            ui.separator();
            let any = !ws.board.selection.is_empty();
            if ui
                .add_enabled(any, egui::Button::new("Duplicate").shortcut_text("Ctrl+D"))
                .clicked()
            {
                ws.duplicate_selection();
                ui.close();
            }
            if ui
                .add_enabled(any, egui::Button::new("Delete").shortcut_text("Del"))
                .clicked()
            {
                ws.delete_selection();
                ui.close();
            }
            if ui
                .add(egui::Button::new("Select all").shortcut_text("Ctrl+A"))
                .clicked()
            {
                ws.select_all();
                ui.close();
            }
        });
        ui.menu_button("View", |ui| {
            if ui
                .add(egui::Button::new("Zoom to fit").shortcut_text("F"))
                .clicked()
            {
                ws.fit_all();
                ui.close();
            }
            if ui
                .add(egui::Button::new("Zoom to selection").shortcut_text("Shift+F"))
                .clicked()
            {
                ws.focus_selection();
                ui.close();
            }
            if ui
                .add(egui::Button::new("Zoom 100%").shortcut_text("Ctrl+0"))
                .clicked()
            {
                ws.camera.zoom_to(1.0);
                ui.close();
            }
            ui.separator();
            ui.checkbox(&mut ws.settings.grid, "Grid");
            ui.checkbox(&mut ws.settings.rulers, "Rulers");
            ui.checkbox(&mut ws.settings.minimap, "Navigator");
            ui.checkbox(&mut ws.settings.animate_all, "Animate all viewports");
            ui.separator();
            ui.checkbox(&mut p.browser, "Browser panel");
            ui.checkbox(&mut p.inspector, "Inspector panel");
            ui.separator();
            if ui
                .button(if p.dark {
                    "Light theme (drafting paper)"
                } else {
                    "Dark theme (graphite)"
                })
                .clicked()
            {
                p.dark = !p.dark;
                ui.close();
            }
        });
        ui.menu_button("Place", |ui| {
            for (category, tools) in formulas::categories() {
                ui.menu_button(category, |ui| {
                    for tool in tools {
                        if tool_row(ui, tool, false).clicked() {
                            ws.set_mode(Mode::Place(tool));
                            ui.close();
                        }
                    }
                });
            }
        });
        ui.menu_button("Help", |ui| {
            if ui.button("Keyboard and mouse").clicked() {
                p.help = true;
                ui.close();
            }
            if ui.button("Commands").clicked() {
                ws.run_command("help");
                ui.close();
            }
        });
    });
}

/// A menu row: tool icon, name and alias.
fn tool_row(ui: &mut egui::Ui, tool: &'static Tool, on: bool) -> egui::Response {
    let height = 22.0;
    let width = ui.available_width().max(220.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    let v = ui.visuals();
    if response.hovered() || on {
        ui.painter().rect_filled(rect, 2.0, v.selection.bg_fill);
    }
    let color = v.text_color();
    let icon = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(12.0, 0.0),
        egui::vec2(15.0, 15.0),
    );
    icons::tool(ui.painter(), icon, tool.id, color);
    ui.painter().text(
        rect.left_center() + egui::vec2(28.0, 0.0),
        egui::Align2::LEFT_CENTER,
        tool.name,
        egui::FontId::proportional(13.0),
        color,
    );
    ui.painter().text(
        rect.right_center() - egui::vec2(6.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        tool.alias,
        mono(10.5),
        v.weak_text_color(),
    );
    response
}

pub fn toolbar(ui: &mut egui::Ui, ws: &mut Workspace, p: &mut Panels, chrome: &Chrome) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        if icons::button(
            ui,
            Icon::Select,
            ws.mode == Mode::Select,
            "Select  (V)\nClick to select, drag to move, grips resize",
        )
        .clicked()
        {
            ws.set_mode(Mode::Select);
        }
        if icons::button(
            ui,
            Icon::Pan,
            ws.mode == Mode::Pan,
            "Pan  (H)\nOr drag with the middle mouse button",
        )
        .clicked()
        {
            ws.set_mode(Mode::Pan);
        }
        separator(ui, chrome);
        if icons::button(ui, Icon::Undo, false, "Undo  (Ctrl+Z)").clicked() {
            ws.undo();
        }
        if icons::button(ui, Icon::Redo, false, "Redo  (Ctrl+Y)").clicked() {
            ws.redo();
        }
        separator(ui, chrome);
        if icons::button(ui, Icon::Fit, false, "Zoom to fit  (F)").clicked() {
            ws.fit_all();
        }
        if icons::button(ui, Icon::ZoomIn, false, "Zoom in  (+ or wheel)").clicked() {
            ws.zoom_by(1.25);
        }
        if icons::button(ui, Icon::ZoomOut, false, "Zoom out  (- or wheel)").clicked() {
            ws.zoom_by(0.8);
        }
        separator(ui, chrome);
        let placing = match ws.mode {
            Mode::Place(t) => Some(t),
            _ => None,
        };
        for (category, tools) in formulas::categories() {
            let active = placing.is_some_and(|t| t.category == category);
            let first = tools[0];
            let shown = if active { placing.unwrap() } else { first };
            let r = category_button(ui, shown.id, short(category), active, chrome);
            egui::Popup::menu(&r).show(|ui| {
                ui.set_min_width(260.0);
                ui.label(
                    RichText::new(category.to_uppercase())
                        .font(mono(10.0))
                        .color(chrome.accent),
                );
                ui.separator();
                for tool in tools {
                    let row = tool_row(ui, tool, placing == Some(tool));
                    if row.hovered() {
                        p.picked = Some(tool);
                    }
                    if row.clicked() {
                        ws.set_mode(Mode::Place(tool));
                        ui.close();
                    }
                }
            });
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icons::button(ui, Icon::Help, p.help, "Keyboard and mouse").clicked() {
                p.help = !p.help;
            }
            if icons::button(
                ui,
                Icon::Theme,
                false,
                if p.dark { "Light theme" } else { "Dark theme" },
            )
            .clicked()
            {
                p.dark = !p.dark;
            }
            if icons::button(ui, Icon::Properties, p.inspector, "Inspector panel").clicked() {
                p.inspector = !p.inspector;
            }
            if icons::button(ui, Icon::Layers, p.browser, "Browser panel").clicked() {
                p.browser = !p.browser;
            }
        });
    });
}

fn separator(ui: &mut egui::Ui, chrome: &Chrome) {
    ui.add_space(4.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 20.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, chrome.border);
    ui.add_space(4.0);
}

fn category_button(
    ui: &mut egui::Ui,
    icon_id: &str,
    label: &str,
    on: bool,
    chrome: &Chrome,
) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(10.0),
        chrome.text,
    );
    let w = galley.size().x.max(34.0) + 10.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(w, 38.0), egui::Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    if on {
        ui.painter()
            .rect_filled(rect, 3.0, chrome.accent.gamma_multiply(0.22));
        ui.painter().rect_stroke(
            rect,
            3.0,
            Stroke::new(1.0, chrome.accent),
            egui::StrokeKind::Inside,
        );
    } else {
        ui.painter()
            .rect_filled(rect, 3.0, chrome.tab.gamma_multiply(hover));
    }
    let color = if on { chrome.accent } else { chrome.text };
    let icon = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 13.0),
        egui::vec2(17.0, 17.0),
    );
    icons::tool(ui.painter(), icon, icon_id, color);
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.bottom() - galley.size().y - 2.0,
        ),
        galley,
        if on { chrome.accent } else { chrome.text_dim },
    );
    // A small dropdown mark beside the icon.
    let c = egui::pos2(rect.right() - 5.0, rect.top() + 13.0);
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            c + egui::vec2(-3.0, -1.5),
            c + egui::vec2(3.0, -1.5),
            c + egui::vec2(0.0, 2.0),
        ],
        chrome.text_dim,
        Stroke::NONE,
    ));
    response.on_hover_text(format!("{label} tools"))
}

/// The Browser: the tool library by category, and the viewports on the sheet.
pub fn browser(ui: &mut egui::Ui, ws: &mut Workspace, p: &mut Panels, chrome: &Chrome) {
    ScrollArea::vertical()
        .id_salt("browser")
        .auto_shrink(false)
        .show(ui, |ui| {
            Frame::new()
                .inner_margin(Margin::symmetric(8, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    heading(ui, chrome, "TOOL LIBRARY");
                    for (category, tools) in formulas::categories() {
                        egui::CollapsingHeader::new(
                            RichText::new(category).size(12.0).color(chrome.text_dim),
                        )
                        .id_salt(("lib", category))
                        .default_open(true)
                        .show(ui, |ui| {
                            for tool in tools {
                                let placing = ws.mode == Mode::Place(tool);
                                let r =
                                    library_row(ui, tool, placing, p.picked == Some(tool), chrome);
                                if r.clicked() {
                                    p.picked = Some(tool);
                                    ws.board.selection.clear();
                                    ws.set_mode(Mode::Place(tool));
                                }
                                if r.double_clicked() {
                                    ws.place(tool, None, egui::Vec2::ZERO);
                                    ws.mode = Mode::Select;
                                }
                            }
                        });
                    }
                    ui.add_space(10.0);
                    heading(ui, chrome, "VIEWPORTS");
                    if ws.board.viewports.is_empty() {
                        ui.label(
                            RichText::new("Nothing placed yet")
                                .small()
                                .color(chrome.text_dim),
                        );
                    }
                    let mut focus = None;
                    let mut select: Option<(u64, bool)> = None;
                    let shift = ui.input(|i| i.modifiers.shift);
                    for vp in ws.board.viewports.iter_mut().rev() {
                        let selected = ws.board.selection.contains(&vp.uid);
                        ui.horizontal(|ui| {
                            // Leave exactly enough room for the two icon buttons.
                            let room = 2.0 * 18.0 + 2.0 * ui.spacing().item_spacing.x;
                            let (rect, r) = ui.allocate_exact_size(
                                egui::vec2((ui.available_width() - room).max(40.0), 22.0),
                                egui::Sense::click(),
                            );
                            if selected {
                                ui.painter().rect_filled(
                                    rect,
                                    2.0,
                                    chrome.accent.gamma_multiply(0.25),
                                );
                            } else if r.hovered() {
                                ui.painter().rect_filled(rect, 2.0, chrome.tab);
                            }
                            let icon = egui::Rect::from_center_size(
                                rect.left_center() + egui::vec2(11.0, 0.0),
                                egui::vec2(14.0, 14.0),
                            );
                            let ink = if vp.hidden {
                                chrome.text_dim.gamma_multiply(0.5)
                            } else {
                                chrome.text
                            };
                            icons::tool(
                                ui.painter(),
                                icon,
                                vp.tool.id,
                                if selected {
                                    chrome.accent
                                } else {
                                    chrome.text_dim
                                },
                            );
                            ui.painter().text(
                                rect.left_center() + egui::vec2(24.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                &vp.title,
                                mono(11.0),
                                ink,
                            );
                            ui.painter().with_clip_rect(rect).text(
                                rect.left_center() + egui::vec2(70.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                vp.tool.name,
                                egui::FontId::proportional(12.0),
                                chrome.text_dim,
                            );
                            if r.clicked() {
                                select = Some((vp.uid, shift));
                            }
                            if r.double_clicked() {
                                focus = Some(vp.uid);
                            }
                            let eye = if vp.hidden { Icon::EyeOff } else { Icon::Eye };
                            if small_icon(ui, eye, !vp.hidden, chrome)
                                .on_hover_text("Show / hide")
                                .clicked()
                            {
                                vp.hidden = !vp.hidden;
                            }
                            let lock = if vp.locked { Icon::Lock } else { Icon::Unlock };
                            if small_icon(ui, lock, vp.locked, chrome)
                                .on_hover_text("Lock position")
                                .clicked()
                            {
                                vp.locked = !vp.locked;
                            }
                        });
                    }
                    if let Some((uid, add)) = select {
                        ws.mode = Mode::Select;
                        if add {
                            ws.board.toggle_selected(uid);
                        } else {
                            ws.board.select_only(uid);
                        }
                        if let Some(vp) = ws.board.get(uid) {
                            let seen = ws.camera.rect_to_world(ws.view, ws.view);
                            if !seen.intersects(vp.rect) {
                                ws.focus(uid);
                            }
                        }
                    }
                    if let Some(uid) = focus {
                        ws.focus(uid);
                    }
                });
        });
}

fn heading(ui: &mut egui::Ui, chrome: &Chrome, text: &str) {
    ui.label(RichText::new(text).font(mono(10.0)).color(chrome.accent));
    ui.add_space(2.0);
}

fn library_row(
    ui: &mut egui::Ui,
    tool: &'static Tool,
    placing: bool,
    picked: bool,
    chrome: &Chrome,
) -> egui::Response {
    let (rect, r) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
    let hover = ui.ctx().animate_bool(r.id, r.hovered());
    if placing {
        ui.painter()
            .rect_filled(rect, 2.0, chrome.accent.gamma_multiply(0.25));
    } else if picked {
        ui.painter().rect_filled(rect, 2.0, chrome.tab);
    } else {
        ui.painter()
            .rect_filled(rect, 2.0, chrome.tab.gamma_multiply(hover));
    }
    let icon = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(11.0, 0.0),
        egui::vec2(14.0, 14.0),
    );
    icons::tool(
        ui.painter(),
        icon,
        tool.id,
        if placing {
            chrome.accent
        } else {
            chrome.text_dim
        },
    );
    let clip = ui.painter().with_clip_rect(rect);
    clip.text(
        rect.left_center() + egui::vec2(24.0, 0.0),
        egui::Align2::LEFT_CENTER,
        tool.name,
        egui::FontId::proportional(12.5),
        chrome.text,
    );
    clip.text(
        rect.right_center() - egui::vec2(4.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        tool.alias,
        mono(9.5),
        chrome.text_dim,
    );
    r.on_hover_text("Click to place on the canvas · double-click to place beside the drawing")
}

fn small_icon(ui: &mut egui::Ui, icon: Icon, on: bool, chrome: &Chrome) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
    let color = if on {
        chrome.text
    } else {
        chrome.text_dim.gamma_multiply(0.6)
    };
    let color = if r.hovered() { chrome.accent } else { color };
    icons::ui(ui.painter(), rect.shrink(3.0), icon, color);
    r
}

/// The command line: the last few messages and a prompt.
pub fn command_line(ui: &mut egui::Ui, ws: &mut Workspace, p: &mut Panels, chrome: &Chrome) {
    Frame::new()
        .fill(chrome.well)
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Always two lines, so the canvas above doesn't jump when the log grows.
            let shown = ws.log.len().min(2);
            for _ in shown..2 {
                ui.label(RichText::new(" ").font(mono(11.0)));
            }
            for msg in &ws.log[ws.log.len() - shown..] {
                ui.label(RichText::new(msg).font(mono(11.0)).color(chrome.text_dim));
            }
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Command:")
                        .font(mono(12.0))
                        .color(chrome.accent),
                );
                let edit = ui.add(
                    TextEdit::singleline(&mut p.command)
                        .id_salt("cad-command")
                        .font(mono(12.0))
                        .frame(Frame::new())
                        .hint_text("type a tool alias (LOR, BEZ, MAN…), a tool name, or help")
                        .desired_width(f32::INFINITY),
                );
                if p.focus_command {
                    edit.request_focus();
                    p.focus_command = false;
                }
                let (enter, up, down) = ui.input(|i| {
                    (
                        i.key_pressed(Key::Enter),
                        i.key_pressed(Key::ArrowUp),
                        i.key_pressed(Key::ArrowDown),
                    )
                });
                if edit.lost_focus() && enter {
                    let line = std::mem::take(&mut p.command);
                    if !line.trim().is_empty() {
                        ws.run_command(&line);
                        p.history.push(line);
                    }
                    p.history_pos = None;
                    p.focus_command = true;
                }
                if edit.has_focus() && !p.history.is_empty() && (up || down) {
                    let last = p.history.len() - 1;
                    let pos = match (p.history_pos, up) {
                        (None, true) => Some(last),
                        (Some(i), true) => Some(i.saturating_sub(1)),
                        (Some(i), false) if i < last => Some(i + 1),
                        _ => None,
                    };
                    p.history_pos = pos;
                    p.command = pos.map(|i| p.history[i].clone()).unwrap_or_default();
                }
                // Suggestions while typing a tool.
                let q = p.command.trim().to_lowercase();
                if edit.has_focus() && q.len() >= 2 {
                    let matches: Vec<&'static Tool> = formulas::TOOLS
                        .iter()
                        .filter(|t| {
                            t.alias.to_lowercase().starts_with(&q)
                                || t.name.to_lowercase().contains(&q)
                        })
                        .take(6)
                        .collect();
                    if !matches.is_empty() {
                        let pos = edit.rect.left_top() - egui::vec2(0.0, 6.0);
                        egui::Area::new(egui::Id::new("cad-command-suggest"))
                            .order(egui::Order::Foreground)
                            .pivot(egui::Align2::LEFT_BOTTOM)
                            .fixed_pos(pos)
                            .show(ui.ctx(), |ui| {
                                Frame::popup(ui.style()).show(ui, |ui| {
                                    ui.set_min_width(260.0);
                                    for t in matches {
                                        if tool_row(ui, t, false).clicked() {
                                            ws.place(t, None, egui::Vec2::ZERO);
                                            p.command.clear();
                                        }
                                    }
                                });
                            });
                    }
                }
            });
        });
    // "/" or ":" jumps to the command line (single letters are shortcuts).
    if !ui.ctx().egui_wants_keyboard_input()
        && ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, Key::Slash)
                || i.consume_key(egui::Modifiers::SHIFT, Key::Semicolon)
                || i.consume_key(egui::Modifiers::NONE, Key::Colon)
        })
    {
        p.focus_command = true;
    }
}

pub fn status_bar(ui: &mut egui::Ui, ws: &mut Workspace, p: &mut Panels, chrome: &Chrome) {
    let dt = ui.input(|i| i.unstable_dt).max(1e-4);
    p.fps += (1.0 / dt - p.fps) * 0.05;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        let coords = match ws.pointer {
            Some(w) => format!("X {:>8.1}   Y {:>8.1}", w.x, w.y),
            None => "X        –   Y        –".to_owned(),
        };
        ui.label(RichText::new(coords).font(mono(11.5)).color(chrome.text));
        let mode = match ws.mode {
            Mode::Select => "SELECT".to_owned(),
            Mode::Pan => "PAN".to_owned(),
            Mode::Place(t) => format!("PLACE {}", t.alias),
        };
        ui.label(RichText::new(mode).font(mono(11.0)).color(chrome.accent));
        for (on, label, tip) in [
            (&mut ws.settings.snap, "SNAP", "Snap to grid (S)"),
            (&mut ws.settings.grid, "GRID", "Show grid (G)"),
            (
                &mut ws.settings.ortho,
                "ORTHO",
                "Move along one axis (O); Shift inverts while dragging",
            ),
        ] {
            if toggle(ui, label, *on, chrome).on_hover_text(tip).clicked() {
                *on = !*on;
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{:.0} fps", p.fps))
                    .font(mono(10.5))
                    .color(chrome.text_dim),
            );
            let n = ws.board.viewports.len();
            ui.label(
                RichText::new(format!("{n} viewports"))
                    .font(mono(10.5))
                    .color(chrome.text_dim),
            );
            if ui.small_button("+").on_hover_text("Zoom in").clicked() {
                ws.zoom_by(1.25);
            }
            let zoom = ui
                .add(
                    egui::Label::new(
                        RichText::new(format!("{:>4.0}%", ws.camera.zoom * 100.0))
                            .font(mono(11.5))
                            .color(chrome.text),
                    )
                    .sense(egui::Sense::click()),
                )
                .on_hover_text("Click for 100%");
            if zoom.clicked() {
                ws.camera.zoom_to(1.0);
            }
            if ui.small_button("−").on_hover_text("Zoom out").clicked() {
                ws.zoom_by(0.8);
            }
            if icons::button(ui, Icon::Fit, false, "Zoom to fit (F)").clicked() {
                ws.fit_all();
            }
        });
    });
}

fn toggle(ui: &mut egui::Ui, label: &str, on: bool, chrome: &Chrome) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), mono(10.5), chrome.text);
    let (rect, r) =
        ui.allocate_exact_size(galley.size() + egui::vec2(12.0, 6.0), egui::Sense::click());
    let fill = if on {
        chrome.accent.gamma_multiply(0.25)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 2.0, fill);
    ui.painter().rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0, if on { chrome.accent } else { chrome.border }),
        egui::StrokeKind::Inside,
    );
    ui.painter().galley(
        rect.min + egui::vec2(6.0, 3.0),
        galley,
        if on { chrome.accent } else { chrome.text_dim },
    );
    r
}

/// The keyboard and mouse reference.
pub fn help_window(ctx: &egui::Context, p: &mut Panels, chrome: &Chrome) {
    egui::Window::new("Keyboard and mouse")
        .open(&mut p.help)
        .resizable(false)
        .collapsible(false)
        .default_pos(egui::pos2(300.0, 140.0))
        .show(ctx, |ui| {
            let rows = [
                (
                    "Place a formula",
                    "pick a tool, click the canvas (or drag a rectangle)",
                ),
                ("Keep placing", "hold Shift while clicking"),
                ("Select", "click a viewport or its title tab; Shift adds"),
                (
                    "Window / crossing select",
                    "drag right / left on empty canvas",
                ),
                ("Use a formula", "select it, then drag inside it"),
                ("Move", "drag the title tab (or an inactive viewport)"),
                ("Resize", "drag a grip; Shift keeps the aspect ratio"),
                ("Pan", "middle-drag, Shift+wheel, or H"),
                ("Zoom", "wheel or pinch, about the pointer"),
                ("Zoom to fit / selection", "F / Shift+F"),
                ("Zoom 100%", "Ctrl+0"),
                ("Undo / redo", "Ctrl+Z / Ctrl+Y"),
                ("Duplicate / delete", "Ctrl+D / Del"),
                ("Nudge", "arrow keys (Shift: ×10)"),
                ("Grid / snap / ortho", "G / S / O"),
                ("Command line", "/ or :, then a tool alias or command"),
                ("Cancel", "Esc"),
            ];
            egui::Grid::new("help-grid")
                .striped(true)
                .spacing([18.0, 4.0])
                .show(ui, |ui| {
                    for (what, how) in rows {
                        ui.label(RichText::new(what).color(chrome.text));
                        ui.label(RichText::new(how).font(mono(11.0)).color(chrome.text_dim));
                        ui.end_row();
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A category without a short name shows up as "Other" in the toolbar.
    #[test]
    fn every_category_has_a_toolbar_name() {
        for tool in formulas::TOOLS {
            assert_ne!(short(tool.category), "Other", "{}", tool.category);
        }
    }
}
