//! Step-through playback for algorithms that are recorded once and then
//! replayed: a log of operations applied to a state, at any speed, forward
//! or backward (backward replays from the start), like scrubbing a video.

/// A state rebuilt by applying recorded operations in order.
pub trait Replay: Clone {
    type Op: Copy;
    fn apply(&mut self, op: Self::Op);
}

/// Plays a recorded log of operations against a state.
pub struct Player<S: Replay> {
    /// The state after `cursor` operations.
    pub state: S,
    pub ops: Vec<S::Op>,
    /// Number of ops applied to `state`.
    pub cursor: usize,
    pub playing: bool,
    /// Steps per second.
    pub speed: f32,
    initial: S,
    /// Fractional steps carried between frames.
    carry: f32,
}

impl<S: Replay> Player<S> {
    pub fn new(initial: S, ops: Vec<S::Op>, speed: f32) -> Self {
        Self {
            state: initial.clone(),
            ops,
            cursor: 0,
            playing: true,
            speed,
            initial,
            carry: 0.0,
        }
    }

    /// Start over with a new recording, keeping the speed.
    pub fn reset(&mut self, initial: S, ops: Vec<S::Op>) {
        *self = Self::new(initial, ops, self.speed);
    }

    /// Move to step `k`, replaying from the start when going backwards.
    pub fn seek(&mut self, k: usize) {
        let k = k.min(self.ops.len());
        if k < self.cursor {
            self.state = self.initial.clone();
            self.cursor = 0;
        }
        while self.cursor < k {
            self.state.apply(self.ops[self.cursor]);
            self.cursor += 1;
        }
    }

    /// Play forward by `dt` seconds. Returns how many steps were applied.
    pub fn advance(&mut self, dt: f32) -> usize {
        if !self.playing || self.done() {
            self.playing = false;
            return 0;
        }
        self.carry += self.speed * dt;
        let steps = (self.carry.floor() as usize).min(self.ops.len() - self.cursor);
        self.carry -= self.carry.floor();
        self.seek(self.cursor + steps);
        if self.done() {
            self.playing = false;
        }
        steps
    }

    pub fn done(&self) -> bool {
        self.cursor >= self.ops.len()
    }

    /// Play or pause; when finished, replay from the start.
    pub fn toggle(&mut self) {
        if self.done() {
            self.seek(0);
            self.playing = true;
        } else {
            self.playing = !self.playing;
        }
    }
}

/// Speed and step sliders, then Restart, Step back, Play/Pause, Step, Finish.
pub fn transport(ui: &mut egui::Ui, p: &mut Player<impl Replay>, max_speed: f32) {
    egui::Grid::new(ui.id().with("transport"))
        .num_columns(2)
        .spacing([12.0, 10.0])
        .show(ui, |ui| {
            ui.label("Speed");
            ui.add(
                egui::Slider::new(&mut p.speed, 1.0..=max_speed)
                    .logarithmic(true)
                    .suffix(" steps/s")
                    .custom_formatter(|v, _| format!("{v:.0}")),
            );
            ui.end_row();
            ui.label("Step");
            let mut k = p.cursor;
            if ui.add(egui::Slider::new(&mut k, 0..=p.ops.len())).changed() {
                p.playing = false;
                p.seek(k);
            }
            ui.end_row();
        });
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        if ui.button("Restart").clicked() {
            p.seek(0);
            p.playing = true;
        }
        if ui
            .add_enabled(p.cursor > 0, egui::Button::new("Step back"))
            .clicked()
        {
            p.playing = false;
            p.seek(p.cursor - 1);
        }
        let label = if p.playing {
            "Pause"
        } else if p.done() {
            "Replay"
        } else {
            "Play"
        };
        if ui.button(label).clicked() {
            p.toggle();
        }
        if ui
            .add_enabled(!p.done(), egui::Button::new("Step"))
            .clicked()
        {
            p.playing = false;
            p.seek(p.cursor + 1);
        }
        if ui
            .add_enabled(!p.done(), egui::Button::new("Finish"))
            .clicked()
        {
            p.playing = false;
            p.seek(p.ops.len());
        }
    });
}

/// Space plays or pauses, Left and Right step. Keys are consumed only when used.
pub fn transport_keys<S: Replay>(ctx: &egui::Context, p: &mut Player<S>) {
    let none = egui::Modifiers::NONE;
    if ctx.input_mut(|i| i.consume_key(none, egui::Key::Space)) {
        p.toggle();
    }
    if p.cursor > 0 && ctx.input_mut(|i| i.consume_key(none, egui::Key::ArrowLeft)) {
        p.playing = false;
        p.seek(p.cursor - 1);
    }
    if !p.done() && ctx.input_mut(|i| i.consume_key(none, egui::Key::ArrowRight)) {
        p.playing = false;
        p.seek(p.cursor + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A toy state: the sum of the ops applied so far.
    #[derive(Clone, Debug, Default, PartialEq)]
    struct Sum(i32);

    impl Replay for Sum {
        type Op = i32;
        fn apply(&mut self, op: i32) {
            self.0 += op;
        }
    }

    fn player() -> Player<Sum> {
        Player::new(Sum(0), vec![1, 2, 3, 4], 2.0)
    }

    #[test]
    fn seeking_backwards_matches_seeking_forwards() {
        let mut p = player();
        p.seek(3);
        assert_eq!(p.state, Sum(6));
        p.seek(1);
        assert_eq!((p.state.clone(), p.cursor), (Sum(1), 1));
        p.seek(99);
        assert_eq!(
            (p.state.clone(), p.cursor),
            (Sum(10), 4),
            "clamped to the end"
        );
    }

    #[test]
    fn advance_carries_fractional_steps_between_frames() {
        let mut p = player(); // 2 steps per second
        assert_eq!(p.advance(0.3), 0);
        assert_eq!(p.advance(0.3), 1);
        assert_eq!(p.advance(0.3), 0);
        assert_eq!(p.cursor, 1);
    }

    #[test]
    fn advance_never_overshoots_and_stops_at_the_end() {
        let mut p = player();
        assert_eq!(p.advance(100.0), 4);
        assert!(p.done() && !p.playing);
        assert_eq!(p.advance(1.0), 0);
        assert_eq!(p.state, Sum(10));
    }

    #[test]
    fn paused_players_do_not_move() {
        let mut p = player();
        p.playing = false;
        assert_eq!(p.advance(10.0), 0);
        assert_eq!(p.cursor, 0);
    }

    #[test]
    fn toggle_replays_from_the_start_when_done() {
        let mut p = player();
        p.seek(4);
        p.playing = false;
        p.toggle();
        assert_eq!((p.cursor, p.playing), (0, true));
        p.toggle();
        assert!(!p.playing);
    }

    #[test]
    fn reset_keeps_the_speed() {
        let mut p = player();
        p.speed = 7.0;
        p.seek(2);
        p.reset(Sum(100), vec![5]);
        assert_eq!((p.cursor, p.speed, p.state.clone()), (0, 7.0, Sum(100)));
        assert!(p.playing);
    }

    #[test]
    fn keys_play_pause_and_step() {
        let ctx = egui::Context::default();
        let mut p = player();
        p.playing = false;
        let key = |k: egui::Key| egui::Event::Key {
            key: k,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        let press = |p: &mut Player<Sum>, k| {
            let input = egui::RawInput {
                events: vec![key(k)],
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| transport_keys(ui.ctx(), p));
            out.textures_delta.clear();
        };
        press(&mut p, egui::Key::ArrowRight);
        assert_eq!(p.cursor, 1);
        press(&mut p, egui::Key::ArrowLeft);
        assert_eq!(p.cursor, 0);
        press(&mut p, egui::Key::Space);
        assert!(p.playing);
    }
}
