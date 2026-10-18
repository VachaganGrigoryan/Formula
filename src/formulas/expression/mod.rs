//! The expression plotter: type `y = ...`, `r = ...` or `(x(t), y(t))` and
//! get a live plot with a slider for every free parameter.

pub mod plot;

use std::collections::BTreeMap;
use std::f64::consts::TAU;

use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Shape, Stroke, pos2, vec2};
use serde_json::{Value, json};

use crate::expr::{self, Definition, Expr, ParseError};
use crate::formula::{Formula, MathText, Palette};
use plot::{View, nice_step, sample_curve, sample_explicit};

/// Most rows one plotter holds.
pub const MAX_ROWS: usize = 6;
/// Samples along a parametric or polar curve.
const CURVE_STEPS: usize = 720;

/// A slider for one free parameter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Param {
    pub value: f64,
    pub min: f64,
    pub max: f64,
}

impl Default for Param {
    fn default() -> Self {
        Self {
            value: 1.0,
            min: -10.0,
            max: 10.0,
        }
    }
}

/// One typed line and what it parsed to.
struct Row {
    src: String,
    def: Result<Definition, ParseError>,
    /// The parameter range of a parametric or polar curve.
    t_range: (f64, f64),
}

impl Row {
    fn new(src: &str) -> Self {
        Self {
            src: src.to_string(),
            def: expr::parse(src),
            t_range: (0.0, TAU),
        }
    }

    /// An empty row is a blank line to type into, not an error.
    fn error(&self) -> Option<&ParseError> {
        match &self.def {
            Err(e) if !self.src.trim().is_empty() => Some(e),
            _ => None,
        }
    }
}

pub struct Expression {
    rows: Vec<Row>,
    params: BTreeMap<String, Param>,
    view: View,
    hover: Option<Pos2>,
}

impl Default for Expression {
    fn default() -> Self {
        let mut e = Self::with_rows(&["y = a sin(bx)", "r = 1 + cos(kθ)"]);
        for (name, value) in [("a", 2.0), ("b", 1.0), ("k", 3.0)] {
            if let Some(p) = e.params.get_mut(name) {
                p.value = value;
            }
        }
        e
    }
}

impl Expression {
    /// A plotter with these rows (at most [`MAX_ROWS`]; an empty list gives one blank row).
    pub fn with_rows(srcs: &[&str]) -> Self {
        let mut rows: Vec<Row> = srcs.iter().take(MAX_ROWS).map(|s| Row::new(s)).collect();
        if rows.is_empty() {
            rows.push(Row::new(""));
        }
        let mut e = Self {
            rows,
            params: BTreeMap::new(),
            view: View::default(),
            hover: None,
        };
        e.sync_params();
        e
    }

    /// Replace the text of row `i` and update the sliders.
    pub fn set_row(&mut self, i: usize, src: &str) {
        if let Some(row) = self.rows.get_mut(i) {
            let range = row.t_range;
            *row = Row::new(src);
            row.t_range = range;
            self.sync_params();
        }
    }

    /// Add a blank row, unless there are already [`MAX_ROWS`].
    pub fn add_row(&mut self) -> bool {
        if self.rows.len() >= MAX_ROWS {
            return false;
        }
        self.rows.push(Row::new(""));
        true
    }

    /// Remove row `i`, keeping at least one row.
    pub fn remove_row(&mut self, i: usize) {
        if self.rows.len() > 1 && i < self.rows.len() {
            self.rows.remove(i);
            self.sync_params();
        }
    }

    /// One slider per free parameter of the rows that parse: names no row
    /// uses go, names that stay keep their values, new names start at 1.
    fn sync_params(&mut self) {
        let names: Vec<String> = self
            .rows
            .iter()
            .filter_map(|r| r.def.as_ref().ok())
            .flat_map(expr::free_params)
            .collect();
        self.params.retain(|name, _| names.contains(name));
        for name in names {
            self.params.entry(name).or_default();
        }
    }

    fn value(&self, name: &str) -> Option<f64> {
        self.params.get(name).map(|p| p.value)
    }

    /// `body` as a function of `var`, with the current parameter values.
    fn func<'a>(&'a self, var: &'a str, body: &'a Expr) -> impl Fn(f64) -> f64 + 'a {
        move |v| expr::eval(body, &|n| if n == var { Some(v) } else { self.value(n) })
    }

    /// The polylines of row `def` for the view `rect` shows.
    fn sample(&self, row: &Row, def: &Definition, rect: Rect) -> Vec<Vec<[f64; 2]>> {
        let (xr, yr) = (self.view.x_range(rect), self.view.y_range(rect));
        match def {
            Definition::Explicit { var, body } => {
                sample_explicit(&self.func(var, body), xr, yr, rect.width())
            }
            Definition::Polar { theta, r } => {
                let r = self.func(theta, r);
                let f = |t: f64| {
                    let r = r(t);
                    [r * t.cos(), r * t.sin()]
                };
                sample_curve(&f, row.t_range, CURVE_STEPS, xr, yr)
            }
            Definition::Parametric { t, x, y } => {
                let (fx, fy) = (self.func(t, x), self.func(t, y));
                sample_curve(&|v| [fx(v), fy(v)], row.t_range, CURVE_STEPS, xr, yr)
            }
        }
    }
}

/// Row colors cycle through three accents; rows 4 to 6 are dashed.
fn row_style(i: usize, palette: &Palette) -> (Color32, bool) {
    let color = [palette.trace, palette.x, palette.y][i % 3];
    (color, i >= 3)
}

/// A tick label with as many decimals as the tick step needs.
fn tick_label(v: f64, step: f64) -> String {
    let decimals = (-step.log10().floor()).max(0.0) as usize;
    let s = format!("{v:.decimals$}");
    if s.trim_start_matches('-')
        .chars()
        .all(|c| c == '0' || c == '.')
    {
        "0".into()
    } else {
        s
    }
}

impl Formula for Expression {
    fn id(&self) -> &'static str {
        "expression"
    }

    fn name(&self) -> &'static str {
        "Expression plotter"
    }

    fn category(&self) -> &'static str {
        super::CURVES
    }

    fn summary(&self) -> &'static str {
        "Type your own functions: y = a sin(bx), r = 1 + cos(3θ) or (cos t, sin t). \
         Every free letter gets a slider; drag to pan and scroll to zoom."
    }

    fn notes(&self) -> &'static str {
        "Graphing calculators, plotting libraries and spreadsheet charts all do this: \
         parse the formula once, then evaluate it at many x values and join the points, \
         breaking the line where the function jumps or is undefined."
    }

    fn equations(&self) -> Vec<MathText> {
        self.rows
            .iter()
            .filter(|r| !r.src.trim().is_empty())
            .map(|r| match &r.def {
                Ok(def) => expr::to_math(def),
                Err(_) => MathText::new().t(&r.src),
            })
            .collect()
    }

    fn params(&self) -> Vec<(String, f64)> {
        self.params
            .iter()
            .map(|(name, p)| (name.clone(), p.value))
            .collect()
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        self.hover = response.hover_pos().filter(|p| rect.contains(*p));
        if response.dragged() {
            let d = response.drag_delta();
            self.view.cx -= d.x as f64 * self.view.scale;
            self.view.cy += d.y as f64 * self.view.scale;
        }
        // Wheel and pinch zoom, only while this is the active viewport (an
        // inactive one is handed a hover-only response).
        if let Some(at) = self.hover
            && response.sense.senses_click()
        {
            let (pinch, wheel) = response
                .ctx
                .input(|i| (i.zoom_delta(), i.smooth_scroll_delta.y));
            let zoom = pinch as f64 * (wheel as f64 * 0.003).exp();
            if zoom != 1.0 {
                self.view.zoom_at(rect, at, zoom);
            }
        }
    }

    fn wants_scroll(&self) -> bool {
        true
    }

    fn is_animating(&self) -> bool {
        false
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, _dt: f32) {
        let painter = painter.with_clip_rect(rect);
        let view = self.view;
        let (xr, yr) = (view.x_range(rect), view.y_range(rect));
        let to_screen = |p: [f64; 2]| view.to_screen(rect, p);

        // Grid lines on round values, then the axes, then tick labels along them.
        let step = nice_step(xr.1 - xr.0, (rect.width() / 70.0).max(2.0) as usize);
        let grid = Stroke::new(1.0, palette.rule);
        let first = |lo: f64| (lo / step).ceil() as i64;
        let xs: Vec<f64> = (first(xr.0)..=(xr.1 / step).floor() as i64)
            .map(|k| k as f64 * step)
            .collect();
        let ys: Vec<f64> = (first(yr.0)..=(yr.1 / step).floor() as i64)
            .map(|k| k as f64 * step)
            .collect();
        for &x in &xs {
            let sx = to_screen([x, 0.0]).x;
            painter.line_segment([pos2(sx, rect.top()), pos2(sx, rect.bottom())], grid);
        }
        for &y in &ys {
            let sy = to_screen([0.0, y]).y;
            painter.line_segment([pos2(rect.left(), sy), pos2(rect.right(), sy)], grid);
        }
        let origin = to_screen([0.0, 0.0]);
        let axis = Stroke::new(1.2, palette.muted);
        painter.line_segment(
            [pos2(rect.left(), origin.y), pos2(rect.right(), origin.y)],
            axis,
        );
        painter.line_segment(
            [pos2(origin.x, rect.top()), pos2(origin.x, rect.bottom())],
            axis,
        );
        let font = FontId::monospace(11.0);
        // Labels stay inside the stage when an axis is off screen.
        let label_y = origin.y.clamp(rect.top() + 2.0, rect.bottom() - 16.0);
        let label_x = origin.x.clamp(rect.left() + 4.0, rect.right() - 40.0);
        for &x in xs.iter().filter(|x| x.abs() > step * 1e-6) {
            painter.text(
                pos2(to_screen([x, 0.0]).x + 3.0, label_y + 2.0),
                Align2::LEFT_TOP,
                tick_label(x, step),
                font.clone(),
                palette.muted,
            );
        }
        for &y in ys.iter().filter(|y| y.abs() > step * 1e-6) {
            painter.text(
                pos2(label_x + 4.0, to_screen([0.0, y]).y - 2.0),
                Align2::LEFT_BOTTOM,
                tick_label(y, step),
                font.clone(),
                palette.muted,
            );
        }

        for (i, row) in self.rows.iter().enumerate() {
            let Ok(def) = &row.def else { continue };
            let (color, dashed) = row_style(i, palette);
            let stroke = Stroke::new(2.2, color);
            for piece in self.sample(row, def, rect) {
                let pts: Vec<Pos2> = piece.into_iter().map(to_screen).collect();
                if dashed {
                    painter.extend(Shape::dashed_line(&pts, stroke, 7.0, 5.0));
                } else {
                    painter.add(Shape::line(pts, stroke));
                }
            }
        }

        // Errors are listed on the stage too, so a broken row is never silent.
        let mut line = rect.bottom() - 10.0;
        for (i, row) in self.rows.iter().enumerate().rev() {
            if let Some(e) = row.error() {
                painter.text(
                    pos2(rect.left() + 10.0, line),
                    Align2::LEFT_BOTTOM,
                    format!("row {}: {}", i + 1, e.message),
                    FontId::proportional(12.0),
                    palette.y,
                );
                line -= 16.0;
            }
        }

        if let Some(at) = self.hover {
            let thin = Stroke::new(1.0, palette.muted.gamma_multiply(0.5));
            painter.line_segment([pos2(at.x, rect.top()), pos2(at.x, rect.bottom())], thin);
            painter.line_segment([pos2(rect.left(), at.y), pos2(rect.right(), at.y)], thin);
            let [wx, wy] = view.to_world(rect, at);
            let mut readout = format!("x = {wx:.3}   y = {wy:.3}");
            for (i, row) in self.rows.iter().enumerate() {
                if let Ok(Definition::Explicit { var, body }) = &row.def {
                    let y = self.func(var, body)(wx);
                    if y.is_finite() && (yr.0..=yr.1).contains(&y) {
                        let (color, _) = row_style(i, palette);
                        painter.circle(
                            to_screen([wx, y]),
                            4.5,
                            color,
                            Stroke::new(1.5, palette.card),
                        );
                        readout.push_str(&format!("   f{}(x) = {y:.3}", i + 1));
                    }
                }
            }
            painter.text(
                rect.left_top() + vec2(10.0, 8.0),
                Align2::LEFT_TOP,
                readout,
                FontId::monospace(12.0),
                palette.ink,
            );
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let mut edit = None;
        let mut remove = None;
        let can_remove = self.rows.len() > 1;
        for (i, row) in self.rows.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let (color, dashed) = row_style(i, palette);
                let (r, _) = ui.allocate_exact_size(vec2(18.0, 14.0), Sense::hover());
                let mid = [r.left_center(), r.right_center()];
                if dashed {
                    ui.painter().extend(Shape::dashed_line(
                        &mid,
                        Stroke::new(2.5, color),
                        4.0,
                        3.0,
                    ));
                } else {
                    ui.painter().line_segment(mid, Stroke::new(2.5, color));
                }
                let mut src = row.src.clone();
                let width = ui.available_width() - if can_remove { 70.0 } else { 0.0 };
                let response = ui.add(
                    egui::TextEdit::singleline(&mut src)
                        .id_salt(("expression-row", i))
                        .font(egui::TextStyle::Monospace)
                        .hint_text("y = ...")
                        .desired_width(width.max(80.0)),
                );
                if response.changed() {
                    edit = Some((i, src));
                }
                if can_remove && ui.small_button("Remove").clicked() {
                    remove = Some(i);
                }
            });
            if let Some(e) = row.error() {
                // The source again with a caret under the error, both in monospace.
                let col = row.src[..e.span.start.min(row.src.len())].chars().count();
                ui.label(RichText::new(&row.src).monospace().color(palette.muted));
                ui.label(
                    RichText::new(format!("{}^ {}", " ".repeat(col), e.message))
                        .monospace()
                        .color(palette.y),
                );
            }
            if let Ok(Definition::Polar { .. } | Definition::Parametric { .. }) = &row.def {
                let var = if matches!(row.def, Ok(Definition::Polar { .. })) {
                    "θ"
                } else {
                    "t"
                };
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    ui.label(format!("{var} from"));
                    ui.add(egui::DragValue::new(&mut row.t_range.0).speed(0.05));
                    ui.label("to");
                    ui.add(egui::DragValue::new(&mut row.t_range.1).speed(0.05));
                });
            }
        }
        if let Some((i, src)) = edit {
            self.set_row(i, &src);
        }
        if let Some(i) = remove {
            self.remove_row(i);
        }
        ui.horizontal(|ui| {
            if self.rows.len() < MAX_ROWS && ui.button("Add row").clicked() {
                self.add_row();
            }
            if ui.button("Reset view").clicked() {
                self.view = View::default();
            }
        });
        if self.params.is_empty() {
            return;
        }
        ui.add_space(6.0);
        egui::Grid::new("expression-params")
            .num_columns(4)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (name, p) in self.params.iter_mut() {
                    ui.label(RichText::new(name).italics());
                    ui.add(
                        egui::Slider::new(&mut p.value, p.min..=p.max)
                            .clamping(egui::SliderClamping::Edits),
                    );
                    ui.add(egui::DragValue::new(&mut p.min).speed(0.1).prefix("min "));
                    ui.add(egui::DragValue::new(&mut p.max).speed(0.1).prefix("max "));
                    if p.min > p.max {
                        std::mem::swap(&mut p.min, &mut p.max);
                    }
                    ui.end_row();
                }
            });
    }

    fn save_state(&self) -> Option<Value> {
        let params: serde_json::Map<String, Value> = self
            .params
            .iter()
            .map(|(n, p)| (n.clone(), json!([p.value, p.min, p.max])))
            .collect();
        Some(json!({
            "rows": self.rows.iter().map(|r| r.src.as_str()).collect::<Vec<_>>(),
            "ranges": self.rows.iter().map(|r| [r.t_range.0, r.t_range.1]).collect::<Vec<_>>(),
            "params": params,
            "view": [self.view.cx, self.view.cy, self.view.scale],
        }))
    }

    fn load_state(&mut self, state: &Value) {
        let numbers = |v: &Value| -> Option<Vec<f64>> {
            v.as_array()?
                .iter()
                .map(|n| n.as_f64().filter(|n| n.is_finite()))
                .collect()
        };
        if let Some(rows) = state.get("rows").and_then(Value::as_array) {
            let srcs: Vec<&str> = rows.iter().filter_map(Value::as_str).collect();
            if !srcs.is_empty() {
                self.rows = srcs.iter().take(MAX_ROWS).map(|s| Row::new(s)).collect();
                self.sync_params();
            }
        }
        if let Some(ranges) = state.get("ranges").and_then(Value::as_array) {
            for (row, range) in self.rows.iter_mut().zip(ranges) {
                if let Some([a, b]) = numbers(range).as_deref() {
                    row.t_range = (*a, *b);
                }
            }
        }
        if let Some(params) = state.get("params").and_then(Value::as_object) {
            for (name, v) in params {
                let Some(p) = self.params.get_mut(name) else {
                    continue;
                };
                match (v.as_f64(), numbers(v).as_deref()) {
                    (Some(value), _) if value.is_finite() => p.value = value,
                    (_, Some([value, min, max])) if min <= max => {
                        *p = Param {
                            value: *value,
                            min: *min,
                            max: *max,
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some([cx, cy, scale]) = state.get("view").and_then(numbers).as_deref()
            && *scale > 0.0
        {
            self.view = View {
                cx: *cx,
                cy: *cy,
                scale: *scale,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(e: &Expression) -> Vec<(String, f64)> {
        e.params()
    }

    #[test]
    fn sliders_follow_the_rows() {
        let mut e = Expression::with_rows(&["y = a sin(bx)"]);
        assert_eq!(names(&e), [("a".into(), 1.0), ("b".into(), 1.0)]);
        e.params.get_mut("a").unwrap().value = 4.0;
        e.set_row(0, "y = a x");
        assert_eq!(names(&e), [("a".into(), 4.0)], "a keeps its value, b goes");
        e.set_row(0, "sin(");
        assert!(e.rows[0].def.is_err());
        assert!(e.params.is_empty());
        e.set_row(0, "y = a x");
        assert_eq!(names(&e), [("a".into(), 1.0)]);
    }

    #[test]
    fn a_broken_row_leaves_the_others_plotting() {
        let mut e = Expression::default();
        e.set_row(1, "sin(");
        assert!(e.rows[1].error().is_some());
        assert_eq!(e.params().len(), 2, "a and b from the first row remain");
        let rect = Rect::from_min_size(Pos2::ZERO, e.min_size());
        let def = e.rows[0].def.clone().unwrap();
        assert!(!e.sample(&e.rows[0], &def, rect).is_empty());
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            e.hover = Some(rect.center());
            e.paint(ui.painter(), rect, &Palette::dark(), 0.016);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn empty_rows_are_blank_lines_not_errors() {
        let mut e = Expression::default();
        assert!(e.add_row());
        assert!(e.rows[2].error().is_none());
        assert_eq!(e.equations().len(), 2);
    }

    #[test]
    fn rows_are_capped() {
        let mut e = Expression::with_rows(&["1"; 8]);
        assert_eq!(e.rows.len(), MAX_ROWS);
        assert!(!e.add_row());
        e.remove_row(0);
        assert!(e.add_row());
        let mut one = Expression::with_rows(&["x"]);
        one.remove_row(0);
        assert_eq!(one.rows.len(), 1, "the last row stays");
    }

    #[test]
    fn state_round_trips() {
        let mut e = Expression::with_rows(&["(cos t, a sin t)", "y = c"]);
        e.params.get_mut("a").unwrap().value = 0.5;
        e.params.get_mut("c").unwrap().max = 3.0;
        e.rows[0].t_range = (0.0, 3.0);
        e.view.cx = 2.0;
        let saved = e.save_state().unwrap();
        let mut fresh = Expression::default();
        fresh.load_state(&saved);
        assert_eq!(fresh.save_state().unwrap(), saved);
        let defaults = Expression::default().save_state();
        let mut garbage = Expression::default();
        garbage.load_state(&json!(42));
        garbage.load_state(&json!({"rows": 7, "params": [1], "view": [0, 0, -1]}));
        assert_eq!(garbage.save_state(), defaults);
    }

    #[test]
    fn rows_alone_load_with_default_sliders() {
        let mut e = Expression::default();
        e.load_state(&json!({"rows": ["sin(x)"]}));
        assert_eq!(e.rows.len(), 1);
        assert!(e.params.is_empty());
        e.load_state(&json!({"rows": ["y = m x"], "params": {"m": 3}}));
        assert_eq!(e.params(), [("m".into(), 3.0)]);
    }

    #[test]
    fn tick_labels_use_the_steps_precision() {
        assert_eq!(tick_label(2.0, 1.0), "2");
        assert_eq!(tick_label(0.4, 0.2), "0.4");
        assert_eq!(tick_label(-0.00000001, 0.5), "0");
        assert_eq!(tick_label(0.05, 0.05), "0.05");
    }
}
