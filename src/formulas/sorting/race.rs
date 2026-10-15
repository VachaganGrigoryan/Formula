//! Racing mode: several algorithms sort the same list side by side.
//!
//! Every lane advances at the same number of recorded steps per second, so
//! the lane with the fewest operations finishes first. Comparisons and moves
//! are tracked live; the measured time comes from actually running each
//! algorithm (without recording) on this machine.

use egui::{Align2, Color32, FontId, Rect, Stroke, pos2, vec2};

use super::algorithms::{Algorithm, State};
use super::skin::{self, Frame};
use super::{Colors, Session, SortVisualizer};
use crate::formula::{Formula, MathText, Palette, Tone};

const LANE_H: f32 = 92.0;
const HEADER_H: f32 = 34.0;

struct Lane {
    view: SortVisualizer,
    total_comparisons: usize,
    total_moves: usize,
    /// Wall-clock microseconds per sort, measured without recording.
    measured_us: f64,
}

impl Lane {
    fn new(alg: Algorithm, session: Session) -> Self {
        let mut view = SortVisualizer::new(alg, session);
        view.player.playing = false;
        let mut finished = State::new(&view.initial);
        for &op in &view.player.ops {
            finished.apply(op);
        }
        let measured_us = alg.time_us(&view.initial);
        Self {
            view,
            total_comparisons: finished.comparisons,
            total_moves: finished.swaps + finished.writes,
            measured_us,
        }
    }

    fn steps(&self) -> usize {
        self.view.player.ops.len()
    }
}

pub struct Race {
    session: Session,
    chosen: Vec<Algorithm>,
    lanes: Vec<Lane>,
    /// The dataset generation and lineup the lanes were built for.
    built: Option<(u64, Vec<Algorithm>)>,
    /// Seconds of race time elapsed.
    clock: f32,
    running: bool,
    /// Steps per second, the same for every lane.
    speed: f32,
}

impl Race {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            chosen: vec![
                Algorithm::Bubble,
                Algorithm::Insertion,
                Algorithm::Merge,
                Algorithm::Quick,
                Algorithm::Heap,
            ],
            lanes: Vec::new(),
            built: None,
            clock: 0.0,
            running: true,
            speed: 60.0,
        }
    }

    fn sync(&mut self) {
        let generation = self.session.data.borrow().generation();
        let key = (generation, self.chosen.clone());
        if self.built.as_ref() == Some(&key) {
            return;
        }
        self.lanes = self
            .chosen
            .iter()
            .map(|&alg| Lane::new(alg, self.session.clone()))
            .collect();
        // Pace the race so the slowest lane takes about fifteen seconds.
        let longest = self.lanes.iter().map(Lane::steps).max().unwrap_or(1);
        self.speed = (longest as f32 / 15.0).clamp(4.0, 5000.0);
        self.built = Some(key);
        self.restart();
    }

    fn restart(&mut self) {
        self.clock = 0.0;
        self.running = true;
        for lane in &mut self.lanes {
            lane.view.seek(0);
        }
    }

    fn finished(&self) -> bool {
        self.lanes.iter().all(|l| l.view.done())
    }

    /// Finishing place of each lane (1 = first), ties sharing a place.
    fn places(&self) -> Vec<usize> {
        self.lanes
            .iter()
            .map(|l| 1 + self.lanes.iter().filter(|o| o.steps() < l.steps()).count())
            .collect()
    }
}

fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

impl Formula for Race {
    fn id(&self) -> &'static str {
        "sorting-race"
    }

    fn name(&self) -> &'static str {
        "Sorting race"
    }

    fn category(&self) -> &'static str {
        super::super::SORTING
    }

    fn summary(&self) -> &'static str {
        "Pick algorithms and race them on the same list. Every lane runs at the same steps per \
         second, so fewer operations means an earlier finish. The bars beside each lane track \
         comparisons and moves live."
    }

    fn notes(&self) -> &'static str {
        "Counting operations shows how an algorithm scales, independent of the machine: try the \
         same race on Random, Nearly sorted and Reversed data and watch the order change. The \
         measured time is a real benchmark of each algorithm on this list, in microseconds per sort."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("steps")
                .tone(Tone::X)
                .t(" = comparisons + swaps + writes + bookkeeping"),
            MathText::new()
                .t("finish time")
                .tone(Tone::Y)
                .t(" = steps / speed"),
        ]
    }

    fn min_size(&self) -> egui::Vec2 {
        // The label column and the metric bars need the width.
        egui::vec2(480.0, self.stage_height(480.0))
    }

    fn stage_height(&self, _width: f32) -> f32 {
        HEADER_H + LANE_H * self.chosen.len().max(1) as f32 + 12.0
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.sync();
        if self.running {
            self.clock += dt;
            let k = (self.clock * self.speed) as usize;
            for lane in &mut self.lanes {
                lane.view.seek(k);
            }
            if self.finished() {
                self.running = false;
            }
        }

        let colors = Colors::new(palette);
        let inner = rect.shrink2(vec2(16.0, 10.0));
        let mono = FontId::monospace(12.0);
        let small = FontId::proportional(12.0);
        let n = self.session.data.borrow().values().len();
        painter.text(
            inner.left_top(),
            Align2::LEFT_TOP,
            format!(
                "{} lanes, same {n} values    {:.0} steps/s    race time {:.1} s",
                self.lanes.len(),
                self.speed,
                self.clock
            ),
            mono.clone(),
            palette.ink,
        );
        if self.lanes.is_empty() {
            painter.text(
                inner.center(),
                Align2::CENTER_CENTER,
                "pick at least one algorithm below",
                small,
                palette.muted,
            );
            return;
        }

        let max_cmp = self
            .lanes
            .iter()
            .map(|l| l.total_comparisons)
            .max()
            .unwrap_or(1)
            .max(1);
        let max_moves = self
            .lanes
            .iter()
            .map(|l| l.total_moves)
            .max()
            .unwrap_or(1)
            .max(1);
        let max_us = self
            .lanes
            .iter()
            .map(|l| l.measured_us)
            .fold(0.0, f64::max)
            .max(1e-9);
        let places = self.places();
        let skin = self.session.view.borrow().skin;

        for (row, lane) in self.lanes.iter().enumerate() {
            let top = inner.top() + HEADER_H + LANE_H * row as f32;
            let band = Rect::from_min_max(
                pos2(inner.left(), top),
                pos2(inner.right(), top + LANE_H - 8.0),
            );
            if row > 0 {
                painter.line_segment(
                    [pos2(band.left(), top - 4.0), pos2(band.right(), top - 4.0)],
                    Stroke::new(1.0, palette.rule),
                );
            }
            let v = &lane.view;
            let s = &v.player.state;
            let done = v.done();

            // Name, place and progress.
            let label_w = 150.0;
            painter.text(
                band.left_top(),
                Align2::LEFT_TOP,
                v.info.name,
                FontId::proportional(15.0),
                palette.ink,
            );
            let status = if done {
                format!("{} place", ordinal(places[row]))
            } else {
                format!(
                    "{:.0}%",
                    100.0 * v.player.cursor as f32 / v.player.ops.len().max(1) as f32
                )
            };
            let status_color = if done && places[row] == 1 {
                colors.sorted
            } else {
                palette.muted
            };
            painter.text(
                band.left_top() + vec2(0.0, 22.0),
                Align2::LEFT_TOP,
                status,
                small.clone(),
                status_color,
            );
            let finish = v.player.ops.len() as f32 / self.speed;
            painter.text(
                band.left_top() + vec2(0.0, 40.0),
                Align2::LEFT_TOP,
                format!("{:.1} s of {:.1} s", self.clock.min(finish), finish),
                small.clone(),
                palette.muted,
            );

            // The list itself, in the chosen skin.
            let mid_w = (band.width() - label_w) * 0.42;
            let array = Rect::from_min_max(
                pos2(band.left() + label_w, band.top()),
                pos2(band.left() + label_w + mid_w, band.bottom()),
            );
            let values: Vec<Option<u32>> = s.a.iter().map(|&x| Some(x)).collect();
            let max = v.initial.iter().copied().max().unwrap_or(1);
            let marks = |i: usize| v.mark(i, &colors);
            skin::draw(
                painter,
                array,
                skin,
                &Frame {
                    values: &values,
                    max,
                    marks: &marks,
                    idle: colors.idle,
                    empty: palette.rule,
                    ghost: None,
                    swapped: None,
                },
            );

            // Live metric bars, scaled to the largest final total in the race.
            let metrics = Rect::from_min_max(
                pos2(array.right() + 18.0, band.top() + 2.0),
                band.right_bottom(),
            );
            let rows: [(&str, f32, String, Color32); 3] = [
                (
                    "comparisons",
                    s.comparisons as f32 / max_cmp as f32,
                    s.comparisons.to_string(),
                    colors.compare,
                ),
                (
                    "swaps + writes",
                    (s.swaps + s.writes) as f32 / max_moves as f32,
                    (s.swaps + s.writes).to_string(),
                    colors.swap,
                ),
                (
                    "measured",
                    if done {
                        (lane.measured_us / max_us) as f32
                    } else {
                        0.0
                    },
                    if done {
                        format!("{:.1} us", lane.measured_us)
                    } else {
                        "...".to_owned()
                    },
                    colors.sorted,
                ),
            ];
            let row_h = metrics.height() / 3.0;
            for (k, (label, frac, text, color)) in rows.into_iter().enumerate() {
                let y = metrics.top() + row_h * k as f32;
                painter.text(
                    pos2(metrics.left(), y),
                    Align2::LEFT_TOP,
                    label,
                    small.clone(),
                    palette.muted,
                );
                let track = Rect::from_min_max(
                    pos2(metrics.left() + 100.0, y + 3.0),
                    pos2(metrics.right() - 64.0, y + row_h - 9.0),
                );
                painter.rect_filled(track, 3.0, palette.rule.gamma_multiply(0.5));
                let fill = Rect::from_min_max(
                    track.min,
                    pos2(
                        track.left() + track.width() * frac.clamp(0.0, 1.0),
                        track.bottom(),
                    ),
                );
                painter.rect_filled(fill, 3.0, color);
                painter.text(
                    pos2(metrics.right(), y),
                    Align2::RIGHT_TOP,
                    text,
                    mono.clone(),
                    palette.ink,
                );
            }
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Racers");
            for alg in Algorithm::ALL {
                let mut on = self.chosen.contains(&alg);
                if ui.checkbox(&mut on, super::info(alg).name).changed() {
                    if on {
                        self.chosen.push(alg);
                        // Keep lanes in the catalogue's order.
                        self.chosen
                            .sort_by_key(|a| Algorithm::ALL.iter().position(|b| b == a));
                    } else {
                        self.chosen.retain(|&a| a != alg);
                    }
                }
            }
        });
        ui.add_space(8.0);
        self.session.data_controls(ui, palette);
        ui.add_space(8.0);
        egui::Grid::new("race-view")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Draw as");
                ui.horizontal_wrapped(|ui| {
                    let mut view = self.session.view.borrow_mut();
                    for skin in skin::Skin::ALL {
                        ui.selectable_value(&mut view.skin, skin, skin.label());
                    }
                });
                ui.end_row();
                ui.label("Speed");
                ui.add(
                    egui::Slider::new(&mut self.speed, 1.0..=5000.0)
                        .logarithmic(true)
                        .suffix(" steps/s")
                        .custom_formatter(|v, _| format!("{v:.0}")),
                );
                ui.end_row();
            });
        ui.add_space(6.0);
        self.sync();
        ui.horizontal_wrapped(|ui| {
            let label = if self.running {
                "Pause"
            } else if self.finished() {
                "Race again"
            } else {
                "Go"
            };
            if ui.button(label).clicked() {
                if self.finished() {
                    self.restart();
                } else {
                    self.running = !self.running;
                }
            }
            if ui.button("Restart").clicked() {
                self.restart();
            }
            if ui.button("Finish").clicked() {
                self.clock =
                    self.lanes.iter().map(Lane::steps).max().unwrap_or(0) as f32 / self.speed;
                for lane in &mut self.lanes {
                    let end = lane.steps();
                    lane.view.seek(end);
                }
                self.running = false;
            }
        });
    }

    fn is_animating(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_follow_step_counts() {
        let mut race = Race::new(Session::new());
        race.sync();
        let places = race.places();
        let fastest = (0..race.lanes.len())
            .min_by_key(|&i| race.lanes[i].steps())
            .unwrap();
        assert_eq!(places[fastest], 1);
        let slowest = (0..race.lanes.len())
            .max_by_key(|&i| race.lanes[i].steps())
            .unwrap();
        assert_eq!(
            places[slowest],
            race.lanes
                .iter()
                .filter(|l| l.steps() < race.lanes[slowest].steps())
                .count()
                + 1
        );
    }

    #[test]
    fn lanes_rebuild_when_data_or_lineup_change() {
        let session = Session::new();
        let mut race = Race::new(session.clone());
        race.sync();
        assert_eq!(race.lanes.len(), 5);
        race.chosen.retain(|&a| a != Algorithm::Bubble);
        race.sync();
        assert_eq!(race.lanes.len(), 4);
        session.data.borrow_mut().shuffle();
        race.sync();
        assert_eq!(race.lanes[0].view.initial, session.data.borrow().values());
    }

    #[test]
    fn ordinals() {
        let got: Vec<String> = [1, 2, 3, 4, 11, 12, 13, 21, 22]
            .iter()
            .map(|&n| ordinal(n))
            .collect();
        assert_eq!(
            got,
            [
                "1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd"
            ]
        );
    }
}
