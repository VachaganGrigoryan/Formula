//! Perlin noise flow field: theta(x, y) = N(x * s, y * s, t) * 2 pi * k,
//! p' = p + v * dt * (cos theta, sin theta).
//!
//! Improved Perlin noise (Ken Perlin, 2002) turns every point into an angle; particles
//! follow the angle under them and paint trails, colored by heading, into an image that
//! fades toward the background each frame. Hover to read the field; click for new noise.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use std::f32::consts::TAU;

use macroquad::color::hsl_to_rgb;
use macroquad::prelude::*;

const W: usize = 800; // trail image size in pixels
const H: usize = 600;
const COUNT: usize = 1500; // particles
const ZOOM: f32 = 0.005; // s in the formula
const TURNS: f32 = 1.5; // k in the formula
const SPEED: f32 = 70.0; // pixels per second
const FADE: f32 = 0.02; // fraction of the trail that fades per frame at 60 fps
const EVOLVE: f32 = 0.06; // noise time per second
const BG: [f32; 3] = [0.04, 0.063, 0.055];

/// Ken Perlin's improved gradient noise, in 3D so the third axis can be time.
struct Perlin {
    perm: [u8; 512],
}

impl Perlin {
    /// A shuffled 0..255, repeated to 512 entries so p[i + 1] never overflows.
    fn new() -> Self {
        let mut p: [u8; 256] = std::array::from_fn(|i| i as u8);
        for i in (1..256).rev() {
            p.swap(i, rand::gen_range(0, i + 1));
        }
        Self {
            perm: std::array::from_fn(|i| p[i & 255]),
        }
    }

    /// Smooth noise in roughly -1..1; exactly 0 on integer lattice points.
    fn noise(&self, x: f32, y: f32, z: f32) -> f32 {
        let (xf, yf, zf) = (x.floor(), y.floor(), z.floor());
        let cell = |f: f32| (f as i32 & 255) as usize;
        let (xi, yi, zi) = (cell(xf), cell(yf), cell(zf));
        let (x, y, z) = (x - xf, y - yf, z - zf);
        let (u, v, w) = (fade(x), fade(y), fade(z));
        let p = |i: usize| self.perm[i] as usize;
        let (a, b) = (p(xi) + yi, p(xi + 1) + yi); // hash the 8 cell corners
        let (aa, ab, ba, bb) = (p(a) + zi, p(a + 1) + zi, p(b) + zi, p(b + 1) + zi);
        // Trilinear blend of the gradient contributions of the 8 corners.
        let g = |h: usize, dx: f32, dy: f32, dz: f32| grad(self.perm[h], x - dx, y - dy, z - dz);
        let y0z0 = lerp(u, g(aa, 0., 0., 0.), g(ba, 1., 0., 0.));
        let y1z0 = lerp(u, g(ab, 0., 1., 0.), g(bb, 1., 1., 0.));
        let y0z1 = lerp(u, g(aa + 1, 0., 0., 1.), g(ba + 1, 1., 0., 1.));
        let y1z1 = lerp(u, g(ab + 1, 0., 1., 1.), g(bb + 1, 1., 1., 1.));
        lerp(w, lerp(v, y0z0, y1z0), lerp(v, y0z1, y1z1))
    }
}

/// 6t^5 - 15t^4 + 10t^3: blends each cell smoothly, so no seams show.
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

/// Dot product with one of 12 gradients pointing at the edges of a cube.
fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

/// N rescaled to 0..1, and theta = N * 2 pi * k.
fn field(noise: &Perlin, p: Vec2, t: f32) -> (f32, f32) {
    let n = (noise.noise(p.x * ZOOM, p.y * ZOOM, t) * 0.5 + 0.5).clamp(0.0, 1.0);
    (n, n * TAU * TURNS)
}

/// Floating-point trail image: fading in f32 avoids the 8-bit rounding that leaves ghosts.
struct Trails {
    rgb: Vec<[f32; 3]>,
}

impl Trails {
    fn fade(&mut self, amount: f32) {
        for px in &mut self.rgb {
            for c in 0..3 {
                px[c] += (BG[c] - px[c]) * amount;
            }
        }
    }

    /// A one-pixel line from `a` to `b`, blended toward `color`.
    fn line(&mut self, a: Vec2, b: Vec2, color: Color, alpha: f32) {
        let steps = (b - a).abs().max_element().ceil().max(1.0) as usize;
        for k in 1..=steps {
            let p = a + (b - a) * (k as f32 / steps as f32);
            if p.x >= 0.0 && p.y >= 0.0 && (p.x as usize) < W && (p.y as usize) < H {
                let px = &mut self.rgb[p.y as usize * W + p.x as usize];
                for (c, target) in [color.r, color.g, color.b].into_iter().enumerate() {
                    px[c] += (target - px[c]) * alpha;
                }
            }
        }
    }

    fn write_to(&self, image: &mut Image) {
        for (out, px) in image.get_image_data_mut().iter_mut().zip(&self.rgb) {
            let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
            *out = [byte(px[0]), byte(px[1]), byte(px[2]), 255];
        }
    }
}

fn random_particle() -> (Vec2, f32) {
    (
        vec2(
            rand::gen_range(0.0, W as f32),
            rand::gen_range(0.0, H as f32),
        ),
        rand::gen_range(2.0, 7.0),
    )
}

#[macroquad::main("Perlin noise flow field")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut noise = Perlin::new();
    let mut particles: Vec<(Vec2, f32)> = (0..COUNT).map(|_| random_particle()).collect(); // (position, life)
    let mut trails = Trails {
        rgb: vec![BG; W * H],
    };
    let mut image = Image::gen_image_color(W as u16, H as u16, BLACK);
    let texture = Texture2D::from_image(&image);
    let mut time = 0.0;

    loop {
        let dt = get_frame_time().min(0.05);
        let to_canvas = vec2(W as f32 / screen_width(), H as f32 / screen_height());
        let mouse = Vec2::from(mouse_position()) * to_canvas;
        if is_mouse_button_pressed(MouseButton::Left) {
            noise = Perlin::new();
            trails.rgb.fill(BG);
        }

        trails.fade(1.0 - (1.0 - FADE).powf(dt * 60.0));
        for (pos, life) in &mut particles {
            let (_, theta) = field(&noise, *pos, time);
            let next = *pos + vec2(theta.cos(), theta.sin()) * SPEED * dt;
            let color = hsl_to_rgb((theta / TAU).rem_euclid(1.0), 0.65, 0.62);
            trails.line(*pos, next, color, 0.35);
            (*pos, *life) = (next, *life - dt);
            let outside = !(0.0..W as f32).contains(&pos.x) || !(0.0..H as f32).contains(&pos.y);
            if outside || *life <= 0.0 {
                (*pos, *life) = random_particle();
            }
        }
        time += EVOLVE * dt;
        trails.write_to(&mut image);
        texture.update(&image);

        clear_background(BLACK);
        let size = vec2(screen_width(), screen_height());
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );

        // Hover: the field's arrow and value under the cursor.
        let (n, theta) = field(&noise, mouse, time);
        let at = Vec2::from(mouse_position());
        let d = vec2(theta.cos(), theta.sin()) * 20.0;
        let side = vec2(-d.y, d.x) * 0.25;
        draw_circle(at.x, at.y, 26.0, Color::new(0.04, 0.06, 0.05, 0.75));
        draw_line(at.x - d.x, at.y - d.y, at.x + d.x, at.y + d.y, 2.4, WHITE);
        draw_triangle(
            at + d * 1.2,
            at + d * 0.6 + side,
            at + d * 0.6 - side,
            WHITE,
        );
        let text = format!(
            "N = {n:.3}   theta = N * 2pi * {TURNS} = {:.0} deg",
            theta.to_degrees() % 360.0
        );
        let (bx, by) = (
            (at.x - 175.0).clamp(6.0, size.x - 356.0),
            at.y + if at.y < size.y - 80.0 { 34.0 } else { -60.0 },
        );
        draw_rectangle(bx, by, 350.0, 26.0, Color::new(0.04, 0.06, 0.05, 0.75));
        draw_text(&text, bx + 8.0, by + 18.0, 18.0, WHITE);
        next_frame().await;
    }
}
