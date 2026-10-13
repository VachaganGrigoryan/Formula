//! "Drawn by circles": Fourier epicycles tracing animal silhouettes.
//!
//! Two chains of spinning circles, one per coordinate, add up their
//! harmonics; where their tips meet, the outline is drawn. Lowering the
//! number of terms `N` keeps only the slowest circles, and detail melts away.

pub mod animals;
pub mod math;

use std::collections::HashMap;
use std::f64::consts::TAU;

use egui::{Color32, FontId, Mesh, Pos2, Rect, Sense, Shape, Stroke, Vec2, pos2, vec2};

use crate::draw::{self, Span};
use crate::formula::{Formula, MathText, Palette, Tone};
use animals::{Animal, GROUPS};
use math::{MAX_TERMS, Point, SAMPLES, Series, Term};

/// Seconds the "grow" animation takes to climb from 1 term to the target.
const GROW_SECONDS: f32 = 1.8;
/// Radians of `t` per second at speed 1.
const RADIANS_PER_SECOND: f64 = 0.132;
const THUMB_ROWS: f32 = 64.0;

struct Prepared {
    outline: Vec<Point>,
    series: Series,
}

pub struct Epicycles {
    animals: Vec<Animal>,
    cache: HashMap<usize, Prepared>,
    thumbs: Vec<Option<(Rect, Vec<Span>)>>,
    current: usize,
    /// Terms currently drawn.
    n: usize,
    /// Terms chosen on the slider (where the grow animation ends).
    target: usize,
    speed: u32,
    t: f64,
    trail: Vec<Point>,
    recon: Vec<Point>,
    playing: bool,
    guide: bool,
    /// Seconds since the grow animation started, if running.
    grow: Option<f32>,
    search: String,
    group: usize,
    rng: u64,
}

impl Default for Epicycles {
    fn default() -> Self {
        let animals = animals::load();
        let thumbs = vec![None; animals.len()];
        let start = animals
            .iter()
            .position(|a| a.name == "Octopus")
            .unwrap_or(0);
        let mut s = Self {
            animals,
            cache: HashMap::new(),
            thumbs,
            current: 0,
            n: 120,
            target: 120,
            speed: 4,
            t: 0.0,
            trail: Vec::new(),
            recon: Vec::new(),
            playing: true,
            guide: true,
            grow: None,
            search: String::new(),
            group: 0,
            rng: 0x9E37_79B9_7F4A_7C15,
        };
        s.select(start);
        s
    }
}

impl Epicycles {
    fn prepared(&mut self) -> &Prepared {
        let animals = &self.animals;
        let current = self.current;
        self.cache.entry(current).or_insert_with(|| {
            let (outline, series) = math::analyze(&animals[current].outline);
            Prepared { outline, series }
        })
    }

    fn set_n(&mut self, n: usize) {
        self.n = n.clamp(1, MAX_TERMS);
        let n = self.n;
        self.recon = self.prepared().series.curve(n, SAMPLES);
    }

    fn select(&mut self, index: usize) {
        self.current = index;
        self.t = 0.0;
        self.trail.clear();
        self.grow = Some(0.0);
        self.set_n(1);
    }

    fn select_random(&mut self) {
        if self.animals.len() < 2 {
            return;
        }
        // xorshift64: no need for a crate to pick an animal.
        loop {
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            let i = (self.rng % self.animals.len() as u64) as usize;
            if i != self.current {
                self.select(i);
                return;
            }
        }
    }

    fn thumb(&mut self, index: usize) -> &(Rect, Vec<Span>) {
        let outline = &self.animals[index].outline;
        self.thumbs[index].get_or_insert_with(|| {
            let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            let pts: Vec<Pos2> = outline
                .iter()
                .map(|p| pos2(p[0] as f32, p[1] as f32))
                .collect();
            for p in &pts {
                x0 = x0.min(p.x);
                x1 = x1.max(p.x);
                y0 = y0.min(p.y);
                y1 = y1.max(p.y);
            }
            let side = (x1 - x0).max(y1 - y0);
            let pad = side * 0.06;
            let frame = Rect::from_center_size(
                pos2((x0 + x1) / 2.0, (y0 + y1) / 2.0),
                Vec2::splat(side + 2.0 * pad),
            );
            (
                frame,
                draw::polygon_spans(&pts, frame.height() / THUMB_ROWS),
            )
        })
    }

    fn advance(&mut self, dt: f32) {
        if let Some(elapsed) = self.grow.as_mut() {
            *elapsed += dt;
            let u = (*elapsed / GROW_SECONDS).min(1.0);
            let eased = 1.0 - (1.0 - u).powi(3);
            let v = (1.0 + eased * (self.target as f32 - 1.0)).round().max(1.0) as usize;
            if u >= 1.0 {
                self.grow = None;
            }
            if v != self.n {
                self.set_n(v);
            }
        }
        if self.playing {
            let n = self.n;
            let t = self.t;
            let p = self.prepared().series.eval(n, t);
            self.trail.push(p);
            self.t += RADIANS_PER_SECOND * self.speed as f64 * dt as f64;
            if self.t >= TAU {
                self.t -= TAU;
                self.trail.clear();
            }
        }
    }
}

/// Draw one chain of circles, largest first. With `swap`, cosine drives the
/// vertical axis so the chain's tip tracks y instead of x. Returns the tip.
#[allow(clippy::too_many_arguments)]
fn chain(
    painter: &egui::Painter,
    terms: &[Term],
    n: usize,
    t: f64,
    origin: Pos2,
    scale: f32,
    swap: bool,
    color: Color32,
) -> Pos2 {
    let mut active: Vec<&Term> = terms.iter().take(n).collect();
    active.sort_by(|a, b| b.r.total_cmp(&a.r));
    let circle = Stroke::new(1.0, color.gamma_multiply(0.3));
    let arm = Stroke::new(1.0, color.gamma_multiply(0.85));
    let mut p = origin;
    for term in active {
        let r = term.r as f32 * scale;
        let ang = term.angle(t) as f32;
        let (s, c) = ang.sin_cos();
        let d = if swap {
            vec2(r * s, r * c)
        } else {
            vec2(r * c, r * s)
        };
        if r > 0.6 {
            painter.circle_stroke(p, r, circle);
            painter.line_segment([p, p + d], arm);
        }
        p += d;
    }
    painter.circle_filled(p, 3.2, color);
    p
}

impl Formula for Epicycles {
    fn id(&self) -> &'static str {
        "fourier-epicycles"
    }

    fn name(&self) -> &'static str {
        "Drawn by circles"
    }

    fn summary(&self) -> &'static str {
        "Pick an animal. Up to 200 spinning circles trace its outline. \
         Slide the number of terms down and watch the details melt away."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("x")
                .tone(Tone::X)
                .sub("N")
                .tone(Tone::X)
                .t("(t)")
                .tone(Tone::X)
                .t(" = Σ")
                .sub("k=1…N")
                .t(" (a")
                .sub("k")
                .t(" cos kt + b")
                .sub("k")
                .t(" sin kt)"),
            MathText::new()
                .t("y")
                .tone(Tone::Y)
                .sub("N")
                .tone(Tone::Y)
                .t("(t)")
                .tone(Tone::Y)
                .t(" = Σ")
                .sub("k=1…N")
                .t(" (c")
                .sub("k")
                .t(" cos kt + d")
                .sub("k")
                .t(" sin kt)"),
        ]
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * if width < 520.0 { 1.12 } else { 0.9 }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.advance(dt);

        let (w, h) = (rect.width(), rect.height());
        let scale = (w * 0.5).min(h * 0.52);
        let center = rect.min + vec2(w * 0.6, h * 0.62);
        let x_origin = pos2(center.x, rect.min.y + h * 0.23);
        let y_origin = pos2(rect.min.x + w * 0.16, center.y);

        let (n, t) = (self.n, self.t);
        let guide = self.guide;
        self.prepared();
        let prepared = &self.cache[&self.current];
        let series = &prepared.series;
        let (a0, c0) = (series.a0, series.c0);
        let to_screen = |p: &Point| {
            pos2(
                center.x + ((p[0] - a0) as f32) * scale,
                center.y + ((p[1] - c0) as f32) * scale,
            )
        };

        if guide {
            let mut outline: Vec<Pos2> = prepared.outline.iter().map(to_screen).collect();
            outline.push(outline[0]);
            let stroke = Stroke::new(1.0, palette.muted.gamma_multiply(0.45));
            painter.extend(Shape::dashed_line(&outline, stroke, 3.0, 4.0));
        }

        let recon: Vec<Pos2> = self.recon.iter().map(to_screen).collect();
        painter.add(draw::fill_polygon(&recon, palette.fill, 1.0));

        let xe = chain(painter, &series.xs, n, t, x_origin, scale, false, palette.x);
        let ye = chain(painter, &series.ys, n, t, y_origin, scale, true, palette.y);
        let tip = pos2(xe.x, ye.y);
        painter.extend(Shape::dashed_line(
            &[xe, tip],
            Stroke::new(1.0, palette.x),
            5.0,
            5.0,
        ));
        painter.extend(Shape::dashed_line(
            &[ye, tip],
            Stroke::new(1.0, palette.y),
            5.0,
            5.0,
        ));

        let trail: Vec<Pos2> = self.trail.iter().map(to_screen).collect();
        if trail.len() > 1 {
            painter.add(Shape::line(trail, Stroke::new(2.2, palette.trace)));
        }
        painter.circle_filled(tip, 3.2, palette.ink);

        let name_font = FontId::proportional((w * 0.07).clamp(22.0, 40.0));
        let count_font = FontId::proportional((w * 0.06).clamp(20.0, 34.0));
        painter.text(
            rect.left_bottom() + vec2(16.0, -10.0),
            egui::Align2::LEFT_BOTTOM,
            &self.animals[self.current].name,
            name_font,
            palette.ink,
        );
        painter.text(
            rect.right_bottom() + vec2(-16.0, -12.0),
            egui::Align2::RIGHT_BOTTOM,
            self.n.to_string(),
            count_font,
            palette.trace,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("fourier-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Terms N");
                let mut target = self.target;
                if ui
                    .add(egui::Slider::new(&mut target, 1..=MAX_TERMS))
                    .changed()
                {
                    self.grow = None;
                    self.target = target;
                    self.set_n(target);
                }
                ui.end_row();
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.speed, 1..=10));
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui.button("Random animal").clicked() {
                self.select_random();
            }
            if ui
                .button(if self.guide {
                    "Hide outline"
                } else {
                    "Show outline"
                })
                .clicked()
            {
                self.guide = !self.guide;
            }
        });
    }

    fn details(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.heading("Choose an animal");
        ui.add_space(4.0);
        ui.add(
            egui::TextEdit::singleline(&mut self.search)
                .hint_text("Octopus")
                .desired_width(f32::INFINITY),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for (i, g) in GROUPS.iter().enumerate() {
                if ui.selectable_label(self.group == i, *g).clicked() {
                    self.group = i;
                }
            }
        });
        ui.add_space(10.0);

        let query = self.search.trim().to_lowercase();
        let matches: Vec<usize> = (0..self.animals.len())
            .filter(|&i| {
                let a = &self.animals[i];
                (self.group == 0 || a.group == GROUPS[self.group])
                    && a.name.to_lowercase().contains(&query)
            })
            .collect();
        if matches.is_empty() {
            ui.colored_label(
                palette.muted,
                "No animal matches that name. Try a shorter word.",
            );
        }

        let cell = vec2(96.0, 92.0);
        let mut clicked = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for i in matches {
                let (rect, response) = ui.allocate_exact_size(cell, Sense::click());
                let selected = i == self.current;
                let border = if selected {
                    Stroke::new(2.0, palette.trace)
                } else if response.hovered() {
                    Stroke::new(1.0, palette.muted)
                } else {
                    Stroke::new(1.0, palette.rule)
                };
                let painter = ui.painter_at(rect);
                painter.rect(
                    rect.shrink(1.0),
                    14.0,
                    palette.card,
                    border,
                    egui::StrokeKind::Inside,
                );
                let icon =
                    Rect::from_center_size(rect.center_top() + vec2(0.0, 36.0), Vec2::splat(56.0));
                let (frame, spans) = self.thumb(i);
                let mut mesh = Mesh::default();
                draw::add_spans(&mut mesh, spans, *frame, icon, palette.ink);
                painter.add(mesh);
                painter.text(
                    rect.center_bottom() - vec2(0.0, 8.0),
                    egui::Align2::CENTER_BOTTOM,
                    &self.animals[i].name,
                    FontId::proportional(12.5),
                    palette.ink,
                );
                if response.on_hover_text(&self.animals[i].name).clicked() {
                    clicked = Some(i);
                }
            }
        });
        if let Some(i) = clicked {
            self.select(i);
        }

        ui.add_space(24.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.small("Silhouettes adapted from ");
            ui.hyperlink_to(
                egui::RichText::new("game-icons.net").small(),
                "https://game-icons.net",
            );
            ui.small(" by Lorc, Delapouite and contributors, licensed ");
            ui.hyperlink_to(
                egui::RichText::new("CC BY 3.0").small(),
                "https://creativecommons.org/licenses/by/3.0/",
            );
            ui.small(". Outlines were traced and simplified.");
        });
    }
}
