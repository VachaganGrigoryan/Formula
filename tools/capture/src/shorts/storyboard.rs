//! The storyboard: what `formula-render` is asked to draw. `tools/shorts`
//! writes it as JSON after the narration is recorded, so every scene has a
//! fixed length.

use egui::{Vec2, vec2};
use serde::{Deserialize, Serialize};

/// Light or dark palette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn is_dark(self) -> bool {
        self == Theme::Dark
    }
}

/// One narration unit: its caption is shown while its audio plays.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    /// Short kicker line above the caption (may be empty).
    #[serde(default)]
    pub heading: String,
    /// The spoken text, shown as the caption.
    #[serde(default)]
    pub caption: String,
    /// How long the scene stays on screen.
    pub seconds: f32,
    /// Formula parameters applied through `Formula::load_state` when the
    /// scene starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<serde_json::Value>,
    /// Whether the equation lines are shown during this scene.
    #[serde(default = "yes")]
    pub equations: bool,
}

fn yes() -> bool {
    true
}

/// A whole video.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Storyboard {
    pub tool: String,
    /// Output size in pixels.
    #[serde(default = "d_width")]
    pub width: u32,
    #[serde(default = "d_height")]
    pub height: u32,
    #[serde(default = "d_fps")]
    pub fps: u32,
    /// Pixels per egui point. 2 draws a Short as a 540 x 960 layout.
    #[serde(default = "d_scale")]
    pub scale: f32,
    #[serde(default)]
    pub theme: Theme,
    /// Show the workbench viewport (frame and title tab) instead of the clean
    /// stage.
    #[serde(default)]
    pub chrome: bool,
    /// Silence before the first scene.
    #[serde(default = "d_lead_in")]
    pub lead_in: f32,
    /// Seconds the tool runs before the first frame, so a trail or a search
    /// is already under way. Half of the README capture's warmup when absent.
    #[serde(default)]
    pub warmup: Option<f32>,
    /// Title at the top; the tool's name when absent.
    #[serde(default)]
    pub title: Option<String>,
    /// Small line under the title; the tool's category when absent.
    #[serde(default)]
    pub kicker: Option<String>,
    #[serde(default = "d_footer")]
    pub footer: String,
    /// Time of the thumbnail frame; the middle of the first scene when absent.
    #[serde(default)]
    pub thumbnail_at: Option<f32>,
    pub scenes: Vec<Scene>,
}

fn d_width() -> u32 {
    1080
}
fn d_height() -> u32 {
    1920
}
fn d_fps() -> u32 {
    30
}
fn d_scale() -> f32 {
    2.0
}
fn d_lead_in() -> f32 {
    0.5
}
fn d_footer() -> String {
    "Formula  ·  github.com/VachaganGrigoryan/Formula".into()
}

/// Where a moment of the video falls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cue {
    /// Index into `scenes`.
    pub index: usize,
    /// Seconds since the scene began.
    pub local: f32,
    /// Seconds until it ends.
    pub remaining: f32,
}

impl Storyboard {
    pub fn parse(json: &str) -> Result<Self, String> {
        let s: Self = serde_json::from_str(json).map_err(|e| format!("storyboard: {e}"))?;
        s.validate()?;
        Ok(s)
    }

    /// Everything the renderer relies on, checked up front with a plain
    /// message: a known tool, even dimensions that divide by the scale, a
    /// frame rate, and scenes with positive lengths.
    pub fn validate(&self) -> Result<(), String> {
        if formula::formulas::tool(&self.tool).is_none() {
            return Err(format!("unknown tool {:?}", self.tool));
        }
        for (name, v) in [("width", self.width), ("height", self.height)] {
            if v == 0 || v % 2 != 0 {
                return Err(format!("{name} must be a positive even number, not {v}"));
            }
            let logical = v as f32 / self.scale;
            if !self.scale.is_finite()
                || self.scale <= 0.0
                || (logical - logical.round()).abs() > 1e-4
            {
                return Err(format!(
                    "{name} {v} is not a whole multiple of scale {}",
                    self.scale
                ));
            }
        }
        if self.fps == 0 {
            return Err("fps must be positive".into());
        }
        if self.scenes.is_empty() {
            return Err("at least one scene is needed".into());
        }
        if let Some((n, s)) = self
            .scenes
            .iter()
            .enumerate()
            .find(|(_, s)| !(s.seconds > 0.0 && s.seconds.is_finite()))
        {
            return Err(format!("scene {} has no length ({})", n + 1, s.seconds));
        }
        if !(self.lead_in >= 0.0 && self.lead_in.is_finite()) {
            return Err(format!(
                "lead_in must be zero or more, not {}",
                self.lead_in
            ));
        }
        Ok(())
    }

    /// The egui layout size in points.
    pub fn logical_size(&self) -> Vec2 {
        vec2(
            self.width as f32 / self.scale,
            self.height as f32 / self.scale,
        )
    }

    pub fn dt(&self) -> f32 {
        1.0 / self.fps as f32
    }

    /// Lead-in plus every scene.
    pub fn total_seconds(&self) -> f32 {
        self.lead_in + self.scenes.iter().map(|s| s.seconds).sum::<f32>()
    }

    pub fn frames(&self) -> usize {
        (self.total_seconds() * self.fps as f32).round() as usize
    }

    /// When each scene starts, after the lead-in.
    pub fn starts(&self) -> Vec<f32> {
        let mut t = self.lead_in;
        self.scenes
            .iter()
            .map(|s| {
                let start = t;
                t += s.seconds;
                start
            })
            .collect()
    }

    /// The scene playing at `t`, if any (none during the lead-in or after the
    /// last scene).
    pub fn cue(&self, t: f32) -> Option<Cue> {
        let mut start = self.lead_in;
        for (index, s) in self.scenes.iter().enumerate() {
            let end = start + s.seconds;
            if t >= start && t < end {
                return Some(Cue {
                    index,
                    local: t - start,
                    remaining: end - t,
                });
            }
            start = end;
        }
        None
    }

    /// The scene whose parameters apply at `t`: the first one during the
    /// lead-in, the last one after the end.
    pub fn active_scene(&self, t: f32) -> usize {
        match self.cue(t) {
            Some(c) => c.index,
            None if t < self.lead_in => 0,
            None => self.scenes.len() - 1,
        }
    }

    pub fn warmup_seconds(&self) -> f32 {
        self.warmup
            .unwrap_or_else(|| crate::warmup(&self.tool) * 0.5)
            .clamp(0.0, 60.0)
    }

    pub fn thumbnail_time(&self) -> f32 {
        self.thumbnail_at
            .unwrap_or_else(|| self.lead_in + self.scenes[0].seconds / 2.0)
            .clamp(0.0, self.total_seconds())
    }

    /// The middle of every scene: the frames of the contact sheet.
    pub fn midpoints(&self) -> Vec<f32> {
        self.starts()
            .iter()
            .zip(&self.scenes)
            .map(|(start, s)| start + s.seconds / 2.0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_scenes() -> Storyboard {
        Storyboard::parse(
            r#"{"tool": "lorenz", "scenes": [
                {"heading": "One", "caption": "First.", "seconds": 2.0, "state": {"rho": 15}},
                {"caption": "Second.", "seconds": 3.0, "equations": false}
            ]}"#,
        )
        .unwrap()
    }

    #[test]
    fn defaults_make_a_short() {
        let s = two_scenes();
        assert_eq!((s.width, s.height, s.fps, s.scale), (1080, 1920, 30, 2.0));
        assert_eq!(s.theme, Theme::Dark);
        assert!(!s.chrome);
        assert_eq!(s.lead_in, 0.5);
        assert_eq!(s.logical_size(), vec2(540.0, 960.0));
        assert!(s.title.is_none() && s.kicker.is_none());
        assert!(s.footer.contains("Formula"));
        assert!(s.scenes[0].equations);
        assert!(!s.scenes[1].equations);
        assert_eq!(s.scenes[0].state, Some(serde_json::json!({"rho": 15})));
        assert_eq!(s.scenes[1].heading, "");
        assert_eq!(s.warmup_seconds(), 3.0);
    }

    #[test]
    fn warmup_can_be_turned_off_and_is_capped() {
        let mut s = two_scenes();
        s.warmup = Some(0.0);
        assert_eq!(s.warmup_seconds(), 0.0);
        s.warmup = Some(1e9);
        assert_eq!(s.warmup_seconds(), 60.0);
    }

    #[test]
    fn timing() {
        let s = two_scenes();
        assert_eq!(s.total_seconds(), 5.5);
        assert_eq!(s.frames(), 165);
        assert_eq!(s.starts(), vec![0.5, 2.5]);
        assert_eq!(s.midpoints(), vec![1.5, 4.0]);
        assert_eq!(s.thumbnail_time(), 1.5);
        assert_eq!(s.cue(0.2), None);
        assert_eq!(
            s.cue(0.5),
            Some(Cue {
                index: 0,
                local: 0.0,
                remaining: 2.0
            })
        );
        let c = s.cue(4.0).unwrap();
        assert_eq!(c.index, 1);
        assert!((c.local - 1.5).abs() < 1e-6 && (c.remaining - 1.5).abs() < 1e-6);
        assert_eq!(s.cue(5.5), None);
    }

    #[test]
    fn active_scene_covers_the_lead_in_and_the_end() {
        let s = two_scenes();
        assert_eq!(s.active_scene(0.0), 0);
        assert_eq!(s.active_scene(1.0), 0);
        assert_eq!(s.active_scene(2.6), 1);
        assert_eq!(s.active_scene(100.0), 1);
    }

    #[test]
    fn thumbnail_at_is_clamped() {
        let mut s = two_scenes();
        s.thumbnail_at = Some(99.0);
        assert_eq!(s.thumbnail_time(), 5.5);
    }

    #[test]
    fn validation_messages() {
        let bad = |json: &str| Storyboard::parse(json).unwrap_err();
        assert!(bad(r#"{"tool": "nope", "scenes": [{"seconds": 1}]}"#).contains("unknown tool"));
        assert!(bad(r#"{"tool": "lerp", "scenes": []}"#).contains("at least one scene"));
        assert!(bad(r#"{"tool": "lerp", "scenes": [{"seconds": 0}]}"#).contains("scene 1"));
        assert!(
            bad(r#"{"tool": "lerp", "width": 1081, "scenes": [{"seconds": 1}]}"#).contains("even")
        );
        assert!(
            bad(r#"{"tool": "lerp", "width": 1082, "scale": 4, "scenes": [{"seconds": 1}]}"#)
                .contains("multiple of scale")
        );
        assert!(bad(r#"{"tool": "lerp", "fps": 0, "scenes": [{"seconds": 1}]}"#).contains("fps"));
        assert!(
            bad(r#"{"tool": "lerp", "lead_in": -1, "scenes": [{"seconds": 1}]}"#)
                .contains("lead_in")
        );
        assert!(bad(r#"{"scenes": []}"#).contains("storyboard"));
    }

    #[test]
    fn other_sizes_and_themes() {
        let s = Storyboard::parse(
            r#"{"tool": "lerp", "width": 1920, "height": 1080, "scale": 1.5, "theme": "light",
                "chrome": true, "scenes": [{"seconds": 1}]}"#,
        )
        .unwrap();
        assert_eq!(s.logical_size(), vec2(1280.0, 720.0));
        assert!(!s.theme.is_dark());
        assert!(s.chrome);
    }

    #[test]
    fn round_trips_through_json() {
        let s = two_scenes();
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(Storyboard::parse(&json).unwrap(), s);
    }
}
