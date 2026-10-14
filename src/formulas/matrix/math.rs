//! 2×2 matrices: applying, composing, interpolating and their eigenvectors.

pub type Vec2 = [f32; 2];

/// The matrix `[[a, b], [c, d]]`, acting on column vectors: its columns are
/// where the basis vectors `i` and `j` land.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Mat2 {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

impl Mat2 {
    pub const IDENTITY: Mat2 = Mat2::new(1.0, 0.0, 0.0, 1.0);

    pub const fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self { a, b, c, d }
    }

    /// Counter-clockwise rotation by `theta` radians (y up).
    pub fn rotation(theta: f32) -> Self {
        let (s, c) = theta.sin_cos();
        Self::new(c, -s, s, c)
    }

    /// `M v`: `(a x + b y, c x + d y)`.
    pub fn apply(self, [x, y]: Vec2) -> Vec2 {
        [self.a * x + self.b * y, self.c * x + self.d * y]
    }

    /// How the matrix scales areas; negative when it flips orientation.
    pub fn det(self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    /// `I + t (M − I)`: the identity at `t = 0`, this matrix at `t = 1`.
    pub fn lerp(self, t: f32) -> Mat2 {
        Mat2::new(
            1.0 + t * (self.a - 1.0),
            t * self.b,
            t * self.c,
            1.0 + t * (self.d - 1.0),
        )
    }

    /// How many independent columns: 2 for an invertible matrix, 1 when the
    /// plane collapses onto a line, 0 when it collapses onto the origin.
    pub fn rank(self) -> u8 {
        let s = self.scale();
        if [self.a, self.b, self.c, self.d]
            .iter()
            .all(|x| x.abs() < 1e-6 * s)
        {
            0
        } else if self.det().abs() < 1e-4 * s * s {
            1
        } else {
            2
        }
    }

    /// The largest entry's size, at least 1: the scale for tolerances.
    fn scale(self) -> f32 {
        [self.a, self.b, self.c, self.d]
            .iter()
            .fold(1.0_f32, |m, x| m.max(x.abs()))
    }

    /// Eigenvalues from `λ² − tr λ + det = 0`, with their unit eigenvectors.
    pub fn eigen(self) -> Eigen {
        let half_trace = (self.a + self.d) / 2.0;
        let disc = half_trace * half_trace - self.det();
        let eps = 1e-6 * self.scale() * self.scale();
        if disc < -eps {
            Eigen::Complex {
                re: half_trace,
                im: (-disc).sqrt(),
            }
        } else if disc > eps {
            let r = disc.sqrt();
            let values = [half_trace + r, half_trace - r];
            // Distinct eigenvalues always have an eigenvector each.
            let vectors = values.map(|l| self.eigenvector(l).unwrap_or([1.0, 0.0]));
            Eigen::Real { values, vectors }
        } else {
            Eigen::Repeated {
                value: half_trace,
                vector: self.eigenvector(half_trace),
            }
        }
    }

    /// A unit vector in the null space of `M − λI`, or `None` when that is
    /// the zero matrix (every vector is an eigenvector).
    fn eigenvector(self, l: f32) -> Option<Vec2> {
        // Each candidate is perpendicular to one row of M − λI.
        let from_row1 = [self.b, l - self.a];
        let from_row2 = [l - self.d, self.c];
        let len = |v: Vec2| v[0].hypot(v[1]);
        let v = if len(from_row1) >= len(from_row2) {
            from_row1
        } else {
            from_row2
        };
        let n = len(v);
        (n > 1e-6 * self.scale()).then(|| [v[0] / n, v[1] / n])
    }
}

/// `A * B`: apply `B` first, then `A`.
impl std::ops::Mul for Mat2 {
    type Output = Mat2;

    fn mul(self, o: Mat2) -> Mat2 {
        Mat2::new(
            self.a * o.a + self.b * o.c,
            self.a * o.b + self.b * o.d,
            self.c * o.a + self.d * o.c,
            self.c * o.b + self.d * o.d,
        )
    }
}

/// The single matrix for applying `chain[0]` first, then `chain[1]`, …:
/// `… · M2 · M1`. The identity for an empty chain.
pub fn product(chain: &[Mat2]) -> Mat2 {
    chain.iter().fold(Mat2::IDENTITY, |acc, &m| m * acc)
}

/// The chain part way through its animation: stage `k` (for `t` in
/// `k..k + 1`) morphs from the identity while the earlier stages are done.
/// `t = 0` is the identity and `t ≥ chain.len()` the full product.
pub fn staged(chain: &[Mat2], t: f32) -> Mat2 {
    let t = t.max(0.0);
    if t >= chain.len() as f32 {
        return product(chain);
    }
    let k = t.floor() as usize;
    chain[k].lerp(smoothstep(t - k as f32)) * product(&chain[..k])
}

/// Ease in and out over `0..=1`.
pub fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// The eigenvalues of a 2×2 matrix, and the directions it only stretches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Eigen {
    /// Two distinct real eigenvalues, largest first, with unit eigenvectors.
    Real {
        values: [f32; 2],
        vectors: [Vec2; 2],
    },
    /// One eigenvalue twice. `vector` is its only eigen direction (a shear),
    /// or `None` when every direction is one (a uniform scale).
    Repeated { value: f32, vector: Option<Vec2> },
    /// `re ± im·i`: the matrix turns every direction (no real eigenvector).
    Complex { re: f32, im: f32 },
}

/// Matrices worth starting from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Identity,
    Scale,
    Rotate,
    ShearX,
    Reflect,
    Project,
    Singular,
}

impl Preset {
    pub const ALL: [Preset; 7] = [
        Preset::Identity,
        Preset::Scale,
        Preset::Rotate,
        Preset::ShearX,
        Preset::Reflect,
        Preset::Project,
        Preset::Singular,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Preset::Identity => "Identity",
            Preset::Scale => "Scale",
            Preset::Rotate => "Rotate 30°",
            Preset::ShearX => "Shear",
            Preset::Reflect => "Reflect",
            Preset::Project => "Project",
            Preset::Singular => "Singular",
        }
    }

    pub fn matrix(self) -> Mat2 {
        match self {
            Preset::Identity => Mat2::IDENTITY,
            Preset::Scale => Mat2::new(1.5, 0.0, 0.0, 0.6),
            Preset::Rotate => Mat2::rotation(std::f32::consts::FRAC_PI_6),
            Preset::ShearX => Mat2::new(1.0, 1.0, 0.0, 1.0),
            // Mirror in the x axis.
            Preset::Reflect => Mat2::new(1.0, 0.0, 0.0, -1.0),
            // Onto the x axis: everything lands on one line.
            Preset::Project => Mat2::new(1.0, 0.0, 0.0, 0.0),
            // Columns on the same line: the plane collapses onto it.
            Preset::Singular => Mat2::new(1.0, 2.0, 0.5, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;
    use std::f32::consts::FRAC_PI_2;

    const EPS: f32 = 1e-4;

    fn close(p: Vec2, q: Vec2) -> bool {
        (p[0] - q[0]).abs() < EPS && (p[1] - q[1]).abs() < EPS
    }

    /// Same line through the origin (eigenvectors have no preferred sign).
    fn parallel(p: Vec2, q: Vec2) -> bool {
        (p[0] * q[1] - p[1] * q[0]).abs() < EPS
    }

    fn finite(e: &Eigen) -> bool {
        match *e {
            Eigen::Real { values, vectors } => values
                .iter()
                .chain(vectors.iter().flatten())
                .all(|x| x.is_finite()),
            Eigen::Repeated { value, vector } => {
                value.is_finite() && vector.iter().flatten().all(|x| x.is_finite())
            }
            Eigen::Complex { re, im } => re.is_finite() && im.is_finite(),
        }
    }

    #[test]
    fn apply_takes_the_basis_to_the_columns() {
        let m = Mat2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m.apply([1.0, 0.0]), [1.0, 3.0]);
        assert_eq!(m.apply([0.0, 1.0]), [2.0, 4.0]);
        assert_eq!(m.apply([1.0, 1.0]), [3.0, 7.0]);
    }

    #[test]
    fn determinant_is_ad_minus_bc() {
        assert_eq!(Mat2::new(1.0, 2.0, 3.0, 4.0).det(), -2.0);
        assert_eq!(Preset::Reflect.matrix().det(), -1.0);
        assert!(Preset::Singular.matrix().det().abs() < EPS);
    }

    #[test]
    fn rank_counts_independent_columns() {
        assert_eq!(Mat2::IDENTITY.rank(), 2);
        assert_eq!(Preset::ShearX.matrix().rank(), 2);
        assert_eq!(Preset::Singular.matrix().rank(), 1);
        assert_eq!(Preset::Project.matrix().rank(), 1);
        assert_eq!(
            Mat2::new(1.0, 2.0, 0.5, 1.000_001).rank(),
            1,
            "nearly singular"
        );
        assert_eq!(Mat2::new(0.0, 0.0, 0.0, 0.0).rank(), 0);
    }

    #[test]
    fn mul_applies_the_right_matrix_first() {
        let a = Preset::Rotate.matrix();
        let b = Preset::ShearX.matrix();
        let v = [0.3, -1.2];
        assert!(close((a * b).apply(v), a.apply(b.apply(v))));
        assert!(!close((a * b).apply(v), (b * a).apply(v)));
    }

    #[test]
    fn lerp_runs_from_identity_to_the_matrix() {
        let m = Mat2::new(2.0, 1.0, -1.0, 3.0);
        assert_eq!(m.lerp(0.0), Mat2::IDENTITY);
        assert_eq!(m.lerp(1.0), m);
        assert_eq!(m.lerp(0.5), Mat2::new(1.5, 0.5, -0.5, 2.0));
    }

    #[test]
    fn product_applies_the_first_stage_first() {
        let (a, b, c) = (
            Preset::Scale.matrix(),
            Preset::Rotate.matrix(),
            Preset::ShearX.matrix(),
        );
        let v = [0.7, -0.4];
        assert!(close(
            product(&[a, b, c]).apply(v),
            c.apply(b.apply(a.apply(v)))
        ));
        assert_eq!(product(&[]), Mat2::IDENTITY);
        assert_eq!(product(&[a]), a);
    }

    #[test]
    fn composition_is_associative() {
        let mut rng = Rng::new(3);
        let mut random = || {
            Mat2::new(
                rng.range(-2.0, 2.0),
                rng.range(-2.0, 2.0),
                rng.range(-2.0, 2.0),
                rng.range(-2.0, 2.0),
            )
        };
        for _ in 0..100 {
            let (a, b, c) = (random(), random(), random());
            let (l, r) = ((a * b) * c, a * (b * c));
            for (x, y) in [(l.a, r.a), (l.b, r.b), (l.c, r.c), (l.d, r.d)] {
                assert!((x - y).abs() < 1e-3, "{l:?} vs {r:?}");
            }
        }
    }

    #[test]
    fn stages_animate_one_after_another() {
        let chain = [Preset::Scale.matrix(), Preset::Rotate.matrix()];
        assert_eq!(staged(&chain, 0.0), Mat2::IDENTITY);
        assert_eq!(staged(&chain, 1.0), chain[0]);
        let half = staged(&chain, 1.5);
        let want = chain[1].lerp(smoothstep(0.5)) * chain[0];
        assert!(close(half.apply([1.0, 0.0]), want.apply([1.0, 0.0])));
        assert!(close(half.apply([0.0, 1.0]), want.apply([0.0, 1.0])));
        assert_eq!(staged(&chain, 2.0), product(&chain));
        assert_eq!(staged(&chain, 9.0), product(&chain), "past the end");
    }

    #[test]
    fn diagonal_matrix_has_the_axes_as_eigenvectors() {
        let Eigen::Real { values, vectors } = Mat2::new(2.0, 0.0, 0.0, 3.0).eigen() else {
            panic!("expected real eigenvalues");
        };
        assert_eq!(values, [3.0, 2.0], "largest first");
        assert!(parallel(vectors[0], [0.0, 1.0]));
        assert!(parallel(vectors[1], [1.0, 0.0]));
    }

    #[test]
    fn quarter_turn_has_complex_eigenvalues() {
        let r = Mat2::rotation(FRAC_PI_2);
        let Eigen::Complex { re, im } = r.eigen() else {
            panic!("expected complex eigenvalues");
        };
        assert!(re.abs() < EPS && (im - 1.0).abs() < EPS);
    }

    #[test]
    fn shear_has_one_repeated_eigenvalue_and_one_direction() {
        let Eigen::Repeated { value, vector } = Preset::ShearX.matrix().eigen() else {
            panic!("expected a repeated eigenvalue");
        };
        assert!((value - 1.0).abs() < EPS);
        assert!(parallel(vector.expect("one eigen direction"), [1.0, 0.0]));
    }

    #[test]
    fn scaling_every_direction_equally_has_no_single_eigenvector() {
        assert_eq!(
            Mat2::IDENTITY.eigen(),
            Eigen::Repeated {
                value: 1.0,
                vector: None
            }
        );
        assert_eq!(
            Mat2::new(0.0, 0.0, 0.0, 0.0).eigen(),
            Eigen::Repeated {
                value: 0.0,
                vector: None
            }
        );
    }

    /// Singular and nearly singular matrices collapse the plane; nothing
    /// about them may come out as NaN.
    #[test]
    fn singular_matrices_give_finite_eigen_data() {
        for m in [
            Preset::Singular.matrix(),
            Preset::Project.matrix(),
            Mat2::new(1.0, 2.0, 0.5, 1.000_001),
            Mat2::new(1e-8, 0.0, 0.0, 1e-8),
            Mat2::new(0.0, 1.0, 0.0, 0.0),
        ] {
            let e = m.eigen();
            assert!(finite(&e), "{m:?} → {e:?}");
        }
        let Eigen::Real { values, .. } = Preset::Singular.matrix().eigen() else {
            panic!("singular preset has real eigenvalues");
        };
        assert!(values[1].abs() < EPS, "one eigenvalue is zero");
    }

    #[test]
    fn eigenpairs_satisfy_av_equals_lambda_v() {
        let mut rng = Rng::new(7);
        for _ in 0..200 {
            let m = Mat2::new(
                rng.range(-3.0, 3.0),
                rng.range(-3.0, 3.0),
                rng.range(-3.0, 3.0),
                rng.range(-3.0, 3.0),
            );
            let e = m.eigen();
            assert!(finite(&e), "{m:?}");
            let check = |l: f32, v: Vec2| {
                let av = m.apply(v);
                let lv = [l * v[0], l * v[1]];
                assert!(
                    (av[0] - lv[0]).abs() < 1e-3 && (av[1] - lv[1]).abs() < 1e-3,
                    "{m:?}: A{v:?} = {av:?}, λv = {lv:?}"
                );
                assert!(
                    ((v[0] * v[0] + v[1] * v[1]).sqrt() - 1.0).abs() < 1e-4,
                    "unit length"
                );
            };
            match e {
                Eigen::Real { values, vectors } => {
                    check(values[0], vectors[0]);
                    check(values[1], vectors[1]);
                }
                Eigen::Repeated { value, vector } => {
                    if let Some(v) = vector {
                        check(value, v);
                    }
                }
                Eigen::Complex { re, im } => {
                    assert!(im > 0.0);
                    assert!((re - (m.a + m.d) / 2.0).abs() < 1e-3);
                    assert!((re * re + im * im - m.det()).abs() < 1e-2);
                }
            }
        }
    }
}
