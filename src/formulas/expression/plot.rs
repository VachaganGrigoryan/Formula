//! The plotter's view and sampling, kept free of drawing so it is unit tested.
//!
//! Curves are sampled into polylines that break where the function is not
//! finite or jumps across an asymptote, so `tan x` never draws its vertical
//! connecting lines.

use egui::{Pos2, Rect, pos2};

/// Deepest subdivision of one initial sampling interval (2^6 = 64 pieces).
const MAX_DEPTH: u32 = 6;
/// A step bigger than this many view heights, with a midpoint that is not in
/// between, is an asymptote.
const JUMP_SPANS: f64 = 5.0;
/// Drawn points are clamped to this many view heights beyond the view.
const CLAMP_SPANS: f64 = 10.0;

/// Which part of the plane the stage shows. The scale is the same on both axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub cx: f64,
    pub cy: f64,
    /// World units per point.
    pub scale: f64,
}

impl Default for View {
    fn default() -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            scale: 0.02,
        }
    }
}

impl View {
    /// The screen position of a world point (world y points up).
    pub fn to_screen(&self, rect: Rect, p: [f64; 2]) -> Pos2 {
        let c = rect.center();
        pos2(
            c.x + ((p[0] - self.cx) / self.scale) as f32,
            c.y - ((p[1] - self.cy) / self.scale) as f32,
        )
    }

    pub fn to_world(&self, rect: Rect, p: Pos2) -> [f64; 2] {
        let c = rect.center();
        [
            self.cx + (p.x - c.x) as f64 * self.scale,
            self.cy - (p.y - c.y) as f64 * self.scale,
        ]
    }

    /// Zoom in by `factor` (above 1 zooms in) keeping the world point under
    /// `at` where it is.
    pub fn zoom_at(&mut self, rect: Rect, at: Pos2, factor: f64) {
        let w = self.to_world(rect, at);
        self.scale = (self.scale / factor).clamp(1e-9, 1e9);
        let c = rect.center();
        self.cx = w[0] - (at.x - c.x) as f64 * self.scale;
        self.cy = w[1] + (at.y - c.y) as f64 * self.scale;
    }

    pub fn x_range(&self, rect: Rect) -> (f64, f64) {
        let half = rect.width() as f64 * 0.5 * self.scale;
        (self.cx - half, self.cx + half)
    }

    pub fn y_range(&self, rect: Rect) -> (f64, f64) {
        let half = rect.height() as f64 * 0.5 * self.scale;
        (self.cy - half, self.cy + half)
    }
}

/// Collects points into polylines, dropping pieces too short to draw.
struct Pieces {
    out: Vec<Vec<[f64; 2]>>,
    cur: Vec<[f64; 2]>,
    lo: f64,
    hi: f64,
}

impl Pieces {
    fn new(y: (f64, f64)) -> Self {
        let span = y.1 - y.0;
        Self {
            out: Vec::new(),
            cur: Vec::new(),
            lo: y.0 - CLAMP_SPANS * span,
            hi: y.1 + CLAMP_SPANS * span,
        }
    }

    fn push(&mut self, p: [f64; 2]) {
        self.cur.push([p[0], p[1].clamp(self.lo, self.hi)]);
    }

    fn close(&mut self) {
        let cur = std::mem::take(&mut self.cur);
        if cur.len() >= 2 {
            self.out.push(cur);
        }
    }

    fn finish(mut self) -> Vec<Vec<[f64; 2]>> {
        self.close();
        self.out
    }
}

/// Polylines for `y = f(x)` over `x`, for a view showing `y` and `px` points
/// wide. Intervals that bend more than half a point or jump are subdivided.
pub fn sample_explicit(
    f: &dyn Fn(f64) -> f64,
    x: (f64, f64),
    y: (f64, f64),
    px: f32,
) -> Vec<Vec<[f64; 2]>> {
    let n = ((px / 2.0) as usize).max(8);
    let ctx = Ctx {
        f,
        tol: 0.5 * (x.1 - x.0) / px.max(1.0) as f64,
        jump: JUMP_SPANS * (y.1 - y.0),
    };
    let mut pieces = Pieces::new(y);
    let mut a = [x.0, f(x.0)];
    if a[1].is_finite() {
        pieces.push(a);
    }
    for i in 1..=n {
        let xb = x.0 + (x.1 - x.0) * i as f64 / n as f64;
        let b = [xb, f(xb)];
        refine(&ctx, a, b, 0, &mut pieces);
        a = b;
    }
    pieces.finish()
}

struct Ctx<'a> {
    f: &'a dyn Fn(f64) -> f64,
    /// Allowed distance of a midpoint from the chord, in world units.
    tol: f64,
    jump: f64,
}

/// Adds the points after `a` up to and including `b`, subdividing as needed.
/// When `a` is finite and nothing broke, `a` is the last point of the piece.
fn refine(ctx: &Ctx, a: [f64; 2], b: [f64; 2], depth: u32, pieces: &mut Pieces) {
    let mx = 0.5 * (a[0] + b[0]);
    let m = [mx, (ctx.f)(mx)];
    let finite = a[1].is_finite() && b[1].is_finite() && m[1].is_finite();
    let bends = finite && (m[1] - 0.5 * (a[1] + b[1])).abs() > ctx.tol;
    let jumps = (b[1] - a[1]).abs() > ctx.jump;
    if depth < MAX_DEPTH && (!finite || bends || jumps) {
        refine(ctx, a, m, depth + 1, pieces);
        refine(ctx, m, b, depth + 1, pieces);
        return;
    }
    let between = (a[1].min(b[1])..=a[1].max(b[1])).contains(&m[1]);
    if !b[1].is_finite() {
        pieces.close();
    } else if !a[1].is_finite() || (jumps && !between) {
        pieces.close();
        pieces.push(b);
    } else {
        pieces.push(b);
    }
}

/// Polylines for a parametric curve over `t` in `steps` equal steps, broken
/// where a point is not finite or the curve jumps by more than five view
/// heights; points are clamped near the view showing `x` by `y`.
pub fn sample_curve(
    f: &dyn Fn(f64) -> [f64; 2],
    t: (f64, f64),
    steps: usize,
    x: (f64, f64),
    y: (f64, f64),
) -> Vec<Vec<[f64; 2]>> {
    let span = (y.1 - y.0).max(x.1 - x.0);
    let jump = JUMP_SPANS * span;
    let (xlo, xhi) = (x.0 - CLAMP_SPANS * span, x.1 + CLAMP_SPANS * span);
    let mut pieces = Pieces::new(y);
    let mut prev: Option<[f64; 2]> = None;
    for i in 0..=steps.max(1) {
        let p = f(t.0 + (t.1 - t.0) * i as f64 / steps.max(1) as f64);
        if !(p[0].is_finite() && p[1].is_finite()) {
            pieces.close();
            prev = None;
            continue;
        }
        if let Some(q) = prev
            && ((p[0] - q[0]).abs() > jump || (p[1] - q[1]).abs() > jump)
        {
            pieces.close();
        }
        pieces.push([p[0].clamp(xlo, xhi), p[1]]);
        prev = Some(p);
    }
    pieces.finish()
}

/// A tick spacing of 1, 2 or 5 times a power of ten that gives about
/// `target` ticks (never fewer) across `span`.
pub fn nice_step(span: f64, target: usize) -> f64 {
    let raw = span.abs() / target.max(1) as f64;
    if !(raw.is_finite() && raw > 0.0) {
        return 1.0;
    }
    let mag = 10f64.powf(raw.log10().floor());
    let norm = raw / mag;
    let nice = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .find(|&s| s >= norm - 1e-9)
        .unwrap_or(10.0);
    nice * mag
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI, TAU};

    const X: (f64, f64) = (-5.0, 5.0);
    const Y: (f64, f64) = (-5.0, 5.0);

    fn all_finite_and_near(pieces: &[Vec<[f64; 2]>]) {
        let lim = 5.0 + CLAMP_SPANS * 10.0;
        for p in pieces.iter().flatten() {
            assert!(p[0].is_finite() && p[1].is_finite(), "{p:?}");
            assert!(p[1].abs() <= lim + 1e-9, "{p:?}");
        }
    }

    #[test]
    fn tan_breaks_at_each_asymptote() {
        let pieces = sample_explicit(&f64::tan, X, Y, 400.0);
        assert_eq!(pieces.len(), 5);
        all_finite_and_near(&pieces);
        let poles = [-3.0 * FRAC_PI_2, -FRAC_PI_2, FRAC_PI_2, 3.0 * FRAC_PI_2];
        for piece in &pieces {
            for w in piece.windows(2) {
                for pole in poles {
                    assert!(
                        !(w[0][0] < pole && w[1][0] > pole),
                        "segment {w:?} crosses {pole}"
                    );
                }
            }
        }
    }

    #[test]
    fn breaks_and_gaps_in_other_functions() {
        let recip = sample_explicit(&|x| 1.0 / x, (-1.0, 1.0), Y, 400.0);
        assert_eq!(recip.len(), 2);
        all_finite_and_near(&recip);
        let root = sample_explicit(&f64::sqrt, (-1.0, 1.0), Y, 400.0);
        assert_eq!(root.len(), 1);
        assert!(root[0].iter().all(|p| p[0] >= 0.0));
        assert!(root[0][0][0] < 0.01, "starts near the origin");
        assert_eq!(sample_explicit(&|x| x * x, X, Y, 400.0).len(), 1);
        assert!(sample_explicit(&|_| f64::NAN, X, Y, 400.0).is_empty());
        let huge = sample_explicit(&|x| x.exp() * 1e300, X, Y, 400.0);
        all_finite_and_near(&huge);
    }

    #[test]
    fn steep_but_continuous_functions_stay_whole() {
        assert_eq!(sample_explicit(&|x| x.powi(9), X, Y, 400.0).len(), 1);
        assert_eq!(sample_explicit(&|x| (50.0 * x).sin(), X, Y, 400.0).len(), 1);
    }

    #[test]
    fn curves_close_and_break() {
        let circle = sample_curve(&|t| [t.cos(), t.sin()], (0.0, TAU), 720, X, Y);
        assert_eq!(circle.len(), 1);
        let (first, last) = (circle[0][0], *circle[0].last().unwrap());
        assert!((first[0] - last[0]).abs() < 1e-9 && (first[1] - last[1]).abs() < 1e-9);
        // r = 1 / cos(t) is the line x = 1, going to infinity at t = pi/2.
        let line = sample_curve(
            &|t| {
                let r = 1.0 / t.cos();
                [r * t.cos(), r * t.sin()]
            },
            (0.0, PI),
            720,
            X,
            Y,
        );
        assert!(line.len() >= 2);
        all_finite_and_near(&line);
    }

    #[test]
    fn tick_steps_are_round() {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(close(nice_step(10.0, 5), 2.0));
        assert!(close(nice_step(1.0, 5), 0.2));
        assert!(close(nice_step(73.0, 8), 10.0));
        assert!(close(nice_step(0.0, 5), 1.0));
    }

    #[test]
    fn view_maps_both_ways_and_zooms_about_the_cursor() {
        let rect = Rect::from_min_size(pos2(10.0, 20.0), egui::vec2(400.0, 300.0));
        let mut v = View {
            cx: 1.5,
            cy: -2.0,
            scale: 0.05,
        };
        let p = [3.25, 0.5];
        let back = v.to_world(rect, v.to_screen(rect, p));
        assert!((back[0] - p[0]).abs() < 1e-4 && (back[1] - p[1]).abs() < 1e-4);
        let at = pos2(100.0, 250.0);
        let before = v.to_world(rect, at);
        v.zoom_at(rect, at, 2.0);
        let after = v.to_world(rect, at);
        assert!((before[0] - after[0]).abs() < 1e-9 && (before[1] - after[1]).abs() < 1e-9);
        assert!((v.scale - 0.025).abs() < 1e-12);
        let (x0, x1) = v.x_range(rect);
        assert!((x1 - x0 - 400.0 * 0.025).abs() < 1e-9);
    }
}
