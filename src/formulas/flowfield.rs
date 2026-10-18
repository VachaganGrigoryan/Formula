//! A flow field: Perlin noise turned into angles, and particles that follow them.

use std::f32::consts::TAU;

use egui::{
    Align2, Color32, ColorImage, FontId, Pos2, Rect, Sense, Stroke, TextureHandle, TextureOptions,
    pos2, vec2,
};

use crate::formula::{Formula, MathText, Palette, Tone};
use crate::formulas::luminance::hsv_to_srgb;
use crate::rng::Rng;

/// Ken Perlin's improved gradient noise (2002), in three dimensions so the
/// third one can be time.
pub struct Perlin {
    perm: [u8; 512],
}

impl Perlin {
    pub fn new(seed: u64) -> Self {
        let mut p: [u8; 256] = std::array::from_fn(|i| i as u8);
        let mut rng = Rng::new(seed);
        for i in (1..256).rev() {
            p.swap(i, rng.index(i + 1));
        }
        Self {
            perm: std::array::from_fn(|i| p[i & 255]),
        }
    }

    /// Smooth noise in roughly `[-1, 1]`; exactly 0 on integer lattice points.
    pub fn noise(&self, x: f32, y: f32, z: f32) -> f32 {
        let (xf, yf, zf) = (x.floor(), y.floor(), z.floor());
        let (xi, yi, zi) = (
            (xf as i32 & 255) as usize,
            (yf as i32 & 255) as usize,
            (zf as i32 & 255) as usize,
        );
        let (x, y, z) = (x - xf, y - yf, z - zf);
        let (u, v, w) = (fade(x), fade(y), fade(z));
        let p = &self.perm;
        let a = p[xi] as usize + yi;
        let (aa, ab) = (p[a] as usize + zi, p[a + 1] as usize + zi);
        let b = p[xi + 1] as usize + yi;
        let (ba, bb) = (p[b] as usize + zi, p[b + 1] as usize + zi);
        lerp(
            w,
            lerp(
                v,
                lerp(u, grad(p[aa], x, y, z), grad(p[ba], x - 1.0, y, z)),
                lerp(
                    u,
                    grad(p[ab], x, y - 1.0, z),
                    grad(p[bb], x - 1.0, y - 1.0, z),
                ),
            ),
            lerp(
                v,
                lerp(
                    u,
                    grad(p[aa + 1], x, y, z - 1.0),
                    grad(p[ba + 1], x - 1.0, y, z - 1.0),
                ),
                lerp(
                    u,
                    grad(p[ab + 1], x, y - 1.0, z - 1.0),
                    grad(p[bb + 1], x - 1.0, y - 1.0, z - 1.0),
                ),
            ),
        )
    }

    /// The noise rescaled to `[0, 1]`: N(x, y, t) in the equations.
    pub fn unit(&self, x: f32, y: f32, z: f32) -> f32 {
        (self.noise(x, y, z) * 0.5 + 0.5).clamp(0.0, 1.0)
    }
}

/// 6t⁵ − 15t⁴ + 10t³: eases each cell's blend so the noise has no visible seams.
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

/// Dot product with one of twelve edge-pointing gradients.
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

/// θ = N · 2π · k
pub fn angle(n: f32, turns: f32) -> f32 {
    n * TAU * turns
}

/// An RGB image the particles draw into. Each frame it fades a little toward
/// the background, which leaves the trails.
struct Canvas {
    w: usize,
    h: usize,
    rgb: Vec<[f32; 3]>,
    bg: [f32; 3],
}

impl Canvas {
    fn new(w: usize, h: usize, bg: [f32; 3]) -> Self {
        Self {
            w,
            h,
            rgb: vec![bg; w * h],
            bg,
        }
    }

    fn fade(&mut self, amount: f32) {
        let bg = self.bg;
        for px in &mut self.rgb {
            for c in 0..3 {
                px[c] += (bg[c] - px[c]) * amount;
            }
        }
    }

    fn blend(&mut self, x: i32, y: i32, color: [f32; 3], alpha: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let px = &mut self.rgb[y as usize * self.w + x as usize];
        for c in 0..3 {
            px[c] += (color[c] - px[c]) * alpha;
        }
    }

    /// A one-pixel line from `a` to `b`, not including `a` itself, so joined
    /// segments don't darken their shared end.
    fn line(&mut self, a: Pos2, b: Pos2, color: [f32; 3], alpha: f32) {
        let d = b - a;
        let steps = d.x.abs().max(d.y.abs()).ceil().max(1.0) as usize;
        for k in 1..=steps {
            let p = a + d * (k as f32 / steps as f32);
            self.blend(p.x.floor() as i32, p.y.floor() as i32, color, alpha);
        }
    }

    fn image(&self) -> ColorImage {
        let pixels = self
            .rgb
            .iter()
            .map(|c| {
                let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                Color32::from_rgb(b(c[0]), b(c[1]), b(c[2]))
            })
            .collect();
        ColorImage::new([self.w, self.h], pixels)
    }
}

fn rgb(c: Color32) -> [f32; 3] {
    [
        c.r() as f32 / 255.0,
        c.g() as f32 / 255.0,
        c.b() as f32 / 255.0,
    ]
}

#[derive(Clone, Copy)]
struct Particle {
    pos: Pos2,
    /// Seconds left before it is dropped somewhere new.
    life: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Coloring {
    Angle,
    Ink,
}

pub struct FlowField {
    noise: Perlin,
    seed: u64,
    rng: Rng,
    particles: Vec<Particle>,
    count: usize,
    /// Noise features per pixel; larger means busier.
    zoom: f32,
    /// k in θ = N · 2π · k.
    turns: f32,
    /// Pixels per second.
    speed: f32,
    /// Fraction of the trail that fades per frame at 60 fps.
    fade: f32,
    /// How fast the field itself changes (noise z per second).
    evolve: f32,
    z: f32,
    coloring: Coloring,
    show_field: bool,
    playing: bool,
    canvas: Option<Canvas>,
    texture: Option<TextureHandle>,
    hover: Option<Pos2>,
}

impl Default for FlowField {
    fn default() -> Self {
        Self {
            noise: Perlin::new(1),
            seed: 1,
            rng: Rng::new(99),
            particles: Vec::new(),
            count: 1500,
            zoom: 0.005,
            turns: 1.5,
            speed: 70.0,
            fade: 0.02,
            evolve: 0.06,
            z: 0.0,
            coloring: Coloring::Angle,
            show_field: false,
            playing: true,
            canvas: None,
            texture: None,
            hover: None,
        }
    }
}

impl FlowField {
    fn theta(&self, p: Pos2) -> (f32, f32) {
        let n = self.noise.unit(p.x * self.zoom, p.y * self.zoom, self.z);
        (n, angle(n, self.turns))
    }

    fn spawn(&mut self, w: f32, h: f32) -> Particle {
        Particle {
            pos: pos2(self.rng.range(0.0, w), self.rng.range(0.0, h)),
            life: self.rng.range(2.0, 7.0),
        }
    }

    fn color(&self, theta: f32, palette: &Palette, dark: bool) -> [f32; 3] {
        match self.coloring {
            Coloring::Angle => {
                let hue = (theta / TAU).rem_euclid(1.0);
                let [r, g, b] = hsv_to_srgb(hue, 0.65, if dark { 0.95 } else { 0.7 });
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
            }
            Coloring::Ink => rgb(palette.trace),
        }
    }

    fn simulate(&mut self, size: [usize; 2], palette: &Palette, dark: bool, dt: f32) {
        let (w, h) = (size[0] as f32, size[1] as f32);
        while self.particles.len() < self.count {
            let p = self.spawn(w, h);
            self.particles.push(p);
        }
        self.particles.truncate(self.count);
        let Some(mut canvas) = self.canvas.take() else {
            return;
        };
        canvas.fade(1.0 - (1.0 - self.fade).powf(dt * 60.0));
        let alpha = if dark { 0.35 } else { 0.28 };
        let step = self.speed * dt;
        for k in 0..self.particles.len() {
            let mut p = self.particles[k];
            let (_, theta) = self.theta(p.pos);
            let next = p.pos + vec2(theta.cos(), theta.sin()) * step;
            canvas.line(p.pos, next, self.color(theta, palette, dark), alpha);
            p.pos = next;
            p.life -= dt;
            let outside = !(0.0..w).contains(&p.pos.x) || !(0.0..h).contains(&p.pos.y);
            if outside || p.life <= 0.0 {
                p = self.spawn(w, h);
            }
            self.particles[k] = p;
        }
        self.z += self.evolve * dt;
        self.canvas = Some(canvas);
    }

    fn clear(&mut self) {
        self.canvas = None;
        self.particles.clear();
    }

    fn paint_arrow(painter: &egui::Painter, at: Pos2, theta: f32, len: f32, stroke: Stroke) {
        let d = vec2(theta.cos(), theta.sin()) * len;
        let (a, b) = (at - d * 0.5, at + d * 0.5);
        painter.line_segment([a, b], stroke);
        let back = -d.normalized() * (len * 0.3).min(7.0);
        let side = vec2(-back.y, back.x) * 0.55;
        painter.line_segment([b, b + back + side], stroke);
        painter.line_segment([b, b + back - side], stroke);
    }
}

/// What a sheet saves of a [`FlowField`].
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct State {
    /// The noise is generated from this seed.
    seed: u64,
    count: usize,
    zoom: f32,
    turns: f32,
    speed: f32,
    fade: f32,
    evolve: f32,
    coloring: Coloring,
    show_field: bool,
    playing: bool,
}

impl FlowField {
    fn state(&self) -> State {
        State {
            seed: self.seed,
            count: self.count,
            zoom: self.zoom,
            turns: self.turns,
            speed: self.speed,
            fade: self.fade,
            evolve: self.evolve,
            coloring: self.coloring,
            show_field: self.show_field,
            playing: self.playing,
        }
    }

    fn apply(&mut self, s: State) {
        if s.seed != self.seed {
            self.seed = s.seed;
            self.noise = Perlin::new(self.seed);
        }
        self.count = s.count.clamp(100, 5000);
        self.zoom = s.zoom.clamp(0.001, 0.03);
        self.turns = s.turns.clamp(0.25, 4.0);
        self.speed = s.speed.clamp(10.0, 240.0);
        self.fade = s.fade.clamp(0.002, 0.2);
        self.evolve = s.evolve.clamp(0.0, 0.5);
        self.coloring = s.coloring;
        self.show_field = s.show_field;
        self.playing = s.playing;
    }
}

impl Formula for FlowField {
    crate::state_hooks!();

    fn params(&self) -> Vec<(String, f64)> {
        vec![
            ("s".into(), self.zoom as f64),
            ("k".into(), self.turns as f64),
        ]
    }

    fn id(&self) -> &'static str {
        "flow-field"
    }

    fn name(&self) -> &'static str {
        "Perlin noise flow field"
    }

    fn category(&self) -> &'static str {
        super::FRACTALS
    }

    fn summary(&self) -> &'static str {
        "Smooth noise sampled at every point becomes an angle; particles read the angle \
         under them and move that way. The trails fade slowly, so the field draws itself \
         like wind through grass. Hover to read the field at a point."
    }

    fn notes(&self) -> &'static str {
        "Perlin noise is the workhorse of procedural graphics: clouds, terrain heights, \
         fire, water and wood grain in games and film. Flow fields built on it are a \
         staple of generative art, and the same vector-field idea visualizes wind maps, \
         ocean currents and magnetic fields."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("θ(x, y)")
                .tone(Tone::Trace)
                .t(" = N(x · s, y · s, t) · 2π · k"),
            MathText::new()
                .t("p'")
                .tone(Tone::X)
                .t(" = p + v Δt · (cos θ, sin θ)"),
            MathText::new().t("N: Perlin noise in [0, 1],  s: zoom,  k: turns"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::hover()
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * if width < 520.0 { 1.0 } else { 0.62 }
    }

    fn interact(&mut self, response: &egui::Response, _rect: Rect) {
        self.hover = response.hover_pos();
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        let dark = palette.paper.r() < 128;
        let bg = if dark {
            Color32::from_rgb(10, 16, 14)
        } else {
            palette.card
        };
        let size = [
            (rect.width().round() as usize).clamp(1, 1400),
            (rect.height().round() as usize).clamp(1, 1000),
        ];
        let stale = self
            .canvas
            .as_ref()
            .is_none_or(|c| [c.w, c.h] != size || c.bg != rgb(bg));
        if stale {
            self.canvas = Some(Canvas::new(size[0], size[1], rgb(bg)));
            self.particles.clear();
        }
        if self.playing {
            self.simulate(size, palette, dark, dt.min(0.05));
        }
        if let Some(canvas) = &self.canvas {
            let image = canvas.image();
            match &mut self.texture {
                Some(t) => t.set(image, TextureOptions::LINEAR),
                None => {
                    self.texture = Some(painter.ctx().load_texture(
                        "flow-field",
                        image,
                        TextureOptions::LINEAR,
                    ))
                }
            }
        }
        if let Some(t) = &self.texture {
            painter.image(
                t.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        let to_canvas = |p: Pos2| pos2(p.x - rect.left(), p.y - rect.top());
        if self.show_field {
            let gap = 34.0;
            let stroke = Stroke::new(1.0, palette.ink.gamma_multiply(0.45));
            let (cols, rows) = (
                (rect.width() / gap) as usize,
                (rect.height() / gap) as usize,
            );
            for j in 0..rows {
                for i in 0..cols {
                    let at = rect.left_top()
                        + vec2(
                            (i as f32 + 0.5) * rect.width() / cols as f32,
                            (j as f32 + 0.5) * rect.height() / rows as f32,
                        );
                    let (_, theta) = self.theta(to_canvas(at));
                    Self::paint_arrow(painter, at, theta, gap * 0.6, stroke);
                }
            }
        }

        if let Some(at) = self.hover.filter(|p| rect.contains(*p)) {
            let (n, theta) = self.theta(to_canvas(at));
            painter.circle_filled(at, 26.0, palette.card.gamma_multiply(0.8));
            Self::paint_arrow(painter, at, theta, 40.0, Stroke::new(2.4, palette.ink));
            let degrees = theta.to_degrees().rem_euclid(360.0);
            let text = format!(
                "N = {n:.3}\nθ = {n:.3} · 2π · {:.1} = {:.0}°",
                self.turns, degrees
            );
            let below = at.y < rect.bottom() - 70.0;
            let anchor = at + vec2(0.0, if below { 34.0 } else { -34.0 });
            let galley = painter.layout_no_wrap(text, FontId::monospace(12.0), palette.ink);
            let mut r = Align2::CENTER_TOP
                .anchor_size(anchor, galley.size())
                .translate(vec2(0.0, if below { 0.0 } else { -galley.size().y }));
            let shift =
                (rect.left() + 6.0 - r.left()).max(0.0) - (r.right() - rect.right() + 6.0).max(0.0);
            r = r.translate(vec2(shift, 0.0));
            painter.rect_filled(r.expand(6.0), 5.0, palette.card);
            painter.rect_stroke(
                r.expand(6.0),
                5.0,
                Stroke::new(1.0, palette.rule),
                egui::StrokeKind::Inside,
            );
            painter.galley(r.min, galley, palette.ink);
        }

        painter.text(
            rect.right_bottom() + vec2(-12.0, -10.0),
            Align2::RIGHT_BOTTOM,
            format!("{} particles", self.particles.len()),
            FontId::proportional(11.0),
            palette.muted,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("flow-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Particles");
                ui.add(egui::Slider::new(&mut self.count, 100..=5000).logarithmic(true));
                ui.end_row();
                ui.label("Zoom s");
                ui.add(
                    egui::Slider::new(&mut self.zoom, 0.001..=0.03)
                        .logarithmic(true)
                        .custom_formatter(|v, _| format!("{v:.4}")),
                );
                ui.end_row();
                ui.label("Turns k");
                ui.add(
                    egui::Slider::new(&mut self.turns, 0.25..=4.0)
                        .step_by(0.05)
                        .clamping(egui::SliderClamping::Edits),
                );
                ui.end_row();
                ui.label("Speed v");
                ui.add(egui::Slider::new(&mut self.speed, 10.0..=240.0).suffix(" px/s"));
                ui.end_row();
                ui.label("Trail fade");
                ui.add(
                    egui::Slider::new(&mut self.fade, 0.002..=0.2)
                        .logarithmic(true)
                        .custom_formatter(|v, _| format!("{:.1}%", v * 100.0)),
                );
                ui.end_row();
                ui.label("Field change");
                ui.add(
                    egui::Slider::new(&mut self.evolve, 0.0..=0.5)
                        .step_by(0.01)
                        .clamping(egui::SliderClamping::Edits),
                );
                ui.end_row();
                ui.label("Color");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.coloring, Coloring::Angle, "By angle");
                    ui.selectable_value(&mut self.coloring, Coloring::Ink, "Ink");
                });
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui.button("Clear").clicked() {
                self.clear();
            }
            if ui.button("New noise").clicked() {
                self.seed += 1;
                self.noise = Perlin::new(self.seed);
                self.clear();
            }
            ui.checkbox(&mut self.show_field, "Show vectors");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_on_the_lattice() {
        let n = Perlin::new(3);
        for (x, y, z) in [(0.0, 0.0, 0.0), (5.0, 2.0, 1.0), (-3.0, 7.0, 4.0)] {
            assert!(n.noise(x, y, z).abs() < 1e-6);
        }
    }

    #[test]
    fn bounded_smooth_and_seeded() {
        let (a, b, c) = (Perlin::new(1), Perlin::new(1), Perlin::new(2));
        let mut differs = false;
        for i in 0..2000 {
            let (x, y, z) = (i as f32 * 0.137, i as f32 * 0.071, i as f32 * 0.013);
            let v = a.noise(x, y, z);
            assert!(v.abs() <= 1.0);
            assert_eq!(v, b.noise(x, y, z));
            differs |= v != c.noise(x, y, z);
            // Continuous: a tiny step barely changes it.
            assert!((v - a.noise(x + 1e-3, y, z)).abs() < 0.02);
        }
        assert!(differs);
        assert!((0.0..=1.0).contains(&a.unit(0.3, 0.6, 0.9)));
    }

    #[test]
    fn angle_wraps_a_full_turn() {
        assert_eq!(angle(0.0, 1.0), 0.0);
        assert!((angle(0.5, 1.0) - std::f32::consts::PI).abs() < 1e-6);
        assert!((angle(1.0, 2.0) - 2.0 * TAU).abs() < 1e-5);
    }

    #[test]
    fn canvas_fades_to_background_and_draws_lines() {
        let bg = [0.1, 0.2, 0.3];
        let mut c = Canvas::new(20, 10, bg);
        c.line(pos2(1.5, 1.5), pos2(15.5, 1.5), [1.0, 1.0, 1.0], 1.0);
        let lit = c.rgb.iter().filter(|p| p[0] == 1.0).count();
        assert_eq!(lit, 14);
        for _ in 0..200 {
            c.fade(0.1);
        }
        assert!(c.rgb.iter().all(|p| (p[0] - bg[0]).abs() < 1e-3));
        // Off-canvas writes are ignored.
        c.line(pos2(-5.0, -5.0), pos2(30.0, 30.0), [1.0; 3], 1.0);
    }

    #[test]
    fn particles_stay_on_canvas() {
        let mut f = FlowField {
            count: 300,
            ..FlowField::default()
        };
        f.canvas = Some(Canvas::new(120, 80, [0.0; 3]));
        let palette = Palette::dark();
        for _ in 0..200 {
            f.simulate([120, 80], &palette, true, 1.0 / 30.0);
        }
        assert_eq!(f.particles.len(), 300);
        assert!(
            f.particles
                .iter()
                .all(|p| (0.0..120.0).contains(&p.pos.x) && (0.0..80.0).contains(&p.pos.y))
        );
    }
}
