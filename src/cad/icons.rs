//! Line-art icons, drawn with the painter so they stay crisp at any size and
//! take the theme's colors. Coordinates are in a unit box, y down.

use std::f32::consts::{PI, TAU};

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, vec2};

/// Icons for workspace commands (tools have one each, by tool id).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ui {
    Select,
    Pan,
    Fit,
    ZoomIn,
    ZoomOut,
    Undo,
    Redo,
    Grid,
    Snap,
    Ortho,
    Delete,
    Duplicate,
    Docs,
    Derive,
    Code,
    Copy,
    Download,
    Eye,
    EyeOff,
    Lock,
    Unlock,
    Theme,
    Help,
    Layers,
    Properties,
    Maximize,
    Restore,
}

struct Pen<'a> {
    painter: &'a Painter,
    rect: Rect,
    stroke: Stroke,
    color: Color32,
}

impl Pen<'_> {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        self.rect.min + vec2(x * self.rect.width(), y * self.rect.height())
    }

    fn line(&self, pts: &[(f32, f32)]) {
        let pts: Vec<Pos2> = pts.iter().map(|&(x, y)| self.at(x, y)).collect();
        self.painter.add(Shape::line(pts, self.stroke));
    }

    fn closed(&self, pts: &[(f32, f32)]) {
        let pts: Vec<Pos2> = pts.iter().map(|&(x, y)| self.at(x, y)).collect();
        self.painter.add(Shape::closed_line(pts, self.stroke));
    }

    fn fill(&self, pts: &[(f32, f32)]) {
        let pts: Vec<Pos2> = pts.iter().map(|&(x, y)| self.at(x, y)).collect();
        self.painter
            .add(Shape::convex_polygon(pts, self.color, Stroke::NONE));
    }

    fn curve(&self, n: usize, f: impl Fn(f32) -> (f32, f32)) {
        let pts: Vec<Pos2> = (0..=n)
            .map(|k| {
                let (x, y) = f(k as f32 / n as f32);
                self.at(x, y)
            })
            .collect();
        self.painter.add(Shape::line(pts, self.stroke));
    }

    fn circle(&self, x: f32, y: f32, r: f32) {
        let s = self.rect.width().min(self.rect.height());
        self.painter
            .circle_stroke(self.at(x, y), r * s, self.stroke);
    }

    fn dot(&self, x: f32, y: f32, r: f32) {
        let s = self.rect.width().min(self.rect.height());
        self.painter
            .circle_filled(self.at(x, y), (r * s).max(1.2), self.color);
    }

    fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32, filled: bool) {
        let r = Rect::from_min_max(self.at(x0, y0), self.at(x1, y1));
        if filled {
            self.painter.rect_filled(r, 0.0, self.color);
        } else {
            self.painter
                .rect_stroke(r, 0.0, self.stroke, egui::StrokeKind::Middle);
        }
    }

    /// Bars along the bottom, heights in 0..1; `hi` marks bars drawn solid.
    fn bars(&self, heights: &[f32], hi: &[usize]) {
        let n = heights.len() as f32;
        for (i, h) in heights.iter().enumerate() {
            let x0 = 0.1 + 0.8 * i as f32 / n;
            let x1 = x0 + 0.8 / n * 0.7;
            self.rect(x0, 0.9 - 0.75 * h, x1, 0.9, hi.contains(&i));
        }
    }
}

fn pen(painter: &Painter, rect: Rect, color: Color32) -> Pen<'_> {
    let w = (rect.width() / 14.0).clamp(1.0, 2.2);
    Pen {
        painter,
        rect,
        stroke: Stroke::new(w, color),
        color,
    }
}

/// The icon of a formula tool.
pub fn tool(painter: &Painter, rect: Rect, id: &str, color: Color32) {
    let p = pen(painter, rect, color);
    match id {
        "polar-to-cartesian" => {
            p.line(&[(0.15, 0.85), (0.9, 0.85)]);
            p.line(&[(0.15, 0.85), (0.15, 0.1)]);
            p.line(&[(0.15, 0.85), (0.72, 0.3)]);
            p.curve(12, |t| {
                let a = t * PI / 4.0;
                (0.15 + 0.35 * a.cos(), 0.85 - 0.35 * a.sin())
            });
            p.dot(0.72, 0.3, 0.07);
        }
        "perspective-projection" => {
            p.closed(&[(0.12, 0.38), (0.6, 0.38), (0.6, 0.86), (0.12, 0.86)]);
            p.closed(&[(0.38, 0.14), (0.86, 0.14), (0.86, 0.62), (0.38, 0.62)]);
            for (a, b) in [
                ((0.12, 0.38), (0.38, 0.14)),
                ((0.6, 0.38), (0.86, 0.14)),
                ((0.6, 0.86), (0.86, 0.62)),
                ((0.12, 0.86), (0.38, 0.62)),
            ] {
                p.line(&[a, b]);
            }
        }
        "lerp" => {
            p.line(&[(0.15, 0.78), (0.85, 0.22)]);
            p.dot(0.15, 0.78, 0.08);
            p.dot(0.85, 0.22, 0.08);
            p.circle(0.5, 0.5, 0.09);
        }
        "cubic-bezier" => {
            p.curve(24, |t| {
                let u = 1.0 - t;
                let x = u * u * u * 0.1
                    + 3.0 * u * u * t * 0.25
                    + 3.0 * u * t * t * 0.75
                    + t * t * t * 0.9;
                let y = u * u * u * 0.85
                    + 3.0 * u * u * t * 0.1
                    + 3.0 * u * t * t * 0.9
                    + t * t * t * 0.15;
                (x, y)
            });
            p.line(&[(0.1, 0.85), (0.25, 0.1)]);
            p.line(&[(0.9, 0.15), (0.75, 0.9)]);
            p.rect(0.2, 0.05, 0.3, 0.15, true);
            p.rect(0.7, 0.85, 0.8, 0.95, true);
        }
        "fourier-epicycles" => {
            p.circle(0.4, 0.5, 0.3);
            p.circle(0.7, 0.5, 0.13);
            p.line(&[(0.4, 0.5), (0.7, 0.5), (0.8, 0.41)]);
            p.dot(0.8, 0.41, 0.06);
        }
        "lissajous" => p.curve(60, |t| {
            let a = t * TAU;
            (
                0.5 + 0.4 * (3.0 * a + PI / 2.0).sin(),
                0.5 + 0.4 * (2.0 * a).sin(),
            )
        }),
        "expression" => {
            p.line(&[(0.1, 0.55), (0.9, 0.55)]);
            p.line(&[(0.3, 0.12), (0.3, 0.9)]);
            p.curve(32, |t| {
                (0.1 + 0.8 * t, 0.55 - 0.28 * (t * TAU * 1.25).sin())
            });
        }
        "force-layout" => {
            let nodes = [
                (0.2, 0.3),
                (0.75, 0.2),
                (0.5, 0.55),
                (0.25, 0.82),
                (0.82, 0.78),
            ];
            for (a, b) in [(0, 2), (1, 2), (2, 3), (2, 4), (0, 3), (1, 4)] {
                p.line(&[nodes[a], nodes[b]]);
            }
            for (x, y) in nodes {
                p.dot(x, y, 0.08);
            }
        }
        "min-max-normalization" => {
            p.line(&[(0.08, 0.15), (0.92, 0.15)]);
            p.line(&[(0.08, 0.9), (0.92, 0.9)]);
            p.bars(&[0.35, 1.0, 0.6, 0.0, 0.8], &[1, 3]);
        }
        "mandelbrot" => {
            p.curve(40, |t| {
                let a = t * TAU;
                let r = 0.5 * (1.0 - a.cos());
                (0.62 + 0.36 * (r * a.cos() - 0.25), 0.5 + 0.36 * r * a.sin())
            });
            p.circle(0.2, 0.5, 0.11);
        }
        "flow-field" => {
            for k in 0..3 {
                let y0 = 0.25 + 0.25 * k as f32;
                p.curve(20, |t| {
                    (0.08 + 0.84 * t, y0 + 0.1 * (t * TAU + k as f32).sin())
                });
            }
        }
        "game-of-life" => {
            for i in 0..4 {
                let v = 0.1 + 0.8 * i as f32 / 3.0;
                p.line(&[(v, 0.1), (v, 0.9)]);
                p.line(&[(0.1, v), (0.9, v)]);
            }
            for (cx, cy) in [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)] {
                let (x, y) = (0.1 + 0.8 * cx as f32 / 3.0, 0.1 + 0.8 * cy as f32 / 3.0);
                p.rect(
                    x + 0.04,
                    y + 0.04,
                    x + 0.8 / 3.0 - 0.04,
                    y + 0.8 / 3.0 - 0.04,
                    true,
                );
            }
        }
        "lorenz" => {
            p.curve(40, |t| {
                let a = t * TAU;
                (0.3 + 0.2 * a.cos(), 0.5 + 0.3 * a.sin())
            });
            p.curve(40, |t| {
                let a = t * TAU;
                (0.68 + 0.22 * a.cos(), 0.45 + 0.33 * a.sin())
            });
        }
        "euclidean-distance" => {
            p.line(&[(0.15, 0.82), (0.85, 0.82), (0.85, 0.18)]);
            p.line(&[(0.15, 0.82), (0.85, 0.18)]);
            p.dot(0.15, 0.82, 0.08);
            p.dot(0.85, 0.18, 0.08);
        }
        "rotation-matrix" => {
            p.closed(&[(0.5, 0.18), (0.82, 0.5), (0.5, 0.82), (0.18, 0.5)]);
            p.curve(16, |t| {
                let a = -PI * 0.15 - t * PI * 0.6;
                (0.5 + 0.42 * a.cos(), 0.5 + 0.42 * a.sin())
            });
            p.fill(&[(0.42, 0.04), (0.56, 0.1), (0.44, 0.18)]);
        }
        "matrix-transform" => {
            // A sheared grid: the unit square's image and its mid-lines.
            p.closed(&[(0.1, 0.88), (0.58, 0.88), (0.9, 0.18), (0.42, 0.18)]);
            p.line(&[(0.34, 0.88), (0.66, 0.18)]);
            p.line(&[(0.26, 0.53), (0.74, 0.53)]);
            p.dot(0.1, 0.88, 0.07);
        }
        "voronoi" => {
            p.closed(&[(0.1, 0.1), (0.9, 0.1), (0.9, 0.9), (0.1, 0.9)]);
            p.line(&[(0.1, 0.45), (0.45, 0.55), (0.62, 0.1)]);
            p.line(&[(0.45, 0.55), (0.55, 0.9)]);
            p.line(&[(0.62, 0.1), (0.9, 0.5)]);
            for (x, y) in [
                (0.27, 0.27),
                (0.27, 0.72),
                (0.76, 0.3),
                (0.72, 0.7),
                (0.47, 0.33),
            ] {
                p.dot(x, y, 0.045);
            }
        }
        "linear-regression" => {
            p.line(&[(0.1, 0.85), (0.9, 0.2)]);
            for (x, y) in [
                (0.18, 0.7),
                (0.32, 0.72),
                (0.45, 0.5),
                (0.6, 0.48),
                (0.72, 0.3),
                (0.84, 0.32),
            ] {
                p.dot(x, y, 0.05);
            }
        }
        "ease-out-cubic" => {
            p.line(&[(0.12, 0.1), (0.12, 0.88), (0.9, 0.88)]);
            p.curve(24, |t| {
                (0.12 + 0.78 * t, 0.88 - 0.78 * (1.0 - (1.0 - t).powi(3)))
            });
        }
        "relative-luminance" => {
            p.circle(0.5, 0.5, 0.36);
            p.curve(16, |t| {
                let a = -PI / 2.0 + t * PI;
                (0.5 + 0.36 * a.cos(), 0.5 + 0.36 * a.sin())
            });
            let s = rect.width().min(rect.height());
            let c = p.at(0.5, 0.5);
            let pts: Vec<Pos2> = (0..=16)
                .map(|k| {
                    let a = -PI / 2.0 + k as f32 / 16.0 * PI;
                    c + vec2(a.cos(), a.sin()) * 0.36 * s
                })
                .collect();
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        "bfs" => {
            // A 3x3 dot grid with rings spreading from the left-middle dot.
            for (x, y) in [
                (0.5, 0.18),
                (0.5, 0.5),
                (0.5, 0.82),
                (0.84, 0.18),
                (0.84, 0.5),
                (0.84, 0.82),
            ] {
                p.dot(x, y, 0.05);
            }
            p.dot(0.16, 0.18, 0.05);
            p.dot(0.16, 0.5, 0.08);
            p.dot(0.16, 0.82, 0.05);
            p.curve(16, |t| {
                let a = -PI / 2.0 + t * PI;
                (0.16 + 0.2 * a.cos(), 0.5 + 0.2 * a.sin())
            });
            p.curve(16, |t| {
                let a = -PI / 2.0 + t * PI;
                (0.16 + 0.34 * a.cos(), 0.5 + 0.34 * a.sin())
            });
        }
        "dfs" => {
            // One long zigzag diving to the bottom-right, ending in a dot.
            p.line(&[
                (0.12, 0.12),
                (0.5, 0.12),
                (0.5, 0.34),
                (0.16, 0.34),
                (0.16, 0.56),
                (0.5, 0.56),
                (0.5, 0.78),
                (0.86, 0.78),
            ]);
            p.dot(0.12, 0.12, 0.06);
            p.dot(0.86, 0.78, 0.08);
        }
        "dijkstra" => {
            // Three dots joined by lines; the shortest leg is drawn thick.
            let thick = Pen {
                stroke: Stroke::new(p.stroke.width * 2.2, color),
                painter: p.painter,
                rect: p.rect,
                color,
            };
            p.line(&[(0.15, 0.75), (0.5, 0.2)]);
            p.line(&[(0.5, 0.2), (0.85, 0.75)]);
            thick.line(&[(0.15, 0.75), (0.85, 0.75)]);
            p.dot(0.15, 0.75, 0.08);
            p.dot(0.5, 0.2, 0.08);
            p.dot(0.85, 0.75, 0.08);
        }
        "binary-search-tree" => {
            // A root with two children and two grandchildren joined by lines.
            p.line(&[(0.5, 0.18), (0.28, 0.5)]);
            p.line(&[(0.5, 0.18), (0.72, 0.5)]);
            p.line(&[(0.28, 0.5), (0.16, 0.84)]);
            p.line(&[(0.72, 0.5), (0.84, 0.84)]);
            p.dot(0.5, 0.18, 0.09);
            p.dot(0.28, 0.5, 0.08);
            p.dot(0.72, 0.5, 0.08);
            p.dot(0.16, 0.84, 0.07);
            p.dot(0.84, 0.84, 0.07);
        }
        "kruskal" => {
            // Four dots: three thick tree edges and one thin rejected edge.
            let thick = Pen {
                stroke: Stroke::new(p.stroke.width * 2.2, color),
                painter: p.painter,
                rect: p.rect,
                color,
            };
            thick.line(&[(0.15, 0.7), (0.4, 0.25)]);
            thick.line(&[(0.4, 0.25), (0.85, 0.35)]);
            thick.line(&[(0.85, 0.35), (0.7, 0.8)]);
            p.line(&[(0.15, 0.7), (0.7, 0.8)]);
            p.dot(0.15, 0.7, 0.08);
            p.dot(0.4, 0.25, 0.08);
            p.dot(0.85, 0.35, 0.08);
            p.dot(0.7, 0.8, 0.08);
        }
        "prim" => {
            // A dot with edges growing out to three dots.
            p.line(&[(0.2, 0.5), (0.55, 0.2)]);
            p.line(&[(0.2, 0.5), (0.6, 0.55)]);
            p.line(&[(0.6, 0.55), (0.85, 0.85)]);
            p.dot(0.2, 0.5, 0.1);
            p.dot(0.55, 0.2, 0.08);
            p.dot(0.6, 0.55, 0.08);
            p.dot(0.85, 0.85, 0.08);
        }
        "a-star" => {
            // A five-pointed star with a dot target at its centre.
            let pts: Vec<(f32, f32)> = (0..10)
                .map(|k| {
                    let a = -PI / 2.0 + k as f32 * PI / 5.0;
                    let r = if k % 2 == 0 { 0.42 } else { 0.18 };
                    (0.5 + r * a.cos(), 0.54 + r * a.sin())
                })
                .collect();
            p.closed(&pts);
            p.dot(0.5, 0.54, 0.06);
        }
        "bubble-sort" => p.bars(&[0.3, 0.55, 0.45, 0.8, 1.0], &[1, 2]),
        "selection-sort" => {
            p.bars(&[0.25, 0.5, 0.9, 0.15, 0.7], &[3]);
            p.line(&[(0.66, 0.08), (0.66, 0.18)]);
        }
        "insertion-sort" => {
            p.bars(&[0.2, 0.45, 0.7, 0.35, 0.9], &[3]);
            p.line(&[(0.55, 0.12), (0.43, 0.12)]);
            p.fill(&[(0.38, 0.12), (0.45, 0.07), (0.45, 0.17)]);
        }
        "merge-sort" => {
            p.bars(&[0.3, 0.7, 0.5, 0.9, 0.2, 0.6], &[]);
            p.line(&[(0.5, 0.05), (0.5, 0.95)]);
        }
        "quick-sort" => {
            p.bars(&[0.2, 0.4, 0.3, 0.6, 0.9, 0.75], &[3]);
            p.line(&[(0.05, 0.45), (0.95, 0.45)]);
        }
        "quick-sort-3way" => {
            p.bars(&[0.2, 0.3, 0.55, 0.55, 0.85, 0.9], &[2, 3]);
            p.line(&[(0.36, 0.05), (0.36, 0.95)]);
            p.line(&[(0.64, 0.05), (0.64, 0.95)]);
        }
        "heap-sort" => {
            p.line(&[(0.5, 0.22), (0.25, 0.55), (0.12, 0.85)]);
            p.line(&[(0.25, 0.55), (0.38, 0.85)]);
            p.line(&[(0.5, 0.22), (0.75, 0.55)]);
            for (x, y) in [
                (0.5, 0.22),
                (0.25, 0.55),
                (0.75, 0.55),
                (0.12, 0.85),
                (0.38, 0.85),
            ] {
                p.dot(x, y, 0.08);
            }
        }
        "counting-sort" => {
            p.bars(&[0.4, 0.8, 0.2, 0.6], &[]);
            for k in 0..4 {
                let x = 0.1 + 0.8 * k as f32 / 4.0 + 0.07;
                p.dot(x, 0.06 + 0.0, 0.035);
            }
        }
        "sorting-race" => {
            for (k, len) in [0.8f32, 0.55, 0.35].iter().enumerate() {
                let y = 0.22 + 0.28 * k as f32;
                p.rect(0.1, y - 0.07, 0.1 + 0.7 * len, y + 0.07, k == 0);
            }
            p.line(&[(0.88, 0.08), (0.88, 0.92)]);
        }
        _ => p.closed(&[(0.15, 0.15), (0.85, 0.15), (0.85, 0.85), (0.15, 0.85)]),
    }
}

/// The icon of a workspace command.
pub fn ui(painter: &Painter, rect: Rect, icon: Ui, color: Color32) {
    let p = pen(painter, rect, color);
    match icon {
        Ui::Select => p.fill(&[(0.25, 0.1), (0.25, 0.85), (0.45, 0.65), (0.75, 0.62)]),
        Ui::Pan => {
            p.line(&[(0.5, 0.08), (0.5, 0.92)]);
            p.line(&[(0.08, 0.5), (0.92, 0.5)]);
            for (a, b, c) in [
                ((0.38, 0.2), (0.5, 0.08), (0.62, 0.2)),
                ((0.38, 0.8), (0.5, 0.92), (0.62, 0.8)),
                ((0.2, 0.38), (0.08, 0.5), (0.2, 0.62)),
                ((0.8, 0.38), (0.92, 0.5), (0.8, 0.62)),
            ] {
                p.line(&[a, b, c]);
            }
        }
        Ui::Fit => {
            for (a, b, c) in [
                ((0.1, 0.35), (0.1, 0.1), (0.35, 0.1)),
                ((0.65, 0.1), (0.9, 0.1), (0.9, 0.35)),
                ((0.9, 0.65), (0.9, 0.9), (0.65, 0.9)),
                ((0.35, 0.9), (0.1, 0.9), (0.1, 0.65)),
            ] {
                p.line(&[a, b, c]);
            }
            p.rect(0.32, 0.32, 0.68, 0.68, false);
        }
        Ui::ZoomIn | Ui::ZoomOut => {
            p.circle(0.42, 0.42, 0.28);
            p.line(&[(0.62, 0.62), (0.9, 0.9)]);
            p.line(&[(0.28, 0.42), (0.56, 0.42)]);
            if icon == Ui::ZoomIn {
                p.line(&[(0.42, 0.28), (0.42, 0.56)]);
            }
        }
        Ui::Undo | Ui::Redo => {
            let m = |x: f32| if icon == Ui::Undo { x } else { 1.0 - x };
            p.curve(16, |t| {
                let a = PI * (1.0 - t) - PI * 0.0;
                (m(0.55 + 0.3 * a.cos()), 0.55 - 0.3 * a.sin())
            });
            p.line(&[(m(0.12), 0.38), (m(0.25), 0.58), (m(0.42), 0.45)]);
        }
        Ui::Grid => {
            for k in 0..4 {
                let v = 0.12 + 0.76 * k as f32 / 3.0;
                p.line(&[(v, 0.12), (v, 0.88)]);
                p.line(&[(0.12, v), (0.88, v)]);
            }
        }
        Ui::Snap => {
            p.circle(0.5, 0.5, 0.3);
            p.line(&[(0.5, 0.05), (0.5, 0.95)]);
            p.line(&[(0.05, 0.5), (0.95, 0.5)]);
            p.dot(0.5, 0.5, 0.08);
        }
        Ui::Ortho => {
            p.line(&[(0.15, 0.1), (0.15, 0.85), (0.9, 0.85)]);
            p.rect(0.15, 0.65, 0.35, 0.85, false);
        }
        Ui::Delete => {
            p.line(&[(0.15, 0.22), (0.85, 0.22)]);
            p.line(&[(0.4, 0.22), (0.42, 0.1), (0.58, 0.1), (0.6, 0.22)]);
            p.closed(&[(0.24, 0.22), (0.76, 0.22), (0.7, 0.9), (0.3, 0.9)]);
        }
        Ui::Duplicate => {
            p.rect(0.12, 0.12, 0.62, 0.62, false);
            p.rect(0.38, 0.38, 0.88, 0.88, false);
        }
        Ui::Docs => {
            p.closed(&[
                (0.2, 0.08),
                (0.65, 0.08),
                (0.82, 0.25),
                (0.82, 0.92),
                (0.2, 0.92),
            ]);
            for y in [0.38, 0.55, 0.72] {
                p.line(&[(0.32, y), (0.7, y)]);
            }
        }
        Ui::Derive => {
            // An equals sign and three lines of working, each shorter than the last.
            p.line(&[(0.1, 0.4), (0.3, 0.4)]);
            p.line(&[(0.1, 0.6), (0.3, 0.6)]);
            p.line(&[(0.42, 0.22), (0.9, 0.22)]);
            p.line(&[(0.42, 0.5), (0.8, 0.5)]);
            p.line(&[(0.42, 0.78), (0.68, 0.78)]);
        }
        Ui::Code => {
            p.line(&[(0.32, 0.25), (0.1, 0.5), (0.32, 0.75)]);
            p.line(&[(0.68, 0.25), (0.9, 0.5), (0.68, 0.75)]);
            p.line(&[(0.56, 0.15), (0.44, 0.85)]);
        }
        Ui::Copy => {
            p.rect(0.3, 0.3, 0.85, 0.88, false);
            p.line(&[(0.15, 0.7), (0.15, 0.12), (0.68, 0.12)]);
        }
        Ui::Download => {
            p.line(&[(0.5, 0.1), (0.5, 0.65)]);
            p.line(&[(0.28, 0.45), (0.5, 0.67), (0.72, 0.45)]);
            p.line(&[(0.12, 0.7), (0.12, 0.88), (0.88, 0.88), (0.88, 0.7)]);
        }
        Ui::Eye | Ui::EyeOff => {
            p.curve(16, |t| (0.08 + 0.84 * t, 0.5 - 0.3 * (t * PI).sin()));
            p.curve(16, |t| (0.08 + 0.84 * t, 0.5 + 0.3 * (t * PI).sin()));
            p.dot(0.5, 0.5, 0.1);
            if icon == Ui::EyeOff {
                p.line(&[(0.15, 0.88), (0.85, 0.12)]);
            }
        }
        Ui::Lock | Ui::Unlock => {
            p.rect(0.22, 0.45, 0.78, 0.9, true);
            let lift = if icon == Ui::Unlock { 0.12 } else { 0.0 };
            p.curve(12, |t| {
                let a = PI + t * PI;
                (0.5 + 0.2 * a.cos() + lift * t, 0.45 - lift + 0.28 * a.sin())
            });
        }
        Ui::Theme => {
            p.circle(0.5, 0.5, 0.32);
            let s = rect.width().min(rect.height());
            let c = p.at(0.5, 0.5);
            let pts: Vec<Pos2> = (0..=16)
                .map(|k| {
                    let a = PI / 2.0 + k as f32 / 16.0 * PI;
                    c + vec2(a.cos(), a.sin()) * 0.32 * s
                })
                .collect();
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        Ui::Help => {
            p.circle(0.5, 0.5, 0.4);
            p.curve(12, |t| {
                let a = PI + t * PI * 1.4;
                (0.5 + 0.13 * a.cos(), 0.4 + 0.13 * a.sin())
            });
            p.line(&[(0.5, 0.53), (0.5, 0.62)]);
            p.dot(0.5, 0.74, 0.04);
        }
        Ui::Layers => {
            p.closed(&[(0.5, 0.15), (0.9, 0.35), (0.5, 0.55), (0.1, 0.35)]);
            p.line(&[(0.1, 0.52), (0.5, 0.72), (0.9, 0.52)]);
            p.line(&[(0.1, 0.68), (0.5, 0.88), (0.9, 0.68)]);
        }
        Ui::Properties => {
            for (k, x) in [0.35f32, 0.65, 0.45].iter().enumerate() {
                let y = 0.22 + 0.28 * k as f32;
                p.line(&[(0.1, y), (0.9, y)]);
                p.rect(x - 0.08, y - 0.08, x + 0.08, y + 0.08, true);
            }
        }
        Ui::Maximize => {
            p.rect(0.12, 0.12, 0.88, 0.88, false);
            p.line(&[(0.36, 0.64), (0.7, 0.3)]);
            p.line(&[(0.46, 0.3), (0.7, 0.3), (0.7, 0.54)]);
        }
        Ui::Restore => {
            p.rect(0.12, 0.34, 0.66, 0.88, false);
            p.line(&[
                (0.34, 0.34),
                (0.34, 0.12),
                (0.88, 0.12),
                (0.88, 0.66),
                (0.66, 0.66),
            ]);
        }
    }
}

/// An icon-only button with a tooltip. `on` draws it as active.
pub fn button(ui: &mut egui::Ui, icon: Ui, on: bool, tip: &str) -> egui::Response {
    paint_button(ui, on, tip, |painter, rect, color| {
        self::ui(painter, rect, icon, color)
    })
}

/// A short text label (like "1:1") drawn as an icon button.
pub fn text_button(ui: &mut egui::Ui, text: &str, tip: &str) -> egui::Response {
    paint_button(ui, false, tip, |painter, rect, color| {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(11.0),
            color,
        );
    })
}

/// A tool's icon as a button.
pub fn tool_button(ui: &mut egui::Ui, id: &str, on: bool, tip: &str) -> egui::Response {
    paint_button(ui, on, tip, |painter, rect, color| {
        tool(painter, rect, id, color)
    })
}

fn paint_button(
    ui: &mut egui::Ui,
    on: bool,
    tip: &str,
    draw: impl FnOnce(&Painter, Rect, Color32),
) -> egui::Response {
    let size = vec2(26.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let v = ui.visuals();
        let hover = ui.ctx().animate_bool(response.id, response.hovered());
        let accent = v.selection.stroke.color;
        let bg = if on {
            v.selection.bg_fill
        } else {
            v.widgets.hovered.weak_bg_fill.gamma_multiply(hover)
        };
        ui.painter().rect_filled(rect, 3.0, bg);
        if on {
            ui.painter().rect_stroke(
                rect,
                3.0,
                Stroke::new(1.0, accent),
                egui::StrokeKind::Inside,
            );
        }
        let color = if on { accent } else { v.text_color() };
        draw(
            ui.painter(),
            Rect::from_center_size(rect.center(), vec2(16.0, 16.0)),
            color,
        );
    }
    response.on_hover_text(tip)
}

/// A small square swatch with the tool's icon, for lists.
pub fn tool_badge(painter: &Painter, rect: Rect, id: &str, fg: Color32, bg: Color32) {
    painter.rect_filled(rect, 3.0, bg);
    tool(painter, rect.shrink(rect.width() * 0.16), id, fg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn every_tool_has_its_own_icon() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let painter = ui.painter();
            let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(16.0, 16.0));
            for t in crate::formulas::TOOLS {
                tool(painter, r, t.id, Color32::WHITE);
            }
        });
        out.textures_delta.clear();
        // The fallback square is only for unknown ids.
        let known: Vec<_> = crate::formulas::TOOLS.iter().map(|t| t.id).collect();
        let src = include_str!("icons.rs");
        for id in known {
            assert!(src.contains(&format!("\"{id}\" =>")), "no icon for {id}");
        }
    }
}
