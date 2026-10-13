//! Perspective projection: 3D points onto a 2D screen by dividing by depth.

use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::draw::lerp_color;
use crate::formula::{Formula, MathText, Palette, Tone};
use crate::rng::Rng;

pub type Vec3 = [f32; 3];

/// Points closer to the camera than this are not drawn.
pub const NEAR: f32 = 0.05;

/// Rotate around the vertical axis (yaw), then the horizontal axis (pitch).
pub fn rotate([x, y, z]: Vec3, yaw: f32, pitch: f32) -> Vec3 {
    let (sy, cy) = yaw.sin_cos();
    let (x, z) = (x * cy + z * sy, -x * sy + z * cy);
    let (sp, cp) = pitch.sin_cos();
    let (y, z) = (y * cp - z * sp, y * sp + z * cp);
    [x, y, z]
}

/// `x' = f·x/z + cx`, `y' = f·y/z + cy` (y flipped so up is up on screen).
pub fn project([x, y, z]: Vec3, f: f32, c: Pos2) -> Option<Pos2> {
    (z > NEAR).then(|| pos2(f * x / z + c.x, -f * y / z + c.y))
}

const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 3),
    (3, 2),
    (2, 0),
    (4, 5),
    (5, 7),
    (7, 6),
    (6, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

fn cube_corner(i: usize) -> Vec3 {
    let s = |bit: usize| if i & bit != 0 { 1.0 } else { -1.0 };
    [s(1), s(2), s(4)]
}

pub struct Projection {
    /// Focal length as a fraction of the stage's shorter side.
    focal: f32,
    /// Distance from the camera to the center of the scene.
    distance: f32,
    yaw: f32,
    pitch: f32,
    spinning: bool,
    scatter: Vec<Vec3>,
}

impl Default for Projection {
    fn default() -> Self {
        let mut rng = Rng::new(3);
        let scatter = (0..70)
            .map(|_| {
                [
                    rng.range(-1.0, 1.0),
                    rng.range(-1.0, 1.0),
                    rng.range(-1.0, 1.0),
                ]
            })
            .collect();
        Self {
            focal: 1.0,
            distance: 4.0,
            yaw: 0.6,
            pitch: 0.35,
            spinning: true,
            scatter,
        }
    }
}

impl Projection {
    fn to_camera(&self, p: Vec3) -> Vec3 {
        let [x, y, z] = rotate(p, self.yaw, self.pitch);
        [x, y, z + self.distance]
    }
}

impl Formula for Projection {
    fn id(&self) -> &'static str {
        "perspective-projection"
    }

    fn name(&self) -> &'static str {
        "Perspective projection"
    }

    fn category(&self) -> &'static str {
        super::COORDINATES
    }

    fn summary(&self) -> &'static str {
        "A 3D scatter plot flattened onto the screen. Dividing by depth z makes far \
         points shrink toward the center. Drag to orbit; the inset shows one ray from above."
    }

    fn notes(&self) -> &'static str {
        "3D scatter plots, bar charts with depth, globe and terrain views, and 3D network \
         graphs all project through this divide-by-z step. Larger z means smaller on \
         screen, which is what gives the picture its depth."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("x'")
                .tone(Tone::X)
                .t(" = f · x / z + c")
                .sub("x"),
            MathText::new()
                .t("y'")
                .tone(Tone::Y)
                .t(" = f · y / z + c")
                .sub("y"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::drag()
    }

    fn interact(&mut self, response: &egui::Response, _rect: Rect) {
        if response.dragged() {
            let d = response.drag_delta();
            self.yaw += d.x * 0.01;
            self.pitch = (self.pitch + d.y * 0.01).clamp(-1.4, 1.4);
            self.spinning = false;
        }
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.spinning {
            self.yaw += 0.35 * dt;
        }
        let f = self.focal * rect.width().min(rect.height());
        let c = rect.center();

        // Floor grid at y = -1 for a sense of depth.
        let floor = Stroke::new(1.0, palette.rule);
        for i in 0..=8 {
            let u = -1.6 + 3.2 * i as f32 / 8.0;
            for (a, b) in [
                ([u, -1.0, -1.6], [u, -1.0, 1.6]),
                ([-1.6, -1.0, u], [1.6, -1.0, u]),
            ] {
                if let (Some(a), Some(b)) = (
                    project(self.to_camera(a), f, c),
                    project(self.to_camera(b), f, c),
                ) {
                    painter.line_segment([a, b], floor);
                }
            }
        }

        let edge = Stroke::new(1.2, palette.muted.gamma_multiply(0.8));
        for &(i, j) in &CUBE_EDGES {
            let a = project(self.to_camera(cube_corner(i)), f, c);
            let b = project(self.to_camera(cube_corner(j)), f, c);
            if let (Some(a), Some(b)) = (a, b) {
                painter.line_segment([a, b], edge);
            }
        }

        // Far points first so near ones paint over them.
        let mut points: Vec<(usize, Vec3)> = self
            .scatter
            .iter()
            .map(|&p| self.to_camera(p))
            .enumerate()
            .collect();
        points.sort_by(|a, b| b.1[2].total_cmp(&a.1[2]));
        let (near, far) = (self.distance - 1.8, self.distance + 1.8);
        for (i, cam) in points {
            let Some(s) = project(cam, f, c) else {
                continue;
            };
            let depth = ((cam[2] - near) / (far - near)).clamp(0.0, 1.0);
            let color = lerp_color(palette.trace, palette.muted.gamma_multiply(0.5), depth);
            let radius = (0.045 * f / cam[2]).clamp(1.5, 18.0);
            if i == 0 {
                painter.circle(s, radius, palette.x, Stroke::new(2.0, palette.card));
            } else {
                painter.circle(s, radius, color, Stroke::new(1.0, palette.card));
            }
        }

        self.paint_side_view(painter, rect, palette);
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("projection-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Focal length f");
                ui.add(egui::Slider::new(&mut self.focal, 0.3..=2.5));
                ui.end_row();
                ui.label("Camera distance");
                ui.add(egui::Slider::new(&mut self.distance, 2.2..=12.0));
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button(if self.spinning { "Stop spinning" } else { "Spin" }).clicked() {
                self.spinning = !self.spinning;
            }
            if ui.button("Dolly zoom").on_hover_text("Move the camera back while lengthening f, so the cube keeps its size but loses its perspective").clicked() {
                self.distance = if self.distance < 7.0 { 10.0 } else { 3.0 };
                self.focal = 0.25 * self.distance;
            }
        });
    }
}

impl Projection {
    /// Top-down diagram of the highlighted point's ray through the image plane.
    fn paint_side_view(&self, painter: &egui::Painter, rect: Rect, palette: &Palette) {
        let size = vec2(
            (rect.width() * 0.36).clamp(150.0, 240.0),
            (rect.height() * 0.3).clamp(110.0, 170.0),
        );
        let inset = Rect::from_min_size(rect.right_bottom() - size - vec2(12.0, 12.0), size);
        painter.rect(
            inset,
            10.0,
            palette.card,
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Inside,
        );

        let [x, _, z] = self.to_camera(self.scatter[0]);
        let max_z = self.distance + 2.0;
        let span_x = 2.2;
        let eye = pos2(inset.left() + 18.0, inset.center().y);
        let w = inset.width() - 30.0;
        let to = |z: f32, x: f32| {
            pos2(
                eye.x + z / max_z * w,
                eye.y - x / span_x * inset.height() * 0.45,
            )
        };

        let plane_z = self.focal;
        let plane_top = to(plane_z, span_x);
        let plane_bottom = to(plane_z, -span_x);
        painter.line_segment(
            [plane_top, plane_bottom],
            Stroke::new(2.0, palette.trace.gamma_multiply(0.6)),
        );
        painter.line_segment([eye, to(max_z, 0.0)], Stroke::new(1.0, palette.rule));

        if z > NEAR {
            let point = to(z, x);
            let hit = to(plane_z, self.focal * x / z);
            painter.extend(Shape::dashed_line(
                &[eye, point],
                Stroke::new(1.0, palette.x),
                4.0,
                3.0,
            ));
            painter.circle_filled(point, 4.5, palette.x);
            painter.circle_filled(hit, 3.5, palette.ink);
            painter.text(
                point + vec2(6.0, -4.0),
                Align2::LEFT_BOTTOM,
                "(x, z)",
                FontId::proportional(11.0),
                palette.x,
            );
            painter.text(
                hit + vec2(-5.0, 0.0),
                Align2::RIGHT_CENTER,
                "x'",
                FontId::proportional(11.0),
                palette.ink,
            );
        }
        painter.circle_filled(eye, 4.0, palette.ink);
        painter.text(
            inset.left_top() + vec2(8.0, 6.0),
            Align2::LEFT_TOP,
            "seen from above",
            FontId::proportional(11.0),
            palette.muted,
        );
        painter.text(
            plane_bottom + vec2(0.0, 2.0),
            Align2::CENTER_TOP,
            "f",
            FontId::proportional(11.0),
            palette.trace,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_maps_to_center() {
        let c = pos2(100.0, 50.0);
        assert_eq!(project([0.0, 0.0, 3.0], 200.0, c), Some(c));
    }

    #[test]
    fn twice_as_far_is_half_as_big() {
        let c = pos2(0.0, 0.0);
        let near = project([1.0, 1.0, 2.0], 100.0, c).unwrap();
        let far = project([1.0, 1.0, 4.0], 100.0, c).unwrap();
        assert!((near.x - 2.0 * far.x).abs() < 1e-4);
        assert!((near.y - 2.0 * far.y).abs() < 1e-4);
    }

    #[test]
    fn behind_camera_is_culled() {
        assert_eq!(project([1.0, 1.0, -1.0], 100.0, pos2(0.0, 0.0)), None);
    }

    #[test]
    fn rotation_preserves_length() {
        let p = [0.3, -0.7, 1.1];
        let q = rotate(p, 1.2, -0.4);
        let len = |v: Vec3| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len(p) - len(q)).abs() < 1e-5);
    }
}
