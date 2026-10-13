//! Perspective projection: x' = f * x / z + cx,  y' = f * y / z + cy.
//!
//! A cube, a floor grid and a cloud of random 3D points are rotated, pushed in front of
//! the camera, and flattened onto the screen by dividing by depth z: far points shrink
//! toward the center. The inset shows the highlighted point's ray from above.
//! Drag to orbit, scroll to move the camera, Up/Down change f, D toggles a dolly zoom.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

/// Points closer than this to the eye are not drawn.
const NEAR: f32 = 0.05;

/// Rotate around the vertical axis (yaw), then the horizontal axis (pitch).
fn rotate(p: Vec3, yaw: f32, pitch: f32) -> Vec3 {
    let (sy, cy) = yaw.sin_cos();
    let (x, z) = (p.x * cy + p.z * sy, -p.x * sy + p.z * cy);
    let (sp, cp) = pitch.sin_cos();
    let (y, z) = (p.y * cp - z * sp, p.y * sp + z * cp);
    vec3(x, y, z)
}

/// Divide by depth; y is flipped so up is up on screen. None when behind the near plane.
fn project(p: Vec3, f: f32, c: Vec2) -> Option<Vec2> {
    (p.z > NEAR).then(|| vec2(f * p.x / p.z + c.x, -f * p.y / p.z + c.y))
}

fn corner(i: usize) -> Vec3 {
    let s = |bit: usize| if i & bit != 0 { 1.0 } else { -1.0 };
    vec3(s(1), s(2), s(4))
}

const CUBE_EDGES: [(usize, usize); 12] =
    [(0, 1), (1, 3), (3, 2), (2, 0), (4, 5), (5, 7), (7, 6), (6, 4), (0, 4), (1, 5), (2, 6), (3, 7)];

fn segment(a: Option<Vec2>, b: Option<Vec2>, thickness: f32, color: Color) {
    if let (Some(a), Some(b)) = (a, b) {
        draw_line(a.x, a.y, b.x, b.y, thickness, color);
    }
}

#[macroquad::main("Perspective projection")]
async fn main() {
    rand::srand(3);
    let scatter: Vec<Vec3> = (0..70)
        .map(|_| vec3(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)))
        .collect();
    let (mut focal, mut distance, mut yaw, mut pitch, mut spinning) = (1.0_f32, 4.0_f32, 0.6_f32, -0.35_f32, true);
    let mut last_mouse = Vec2::from(mouse_position());
    let highlight = Color::from_rgba(239, 107, 91, 255);

    loop {
        // Input: drag to orbit, wheel for distance, keys for f and the dolly zoom.
        let mouse = Vec2::from(mouse_position());
        if is_mouse_button_down(MouseButton::Left) {
            let d = mouse - last_mouse;
            yaw += d.x * 0.01;
            pitch = (pitch + d.y * 0.01).clamp(-1.4, 1.4);
            if d.length() > 0.0 {
                spinning = false;
            }
        }
        last_mouse = mouse;
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            distance = (distance - wheel.signum() * 0.5).clamp(2.2, 12.0);
        }
        if is_key_pressed(KeyCode::Up) {
            focal = (focal + 0.1).min(2.5);
        }
        if is_key_pressed(KeyCode::Down) {
            focal = (focal - 0.1).max(0.3);
        }
        if is_key_pressed(KeyCode::D) {
            // Dolly zoom: move the camera and scale f with it, so the cube keeps its size.
            distance = if distance < 7.0 { 10.0 } else { 3.0 };
            focal = 0.25 * distance;
        }
        if spinning {
            yaw += 0.35 * get_frame_time();
        }

        let f = focal * screen_width().min(screen_height());
        let c = vec2(screen_width() / 2.0, screen_height() / 2.0);
        let to_camera = |p: Vec3| rotate(p, yaw, pitch) + vec3(0.0, 0.0, distance);
        let screen = |p: Vec3| project(to_camera(p), f, c);
        clear_background(Color::from_rgba(22, 32, 28, 255));

        // Floor grid at y = -1 for a sense of depth, then the cube.
        let floor = Color::from_rgba(44, 58, 52, 255);
        for i in 0..=8 {
            let u = -1.6 + 3.2 * i as f32 / 8.0;
            segment(screen(vec3(u, -1.0, -1.6)), screen(vec3(u, -1.0, 1.6)), 1.0, floor);
            segment(screen(vec3(-1.6, -1.0, u)), screen(vec3(1.6, -1.0, u)), 1.0, floor);
        }
        for &(i, j) in &CUBE_EDGES {
            segment(screen(corner(i)), screen(corner(j)), 1.2, GRAY);
        }

        // Painter's algorithm: far points first so near ones cover them.
        let mut points: Vec<(usize, Vec3)> = scatter.iter().map(|&p| to_camera(p)).enumerate().collect();
        points.sort_by(|a, b| b.1.z.total_cmp(&a.1.z));
        let (near, far) = (distance - 1.8, distance + 1.8);
        for (i, q) in points {
            let Some(s) = project(q, f, c) else { continue };
            let depth = ((q.z - near) / (far - near)).clamp(0.0, 1.0);
            let shade = 0.9 - 0.6 * depth;
            let radius = (0.045 * f / q.z).clamp(1.5, 18.0);
            let color = if i == 0 { highlight } else { Color::new(shade, shade * 0.75, 0.24, 1.0) };
            draw_circle(s.x, s.y, radius + 1.0, Color::from_rgba(22, 32, 28, 255));
            draw_circle(s.x, s.y, radius, color);
        }

        // Inset: the highlighted point's ray seen from above, crossing the image plane at f.
        let (bx, by, bw, bh) = (screen_width() - 240.0, screen_height() - 160.0, 225.0, 145.0);
        draw_rectangle(bx, by, bw, bh, Color::from_rgba(30, 42, 37, 255));
        let q = to_camera(scatter[0]);
        let eye = vec2(bx + 16.0, by + bh / 2.0);
        let to = |z: f32, x: f32| vec2(eye.x + z / (distance + 2.0) * (bw - 28.0), eye.y - x / 2.2 * bh * 0.42);
        segment(Some(to(focal, 2.2)), Some(to(focal, -2.2)), 2.0, Color::from_rgba(240, 180, 41, 255));
        let (point, hit) = (to(q.z, q.x), to(focal, focal * q.x / q.z));
        segment(Some(eye), Some(point), 1.0, highlight);
        draw_circle(point.x, point.y, 4.5, highlight);
        draw_circle(hit.x, hit.y, 3.5, WHITE);
        draw_circle(eye.x, eye.y, 4.0, WHITE);
        draw_text("seen from above", bx + 8.0, by + 18.0, 18.0, GRAY);
        draw_text("x'", hit.x - 16.0, hit.y - 4.0, 18.0, WHITE);

        let text = format!("f = {focal:.2}   camera distance = {distance:.1}");
        draw_text(&text, 16.0, 28.0, 22.0, WHITE);
        draw_text("drag: orbit   wheel: distance   Up/Down: f   D: dolly zoom", 16.0, 50.0, 18.0, GRAY);
        next_frame().await;
    }
}
