//! Euclidean distance: d = sqrt((x2 - x1)^2 + (y2 - y1)^2).
//!
//! Nearest point: a probe follows the mouse (or wanders on its own) and snaps to the
//! closest point of a scatter plot, as a chart tooltip does. The legs dx and dy and the
//! empty circle of radius d show the Pythagorean triangle. K-means: every point joins
//! its nearest centroid, then each centroid moves to the mean of its points.
//! Click the tabs to switch modes; in K-means, click the plot for new centroids and
//! press 2-6 to change k.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const W: f32 = 100.0;
const H: f32 = 60.0; // data space, drawn with equal scale on both axes
const SCALE: f32 = 7.0;
const LEFT: f32 = 50.0;
const TOP: f32 = 70.0;
const TAB_X: [f32; 2] = [50.0, 210.0];
const COLORS: [Color; 6] = [
    Color::new(0.35, 0.71, 0.90, 1.0),
    Color::new(0.94, 0.71, 0.35, 1.0),
    Color::new(0.48, 0.78, 0.48, 1.0),
    Color::new(0.90, 0.44, 0.49, 1.0),
    Color::new(0.70, 0.62, 0.86, 1.0),
    Color::new(0.83, 0.76, 0.42, 1.0),
];

fn distance(a: Vec2, b: Vec2) -> f32 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

/// Index of the point closest to `at`, and its distance (a linear scan).
fn nearest(points: &[Vec2], at: Vec2) -> (usize, f32) {
    points
        .iter()
        .enumerate()
        .map(|(i, &p)| (i, distance(p, at)))
        .fold((0, f32::INFINITY), |best, c| if c.1 < best.1 { c } else { best })
}

/// One K-means round: assign each point to its nearest centroid, then move each
/// centroid to the mean of its points. Returns whether any centroid moved.
fn kmeans_step(points: &[Vec2], centroids: &mut [Vec2]) -> bool {
    let assign: Vec<usize> = points.iter().map(|&p| nearest(centroids, p).0).collect();
    let mut moved = false;
    for (k, centroid) in centroids.iter_mut().enumerate() {
        let members: Vec<Vec2> = points
            .iter()
            .zip(&assign)
            .filter(|(_, a)| **a == k)
            .map(|(p, _)| *p)
            .collect();
        if members.is_empty() {
            continue; // an empty cluster keeps its centroid
        }
        let mean = members.iter().fold(Vec2::ZERO, |s, &p| s + p) / members.len() as f32;
        moved |= distance(mean, *centroid) > 1e-4;
        *centroid = mean;
    }
    moved
}

fn make_data() -> Vec<Vec2> {
    let blobs: Vec<Vec2> = [(25.0, 18.0), (75.0, 18.0), (25.0, 42.0), (75.0, 42.0)]
        .iter()
        .map(|&(x, y)| vec2(x + rand::gen_range(-8.0, 8.0), y + rand::gen_range(-5.0, 5.0)))
        .collect();
    // The sum of three uniforms is a cheap bell curve.
    let bell = || ((0..3).map(|_| rand::gen_range(0.0, 1.0)).sum::<f32>() - 1.5) * 9.0;
    (0..90)
        .map(|i| {
            let c = blobs[i % blobs.len()];
            vec2((c.x + bell()).clamp(1.0, W - 1.0), (c.y + bell()).clamp(1.0, H - 1.0))
        })
        .collect()
}

fn random_centroids(k: usize) -> Vec<Vec2> {
    (0..k).map(|_| vec2(rand::gen_range(5.0, W - 5.0), rand::gen_range(5.0, H - 5.0))).collect()
}

fn to_screen(p: Vec2) -> Vec2 {
    vec2(LEFT + p.x * SCALE, TOP + (H - p.y) * SCALE)
}

fn to_data(s: Vec2) -> Option<Vec2> {
    let p = vec2((s.x - LEFT) / SCALE, H - (s.y - TOP) / SCALE);
    (p.x >= 0.0 && p.x <= W && p.y >= 0.0 && p.y <= H).then_some(p)
}

fn line(a: Vec2, b: Vec2, width: f32, color: Color) {
    draw_line(a.x, a.y, b.x, b.y, width, color);
}

fn draw_nearest(points: &[Vec2], probe: Vec2) -> String {
    for &p in points {
        let s = to_screen(p);
        draw_circle(s.x, s.y, 3.5, Color::from_rgba(111, 125, 118, 255));
    }
    let (i, d) = nearest(points, probe);
    let b = points[i];
    let (pa, pb, corner) = (to_screen(probe), to_screen(b), to_screen(vec2(b.x, probe.y)));
    // Nothing lies inside this circle: that is what "nearest" means.
    draw_poly(pa.x, pa.y, 64, d * SCALE, 0.0, Color::new(0.62, 0.70, 1.0, 0.12));
    line(pa, corner, 2.0, COLORS[0]);
    line(corner, pb, 2.0, COLORS[3]);
    line(pa, pb, 2.5, COLORS[1]);
    draw_circle(pb.x, pb.y, 7.0, COLORS[1]);
    draw_circle(pa.x, pa.y, 4.0, WHITE);
    let (dx, dy) = ((b.x - probe.x).abs(), (b.y - probe.y).abs());
    draw_text(&format!("dx {dx:.1}"), (pa.x + corner.x) / 2.0 - 16.0, corner.y + 18.0, 18.0, COLORS[0]);
    draw_text(&format!("dy {dy:.1}"), corner.x + 6.0, (corner.y + pb.y) / 2.0, 18.0, COLORS[3]);
    // The tooltip, as a scatter plot would show it, on the side away from the probe.
    let tx = if pa.x > pb.x { pb.x - 138.0 } else { pb.x + 12.0 };
    let ty = if pa.y < pb.y { pb.y + 12.0 } else { pb.y - 70.0 };
    let tx = tx.clamp(LEFT + 4.0, LEFT + W * SCALE - 130.0);
    let ty = ty.clamp(TOP + 4.0, TOP + H * SCALE - 62.0);
    draw_rectangle(tx, ty, 126.0, 58.0, Color::from_rgba(36, 48, 43, 255));
    let tip = [format!("point #{i}"), format!("({:.1}, {:.1})", b.x, b.y), format!("d = {d:.2}")];
    for (k, s) in tip.iter().enumerate() {
        draw_text(s, tx + 8.0, ty + 18.0 + 16.0 * k as f32, 18.0, WHITE);
    }
    format!("checked {} distances, nearest d = {d:.2}", points.len())
}

fn draw_kmeans(points: &[Vec2], centroids: &[Vec2]) {
    for &p in points {
        let c = nearest(centroids, p).0;
        let (s, t) = (to_screen(p), to_screen(centroids[c]));
        line(s, t, 1.0, COLORS[c].with_alpha(0.2));
        draw_circle(s.x, s.y, 4.0, COLORS[c]);
    }
    for (i, &c) in centroids.iter().enumerate() {
        let s = to_screen(c);
        draw_circle(s.x, s.y, 10.0, Color::from_rgba(22, 32, 28, 255));
        draw_circle_lines(s.x, s.y, 9.0, 3.0, COLORS[i]);
        line(s - vec2(5.0, 0.0), s + vec2(5.0, 0.0), 2.0, COLORS[i]);
        line(s - vec2(0.0, 5.0), s + vec2(0.0, 5.0), 2.0, COLORS[i]);
    }
}

#[macroquad::main("Euclidean distance")]
async fn main() {
    rand::srand(2);
    let points = make_data();
    let mut kmeans_mode = false;
    let mut k = 4;
    let mut centroids = random_centroids(k);
    let (mut iteration, mut converged, mut timer) = (0, false, 0.8);

    loop {
        clear_background(Color::from_rgba(22, 32, 28, 255));
        let mouse = Vec2::from(mouse_position());

        // Input: tabs, new centroids on click, keys 2-6 for k.
        if is_mouse_button_pressed(MouseButton::Left) {
            let on_tab = |x: f32| mouse.x >= x && mouse.x <= x + 150.0 && mouse.y >= 18.0 && mouse.y <= 50.0;
            if let Some(tab) = TAB_X.iter().position(|&x| on_tab(x)) {
                kmeans_mode = tab == 1;
            } else if kmeans_mode && to_data(mouse).is_some() {
                (centroids, iteration, converged, timer) = (random_centroids(k), 0, false, 0.8);
            }
        }
        let digits = [KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6];
        if let Some(n) = digits.iter().position(|&key| is_key_pressed(key)) {
            k = n + 2;
            (centroids, iteration, converged, timer) = (random_centroids(k), 0, false, 0.8);
        }

        for (t, &x) in TAB_X.iter().enumerate() {
            let active = (t == 1) == kmeans_mode;
            let (r, g, b) = if active { (58, 74, 138) } else { (36, 48, 43) };
            draw_rectangle(x, 18.0, 150.0, 32.0, Color::from_rgba(r, g, b, 255));
            draw_text(["Nearest point", "K-means"][t], x + 14.0, 40.0, 22.0, WHITE);
        }
        // Grid, one line every 10 data units.
        let grid = Color::from_rgba(38, 51, 46, 255);
        for g in (0..=100).step_by(10) {
            line(to_screen(vec2(g as f32, 0.0)), to_screen(vec2(g as f32, H)), 1.0, grid);
        }
        for g in (0..=60).step_by(10) {
            line(to_screen(vec2(0.0, g as f32)), to_screen(vec2(W, g as f32)), 1.0, grid);
        }

        let (caption, hint) = if kmeans_mode {
            timer -= get_frame_time();
            if timer <= 0.0 {
                if converged {
                    (centroids, iteration, converged) = (random_centroids(k), 0, false);
                    timer = 0.8;
                } else {
                    converged = !kmeans_step(&points, &mut centroids);
                    iteration += 1;
                    timer = if converged { 2.5 } else { 0.7 };
                }
            }
            draw_kmeans(&points, &centroids);
            let status = if converged { "converged" } else { "running" };
            let caption = format!("k = {k}   iteration {iteration}  ({status})");
            (caption, "click the plot for new centroids, keys 2-6 set k")
        } else {
            let s = get_time() as f32 / 1.6;
            let wander = vec2(W / 2.0 + 40.0 * (1.3 * s).sin(), H / 2.0 + 22.0 * (2.1 * s + 0.5).sin());
            (draw_nearest(&points, to_data(mouse).unwrap_or(wander)), "move the mouse over the plot")
        };
        draw_text(&caption, LEFT, 530.0, 20.0, LIGHTGRAY);
        draw_text(hint, LEFT, 570.0, 20.0, GRAY);
        next_frame().await;
    }
}
