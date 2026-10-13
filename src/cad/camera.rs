//! The view onto the infinite drafting plane: where it is looking and how far
//! it is zoomed in, with smooth motion toward wherever it is asked to go.
//!
//! World units are points at 100% zoom, so a formula laid out in a 640-unit
//! viewport looks exactly as it would in a 640-point window.

use egui::emath::TSTransform;
use egui::{Pos2, Rect, Vec2, pos2};

pub const MIN_ZOOM: f32 = 0.1;
pub const MAX_ZOOM: f32 = 4.0;
/// Time constant of the wheel-zoom smoothing, in seconds.
const ZOOM_TAU: f32 = 0.06;
/// Time constant of fly-to moves (fit, focus), in seconds.
const FLY_TAU: f32 = 0.09;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Camera {
    /// The world point shown at the center of the canvas.
    pub center: Pos2,
    pub zoom: f32,
    #[serde(skip)]
    goal: Option<Goal>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Goal {
    /// Zoom toward `zoom`, keeping `world` under the screen point `screen`.
    Anchored {
        zoom: f32,
        screen: Pos2,
        world: Pos2,
    },
    /// Glide to show `center` at `zoom`.
    Fly { center: Pos2, zoom: f32 },
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: Pos2::ZERO,
            zoom: 1.0,
            goal: None,
        }
    }
}

impl Camera {
    pub fn to_screen(&self, view: Rect, p: Pos2) -> Pos2 {
        view.center() + (p - self.center) * self.zoom
    }

    pub fn to_world(&self, view: Rect, p: Pos2) -> Pos2 {
        self.center + (p - view.center()) / self.zoom
    }

    pub fn rect_to_screen(&self, view: Rect, r: Rect) -> Rect {
        Rect::from_min_max(self.to_screen(view, r.min), self.to_screen(view, r.max))
    }

    pub fn rect_to_world(&self, view: Rect, r: Rect) -> Rect {
        Rect::from_min_max(self.to_world(view, r.min), self.to_world(view, r.max))
    }

    /// The world-to-screen transform for egui layers, with the translation
    /// rounded to whole physical pixels so text and lines stay sharp.
    pub fn transform(&self, view: Rect, pixels_per_point: f32) -> TSTransform {
        let t = view.center().to_vec2() - self.center.to_vec2() * self.zoom;
        let snap = |v: f32| (v * pixels_per_point).round() / pixels_per_point;
        TSTransform::new(egui::vec2(snap(t.x), snap(t.y)), self.zoom)
    }

    /// Pan by a screen-space drag, immediately (drags must not lag).
    pub fn pan(&mut self, screen_delta: Vec2) {
        self.center -= screen_delta / self.zoom;
        match &mut self.goal {
            Some(Goal::Anchored { world, .. }) => *world -= screen_delta / self.zoom,
            Some(Goal::Fly { .. }) => self.goal = None,
            None => {}
        }
    }

    /// Zoom by `factor` about the screen point `at`, smoothly.
    pub fn zoom_at(&mut self, view: Rect, at: Pos2, factor: f32) {
        let current_goal = match self.goal {
            Some(Goal::Anchored { zoom, .. }) | Some(Goal::Fly { zoom, .. }) => zoom,
            None => self.zoom,
        };
        let zoom = (current_goal * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let world = self.to_world(view, at);
        self.goal = Some(Goal::Anchored {
            zoom,
            screen: at,
            world,
        });
    }

    /// Glide so `world` fills the canvas (with a margin), never zooming in
    /// past `max_zoom`.
    pub fn fly_to(&mut self, view: Rect, world: Rect, max_zoom: f32) {
        if !world.is_positive() {
            return;
        }
        let margin = 48.0;
        let fit = ((view.width() - 2.0 * margin) / world.width())
            .min((view.height() - 2.0 * margin) / world.height());
        self.goal = Some(Goal::Fly {
            center: world.center(),
            zoom: fit.min(max_zoom).clamp(MIN_ZOOM, MAX_ZOOM),
        });
    }

    /// Glide to `zoom` about the canvas center.
    pub fn zoom_to(&mut self, zoom: f32) {
        self.goal = Some(Goal::Fly {
            center: self.center,
            zoom: zoom.clamp(MIN_ZOOM, MAX_ZOOM),
        });
    }

    pub fn is_moving(&self) -> bool {
        self.goal.is_some()
    }

    /// Advance the smoothing by `dt` seconds. Returns whether still moving.
    pub fn update(&mut self, view: Rect, dt: f32) -> bool {
        let Some(goal) = self.goal else {
            return false;
        };
        let dt = dt.clamp(0.0, 0.1);
        match goal {
            Goal::Anchored {
                zoom,
                screen,
                world,
            } => {
                let k = 1.0 - (-dt / ZOOM_TAU).exp();
                self.zoom = (self.zoom.ln() + (zoom.ln() - self.zoom.ln()) * k).exp();
                // Keep the anchor under the pointer at every step.
                self.center = world - (screen - view.center()) / self.zoom;
                if (self.zoom / zoom - 1.0).abs() < 1e-3 {
                    self.zoom = zoom;
                    self.center = world - (screen - view.center()) / self.zoom;
                    self.goal = None;
                }
            }
            Goal::Fly { center, zoom } => {
                let k = 1.0 - (-dt / FLY_TAU).exp();
                self.zoom = (self.zoom.ln() + (zoom.ln() - self.zoom.ln()) * k).exp();
                self.center += (center - self.center) * k;
                let close = (center - self.center).length() * self.zoom < 0.5;
                if close && (self.zoom / zoom - 1.0).abs() < 1e-3 {
                    self.center = center;
                    self.zoom = zoom;
                    self.goal = None;
                }
            }
        }
        if self.goal.is_none() {
            self.settle();
        }
        self.goal.is_some()
    }

    /// Text and images are sharpest at exactly 100%: snap to it when close.
    fn settle(&mut self) {
        if (self.zoom - 1.0).abs() < 0.03 && self.zoom != 1.0 {
            self.zoom = 1.0;
        }
    }

    /// Grid spacing in world units for the current zoom: a power of ten
    /// times 1, 2 or 5, so minor lines stay about 12–30 points apart.
    pub fn grid_step(&self) -> f32 {
        let target = 16.0 / self.zoom;
        let base = 10f32.powf(target.log10().floor());
        [1.0, 2.0, 5.0, 10.0]
            .into_iter()
            .map(|m| base * m)
            .find(|s| *s >= target)
            .unwrap_or(base * 10.0)
    }

    pub fn snap(p: Pos2, step: f32) -> Pos2 {
        pos2((p.x / step).round() * step, (p.y / step).round() * step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    fn view() -> Rect {
        Rect::from_min_size(pos2(100.0, 50.0), vec2(800.0, 600.0))
    }

    fn settle(c: &mut Camera) {
        for _ in 0..600 {
            if !c.update(view(), 1.0 / 60.0) {
                break;
            }
        }
        assert!(!c.is_moving());
    }

    #[test]
    fn round_trip() {
        let c = Camera {
            center: pos2(30.0, -40.0),
            zoom: 1.7,
            goal: None,
        };
        let p = pos2(123.0, 456.0);
        let back = c.to_world(view(), c.to_screen(view(), p));
        assert!((back - p).length() < 1e-3);
        let t = c.transform(view(), 1.0);
        assert!((t * p - c.to_screen(view(), p)).length() < 1.0);
    }

    #[test]
    fn zoom_keeps_the_point_under_the_cursor() {
        let mut c = Camera::default();
        let at = pos2(700.0, 120.0);
        let world = c.to_world(view(), at);
        c.zoom_at(view(), at, 2.5);
        // Halfway through the animation too, not just at the end.
        c.update(view(), 1.0 / 60.0);
        assert!((c.to_screen(view(), world) - at).length() < 1e-2);
        settle(&mut c);
        assert!((c.zoom - 2.5).abs() < 1e-4);
        assert!((c.to_screen(view(), world) - at).length() < 1e-2);
    }

    #[test]
    fn zoom_is_clamped_and_snaps_to_one() {
        let mut c = Camera::default();
        c.zoom_at(view(), view().center(), 100.0);
        settle(&mut c);
        assert_eq!(c.zoom, MAX_ZOOM);
        c.zoom_to(1.02);
        settle(&mut c);
        assert_eq!(c.zoom, 1.0);
    }

    #[test]
    fn fly_to_fits_the_rect() {
        let mut c = Camera::default();
        let target = Rect::from_min_size(pos2(1000.0, 2000.0), vec2(1400.0, 300.0));
        c.fly_to(view(), target, 1.0);
        settle(&mut c);
        assert!((c.center - target.center()).length() < 1.0);
        let shown = c.rect_to_screen(view(), target);
        assert!(view().contains_rect(shown.shrink(1.0)));
        assert!((shown.width() - (800.0 - 96.0)).abs() < 1.0);
    }

    #[test]
    fn pan_is_immediate() {
        let mut c = Camera {
            zoom: 2.0,
            ..Camera::default()
        };
        c.pan(vec2(20.0, -10.0));
        assert_eq!(c.center, pos2(-10.0, 5.0));
    }

    #[test]
    fn grid_step_is_a_nice_number() {
        for zoom in [0.1, 0.33, 1.0, 2.7, 4.0] {
            let c = Camera {
                zoom,
                ..Camera::default()
            };
            let s = c.grid_step();
            let on_screen = s * zoom;
            assert!((16.0..=40.0).contains(&on_screen), "{zoom}: {on_screen}");
        }
    }
}
