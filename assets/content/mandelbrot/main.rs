//! Mandelbrot set: z_{n+1} = z_n^2 + c, z_0 = 0; c is in the set while |z_n| <= 2.
//!
//! Every pixel is a value of c, colored by its smooth escape count
//! mu = n + 1 - log2(log2 |z_n|); black points never escape. Hover to see the orbit of
//! the point under the cursor; click to zoom in 2x, right-click to zoom out.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const BAILOUT: f64 = 256.0; // |z|^2 escape test; large so the smooth count is smooth
const HOME_SPAN: f64 = 2.6;

/// Main cardioid or period-2 disc: these never escape, so skip iterating them.
fn in_main_bulbs(x: f64, y: f64) -> bool {
    let q = (x - 0.25) * (x - 0.25) + y * y;
    q * (q + (x - 0.25)) <= 0.25 * y * y || (x + 1.0) * (x + 1.0) + y * y <= 0.0625
}

/// Smooth escape count of c = cx + i cy, or None if z stays bounded for max_iter steps.
fn escape(cx: f64, cy: f64, max_iter: u32) -> Option<f64> {
    if in_main_bulbs(cx, cy) {
        return None;
    }
    let (mut x, mut y) = (0.0f64, 0.0f64);
    for n in 0..max_iter {
        let (x2, y2) = (x * x, y * y);
        if x2 + y2 > BAILOUT {
            return Some(n as f64 + 1.0 - ((x2 + y2).log2() / 2.0).log2());
        }
        y = 2.0 * x * y + cy;
        x = x2 - y2 + cx;
    }
    None
}

/// The first points of the orbit of c, stopping once it clearly escapes.
fn orbit(cx: f64, cy: f64, steps: usize) -> Vec<(f64, f64)> {
    let mut out = vec![(0.0, 0.0)];
    let (mut x, mut y) = (0.0f64, 0.0f64);
    for _ in 0..steps {
        (x, y) = (x * x - y * y + cx, 2.0 * x * y + cy);
        out.push((x, y));
        if x * x + y * y > 16.0 {
            break;
        }
    }
    out
}

/// A repeating color ramp, indexed by the square root of the escape count.
fn color(mu: f64) -> Color {
    const STOPS: [[f32; 3]; 5] = [
        [0.05, 0.07, 0.12],
        [0.16, 0.45, 0.62],
        [0.95, 0.8, 0.45],
        [0.85, 0.33, 0.25],
        [0.05, 0.07, 0.12],
    ];
    let t = (((mu - 1.0).max(0.0).sqrt() * 0.06).fract() * 4.0) as f32;
    let (k, f) = ((t as usize).min(3), t.fract());
    let (a, b) = (STOPS[k], STOPS[k + 1]);
    Color::new(
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
        1.0,
    )
}

struct View {
    cx: f64,
    cy: f64,
    /// Height of the view in the complex plane.
    span: f64,
}

impl View {
    fn pixel(&self) -> f64 {
        self.span / screen_height() as f64
    }

    fn to_complex(&self, p: Vec2) -> (f64, f64) {
        let px = self.pixel();
        let (dx, dy) = (
            (p.x - screen_width() / 2.0) as f64,
            (p.y - screen_height() / 2.0) as f64,
        );
        (self.cx + dx * px, self.cy - dy * px)
    }

    fn to_screen(&self, (x, y): (f64, f64)) -> Vec2 {
        let px = self.pixel();
        vec2(
            screen_width() / 2.0 + ((x - self.cx) / px) as f32,
            screen_height() / 2.0 - ((y - self.cy) / px) as f32,
        )
    }

    /// Deeper zooms need more iterations to separate the set from its surroundings.
    fn max_iter(&self) -> u32 {
        300 + (60.0 * (HOME_SPAN / self.span).log2().max(0.0)) as u32
    }

    /// Zoom by `factor` (> 1 zooms in) keeping the point under the cursor fixed.
    fn zoom_at(&mut self, at: Vec2, factor: f64) {
        let (x, y) = self.to_complex(at);
        self.cx = x + (self.cx - x) / factor;
        self.cy = y + (self.cy - y) / factor;
        self.span /= factor;
    }

    fn render(&self, w: u16, h: u16) -> Image {
        let mut image = Image::gen_image_color(w, h, BLACK);
        let iter = self.max_iter();
        for j in 0..h as u32 {
            for i in 0..w as u32 {
                let (cx, cy) = self.to_complex(vec2(i as f32 + 0.5, j as f32 + 0.5));
                if let Some(mu) = escape(cx, cy, iter) {
                    image.set_pixel(i, j, color(mu));
                }
            }
        }
        image
    }
}

#[macroquad::main("Mandelbrot set")]
async fn main() {
    let mut view = View {
        cx: -0.6,
        cy: 0.0,
        span: HOME_SPAN,
    };
    let mut texture: Option<Texture2D> = None;
    let mut size = (0, 0);

    loop {
        let mouse = Vec2::from(mouse_position());
        if is_mouse_button_pressed(MouseButton::Left) {
            view.zoom_at(mouse, 2.0);
            texture = None;
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            view.zoom_at(mouse, 0.5);
            texture = None;
        }
        let now = (screen_width() as u16, screen_height() as u16);
        if texture.is_none() || now != size {
            size = now;
            texture = Some(Texture2D::from_image(&view.render(size.0, size.1)));
        }
        clear_background(BLACK);
        if let Some(t) = &texture {
            draw_texture(t, 0.0, 0.0, WHITE);
        }

        // The orbit of the point under the cursor: inside it settles, outside it flies off.
        let (cx, cy) = view.to_complex(mouse);
        let pts: Vec<Vec2> = orbit(cx, cy, 60)
            .into_iter()
            .map(|z| view.to_screen(z))
            .collect();
        for w in pts.windows(2) {
            draw_line(
                w[0].x,
                w[0].y,
                w[1].x,
                w[1].y,
                1.2,
                Color::new(1.0, 1.0, 1.0, 0.8),
            );
        }
        for p in &pts {
            draw_circle(p.x, p.y, 2.2, WHITE);
        }
        draw_circle_lines(mouse.x, mouse.y, 6.0, 2.0, GOLD);
        let fate = match escape(cx, cy, view.max_iter()) {
            Some(mu) => format!("escapes after {:.0} iterations", mu.max(1.0)),
            None => format!("bounded for {} iterations", view.max_iter()),
        };
        let sign = if cy < 0.0 { "-" } else { "+" };
        draw_rectangle(10.0, 10.0, 350.0, 50.0, Color::new(0.0, 0.0, 0.0, 0.65));
        draw_text(
            &format!("c = {cx:.6} {sign} {:.6}i", cy.abs()),
            18.0,
            30.0,
            20.0,
            WHITE,
        );
        draw_text(&fate, 18.0, 50.0, 20.0, WHITE);
        let zoom = format!(
            "zoom x{:.0}   click: in   right-click: out",
            HOME_SPAN / view.span
        );
        draw_rectangle(
            screen_width() - 390.0,
            screen_height() - 36.0,
            380.0,
            26.0,
            Color::new(0.0, 0.0, 0.0, 0.65),
        );
        draw_text(
            &zoom,
            screen_width() - 382.0,
            screen_height() - 17.0,
            20.0,
            GOLD,
        );
        next_frame().await;
    }
}
