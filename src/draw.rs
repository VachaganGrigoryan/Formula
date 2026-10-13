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
    fn degenerate_input_is_empty() {
        assert!(polygon_spans(&[pos2(0.0, 0.0), pos2(1.0, 1.0)], 1.0).is_empty());
    }
}
