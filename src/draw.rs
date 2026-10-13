//! Drawing helpers that egui does not provide out of the box.

use egui::{Color32, Mesh, Pos2, Rect, pos2};

/// A horizontal run of filled pixels: `y` row from `y0` to `y1`, spanning `x0..x1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub y0: f32,
    pub y1: f32,
    pub x0: f32,
    pub x1: f32,
}

/// Scan-convert a closed polygon using the non-zero winding rule (the same
/// rule a 2D canvas uses for `fill()`), sampling one row every `step` units.
///
/// egui only fills convex paths; outlines like animal silhouettes and
/// partial Fourier reconstructions are anything but, so we build spans.
pub fn polygon_spans(points: &[Pos2], step: f32) -> Vec<Span> {
    let mut spans = Vec::new();
    if points.len() < 3 || step <= 0.0 {
        return spans;
    }
    let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
    for p in points {
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }
    if !min_y.is_finite() || !max_y.is_finite() {
        return spans;
    }

    let mut hits: Vec<(f32, i32)> = Vec::new();
    let mut y0 = (min_y / step).floor() * step;
    while y0 < max_y {
        let y = y0 + step * 0.5;
        hits.clear();
        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            if (a.y <= y) != (b.y <= y) {
                let u = (y - a.y) / (b.y - a.y);
                let dir = if b.y > a.y { 1 } else { -1 };
                hits.push((a.x + u * (b.x - a.x), dir));
            }
        }
        hits.sort_by(|p, q| p.0.total_cmp(&q.0));
        let mut winding = 0;
        let mut start = 0.0;
        for &(x, dir) in &hits {
            let was_inside = winding != 0;
            winding += dir;
            if !was_inside && winding != 0 {
                start = x;
            } else if was_inside && winding == 0 && x > start {
                spans.push(Span {
                    y0,
                    y1: y0 + step,
                    x0: start,
                    x1: x,
                });
            }
        }
        y0 += step;
    }
    spans
}

/// Append spans as quads to a mesh, mapping each span from `from` to `to`.
pub fn add_spans(mesh: &mut Mesh, spans: &[Span], from: Rect, to: Rect, color: Color32) {
    let sx = to.width() / from.width();
    let sy = to.height() / from.height();
    let map = |x: f32, y: f32| {
        pos2(
            to.min.x + (x - from.min.x) * sx,
            to.min.y + (y - from.min.y) * sy,
        )
    };
    for s in spans {
        let r = Rect::from_min_max(map(s.x0, s.y0), map(s.x1, s.y1));
        mesh.add_colored_rect(r, color);
    }
}

/// Fill an arbitrary (possibly concave or self-intersecting) closed polygon.
pub fn fill_polygon(points: &[Pos2], color: Color32, step: f32) -> Mesh {
    let mut mesh = Mesh::default();
    let spans = polygon_spans(points, step);
    let unit = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    add_spans(&mut mesh, &spans, unit, unit, color);
    mesh
}

/// Linear interpolation `a + t (b − a)`.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Interpolate two colors channel by channel (in gamma space, like CSS).
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let ch = |x: u8, y: u8| lerp(x as f32, y as f32, t).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgba_premultiplied(
        ch(a.r(), b.r()),
        ch(a.g(), b.g()),
        ch(a.b(), b.b()),
        ch(a.a(), b.a()),
    )
}

/// Sample a multi-stop gradient at `t` in `[0, 1]`.
pub fn gradient(stops: &[Color32], t: f32) -> Color32 {
    match stops.len() {
        0 => Color32::TRANSPARENT,
        1 => stops[0],
        n => {
            let x = t.clamp(0.0, 1.0) * (n - 1) as f32;
            let i = (x.floor() as usize).min(n - 2);
            lerp_color(stops[i], stops[i + 1], x - i as f32)
        }
    }
}

/// A round draggable handle, highlighted while hovered or dragged.
pub fn handle(painter: &egui::Painter, at: Pos2, color: Color32, ring: Color32, active: bool) {
    let r = if active { 8.0 } else { 6.5 };
    painter.circle(at, r, color, egui::Stroke::new(2.0, ring));
}

/// Index of the point nearest to `at` within `radius`, if any.
pub fn pick(points: &[Pos2], at: Pos2, radius: f32) -> Option<usize> {
    points
        .iter()
        .enumerate()
        .map(|(i, p)| (i, p.distance(at)))
        .filter(|&(_, d)| d <= radius)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(spans: &[Span]) -> f32 {
        spans.iter().map(|s| (s.x1 - s.x0) * (s.y1 - s.y0)).sum()
    }

    #[test]
    fn square_area() {
        let sq = [
            pos2(0.0, 0.0),
            pos2(10.0, 0.0),
            pos2(10.0, 10.0),
            pos2(0.0, 10.0),
        ];
        assert!((area(&polygon_spans(&sq, 1.0)) - 100.0).abs() < 1e-3);
    }

    #[test]
    fn concave_l_shape() {
        let l = [
            pos2(0.0, 0.0),
            pos2(4.0, 0.0),
            pos2(4.0, 2.0),
            pos2(2.0, 2.0),
            pos2(2.0, 4.0),
            pos2(0.0, 4.0),
        ];
        assert!((area(&polygon_spans(&l, 0.5)) - 12.0).abs() < 1e-3);
    }

    #[test]
    fn gradient_hits_stops() {
        let stops = [Color32::BLACK, Color32::WHITE, Color32::BLACK];
        assert_eq!(gradient(&stops, 0.0), Color32::BLACK);
        assert_eq!(gradient(&stops, 0.5), Color32::WHITE);
        assert_eq!(gradient(&stops, 1.0), Color32::BLACK);
        assert_eq!(gradient(&stops, 0.25), Color32::from_gray(128));
    }

    #[test]
    fn degenerate_input_is_empty() {
        assert!(polygon_spans(&[pos2(0.0, 0.0), pos2(1.0, 1.0)], 1.0).is_empty());
    }
}
