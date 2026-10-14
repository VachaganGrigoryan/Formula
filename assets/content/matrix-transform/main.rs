//! Matrix transformation: x' = a x + b y,  y' = c x + d y.
//!
//! A 2x2 matrix moves every point of the plane. Its columns are where the basis
//! vectors i = (1, 0) and j = (0, 1) land: drag the tips of the two arrows and the
//! whole grid follows. The shaded cell is the image of the unit square; its area is
//! |det| and it turns red when det < 0 (the plane is mirrored). Dashed lines are the
//! eigenvectors, the directions the matrix only stretches: A v = lambda v.
//! Keys 1-7 load a preset and play the morph from the identity; space replays it.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

/// [[a, b], [c, d]] acting on column vectors.
#[derive(Clone, Copy)]
struct Mat2 {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

impl Mat2 {
    const IDENTITY: Mat2 = Mat2 { a: 1.0, b: 0.0, c: 0.0, d: 1.0 };

    fn apply(self, p: Vec2) -> Vec2 {
        vec2(self.a * p.x + self.b * p.y, self.c * p.x + self.d * p.y)
    }

    fn det(self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    /// Where i (k = 0) or j (k = 1) lands.
    fn column(self, k: usize) -> Vec2 {
        if k == 0 { vec2(self.a, self.c) } else { vec2(self.b, self.d) }
    }

    fn scale(self) -> f32 {
        [self.a, self.b, self.c, self.d].iter().fold(1.0_f32, |s, x| s.max(x.abs()))
    }
}

enum Eigen {
    /// Real eigenvalues, largest first, with unit eigenvectors.
    Real(Vec<(f32, Vec2)>),
    /// A uniform scale: every direction is an eigenvector.
    All(f32),
    Complex(f32, f32),
}

fn eigen(m: Mat2) -> Eigen {
    let h = (m.a + m.d) / 2.0;
    let disc = h * h - m.det();
    let eps = 1e-6 * m.scale() * m.scale();
    if disc < -eps {
        return Eigen::Complex(h, (-disc).sqrt());
    }
    let lambdas = if disc <= eps { vec![h] } else { vec![h + disc.sqrt(), h - disc.sqrt()] };
    let mut pairs = Vec::new();
    for lambda in lambdas {
        // An eigenvector is perpendicular to a nonzero row of A - lambda I.
        let u = vec2(m.b, lambda - m.a);
        let w = vec2(lambda - m.d, m.c);
        let v = if u.length() >= w.length() { u } else { w };
        if v.length() > 1e-6 {
            pairs.push((lambda, v.normalize()));
        }
    }
    if pairs.is_empty() { Eigen::All(h) } else { Eigen::Real(pairs) }
}

const C30: f32 = 0.866_025_4;
const PRESETS: [Mat2; 7] = [
    Mat2::IDENTITY,
    Mat2 { a: 1.5, b: 0.0, c: 0.0, d: 0.6 },   // scale
    Mat2 { a: C30, b: -0.5, c: 0.5, d: C30 },  // rotate 30 degrees
    Mat2 { a: 1.0, b: 1.0, c: 0.0, d: 1.0 },   // shear
    Mat2 { a: 1.0, b: 0.0, c: 0.0, d: -1.0 },  // reflect in the x axis
    Mat2 { a: 1.0, b: 0.0, c: 0.0, d: 0.0 },   // project onto the x axis
    Mat2 { a: 1.0, b: 2.0, c: 0.5, d: 1.0 },   // singular: both columns on one line
];
const KEYS: [KeyCode; 7] = [
    KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
    KeyCode::Key5, KeyCode::Key6, KeyCode::Key7,
];
const HOUSE: [(f32, f32); 7] =
    [(0.2, 0.0), (0.8, 0.0), (0.8, 0.6), (1.0, 0.6), (0.5, 1.05), (0.0, 0.6), (0.2, 0.6)];
const SQUARE: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
const SIZE: f32 = 700.0;
const UNIT: f32 = 100.0; // pixels per unit
const GRID: i32 = 8; // grid lines on each side of the origin
const LIMIT: f32 = 5.0; // matrix entries stay within +-LIMIT
const SECONDS: f32 = 1.6; // length of the morph from the identity
const ORANGE: Color = Color::new(0.94, 0.63, 0.25, 1.0);
const GREEN: Color = Color::new(0.30, 0.76, 0.54, 1.0);
const BLUE: Color = Color::new(0.42, 0.56, 0.85, 1.0);
const RED: Color = Color::new(0.90, 0.44, 0.49, 1.0);

fn to_screen(p: Vec2) -> Vec2 {
    vec2(SIZE / 2.0 + p.x * UNIT, SIZE / 2.0 - p.y * UNIT)
}

fn to_world(s: Vec2) -> Vec2 {
    vec2((s.x - SIZE / 2.0) / UNIT, (SIZE / 2.0 - s.y) / UNIT)
}

fn line(p: Vec2, q: Vec2, width: f32, color: Color) {
    let (a, b) = (to_screen(p), to_screen(q));
    draw_line(a.x, a.y, b.x, b.y, width, color);
}

fn dashed(p: Vec2, q: Vec2, color: Color) {
    let n = 60;
    for k in (0..n).step_by(2) {
        line(p.lerp(q, k as f32 / n as f32), p.lerp(q, (k + 1) as f32 / n as f32), 1.5, color);
    }
}

fn outline(points: &[Vec2], width: f32, color: Color) {
    for k in 0..points.len() {
        line(points[k], points[(k + 1) % points.len()], width, color);
    }
}

/// The matrix on screen: part way from the identity while the morph plays.
fn shown(target: Mat2, t: f32) -> Mat2 {
    let s = t.min(1.0);
    let e = s * s * (3.0 - 2.0 * s);
    Mat2 {
        a: 1.0 + e * (target.a - 1.0),
        b: e * target.b,
        c: e * target.c,
        d: 1.0 + e * (target.d - 1.0),
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Matrix transformation".to_owned(),
        window_width: SIZE as i32,
        window_height: SIZE as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut target = Mat2 { a: 1.5, b: 0.5, c: 0.25, d: 1.0 };
    let mut t = 0.0_f32;
    let mut dragging: Option<usize> = None;
    loop {
        t += get_frame_time() / SECONDS;
        let mouse = Vec2::from(mouse_position());
        let m = shown(target, t);
        if is_mouse_button_pressed(MouseButton::Left) {
            for k in 0..2 {
                if to_screen(m.column(k)).distance(mouse) < 16.0 {
                    target = m;
                    t = 1.0;
                    dragging = Some(k);
                }
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            dragging = None;
        }
        if let Some(k) = dragging {
            let p = to_world(mouse);
            let snap = |x: f32| ((x * 2.0).round() / 2.0).clamp(-LIMIT, LIMIT);
            if k == 0 {
                target.a = snap(p.x);
                target.c = snap(p.y);
            } else {
                target.b = snap(p.x);
                target.d = snap(p.y);
            }
        }
        for (key, preset) in KEYS.iter().zip(PRESETS) {
            if is_key_pressed(*key) {
                target = preset;
                t = 0.0;
            }
        }
        if is_key_pressed(KeyCode::Space) {
            t = 0.0;
        }

        let m = shown(target, t);
        clear_background(Color::from_hex(0x16201c));
        let g = GRID as f32;
        for k in -GRID..=GRID {
            let k = k as f32;
            let faint = Color::new(1.0, 1.0, 1.0, 0.08);
            line(vec2(k, -g), vec2(k, g), 1.0, faint);
            line(vec2(-g, k), vec2(g, k), 1.0, faint);
        }
        for k in -GRID..=GRID {
            let color = if k == 0 { Color::new(1.0, 1.0, 1.0, 0.55) } else { Color::new(0.42, 0.56, 0.85, 0.35) };
            let k = k as f32;
            line(m.apply(vec2(k, -g)), m.apply(vec2(k, g)), 1.0, color);
            line(m.apply(vec2(-g, k)), m.apply(vec2(g, k)), 1.0, color);
        }
        let det = m.det();
        let sq: Vec<Vec2> = SQUARE.iter().map(|&(x, y)| to_screen(m.apply(vec2(x, y)))).collect();
        let fill = Color { a: 0.33, ..if det < 0.0 { RED } else { BLUE } };
        draw_triangle(sq[0], sq[1], sq[2], fill);
        draw_triangle(sq[0], sq[2], sq[3], fill);
        let house: Vec<Vec2> = HOUSE.iter().map(|&(x, y)| vec2(x, y)).collect();
        outline(&house, 1.0, Color::new(1.0, 1.0, 1.0, 0.3));
        let moved: Vec<Vec2> = house.iter().map(|&p| m.apply(p)).collect();
        outline(&moved, 2.0, Color::from_hex(0xa8c4ff));

        let text = match eigen(m) {
            Eigen::Real(pairs) => {
                for &(_, v) in &pairs {
                    dashed(-9.0 * v, 9.0 * v, Color::new(1.0, 1.0, 1.0, 0.65));
                }
                let values: Vec<String> = pairs.iter().map(|(l, _)| format!("{l:.2}")).collect();
                format!("lambda = {}", values.join(", "))
            }
            Eigen::All(l) => format!("lambda = {l:.2}: every direction is an eigenvector"),
            Eigen::Complex(re, im) => format!("lambda = {re:.2} +- {im:.2}i: no real eigenvectors"),
        };
        for (k, color) in [(0, ORANGE), (1, GREEN)] {
            line(Vec2::ZERO, m.column(k), 3.0, color);
            let tip = to_screen(m.column(k));
            draw_circle(tip.x, tip.y, 7.0, color);
        }

        let area = if det.abs() < 1e-4 * m.scale() * m.scale() {
            "det = 0: the plane collapses".to_owned()
        } else {
            format!("det = {det:.2}{}", if det < 0.0 { " (mirrored)" } else { "" })
        };
        let rows = [
            format!("[{:6.2} {:6.2} ]", m.a, m.b),
            format!("[{:6.2} {:6.2} ]", m.c, m.d),
            area,
            text,
            "drag the tips | keys 1-7 presets | space replays".to_owned(),
        ];
        for (k, row) in rows.iter().enumerate() {
            draw_text(row, 12.0, 24.0 + 18.0 * k as f32, 20.0, WHITE);
        }
        next_frame().await;
    }
}
