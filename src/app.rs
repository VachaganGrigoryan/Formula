//! The app shell: a catalogue of formulas and a page for the selected one.

use egui::{Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Sense, Stroke};

use crate::formula::{Formula, Palette};
use crate::formulas;

const PAGE_WIDTH: f32 = 720.0;
/// Below this window width the catalogue moves to a dropdown above the page.
const NARROW: f32 = 760.0;

pub struct FormulaApp {
    formulas: Vec<Box<dyn Formula>>,
    selected: usize,
}

impl FormulaApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let formulas = formulas::registry();
        let selected = cc
            .storage
            .and_then(|s| s.get_string("formula"))
            .and_then(|id| formulas.iter().position(|f| f.id() == id))
            .unwrap_or(0);
        Self { formulas, selected }
    }

    fn catalogue(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.add_space(12.0);
        ui.label(RichText::new("Formulas").strong().color(palette.muted));
        ui.add_space(6.0);
        for (i, f) in self.formulas.iter().enumerate() {
            if ui.selectable_label(i == self.selected, f.name()).clicked() {
                self.selected = i;
            }
        }
        ui.add_space(16.0);
        egui::widgets::global_theme_preference_buttons(ui);
    }

    fn page(&mut self, ui: &mut egui::Ui, palette: &Palette, narrow: bool) {
        if narrow {
            ui.horizontal(|ui| {
                let current = self.formulas[self.selected].name();
                egui::ComboBox::from_id_salt("formula-picker")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for (i, f) in self.formulas.iter().enumerate() {
                            ui.selectable_value(&mut self.selected, i, f.name());
                        }
                    });
                egui::widgets::global_theme_preference_switch(ui);
            });
        }

        let formula = &mut self.formulas[self.selected];
        ui.add_space(8.0);
        let title_size = (ui.available_width() * 0.09).clamp(34.0, 56.0);
        ui.label(
            RichText::new(formula.name())
                .size(title_size)
                .strong()
                .color(palette.ink),
        );
        ui.add_space(4.0);
        ui.label(RichText::new(formula.summary()).color(palette.muted));
        ui.add_space(16.0);

        let width = ui.available_width();
        let height = formula.stage_height(width);
        Frame::new()
            .fill(palette.card)
            .stroke(Stroke::new(1.0, palette.rule))
            .corner_radius(CornerRadius::same(20))
            .show(ui, |ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::vec2(width - 2.0, height), Sense::hover());
                let dt = ui.input(|i| i.stable_dt).min(0.1);
                formula.paint(
                    &painter.with_clip_rect(response.rect),
                    response.rect,
                    palette,
                    dt,
                );
            });

        ui.add_space(14.0);
        ui.scope(|ui| {
            // Leave room for the label and value box beside each slider.
            ui.spacing_mut().slider_width = (ui.available_width() - 150.0).max(120.0);
            formula.controls(ui, palette);
        });
        ui.add_space(14.0);
        for eq in formula.equations() {
            ui.label(eq.layout(palette, 17.0));
        }
        ui.add_space(24.0);
        formula.details(ui, palette);
        ui.add_space(32.0);
    }
}

impl eframe::App for FormulaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let palette = Palette::for_dark_mode(ui.visuals().dark_mode);
        let narrow = ui.available_width() < NARROW;

        if !narrow {
            egui::Panel::left("catalogue")
                .resizable(false)
                .default_size(200.0)
                .frame(
                    Frame::new()
                        .fill(palette.card)
                        .inner_margin(Margin::symmetric(12, 8)),
                )
                .show(ui, |ui| self.catalogue(ui, &palette));
        }

        egui::CentralPanel::default()
            .frame(Frame::new().fill(palette.paper))
            .show(ui, |ui| {
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    let gutter = 16.0;
                    let width = (ui.available_width() - 2.0 * gutter).min(PAGE_WIDTH);
                    let side = (ui.available_width() - width) / 2.0;
                    ui.horizontal_top(|ui| {
                        ui.add_space(side);
                        ui.vertical(|ui| {
                            ui.set_width(width);
                            ui.add_space(12.0);
                            self.page(ui, &palette, narrow);
                        });
                    });
                });
            });

        if self.formulas[self.selected].is_animating() {
            ui.ctx().request_repaint();
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("formula", self.formulas[self.selected].id().to_owned());
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }
}
