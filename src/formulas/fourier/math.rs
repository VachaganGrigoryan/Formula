//! Fourier series of a closed outline, independent of any UI.
//!
//! A closed curve is resampled to `M` points evenly spaced by arc length,
//! normalized into a unit box, and each coordinate is expanded as
//!
//! ```text
//! x_N(t) = a0 + Σ_{k=1..N} (a_k cos kt + b_k sin kt)
//! y_N(t) = c0 + Σ_{k=1..N} (c_k cos kt + d_k sin kt)
//! ```
//!
//! Each `(a_k, b_k)` pair is stored in polar form as one spinning circle:
//! `a_k cos kt + b_k sin kt = r cos(kt − φ)` with `r = hypot(a, b)`, `φ = atan2(b, a)`.

use std::f64::consts::TAU;

/// Samples taken along the outline.
pub const SAMPLES: usize = 512;
/// Highest harmonic computed.
pub const MAX_TERMS: usize = 200;

pub type Point = [f64; 2];

/// One harmonic, as a circle of radius `r` spinning at `k` turns per period.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Term {
    pub k: usize,
    pub r: f64,
    pub phase: f64,
}

impl Term {
    pub fn angle(&self, t: f64) -> f64 {
        self.k as f64 * t - self.phase
    }
}

#[derive(Clone, Debug)]
pub struct Series {
    /// Mean x (the `a0` term).
    pub a0: f64,
    /// Mean y (the `c0` term).
    pub c0: f64,
    /// Harmonics of x, ordered by `k`.
    pub xs: Vec<Term>,
    /// Harmonics of y, ordered by `k`.
    pub ys: Vec<Term>,
}

impl Series {
    /// Evaluate the partial sum with the first `n` harmonics at angle `t`.
    pub fn eval(&self, n: usize, t: f64) -> Point {
        let n = n.min(self.xs.len());
        let mut x = self.a0;
        let mut y = self.c0;
        for (p, q) in self.xs[..n].iter().zip(&self.ys[..n]) {
            x += p.r * p.angle(t).cos();
            y += q.r * q.angle(t).cos();
        }
        [x, y]
    }

    /// The partial-sum curve sampled at `samples + 1` points (closed).
    pub fn curve(&self, n: usize, samples: usize) -> Vec<Point> {
        (0..=samples)
            .map(|j| self.eval(n, TAU * j as f64 / samples as f64))
            .collect()
    }
}

/// Resample a closed polyline to `m` points evenly spaced along its length.
pub fn resample(raw: &[Point], m: usize) -> Vec<Point> {
    if raw.is_empty() || m == 0 {
        return Vec::new();
    }
    let mut p = raw.to_vec();
    p.push(raw[0]);
    let mut len = vec![0.0];
    for i in 1..p.len() {
        let d = (p[i][0] - p[i - 1][0]).hypot(p[i][1] - p[i - 1][1]);
        len.push(len[i - 1] + d);
    }
    let total = *len.last().unwrap();
    let mut out = Vec::with_capacity(m);
    let mut j = 1;
    for i in 0..m {
        let s = total * i as f64 / m as f64;
        while j < len.len() - 1 && len[j] < s {
            j += 1;
        }
        let seg = len[j] - len[j - 1];
        let u = if seg > 0.0 {
            (s - len[j - 1]) / seg
        } else {
            0.0
        };
        out.push([
            p[j - 1][0] + u * (p[j][0] - p[j - 1][0]),
            p[j - 1][1] + u * (p[j][1] - p[j - 1][1]),
        ]);
    }
    out
}

/// Center the points on their bounding box and scale the longer side to 1.
pub fn normalize(p: &[Point]) -> Vec<Point> {
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &[x, y] in p {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    let side = (x1 - x0).max(y1 - y0);
    let s = if side > 0.0 { 1.0 / side } else { 1.0 };
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    p.iter()
        .map(|&[x, y]| [(x - cx) * s, (y - cy) * s])
        .collect()
}

/// Discrete Fourier coefficients of evenly spaced samples, up to `terms` harmonics.
pub fn fourier(p: &[Point], terms: usize) -> Series {
    let m = p.len().max(1) as f64;
    let a0 = p.iter().map(|q| q[0]).sum::<f64>() / m;
    let c0 = p.iter().map(|q| q[1]).sum::<f64>() / m;
    let mut xs = Vec::with_capacity(terms);
    let mut ys = Vec::with_capacity(terms);
    for k in 1..=terms {
        let (mut a, mut b, mut c, mut d) = (0.0, 0.0, 0.0, 0.0);
        for (j, &[x, y]) in p.iter().enumerate() {
            let (sn, cs) = (TAU * k as f64 * j as f64 / m).sin_cos();
            a += x * cs;
            b += x * sn;
            c += y * cs;
            d += y * sn;
        }
        let f = 2.0 / m;
        let (a, b, c, d) = (a * f, b * f, c * f, d * f);
        xs.push(Term {
            k,
            r: a.hypot(b),
            phase: b.atan2(a),
        });
        ys.push(Term {
            k,
            r: c.hypot(d),
            phase: d.atan2(c),
        });
    }
    Series { a0, c0, xs, ys }
}

/// Full pipeline: raw outline → evenly sampled unit outline + its series.
pub fn analyze(raw: &[Point]) -> (Vec<Point>, Series) {
    let pts = normalize(&resample(raw, SAMPLES));
    let series = fourier(&pts, MAX_TERMS);
    (pts, series)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(n: usize, r: f64) -> Vec<Point> {
        (0..n)
            .map(|i| {
                let t = TAU * i as f64 / n as f64;
                [r * t.cos(), r * t.sin()]
            })
            .collect()
    }

    #[test]
    fn resample_keeps_count_and_spacing() {
        let sq = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        let out = resample(&sq, 16);
        assert_eq!(out.len(), 16);
        for w in out.windows(2) {
            let d = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
            assert!((d - 1.0).abs() < 1e-9, "spacing {d}");
        }
    }

    #[test]
    fn normalize_fits_unit_box() {
        let out = normalize(&[[10.0, 10.0], [30.0, 20.0], [20.0, 15.0]]);
        let w = out.iter().map(|p| p[0]).fold(f64::MIN, f64::max)
            - out.iter().map(|p| p[0]).fold(f64::MAX, f64::min);
        assert!((w - 1.0).abs() < 1e-12);
        assert!(out.iter().all(|p| p[0].abs() <= 0.5 && p[1].abs() <= 0.5));
    }

    #[test]
    fn circle_is_one_harmonic() {
        let s = fourier(&circle(SAMPLES, 0.5), 10);
        assert!((s.xs[0].r - 0.5).abs() < 1e-9);
        assert!((s.ys[0].r - 0.5).abs() < 1e-9);
        assert!(s.xs[1..].iter().all(|t| t.r < 1e-9));
    }

    #[test]
    fn full_series_reconstructs_samples() {
        let sq = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let (pts, s) = analyze(&sq);
        // With all harmonics up to Nyquist the DFT interpolates exactly;
        // 200 of 256 already lands close to every sample.
        let err = pts
            .iter()
            .enumerate()
            .map(|(j, p)| {
                let q = s.eval(MAX_TERMS, TAU * j as f64 / SAMPLES as f64);
                (p[0] - q[0]).hypot(p[1] - q[1])
            })
            .fold(0.0, f64::max);
        assert!(err < 0.02, "max error {err}");
    }

    #[test]
    fn eval_clamps_terms() {
        let s = fourier(&circle(64, 1.0), 3);
        assert_eq!(s.eval(99, 0.0), s.eval(3, 0.0));
    }
}
