//! Ways to draw an array: bars, a radial clock face, a hue spectrum, or dots.
//!
//! Every skin takes the same inputs: each slot's value and its state (idle,
//! comparing, sorted, ...). State colors are chosen by the caller, so the
//! skins stay interchangeable.

use std::f32::consts::{PI, TAU};

use egui::{Color32, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::formulas::luminance::hsv_to_srgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skin {
    Bars,
    Radial,
    Spectrum,
    Dots,
}

impl Skin {
    pub const ALL: [Skin; 4] = [Skin::Bars, Skin::Radial, Skin::Spectrum, Skin::Dots];

    pub fn label(self) -> &'static str {
        match self {
            Skin::Bars => "Bars",
            Skin::Radial => "Radial",
            Skin::Spectrum => "Color spectrum",
            Skin::Dots => "Dot scatter",
        }
    }
}

/// How a slot should be emphasized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mark {
    /// Untouched: drawn in the skin's resting color.
    Idle,
    /// Outside the region being worked on.
    Faded,
    /// In its final place.
    Sorted(Color32),
    /// Being compared, swapped, written, or otherwise singled out.
    Active(Color32),
}

/// The value's hue on the spectrum skin: 0° (red) for the smallest up to
/// 300° (magenta) for the largest, stopping short of wrapping back to red.
pub fn hue(norm: f32) -> Color32 {
    let [r, g, b] = hsv_to_srgb(norm.clamp(0.0, 1.0) * (300.0 / 360.0), 0.75, 0.95);
    Color32::from_rgb(r, g, b)
}

pub struct Frame<'a> {
    pub values: &'a [Option<u32>],
    pub max: u32,
    pub marks: &'a dyn Fn(usize) -> Mark,
    /// The resting color for bars, dots and radial spokes.
    pub idle: Color32,
    pub empty: Color32,
    /// Insertion sort's lifted key: (slot, value, outline color).
    pub ghost: Option<(usize, u32, Color32)>,
    /// A pair just swapped, drawn as a chord on the radial skin.
    pub swapped: Option<(usize, usize, Color32)>,
}

impl Frame<'_> {
    fn norm(&self, v: u32) -> f32 {
        v as f32 / self.max.max(1) as f32
    }

    fn fill(&self, i: usize, resting: Color32) -> Color32 {
        match (self.marks)(i) {
            Mark::Idle => resting,
            Mark::Faded => resting.gamma_multiply(0.3),
            Mark::Sorted(c) | Mark::Active(c) => c,
        }
    }
}

pub fn draw(painter: &egui::Painter, area: Rect, skin: Skin, f: &Frame) {
    match skin {
        Skin::Bars => bars(painter, area, f),
        Skin::Radial => radial(painter, area, f),
        Skin::Spectrum => spectrum(painter, area, f),
        Skin::Dots => dots(painter, area, f),
    }
}

fn slot(area: Rect, n: usize) -> f32 {
    area.width() / n.max(1) as f32
}

fn bars(painter: &egui::Painter, area: Rect, f: &Frame) {
    let w = slot(area, f.values.len());
    let gap = if w > 6.0 {
        w * 0.18
    } else if w > 3.0 {
        1.0
    } else {
        0.0
    };
    for (i, v) in f.values.iter().enumerate() {
        let x0 = area.left() + w * i as f32 + gap / 2.0;
        let x1 = x0 + (w - gap).max(1.0);
        let r = |h: f32| Rect::from_min_max(pos2(x0, area.bottom() - h), pos2(x1, area.bottom()));
        match v {
            Some(v) => {
                painter.rect_filled(
                    r((f.norm(*v) * area.height()).max(2.0)),
                    1.5,
                    f.fill(i, f.idle),
                );
            }
            None => {
                painter.rect_filled(r(2.0), 0.0, f.empty);
            }
        }
    }
    if let Some((i, v, edge)) = f.ghost {
        let x0 = area.left() + w * i as f32;
        let r = Rect::from_min_max(
            pos2(x0 + w * 0.1, area.bottom() - f.norm(v) * area.height()),
            pos2(x0 + w * 0.9, area.bottom()),
        );
        painter.rect(
            r,
            1.5,
            edge.gamma_multiply(0.25),
            Stroke::new(2.0, edge),
            egui::StrokeKind::Inside,
        );
    }
}

fn dots(painter: &egui::Painter, area: Rect, f: &Frame) {
    let w = slot(area, f.values.len());
    let r = (w * 0.38).clamp(1.8, 6.0);
    let at = |i: usize, v: u32| {
        pos2(
            area.left() + w * (i as f32 + 0.5),
            area.bottom() - f.norm(v) * (area.height() - r * 2.0) - r,
        )
    };
    // The sorted diagonal the dots are heading for.
    painter.line_segment(
        [area.left_bottom(), area.right_top()],
        Stroke::new(1.0, f.empty),
    );
    for (i, v) in f.values.iter().enumerate() {
        if let Some(v) = v {
            let active = matches!((f.marks)(i), Mark::Active(_));
            painter.circle_filled(
                at(i, *v),
                if active { r * 1.6 } else { r },
                f.fill(i, f.idle),
            );
        }
    }
    if let Some((i, v, edge)) = f.ghost {
        painter.circle_stroke(at(i, v), r * 1.6, Stroke::new(2.0, edge));
    }
}

fn spectrum(painter: &egui::Painter, area: Rect, f: &Frame) {
    let w = slot(area, f.values.len());
    let band = Rect::from_min_max(area.min, pos2(area.right(), area.bottom() - 14.0));
    for (i, v) in f.values.iter().enumerate() {
        let x0 = band.left() + w * i as f32;
        let col = Rect::from_min_max(pos2(x0, band.top()), pos2(x0 + w + 0.5, band.bottom()));
        let Some(v) = v else {
            painter.rect_filled(col, 0.0, f.empty);
            continue;
        };
        let color = hue(f.norm(*v));
        match (f.marks)(i) {
            Mark::Faded => {
                painter.rect_filled(col, 0.0, color.gamma_multiply(0.35));
            }
            Mark::Idle => {
                painter.rect_filled(col, 0.0, color);
            }
            Mark::Sorted(c) => {
                painter.rect_filled(col, 0.0, color);
                painter.rect_filled(
                    Rect::from_min_max(
                        pos2(col.left(), band.bottom() + 4.0),
                        pos2(col.right(), band.bottom() + 9.0),
                    ),
                    0.0,
                    c,
                );
            }
            Mark::Active(c) => {
                painter.rect_filled(col, 0.0, color);
                let tip = pos2(col.center().x, band.bottom() + 2.0);
                let s = (w * 0.5).clamp(4.0, 8.0);
                painter.add(Shape::convex_polygon(
                    vec![tip, tip + vec2(-s, s * 1.4), tip + vec2(s, s * 1.4)],
                    c,
                    Stroke::NONE,
                ));
                painter.rect_stroke(col, 0.0, Stroke::new(2.0, c), egui::StrokeKind::Inside);
            }
        }
    }
    if let Some((i, v, edge)) = f.ghost {
        let x0 = band.left() + w * i as f32;
        let col = Rect::from_min_max(pos2(x0, band.top()), pos2(x0 + w, band.bottom()));
        painter.rect(
            col,
            0.0,
            hue(f.norm(v)).gamma_multiply(0.5),
            Stroke::new(2.0, edge),
            egui::StrokeKind::Inside,
        );
    }
}

fn radial(painter: &egui::Painter, area: Rect, f: &Frame) {
    let n = f.values.len().max(1);
    let c = area.center();
    let outer = area.width().min(area.height()) * 0.5 - 4.0;
    let inner = outer * 0.2;
    let angle = |i: usize| -PI / 2.0 + TAU * (i as f32 + 0.5) / n as f32;
    let point = |i: usize, r: f32| c + vec2(angle(i).cos(), angle(i).sin()) * r;
    let tip = |i: usize, v: u32| point(i, inner + f.norm(v) * (outer - inner));
    let thickness = (TAU * outer / n as f32 * 0.55).clamp(1.0, 7.0);
    painter.circle_stroke(c, inner, Stroke::new(1.0, f.empty));
    for (i, v) in f.values.iter().enumerate() {
        match v {
            Some(v) => painter.line_segment(
                [point(i, inner), tip(i, *v)],
                Stroke::new(thickness, f.fill(i, f.idle)),
            ),
            None => painter.circle_filled(point(i, inner), thickness * 0.5, f.empty),
        };
    }
    if let Some((i, v, edge)) = f.ghost {
        painter.line_segment(
            [point(i, inner), tip(i, v)],
            Stroke::new(thickness.max(2.0), edge.gamma_multiply(0.6)),
        );
    }
    // A swap sweeps a curve across the face, bowed toward the center.
    if let Some((a, b, color)) = f.swapped
        && let (Some(Some(va)), Some(Some(vb))) = (f.values.get(a), f.values.get(b))
    {
        let (p, q) = (tip(a, *va), tip(b, *vb));
        let ctrl = c + ((p - c) + (q - c)) * 0.25;
        let curve: Vec<Pos2> = (0..=24)
            .map(|k| {
                let t = k as f32 / 24.0;
                let u = 1.0 - t;
                (p.to_vec2() * u * u + ctrl.to_vec2() * 2.0 * u * t + q.to_vec2() * t * t).to_pos2()
            })
            .collect();
        painter.add(Shape::line(curve, Stroke::new(2.0, color)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_runs_red_to_magenta() {
        let low = hue(0.0);
        assert!(low.r() > low.g() && low.r() > low.b(), "smallest is red");
        let high = hue(1.0);
        assert!(
            high.r() > high.g() && high.b() > high.g(),
            "largest is magenta"
        );
        assert_ne!(hue(0.0), hue(1.0));
    }
}
