//! Voronoi diagram: V(p_i) = { x : |x - p_i| <= |x - p_j| for all j != i }.
//!
//! Each seed owns every location closer to it than to any other seed. Cells are computed
//! exactly: start from the whole window and cut away, for every other seed, the half-plane
//! beyond the perpendicular bisector. Seeds drift; hover to compare the distances to the
//! nearest seed and the runner-up. Drag a seed, click to add one, right-click to remove.
//! Keys: R toggles Lloyd relaxation (seeds slide to their cell centroids), D toggles drift.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const W: f32 = 800.0;
const H: f32 = 540.0; // the diagram's box; a status line sits below it
const BG: Color = Color::from_rgba(22, 32, 28, 255);

/// Keep the part of a convex polygon closer to p than to q (one Sutherland-Hodgman pass).
fn clip_closer(poly: &[Vec2], p: Vec2, q: Vec2) -> Vec<Vec2> {
    let m = (p + q) / 2.0;
    let side = |a: Vec2| (a - m).dot(q - p); // <= 0 on p's side of the bisector
    let mut out = Vec::with_capacity(poly.len() + 1);
    for k in 0..poly.len() {
        let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
        let (sa, sb) = (side(a), side(b));
        if sa <= 0.0 {
            out.push(a);
        }
        if (sa <= 0.0) != (sb <= 0.0) {
            out.push(a + (b - a) * (sa / (sa - sb))); // where the edge crosses the bisector
        }
    }
    out
}

/// The Voronoi cell of points[i] inside the box.
fn cell(points: &[Vec2], i: usize) -> Vec<Vec2> {
    let mut poly = vec![vec2(0.0, 0.0), vec2(W, 0.0), vec2(W, H), vec2(0.0, H)];
    for (j, &q) in points.iter().enumerate() {
        if j != i && q != points[i] && !poly.is_empty() {
            poly = clip_closer(&poly, points[i], q);
        }
    }
    poly
}

/// Center of mass of a polygon (shoelace formula).
fn centroid(poly: &[Vec2]) -> Option<Vec2> {
    let (mut a2, mut c) = (0.0, Vec2::ZERO);
    for k in 0..poly.len() {
        let (p, q) = (poly[k], poly[(k + 1) % poly.len()]);
        let cross = p.x * q.y - q.x * p.y;
        a2 += cross;
        c += (p + q) * cross;
    }
    (a2.abs() > 1e-6).then(|| c / (3.0 * a2))
}

struct Seed {
    at: Vec2,
    vel: Vec2,
    hue: f32,
}

fn new_seed(at: Vec2) -> Seed {
    let angle = rand::gen_range(0.0, std::f32::consts::TAU);
    Seed { at, vel: Vec2::from_angle(angle) * rand::gen_range(12.0, 32.0), hue: rand::gen_range(0.0, 1.0) }
}

fn animate(seeds: &mut [Seed], drift: bool, relax: bool, dragging: Option<usize>, dt: f32) {
    if drift {
        for (k, s) in seeds.iter_mut().enumerate() {
            if Some(k) == dragging {
                continue;
            }
            s.at += s.vel * dt;
            // Bounce off the walls.
            if s.at.x < 5.0 || s.at.x > W - 5.0 {
                s.vel.x = -s.vel.x;
            }
            if s.at.y < 5.0 || s.at.y > H - 5.0 {
                s.vel.y = -s.vel.y;
            }
            s.at = s.at.clamp(vec2(5.0, 5.0), vec2(W - 5.0, H - 5.0));
        }
    }
    if relax {
        // Lloyd's algorithm: each seed slides toward the centroid of its cell.
        let points: Vec<Vec2> = seeds.iter().map(|s| s.at).collect();
        for (k, s) in seeds.iter_mut().enumerate() {
            if let Some(c) = centroid(&cell(&points, k)).filter(|_| Some(k) != dragging) {
                s.at += (c - s.at) * (dt * 2.0).min(1.0);
            }
        }
    }
}

#[macroquad::main("Voronoi diagram")]
async fn main() {
    rand::srand(11);
    let mut seeds: Vec<Seed> = (0..24)
        .map(|_| new_seed(vec2(rand::gen_range(20.0, W - 20.0), rand::gen_range(20.0, H - 20.0))))
        .collect();
    let (mut drift, mut relax) = (true, false);
    let mut dragging: Option<usize> = None;

    loop {
        let mouse = Vec2::from(mouse_position());
        let inside = mouse.x >= 0.0 && mouse.x <= W && mouse.y >= 0.0 && mouse.y <= H;
        let picked = seeds.iter().position(|s| s.at.distance(mouse) < 12.0);
        if inside && is_mouse_button_pressed(MouseButton::Left) {
            dragging = picked;
            if picked.is_none() && seeds.len() < 200 {
                seeds.push(new_seed(mouse));
            }
        }
        if is_mouse_button_pressed(MouseButton::Right) && seeds.len() > 2 {
            if let Some(k) = picked {
                seeds.remove(k);
                dragging = None;
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            dragging = None;
        }
        if let Some(k) = dragging {
            seeds[k].at = mouse.clamp(Vec2::ZERO, vec2(W, H));
        }
        if is_key_pressed(KeyCode::R) {
            relax = !relax;
        }
        if is_key_pressed(KeyCode::D) {
            drift = !drift;
        }
        animate(&mut seeds, drift, relax, dragging, get_frame_time().min(0.1));

        clear_background(BG);
        let points: Vec<Vec2> = seeds.iter().map(|s| s.at).collect();
        let mut order: Vec<usize> = (0..points.len()).collect();
        order.sort_by(|&a, &b| points[a].distance(mouse).total_cmp(&points[b].distance(mouse)));
        for (i, s) in seeds.iter().enumerate() {
            let poly = cell(&points, i);
            let lightness = if inside && order[0] == i { 0.52 } else { 0.40 };
            let fill = macroquad::color::hsl_to_rgb(s.hue, 0.45, lightness);
            // Cells are convex, so a triangle fan from the first corner fills them.
            for k in 1..poly.len().saturating_sub(1) {
                draw_triangle(poly[0], poly[k], poly[k + 1], fill);
            }
            for k in 0..poly.len() {
                let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
                draw_line(a.x, a.y, b.x, b.y, 1.5, BG);
            }
        }
        for (k, p) in points.iter().enumerate() {
            let big = dragging == Some(k);
            draw_circle(p.x, p.y, if big { 6.0 } else { 3.5 }, if big { ORANGE } else { WHITE });
        }

        // Hover: the nearest seed wins; the runner-up is shown for comparison.
        if inside && points.len() > 1 {
            let (i, j) = (points[order[0]], points[order[1]]);
            let blue = Color::from_rgba(90, 180, 230, 255);
            let red = Color::from_rgba(229, 112, 126, 255);
            draw_line(mouse.x, mouse.y, j.x, j.y, 1.5, red);
            draw_line(mouse.x, mouse.y, i.x, i.y, 2.5, blue);
            let (di, dj) = (i.distance(mouse), j.distance(mouse));
            for (p, text, color) in [(j, format!("{dj:.0}"), red), (i, format!("{di:.0} nearest"), blue)] {
                let mid = (mouse + p) / 2.0;
                let w = measure_text(&text, None, 18, 1.0).width;
                draw_rectangle(mid.x - w / 2.0 - 4.0, mid.y - 11.0, w + 8.0, 18.0, BG);
                draw_text(&text, mid.x - w / 2.0, mid.y + 3.0, 18.0, color);
            }
        }

        let on_off = |b: bool| if b { "on" } else { "off" };
        let (n, d, r) = (seeds.len(), on_off(drift), on_off(relax));
        let status = format!("{n} seeds   drift {d} (D)   Lloyd relax {r} (R)");
        draw_text(&status, 16.0, H + 26.0, 20.0, LIGHTGRAY);
        draw_text("drag a seed, click to add, right-click to remove", 16.0, H + 50.0, 20.0, GRAY);
        next_frame().await;
    }
}
