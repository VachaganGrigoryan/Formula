//! Drawing for the drafting chrome: grid, rulers, frames, grips, dimension
//! lines, guides, crosshair, marquee and minimap. All of it is drawn in
//! screen space, so it stays one pixel crisp at any zoom.

use std::f32::consts::FRAC_PI_2;

use egui::emath::TSTransform;
use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
    pos2, vec2,
};

use super::board::Guide;
use super::theme::{Chrome, mono};

/// Width of the rulers along the top and left of the canvas.
pub const RULER: f32 = 20.0;
/// Height of a viewport's title tab.
pub const TAB_H: f32 = 20.0;

/// The canvas background with the grid and the world axes.
pub fn paper(
    painter: &Painter,
    view: Rect,
    t: TSTransform,
    step: f32,
    chrome: &Chrome,
    grid: bool,
) {
    painter.rect_filled(view, 0.0, chrome.canvas);
    let inv = t.inverse();
    let world = inv.mul_rect(view);
    if grid {
        let major = step * 5.0;
        let mut x = (world.left() / step).floor() * step;
        while x <= world.right() {
            let sx = t.mul_pos(pos2(x, 0.0)).x.round() + 0.5;
            let is_major = (x / major).round() * major == x;
            let color = if is_major {
                chrome.grid_major
            } else {
                chrome.grid_minor
            };
            painter.line_segment(
                [pos2(sx, view.top()), pos2(sx, view.bottom())],
                Stroke::new(1.0, color),
            );
            x += step;
        }
        let mut y = (world.top() / step).floor() * step;
        while y <= world.bottom() {
            let sy = t.mul_pos(pos2(0.0, y)).y.round() + 0.5;
            let is_major = (y / major).round() * major == y;
            let color = if is_major {
                chrome.grid_major
            } else {
                chrome.grid_minor
            };
            painter.line_segment(
                [pos2(view.left(), sy), pos2(view.right(), sy)],
                Stroke::new(1.0, color),
            );
            y += step;
        }
    }
    // World axes through the origin.
    let o = t.mul_pos(Pos2::ZERO);
    if view.x_range().contains(o.x) {
        painter.line_segment(
            [pos2(o.x, view.top()), pos2(o.x, view.bottom())],
            Stroke::new(1.0, chrome.axis_y.gamma_multiply(0.45)),
        );
    }
    if view.y_range().contains(o.y) {
        painter.line_segment(
            [pos2(view.left(), o.y), pos2(view.right(), o.y)],
            Stroke::new(1.0, chrome.axis_x.gamma_multiply(0.45)),
        );
    }
    if view.contains(o) {
        // The UCS icon: short X and Y arrows at the origin.
        let s = 26.0;
        painter.arrow(o, vec2(s, 0.0), Stroke::new(1.5, chrome.axis_x));
        painter.arrow(o, vec2(0.0, -s), Stroke::new(1.5, chrome.axis_y));
        painter.text(
            o + vec2(s + 4.0, 0.0),
            Align2::LEFT_CENTER,
            "X",
            mono(10.0),
            chrome.axis_x,
        );
        painter.text(
            o + vec2(0.0, -s - 4.0),
            Align2::CENTER_BOTTOM,
            "Y",
            mono(10.0),
            chrome.axis_y,
        );
    }
}

/// Tick labels: whole numbers, with a minus sign for negatives.
fn tick_label(v: f32) -> String {
    let v = if v.abs() < 1e-3 { 0.0 } else { v };
    if v.fract().abs() < 1e-3 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

/// Rulers along the top and left of the canvas, marking the pointer and the
/// extent of the selection.
pub fn rulers(
    painter: &Painter,
    view: Rect,
    t: TSTransform,
    step: f32,
    chrome: &Chrome,
    pointer: Option<Pos2>,
    selection: Option<Rect>,
) {
    let top = Rect::from_min_max(
        view.min + vec2(RULER, 0.0),
        pos2(view.right(), view.top() + RULER),
    );
    let left = Rect::from_min_max(
        view.min + vec2(0.0, RULER),
        pos2(view.left() + RULER, view.bottom()),
    );
    for r in [top, left] {
        painter.rect_filled(r, 0.0, chrome.well);
    }
    let corner = Rect::from_min_size(view.min, vec2(RULER, RULER));
    painter.rect_filled(corner, 0.0, chrome.panel);

    let inv = t.inverse();
    let world = inv.mul_rect(view);
    // Selection extents as bands.
    if let Some(sel) = selection {
        let s = t.mul_rect(sel);
        let band = chrome.accent.gamma_multiply(0.25);
        painter.rect_filled(
            Rect::from_x_y_ranges(
                s.left().max(top.left())..=s.right().min(top.right()),
                top.y_range(),
            ),
            0.0,
            band,
        );
        painter.rect_filled(
            Rect::from_x_y_ranges(
                left.x_range(),
                s.top().max(left.top())..=s.bottom().min(left.bottom()),
            ),
            0.0,
            band,
        );
    }
    let label_every = step * 5.0;
    let tick = Stroke::new(1.0, chrome.text_dim.gamma_multiply(0.7));
    let mut x = (world.left() / step).floor() * step;
    while x <= world.right() {
        let sx = t.mul_pos(pos2(x, 0.0)).x.round() + 0.5;
        if sx > top.left() {
            let labeled = (x / label_every).round() * label_every == x;
            let len = if labeled { RULER * 0.6 } else { RULER * 0.25 };
            painter.line_segment([pos2(sx, top.bottom() - len), pos2(sx, top.bottom())], tick);
            if labeled {
                painter.text(
                    pos2(sx + 3.0, top.top() + 1.0),
                    Align2::LEFT_TOP,
                    tick_label(x),
                    mono(9.0),
                    chrome.text_dim,
                );
            }
        }
        x += step;
    }
    let mut y = (world.top() / step).floor() * step;
    while y <= world.bottom() {
        let sy = t.mul_pos(pos2(0.0, y)).y.round() + 0.5;
        if sy > left.top() {
            let labeled = (y / label_every).round() * label_every == y;
            let len = if labeled { RULER * 0.6 } else { RULER * 0.25 };
            painter.line_segment([pos2(left.right() - len, sy), pos2(left.right(), sy)], tick);
            if labeled {
                let galley = painter.layout_no_wrap(tick_label(y), mono(9.0), chrome.text_dim);
                let pos = pos2(left.left() + 2.0, sy + 3.0 + galley.size().x);
                painter.add(
                    egui::epaint::TextShape::new(pos, galley, chrome.text_dim)
                        .with_angle(-FRAC_PI_2),
                );
            }
        }
        y += step;
    }
    painter.line_segment(
        [top.left_bottom(), top.right_bottom()],
        Stroke::new(1.0, chrome.border),
    );
    painter.line_segment(
        [left.right_top(), left.right_bottom()],
        Stroke::new(1.0, chrome.border),
    );
    if let Some(p) = pointer {
        let mark = Stroke::new(1.0, chrome.accent);
        if p.x > top.left() {
            painter.line_segment([pos2(p.x, top.top()), pos2(p.x, top.bottom())], mark);
        }
        if p.y > left.top() {
            painter.line_segment([pos2(left.left(), p.y), pos2(left.right(), p.y)], mark);
        }
    }
}

/// The outline of a viewport.
pub fn frame(painter: &Painter, r: Rect, stroke: Stroke) {
    painter.rect_stroke(r, 0.0, stroke, StrokeKind::Outside);
}

/// A viewport's title tab, sitting on its top-left corner. Returns its rect.
pub fn tab_rect(painter: &Painter, r: Rect, text: &str) -> Rect {
    let w = painter
        .layout_no_wrap(text.to_owned(), FontId::proportional(11.5), Color32::WHITE)
        .size()
        .x
        + 16.0;
    let w = w.min(r.width().max(48.0));
    Rect::from_min_size(pos2(r.left(), r.top() - TAB_H), vec2(w, TAB_H))
}

pub fn tab(
    painter: &Painter,
    rect: Rect,
    text: &str,
    selected: bool,
    hovered: bool,
    locked: bool,
    chrome: &Chrome,
) {
    let fill = if selected {
        chrome.accent
    } else if hovered {
        chrome.tab.gamma_multiply(1.4)
    } else {
        chrome.tab
    };
    painter.rect_filled(
        rect,
        CornerRadius {
            nw: 3,
            ne: 3,
            sw: 0,
            se: 0,
        },
        fill,
    );
    let ink = if selected { chrome.well } else { chrome.text };
    let clip = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let mut x = rect.left() + 8.0;
    if locked {
        let lock = Rect::from_center_size(pos2(x + 4.0, rect.center().y), vec2(9.0, 9.0));
        super::icons::ui(&clip, lock, super::icons::Ui::Lock, ink);
        x += 12.0;
    }
    clip.text(
        pos2(x, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(11.5),
        ink,
    );
}

pub fn grip(painter: &Painter, at: Pos2, hovered: bool, chrome: &Chrome) {
    let r = Rect::from_center_size(at, vec2(8.0, 8.0));
    painter.rect_filled(r, 0.0, if hovered { chrome.accent } else { chrome.well });
    painter.rect_stroke(r, 0.0, Stroke::new(1.0, chrome.accent), StrokeKind::Inside);
}

/// A dimension line with extension lines and architectural ticks, labeled
/// with `value`. `a` and `b` are the measured points; the line sits `offset`
/// away from them along `normal`.
fn dimension(
    painter: &Painter,
    a: Pos2,
    b: Pos2,
    normal: Vec2,
    offset: f32,
    value: f32,
    chrome: &Chrome,
) {
    let c = chrome.dimension;
    let st = Stroke::new(1.0, c);
    let (da, db) = (a + normal * offset, b + normal * offset);
    painter.line_segment([a + normal * 3.0, da + normal * 5.0], st);
    painter.line_segment([b + normal * 3.0, db + normal * 5.0], st);
    painter.line_segment([da, db], st);
    // 45° ticks at both ends.
    let along = (db - da).normalized();
    let tick = (along + normal).normalized() * 4.5;
    for p in [da, db] {
        painter.line_segment([p - tick, p + tick], Stroke::new(1.6, c));
    }
    let mid = da + (db - da) * 0.5;
    let text = format!("{value:.0}");
    let galley = painter.layout_no_wrap(text, mono(10.5), c);
    let vertical = along.x.abs() < 0.5;
    let size = if vertical {
        vec2(galley.size().y, galley.size().x)
    } else {
        galley.size()
    };
    let bg = Rect::from_center_size(mid, size + vec2(6.0, 4.0));
    painter.rect_filled(bg, 2.0, chrome.canvas);
    if vertical {
        let pos = pos2(
            bg.center().x - galley.size().y / 2.0,
            bg.center().y + galley.size().x / 2.0,
        );
        painter.add(egui::epaint::TextShape::new(pos, galley, c).with_angle(-FRAC_PI_2));
    } else {
        painter.galley(bg.center() - galley.size() / 2.0, galley, c);
    }
}

/// Width below and height to the right of a selected viewport.
pub fn dimensions(painter: &Painter, screen: Rect, world: Rect, chrome: &Chrome) {
    dimension(
        painter,
        screen.left_bottom(),
        screen.right_bottom(),
        Vec2::Y,
        18.0,
        world.width(),
        chrome,
    );
    dimension(
        painter,
        screen.right_top(),
        screen.right_bottom(),
        Vec2::X,
        18.0,
        world.height(),
        chrome,
    );
}

fn dashed_rect(painter: &Painter, r: Rect, stroke: Stroke) {
    let pts = [
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.left_bottom(),
        r.left_top(),
    ];
    painter.extend(Shape::dashed_line(&pts, stroke, 6.0, 4.0));
}

pub fn guides(painter: &Painter, t: TSTransform, guides: &[Guide], chrome: &Chrome) {
    let st = Stroke::new(1.0, chrome.guide);
    for g in guides {
        let (a, b) = if g.vertical {
            (pos2(g.at, g.from), pos2(g.at, g.to))
        } else {
            (pos2(g.from, g.at), pos2(g.to, g.at))
        };
        let (a, b) = (t.mul_pos(a), t.mul_pos(b));
        painter.extend(Shape::dashed_line(&[a, b], st, 5.0, 3.0));
        for p in [a, b] {
            painter.line_segment([p - vec2(3.0, 3.0), p + vec2(3.0, 3.0)], st);
            painter.line_segment([p - vec2(3.0, -3.0), p + vec2(3.0, -3.0)], st);
        }
    }
}

/// Full-canvas crosshair with a pickbox at the pointer.
pub fn crosshair(painter: &Painter, view: Rect, p: Pos2, chrome: &Chrome) {
    let st = Stroke::new(1.0, chrome.accent.gamma_multiply(0.45));
    let gap = 6.0;
    painter.line_segment([pos2(view.left(), p.y), pos2(p.x - gap, p.y)], st);
    painter.line_segment([pos2(p.x + gap, p.y), pos2(view.right(), p.y)], st);
    painter.line_segment([pos2(p.x, view.top()), pos2(p.x, p.y - gap)], st);
    painter.line_segment([pos2(p.x, p.y + gap), pos2(p.x, view.bottom())], st);
    painter.rect_stroke(
        Rect::from_center_size(p, vec2(9.0, 9.0)),
        0.0,
        Stroke::new(1.0, chrome.accent),
        StrokeKind::Middle,
    );
}

/// Window selection (dragged left to right: solid, selects what is fully
/// inside) or crossing selection (right to left: dashed, selects what it
/// touches), as in AutoCAD.
pub fn marquee(painter: &Painter, r: Rect, window: bool, chrome: &Chrome) {
    let color = if window { chrome.accent } else { chrome.axis_y };
    painter.rect_filled(r, 0.0, color.gamma_multiply(0.08));
    if window {
        painter.rect_stroke(r, 0.0, Stroke::new(1.0, color), StrokeKind::Middle);
    } else {
        dashed_rect(painter, r, Stroke::new(1.0, color));
    }
}

/// The outline of a viewport about to be placed.
pub fn ghost(painter: &Painter, r: Rect, label: &str, chrome: &Chrome) {
    painter.rect_filled(r, 0.0, chrome.accent.gamma_multiply(0.06));
    dashed_rect(painter, r, Stroke::new(1.2, chrome.accent));
    painter.text(
        r.left_top() + vec2(6.0, 6.0),
        Align2::LEFT_TOP,
        label,
        FontId::proportional(12.0),
        chrome.accent,
    );
}

/// A small value tag next to the pointer (coordinates, sizes).
pub fn tag(painter: &Painter, at: Pos2, text: &str, chrome: &Chrome) {
    let galley = painter.layout_no_wrap(text.to_owned(), mono(11.0), chrome.well);
    // Above-right of the pointer, clear of a ghost's label below it.
    let size = galley.size() + vec2(10.0, 6.0);
    let r = Rect::from_min_size(at + vec2(16.0, -16.0 - size.y), size);
    painter.rect_filled(r, 3.0, chrome.dimension);
    painter.galley(r.min + vec2(5.0, 3.0), galley, chrome.well);
}

/// Where the minimap sits in the canvas.
pub fn minimap_rect(view: Rect) -> Rect {
    let size = vec2(200.0, 128.0);
    Rect::from_min_size(view.right_bottom() - size - vec2(14.0, 14.0), size)
}

/// The world-to-minimap mapping: (scale, offset) for the board bounds.
pub fn minimap_map(map: Rect, bounds: Rect) -> TSTransform {
    let inner = map.shrink(8.0);
    let scale = (inner.width() / bounds.width()).min(inner.height() / bounds.height());
    let offset = inner.center().to_vec2() - bounds.center().to_vec2() * scale;
    TSTransform::new(offset, scale)
}

pub fn minimap(
    painter: &Painter,
    map: Rect,
    m: TSTransform,
    rects: &[(Rect, bool)],
    seen: Rect,
    chrome: &Chrome,
) {
    painter.rect_filled(map, 3.0, chrome.panel.gamma_multiply(0.92));
    painter.rect_stroke(
        map,
        3.0,
        Stroke::new(1.0, chrome.border),
        StrokeKind::Inside,
    );
    let clip = painter.with_clip_rect(map.shrink(1.0));
    for &(r, selected) in rects {
        let s = m.mul_rect(r);
        clip.rect_filled(
            s,
            0.0,
            if selected {
                chrome.accent.gamma_multiply(0.6)
            } else {
                chrome.tab
            },
        );
        clip.rect_stroke(s, 0.0, Stroke::new(1.0, chrome.border), StrokeKind::Inside);
    }
    clip.rect_stroke(
        m.mul_rect(seen),
        0.0,
        Stroke::new(1.2, chrome.dimension),
        StrokeKind::Middle,
    );
}

/// The drawing's title block, bottom-left, like a printed sheet.
pub fn title_block(painter: &Painter, view: Rect, chrome: &Chrome, lines: [&str; 2]) {
    let w = 250.0;
    let r = Rect::from_min_size(
        pos2(view.left() + RULER + 12.0, view.bottom() - 56.0),
        vec2(w, 44.0),
    );
    painter.rect_filled(r, 0.0, chrome.panel.gamma_multiply(0.92));
    painter.rect_stroke(r, 0.0, Stroke::new(1.0, chrome.border), StrokeKind::Inside);
    let split = r.top() + 22.0;
    painter.line_segment(
        [pos2(r.left(), split), pos2(r.right(), split)],
        Stroke::new(1.0, chrome.border),
    );
    painter.text(
        pos2(r.left() + 8.0, r.top() + 11.0),
        Align2::LEFT_CENTER,
        lines[0],
        FontId::proportional(12.0),
        chrome.text,
    );
    painter.text(
        pos2(r.left() + 8.0, split + 11.0),
        Align2::LEFT_CENTER,
        lines[1],
        mono(10.0),
        chrome.text_dim,
    );
}

/// Today's date as YYYY-MM-DD, from the system clock.
pub fn today() -> String {
    let secs = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    format!("{y:04}-{m:02}-{d:02}")
}

/// Days since 1970-01-01 to a calendar date (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
        assert_eq!(civil_from_days(20_739), (2026, 10, 13));
    }

    #[test]
    fn tick_labels() {
        assert_eq!(tick_label(100.0), "100");
        assert_eq!(tick_label(-0.0001), "0");
        assert_eq!(tick_label(-2.5), "-2.5");
    }

    #[test]
    fn minimap_fits_bounds() {
        let map = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 128.0));
        let bounds = Rect::from_min_size(pos2(-500.0, 100.0), vec2(3000.0, 900.0));
        let m = minimap_map(map, bounds);
        let shown = m.mul_rect(bounds);
        assert!(map.contains_rect(shown));
        assert!((shown.width() - 184.0).abs() < 0.01);
    }
}
