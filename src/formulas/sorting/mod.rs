//! Sorting algorithms, visualized by replaying every step they take.
//!
//! All sorting viewports share one [`data::Dataset`], so a shuffle in one
//! shows up in every other and algorithms can be compared on the same input.

pub mod algorithms;
pub mod data;
pub mod race;
pub mod skin;

use std::cell::RefCell;
use std::rc::Rc;

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, pos2, vec2};

use crate::audio::{self, Audio};
use crate::formula::{Formula, MathText, Palette, Tone, fit_text};
use algorithms::{Algorithm, Op, State};
use data::{Distribution, Shared};
use skin::{Mark, Skin};

/// View settings shared by every sorting viewport.
pub struct View {
    pub skin: Skin,
}

/// Everything the sorting viewports share: the list, how it is drawn, and the
/// one audio output.
#[derive(Clone)]
pub struct Session {
    pub data: Shared,
    pub view: Rc<RefCell<View>>,
    pub audio: Rc<RefCell<Audio>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            data: data::shared(),
            view: Rc::new(RefCell::new(View { skin: Skin::Bars })),
            audio: Rc::new(RefCell::new(Audio::default())),
        }
    }

    /// The list controls: size, distribution and the randomizer.
    pub fn data_controls(&self, ui: &mut egui::Ui, palette: &Palette) {
        let mut data = self.data.borrow_mut();
        let mut changed = false;
        egui::Grid::new("sort-data")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("List size");
                changed |= ui.add(egui::Slider::new(&mut data.n, 5..=200)).changed();
                ui.end_row();
                ui.label("Data");
                ui.horizontal_wrapped(|ui| {
                    for d in Distribution::ALL {
                        if ui.selectable_label(data.dist == d, d.label()).clicked() {
                            data.dist = d;
                            changed = true;
                        }
                    }
                });
                ui.end_row();
            });
        if changed {
            data.regenerate();
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Randomize list").clicked() {
                data.shuffle();
            }
            ui.label(
                egui::RichText::new("shared by every sorting viewport")
                    .small()
                    .color(palette.muted),
            );
        });
    }

    /// The skin picker and sound switch.
    pub fn view_controls(&self, ui: &mut egui::Ui, palette: &Palette) {
        egui::Grid::new("sort-view")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Draw as");
                ui.horizontal_wrapped(|ui| {
                    let mut view = self.view.borrow_mut();
                    for skin in Skin::ALL {
                        ui.selectable_value(&mut view.skin, skin, skin.label());
                    }
                });
                ui.end_row();
                ui.label("Sound");
                ui.horizontal_wrapped(|ui| {
                    let mut audio = self.audio.borrow_mut();
                    let mut on = audio.enabled();
                    let can = Audio::available();
                    if ui
                        .add_enabled(can, egui::Checkbox::new(&mut on, "Play tones"))
                        .changed()
                    {
                        audio.set_enabled(on);
                    }
                    if audio.enabled() {
                        ui.add(
                            egui::Slider::new(&mut audio.volume, 0.0..=1.0)
                                .show_value(false)
                                .text("volume"),
                        );
                    }
                    if let Some(why) = audio.unavailable_reason() {
                        ui.label(egui::RichText::new(why).small().color(palette.muted));
                    } else {
                        ui.label(
                            egui::RichText::new(format!(
                                "pitch = {:.0} + value \u{b7} {:.0} Hz",
                                audio::LOW_HZ,
                                audio::HIGH_HZ - audio::LOW_HZ
                            ))
                            .small()
                            .color(palette.muted),
                        );
                    }
                });
                ui.end_row();
            });
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Colors for the states a bar can be in.
#[derive(Clone, Copy)]
struct Colors {
    idle: Color32,
    compare: Color32,
    swap: Color32,
    sorted: Color32,
    min: Color32,
    pivot: Color32,
    faded: Color32,
}

impl Colors {
    fn new(p: &Palette) -> Self {
        let dark = p.paper.r() < 128;
        let pick = |light: [u8; 3], night: [u8; 3]| {
            let c = if dark { night } else { light };
            Color32::from_rgb(c[0], c[1], c[2])
        };
        Self {
            idle: p.trace,
            compare: pick([214, 69, 65], [255, 107, 107]),
            swap: p.y,
            sorted: pick([46, 158, 79], [92, 203, 122]),
            min: pick([142, 91, 214], [180, 140, 255]),
            pivot: pick([224, 86, 155], [255, 122, 184]),
            faded: p.trace.gamma_multiply(0.3),
        }
    }

    fn of(&self, mark: Mark) -> Color32 {
        match mark {
            Mark::Idle => self.idle,
            Mark::Faded => self.faded,
            Mark::Sorted(c) | Mark::Active(c) => c,
        }
    }

    fn legend(&self, alg: Algorithm) -> Vec<(&'static str, Color32)> {
        let mut v = if alg == Algorithm::Counting {
            vec![
                ("unsorted", self.idle),
                ("reading a value", self.compare),
                ("counting / placing", self.swap),
            ]
        } else {
            vec![
                ("unsorted", self.idle),
                ("comparing", self.compare),
                ("swapping / writing", self.swap),
            ]
        };
        match alg {
            Algorithm::Selection => v.push(("current minimum", self.min)),
            Algorithm::Insertion => v.push(("key being inserted", self.min)),
            Algorithm::Quick | Algorithm::Quick3 => v.push(("pivot", self.pivot)),
            _ => {}
        }
        v.push(("in final place", self.sorted));
        v
    }
}

struct Info {
    id: &'static str,
    name: &'static str,
    summary: &'static str,
    notes: &'static str,
    time: &'static str,
    space: &'static str,
    code: &'static str,
}

fn info(alg: Algorithm) -> Info {
    match alg {
        Algorithm::Bubble => Info {
            id: "bubble-sort",
            name: "Bubble sort",
            summary: "Walk the list comparing neighbours and swap any pair that is out of order. \
                      Each pass carries the largest remaining value to the end, where it turns green.",
            notes: "A teaching algorithm: simple and stable, but quadratic. It stops early when a \
                    pass makes no swaps, so already-sorted input takes a single pass. Watch the big \
                    values race right while small ones creep left one step per pass.",
            time: "O(n²) worst and average, O(n) best",
            space: "O(1)",
            code: "def bubble_sort(arr):\n    n = len(arr)\n    for i in range(n):\n        swapped = False\n        for j in range(0, n - i - 1):\n            if arr[j] > arr[j + 1]:\n                arr[j], arr[j + 1] = arr[j + 1], arr[j]\n                swapped = True\n        if not swapped:\n            break\n    return arr",
        },
        Algorithm::Selection => Info {
            id: "selection-sort",
            name: "Selection sort",
            summary: "Scan the unsorted part for its smallest value (purple), then swap it to the \
                      front. The sorted region grows from the left by one bar per scan.",
            notes: "Always makes n(n − 1)/2 comparisons, whatever the input, but at most n − 1 swaps, \
                    which matters when writes are expensive. Not stable.",
            time: "O(n²) in every case",
            space: "O(1)",
            code: "def selection_sort(arr):\n    n = len(arr)\n    for i in range(n):\n        min_idx = i\n        for j in range(i + 1, n):\n            if arr[j] < arr[min_idx]:\n                min_idx = j\n        arr[i], arr[min_idx] = arr[min_idx], arr[i]\n    return arr",
        },
        Algorithm::Insertion => Info {
            id: "insertion-sort",
            name: "Insertion sort",
            summary: "Lift the next value out (the purple outline), shift bigger values in the sorted \
                      left part one step right, and drop it into the gap. The faded bars are not yet sorted.",
            notes: "How people sort a hand of cards. Fast on small or nearly-sorted lists (try \
                    \"Nearly sorted\"), stable, and works in place; hybrid sorts such as Timsort use \
                    it for short runs.",
            time: "O(n²) worst and average, O(n) best",
            space: "O(1)",
            code: "def insertion_sort(arr):\n    for i in range(1, len(arr)):\n        key = arr[i]\n        j = i - 1\n        while j >= 0 and key < arr[j]:\n            arr[j + 1] = arr[j]\n            j -= 1\n        arr[j + 1] = key\n    return arr",
        },
        Algorithm::Merge => Info {
            id: "merge-sort",
            name: "Merge sort",
            summary: "Split the list in half until each piece has one value, then merge sorted halves \
                      back together, overwriting the array left to right. The rows below show \
                      the list splitting level by level; the bright one is the merge in progress.",
            notes: "Guaranteed n log n and stable, at the cost of a second buffer the size of the \
                    input. The basis of external sorting (data larger than memory) and of Timsort.",
            time: "O(n log n) in every case",
            space: "O(n)",
            code: "def merge_sort(arr):\n    if len(arr) > 1:\n        mid = len(arr) // 2\n        L = arr[:mid]\n        R = arr[mid:]\n        merge_sort(L)\n        merge_sort(R)\n        i = j = k = 0\n        while i < len(L) and j < len(R):\n            if L[i] <= R[j]:\n                arr[k] = L[i]; i += 1\n            else:\n                arr[k] = R[j]; j += 1\n            k += 1\n        while i < len(L):\n            arr[k] = L[i]; i += 1; k += 1\n        while j < len(R):\n            arr[k] = R[j]; j += 1; k += 1\n    return arr",
        },
        Algorithm::Quick => Info {
            id: "quick-sort",
            name: "Quick sort",
            summary: "Pick the last value as pivot (pink), move everything not larger to its left, and \
                      drop the pivot into its final place. Then do the same to each side.",
            notes: "Usually the fastest in practice and sorts in place, but a bad pivot choice makes it \
                    quadratic: try \"Reversed\" or \"Nearly sorted\" input with this last-element pivot. \
                    Library versions pick pivots more carefully.",
            time: "O(n log n) average, O(n²) worst",
            space: "O(log n)",
            code: "def quick_sort(arr, low, high):\n    if low < high:\n        pi = partition(arr, low, high)\n        quick_sort(arr, low, pi - 1)\n        quick_sort(arr, pi + 1, high)\n\ndef partition(arr, low, high):\n    pivot = arr[high]\n    i = low - 1\n    for j in range(low, high):\n        if arr[j] <= pivot:\n            i += 1\n            arr[i], arr[j] = arr[j], arr[i]\n    arr[i + 1], arr[high] = arr[high], arr[i + 1]\n    return i + 1",
        },
        Algorithm::Quick3 => Info {
            id: "quick-sort-3way",
            name: "Quick sort (3-way)",
            summary: "Split around the pivot into three bands: smaller, equal and larger. Every copy \
                      of the pivot value lands in its final place in one pass. Try it on \
                      \"Few unique\" data next to the 2-way version.",
            notes: "Dijkstra's \"Dutch national flag\" partition. Lists with many duplicates are \
                    common in practice (categories, ratings, flags), and on them the plain 2-way \
                    quick sort keeps re-partitioning equal values, while 3-way finishes them all \
                    at once. Library sorts use the same idea.",
            time: "O(n log n) average, close to O(n) with few distinct values",
            space: "O(log n)",
            code: "def quick_sort_3way(arr, lo, hi):\n    if lo >= hi:\n        return\n    pivot = arr[lo]\n    lt, i, gt = lo, lo + 1, hi\n    while i <= gt:\n        if arr[i] < pivot:\n            arr[lt], arr[i] = arr[i], arr[lt]\n            lt += 1; i += 1\n        elif arr[i] > pivot:\n            arr[i], arr[gt] = arr[gt], arr[i]\n            gt -= 1\n        else:\n            i += 1\n    # arr[lt..gt] all equal the pivot\n    quick_sort_3way(arr, lo, lt - 1)\n    quick_sort_3way(arr, gt + 1, hi)",
        },
        Algorithm::Heap => Info {
            id: "heap-sort",
            name: "Heap sort",
            summary: "Rearrange the list into a max-heap, where every parent is at least as big as its \
                      children, so the largest value sits at the root. Swap the root to the end, shrink \
                      the heap, repair it, repeat.",
            notes: "n log n in every case with no extra memory, but it jumps around the array, so it is \
                    cache-unfriendly and not stable. Heaps also power priority queues. Below about 64 \
                    values the tree view shows the heap itself.",
            time: "O(n log n) in every case",
            space: "O(1)",
            code: "def heapify(arr, n, i):\n    largest = i\n    l = 2 * i + 1\n    r = 2 * i + 2\n    if l < n and arr[l] > arr[largest]:\n        largest = l\n    if r < n and arr[r] > arr[largest]:\n        largest = r\n    if largest != i:\n        arr[i], arr[largest] = arr[largest], arr[i]\n        heapify(arr, n, largest)\n\ndef heap_sort(arr):\n    n = len(arr)\n    for i in range(n // 2 - 1, -1, -1):\n        heapify(arr, n, i)\n    for i in range(n - 1, 0, -1):\n        arr[i], arr[0] = arr[0], arr[i]\n        heapify(arr, i, 0)\n    return arr",
        },
        Algorithm::Counting => Info {
            id: "counting-sort",
            name: "Counting sort",
            summary: "Never compares two values. Tally how often each value occurs (middle), turn the \
                      tallies into end positions with a running sum, then place every value straight \
                      into the output (bottom).",
            notes: "Linear time when the range of values k is small compared with n, as with ages, \
                    grades or bytes. Walking the input backwards keeps equal values in order, which \
                    radix sort relies on.",
            time: "O(n + k), k = range of values",
            space: "O(n + k)",
            code: "def counting_sort(arr):\n    max_val = max(arr)\n    count = [0] * (max_val + 1)\n    output = [0] * len(arr)\n    for num in arr:\n        count[num] += 1\n    for i in range(1, len(count)):\n        count[i] += count[i - 1]\n    for num in reversed(arr):\n        output[count[num] - 1] = num\n        count[num] -= 1\n    return output",
        },
    }
}

pub struct SortVisualizer {
    alg: Algorithm,
    info: Info,
    session: Session,
    /// The dataset generation the recording was made from.
    generation: u64,
    initial: Vec<u32>,
    ops: Vec<Op>,
    state: State,
    /// Number of ops applied to `state`.
    cursor: usize,
    playing: bool,
    /// Steps per second.
    speed: f32,
    /// Fractional steps carried between frames.
    carry: f32,
    /// Seconds until another tone may play.
    tone_cooldown: f32,
    /// The closing sweep across the sorted list, in seconds since it began.
    sweep: Option<f32>,
}

/// How long the closing sweep takes.
const SWEEP_SECONDS: f32 = 1.2;

impl SortVisualizer {
    pub fn new(alg: Algorithm, session: Session) -> Self {
        let mut s = Self {
            alg,
            info: info(alg),
            session,
            generation: u64::MAX,
            initial: Vec::new(),
            ops: Vec::new(),
            state: State::default(),
            cursor: 0,
            playing: true,
            speed: 30.0,
            carry: 0.0,
            tone_cooldown: 0.0,
            sweep: None,
        };
        s.sync();
        s
    }

    /// Re-record if the shared data changed since the last recording.
    fn sync(&mut self) {
        let data = self.session.data.borrow();
        if data.generation() == self.generation {
            return;
        }
        self.generation = data.generation();
        self.initial = data.values().to_vec();
        drop(data);
        self.ops = self.alg.record(&self.initial).ops;
        self.state = State::new(&self.initial);
        self.cursor = 0;
        self.carry = 0.0;
        self.sweep = None;
        self.playing = true;
        // Aim for a run of about fifteen seconds at the default speed.
        self.speed = (self.ops.len() as f32 / 15.0).clamp(4.0, 3000.0);
    }

    /// Move to step `k`, replaying from the start when going backwards.
    fn seek(&mut self, k: usize) {
        let k = k.min(self.ops.len());
        if k < self.cursor {
            self.state = State::new(&self.initial);
            self.cursor = 0;
        }
        while self.cursor < k {
            self.state.apply(self.ops[self.cursor]);
            self.cursor += 1;
        }
    }

    fn done(&self) -> bool {
        self.cursor >= self.ops.len()
    }

    fn uses_tree(&self) -> bool {
        self.alg == Algorithm::Heap && self.initial.len() <= 63
    }

    /// The state of slot `i` in the main array.
    fn mark(&self, i: usize, c: &Colors) -> Mark {
        if let Some(t) = self.sweep {
            // The closing sweep: a bright front running left to right.
            let front = (t / SWEEP_SECONDS * self.initial.len() as f32) as usize;
            if i == front {
                return Mark::Active(c.swap);
            }
            return Mark::Sorted(c.sorted);
        }
        let s = &self.state;
        let active = match s.last {
            Some(Op::Swap(a, b)) if i == a || i == b => Some(c.swap),
            Some(Op::Write(a, _)) if i == a => Some(c.swap),
            Some(Op::Place { from, .. }) if i == from => Some(c.swap),
            Some(Op::Compare(a, b)) if i == a || i == b => Some(c.compare),
            Some(Op::CompareKey(a)) | Some(Op::Count(a)) if i == a => Some(c.compare),
            _ => None,
        };
        if let Some(col) = active {
            return Mark::Active(col);
        }
        if s.pivot == Some(i) {
            return Mark::Active(c.pivot);
        }
        if s.min == Some(i) {
            return Mark::Active(c.min);
        }
        if s.sorted.get(i).copied().unwrap_or(false) {
            return Mark::Sorted(c.sorted);
        }
        if let Some((lo, hi, _)) = s.stack.last().copied().or(s.range)
            && (i < lo || i >= hi)
        {
            return Mark::Faded;
        }
        if let Some(h) = s.heap_size
            && i >= h
        {
            return Mark::Faded;
        }
        Mark::Idle
    }

    fn bar_color(&self, i: usize, c: &Colors) -> Color32 {
        c.of(self.mark(i, c))
    }

    /// The value an op touches, for choosing a pitch.
    fn sounded_value(&self, op: Op) -> Option<u32> {
        let a = &self.state.a;
        match op {
            Op::Compare(i, _) | Op::Swap(i, _) | Op::Count(i) | Op::Place { from: i, .. } => {
                a.get(i).copied()
            }
            Op::Write(_, v) => Some(v),
            Op::CompareKey(_) => self.state.key.map(|(k, _)| k),
            _ => None,
        }
    }

    /// Play the pitch of a value, at most about forty tones a second.
    fn sound(&mut self, value: u32, seconds: f32) {
        if self.tone_cooldown > 0.0 {
            return;
        }
        let max = self.initial.iter().copied().max().unwrap_or(1).max(1);
        let hz = audio::frequency(value as f32 / max as f32);
        self.session.audio.borrow_mut().tone(hz, seconds);
        self.tone_cooldown = 1.0 / 40.0;
    }
}

/// Draw `values` as bars in `area`; `None` leaves an empty slot.
fn bars(
    painter: &egui::Painter,
    area: Rect,
    values: &[Option<u32>],
    max: u32,
    color: impl Fn(usize) -> Color32,
    empty: Color32,
) {
    let n = values.len().max(1);
    let slot = area.width() / n as f32;
    let gap = if slot > 6.0 {
        slot * 0.18
    } else if slot > 3.0 {
        1.0
    } else {
        0.0
    };
    for (i, v) in values.iter().enumerate() {
        let x0 = area.left() + slot * i as f32 + gap / 2.0;
        let x1 = x0 + (slot - gap).max(1.0);
        match v {
            Some(v) => {
                let h = (*v as f32 / max.max(1) as f32 * area.height()).max(2.0);
                painter.rect_filled(
                    Rect::from_min_max(pos2(x0, area.bottom() - h), pos2(x1, area.bottom())),
                    1.5,
                    color(i),
                );
            }
            None => {
                painter.rect_filled(
                    Rect::from_min_max(pos2(x0, area.bottom() - 2.0), pos2(x1, area.bottom())),
                    0.0,
                    empty,
                );
            }
        }
    }
}

fn slot_x(area: Rect, n: usize, i: f32) -> f32 {
    area.left() + area.width() / n.max(1) as f32 * i
}

impl Formula for SortVisualizer {
    fn id(&self) -> &'static str {
        self.info.id
    }

    fn name(&self) -> &'static str {
        self.info.name
    }

    fn category(&self) -> &'static str {
        super::SORTING
    }

    fn summary(&self) -> &'static str {
        self.info.summary
    }

    fn notes(&self) -> &'static str {
        self.info.notes
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new()
                .t("Time")
                .tone(Tone::X)
                .t("   ")
                .t(self.info.time),
            MathText::new()
                .t("Space")
                .tone(Tone::Y)
                .t("   ")
                .t(self.info.space),
        ]
    }

    fn stage_height(&self, width: f32) -> f32 {
        let base = (width * 0.5).max(260.0);
        match self.alg {
            Algorithm::Counting => base + 190.0,
            Algorithm::Heap if self.uses_tree() => base + 170.0,
            _ => base + 52.0,
        }
    }

    fn min_size(&self) -> egui::Vec2 {
        // The step / comparisons / swaps / writes line needs the width.
        egui::vec2(480.0, self.stage_height(480.0))
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.sync();
        self.tone_cooldown -= dt;
        if self.playing && !self.done() {
            self.carry += self.speed * dt;
            let steps = self.carry.floor() as usize;
            self.carry -= steps as f32;
            self.seek(self.cursor + steps);
            if steps > 0
                && let Some(v) = self.state.last.and_then(|op| self.sounded_value(op))
            {
                self.sound(v, (2.0 / self.speed).clamp(0.05, 0.18));
            }
            if self.done() && !self.initial.is_empty() {
                self.sweep = Some(0.0);
            }
        }
        if self.done() {
            self.playing = false;
        }
        if let Some(t) = self.sweep {
            let n = self.initial.len();
            let before = (t / SWEEP_SECONDS * n as f32) as usize;
            let t = t + dt;
            let after = (t / SWEEP_SECONDS * n as f32) as usize;
            if after != before && after < n {
                self.sound(self.state.a[after], 0.08);
            }
            self.sweep = (after <= n).then_some(t);
        }

        let colors = Colors::new(palette);
        let n = self.initial.len();
        let max = self.initial.iter().copied().max().unwrap_or(1);
        let inner = rect.shrink2(vec2(18.0, 14.0));
        let s = &self.state;

        let mut fields = vec![
            format!("step {} / {}", self.cursor, self.ops.len()),
            format!("comparisons {}", s.comparisons),
            format!("swaps {}", s.swaps),
            format!("writes {}", s.writes),
        ];
        if !s.stack.is_empty() {
            fields.push(format!("recursion depth {}", s.stack.len()));
        }
        let status = fit_line(painter, &fields, inner.width(), 12.5, palette.ink);
        painter.galley(inner.left_top(), status, palette.ink);

        let extra = match self.alg {
            Algorithm::Counting => 190.0,
            Algorithm::Heap if self.uses_tree() => 170.0,
            _ => 52.0,
        };
        let main = Rect::from_min_max(
            pos2(inner.left(), inner.top() + 26.0),
            pos2(inner.right(), inner.bottom() - extra),
        );

        // Main array; insertion sort's hole shows the lifted key instead.
        let skin = self.session.view.borrow().skin;
        let mut values: Vec<Option<u32>> = s.a.iter().map(|&v| Some(v)).collect();
        let ghost = s.key.map(|(key, hole)| {
            values[hole] = None;
            let edge = if matches!(s.last, Some(Op::CompareKey(_))) {
                colors.compare
            } else {
                colors.min
            };
            (hole, key, edge)
        });
        let swapped = match s.last {
            Some(Op::Swap(a, b)) => Some((a, b, colors.swap)),
            _ => None,
        };
        let marks = |i: usize| self.mark(i, &colors);
        skin::draw(
            painter,
            main,
            skin,
            &skin::Frame {
                values: &values,
                max,
                marks: &marks,
                idle: colors.idle,
                empty: palette.rule,
                ghost,
                swapped,
            },
        );

        // One bracket row per open recursive call, outermost on top: the list
        // visibly splitting into smaller pieces. The innermost call is
        // highlighted, with merge sort's two halves in two colors.
        let rows: Vec<(usize, usize, Option<usize>)> = match (s.stack.is_empty(), s.range) {
            (true, Some(r)) => vec![r],
            _ => s.stack.clone(),
        };
        let max_rows = if skin == Skin::Radial { 0 } else { 7 };
        let skip = rows.len().saturating_sub(max_rows);
        for (depth, &(lo, hi, mid)) in rows.iter().enumerate().skip(skip) {
            let y = main.bottom() + 7.0 + 5.0 * (depth - skip) as f32;
            let (x0, x1) = (
                slot_x(main, n, lo as f32) + 1.0,
                slot_x(main, n, hi as f32) - 1.0,
            );
            if depth + 1 < rows.len() {
                painter.line_segment(
                    [pos2(x0, y), pos2(x1, y)],
                    Stroke::new(2.0, palette.muted.gamma_multiply(0.5)),
                );
                continue;
            }
            match mid {
                Some(m) => {
                    let xm = slot_x(main, n, m as f32);
                    painter.line_segment(
                        [pos2(x0, y), pos2(xm - 1.5, y)],
                        Stroke::new(3.0, palette.x),
                    );
                    painter.line_segment(
                        [pos2(xm + 1.5, y), pos2(x1, y)],
                        Stroke::new(3.0, palette.y),
                    );
                }
                None => {
                    painter.line_segment(
                        [pos2(x0, y), pos2(x1, y)],
                        Stroke::new(3.0, palette.ink.gamma_multiply(0.7)),
                    );
                }
            }
        }
        match self.alg {
            Algorithm::Counting => self.paint_counting(painter, main, inner, max, &colors, palette),
            Algorithm::Heap if self.uses_tree() => self.paint_tree(
                painter,
                Rect::from_min_max(pos2(inner.left(), main.bottom() + 24.0), inner.max),
                &colors,
                palette,
            ),
            _ => {}
        }

        if self.done() && n > 0 {
            painter.text(
                inner.right_top(),
                Align2::RIGHT_TOP,
                "sorted",
                FontId::proportional(14.0),
                colors.sorted,
            );
        }
    }

    /// Space plays or pauses; the arrow keys step back and forward.
    fn handle_keys(&mut self, ctx: &egui::Context) {
        let none = egui::Modifiers::NONE;
        let (space, left, right) = ctx.input_mut(|i| {
            (
                i.consume_key(none, egui::Key::Space),
                i.consume_key(none, egui::Key::ArrowLeft),
                i.consume_key(none, egui::Key::ArrowRight),
            )
        });
        if space {
            if self.done() {
                self.seek(0);
                self.playing = true;
            } else {
                self.playing = !self.playing;
            }
        }
        if left && self.cursor > 0 {
            self.playing = false;
            self.seek(self.cursor - 1);
        }
        if right && !self.done() {
            self.playing = false;
            self.seek(self.cursor + 1);
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        self.session.data_controls(ui, palette);
        self.sync();
        ui.add_space(10.0);
        self.session.view_controls(ui, palette);

        // The player.
        ui.add_space(10.0);
        egui::Grid::new("sort-player")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Speed");
                ui.add(
                    egui::Slider::new(&mut self.speed, 1.0..=5000.0)
                        .logarithmic(true)
                        .suffix(" steps/s")
                        .custom_formatter(|v, _| format!("{v:.0}")),
                );
                ui.end_row();
                ui.label("Step");
                let mut k = self.cursor;
                if ui
                    .add(egui::Slider::new(&mut k, 0..=self.ops.len()))
                    .changed()
                {
                    self.playing = false;
                    self.seek(k);
                }
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Restart").clicked() {
                self.seek(0);
                self.playing = true;
            }
            if ui
                .add_enabled(self.cursor > 0, egui::Button::new("Step back"))
                .clicked()
            {
                self.playing = false;
                self.seek(self.cursor - 1);
            }
            let label = if self.playing {
                "Pause"
            } else if self.done() {
                "Replay"
            } else {
                "Play"
            };
            if ui.button(label).clicked() {
                if self.done() {
                    self.seek(0);
                    self.playing = true;
                } else {
                    self.playing = !self.playing;
                }
            }
            if ui
                .add_enabled(!self.done(), egui::Button::new("Step"))
                .clicked()
            {
                self.playing = false;
                self.seek(self.cursor + 1);
            }
            if ui
                .add_enabled(!self.done(), egui::Button::new("Finish"))
                .clicked()
            {
                self.playing = false;
                self.seek(self.ops.len());
            }
        });

        ui.add_space(10.0);
        let colors = Colors::new(palette);
        ui.horizontal_wrapped(|ui| {
            for (label, color) in colors.legend(self.alg) {
                let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 3.0, color);
                ui.label(egui::RichText::new(label).small().color(palette.muted));
                ui.add_space(8.0);
            }
        });
        ui.label(
            egui::RichText::new("Keys: Space play / pause, Left and Right arrows step")
                .small()
                .color(palette.muted),
        );
    }

    fn details(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        egui::CollapsingHeader::new("Reference implementation (Python)")
            .default_open(false)
            .show(ui, |ui| {
                egui::Frame::new()
                    .fill(palette.card)
                    .stroke(Stroke::new(1.0, palette.rule))
                    .corner_radius(egui::CornerRadius::same(10))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            egui::RichText::new(self.info.code)
                                .monospace()
                                .color(palette.ink),
                        );
                    });
            });
    }
}

impl SortVisualizer {
    /// Counting sort's tallies (middle) and output array (bottom).
    fn paint_counting(
        &self,
        painter: &egui::Painter,
        main: Rect,
        inner: Rect,
        max: u32,
        c: &Colors,
        palette: &Palette,
    ) {
        let s = &self.state;
        let font = FontId::proportional(12.0);
        let hist = Rect::from_min_max(
            pos2(inner.left(), main.bottom() + 30.0),
            pos2(inner.right(), main.bottom() + 90.0),
        );
        let out = Rect::from_min_max(
            pos2(inner.left(), hist.bottom() + 30.0),
            pos2(inner.right(), inner.bottom()),
        );
        painter.text(
            hist.left_top() - vec2(0.0, 16.0),
            Align2::LEFT_TOP,
            "count[value]",
            font.clone(),
            palette.muted,
        );
        painter.text(
            out.left_top() - vec2(0.0, 16.0),
            Align2::LEFT_TOP,
            "output",
            font.clone(),
            palette.muted,
        );
        painter.line_segment(
            [hist.left_bottom(), hist.right_bottom()],
            Stroke::new(1.0, palette.rule),
        );
        painter.line_segment(
            [out.left_bottom(), out.right_bottom()],
            Stroke::new(1.0, palette.rule),
        );

        let Some(counts) = &s.counts else {
            painter.text(
                hist.center(),
                Align2::CENTER_CENTER,
                if self.done() {
                    "done: output copied back"
                } else {
                    ""
                },
                font,
                palette.muted,
            );
            return;
        };
        let touched = match s.last {
            Some(Op::Count(i)) | Some(Op::Place { from: i, .. }) => Some(s.a[i] as usize),
            Some(Op::Prefix(v)) => Some(v),
            _ => None,
        };
        let peak = counts.iter().copied().max().unwrap_or(1).max(1);
        let tallies: Vec<Option<u32>> = counts.iter().map(|&k| Some(k)).collect();
        bars(
            painter,
            hist,
            &tallies,
            peak,
            |v| match (touched, s.last) {
                (Some(t), _) if t == v => c.swap,
                (_, Some(Op::Prefix(p))) if p == v + 1 => c.compare,
                _ => c.idle.gamma_multiply(0.7),
            },
            palette.rule,
        );
        if let Some(output) = &s.output {
            let placed = match s.last {
                Some(Op::Place { to, .. }) => Some(to),
                _ => None,
            };
            bars(
                painter,
                out,
                output,
                max,
                |i| if Some(i) == placed { c.swap } else { c.idle },
                palette.rule,
            );
        }
    }

    /// The heap drawn as a binary tree: node i has children 2i + 1 and 2i + 2.
    fn paint_tree(&self, painter: &egui::Painter, area: Rect, c: &Colors, palette: &Palette) {
        let n = self.state.a.len();
        if n == 0 {
            return;
        }
        let levels = (usize::BITS - n.leading_zeros()) as usize;
        let row_h = area.height() / levels as f32;
        let pos = |i: usize| -> Pos2 {
            let level = (usize::BITS - (i + 1).leading_zeros() - 1) as usize;
            let first = (1 << level) - 1;
            let width = 1usize << level;
            let x = area.left() + area.width() * ((i - first) as f32 + 0.5) / width as f32;
            pos2(x, area.top() + row_h * (level as f32 + 0.5))
        };
        let in_heap = |i: usize| self.state.heap_size.is_some_and(|h| i < h);
        for i in 1..n {
            let parent = (i - 1) / 2;
            let stroke = if in_heap(i) {
                Stroke::new(1.2, palette.muted)
            } else {
                Stroke::new(1.0, palette.rule)
            };
            painter.line_segment([pos(parent), pos(i)], stroke);
        }
        let radius = (area.width() / (1 << (levels - 1)) as f32 * 0.38).clamp(3.0, 13.0);
        for i in 0..n {
            let fill = self.bar_color(i, c);
            painter.circle(pos(i), radius, fill, Stroke::new(1.0, palette.card));
            if radius >= 9.0 {
                painter.text(
                    pos(i),
                    Align2::CENTER_CENTER,
                    self.state.a[i].to_string(),
                    FontId::proportional(radius * 0.95),
                    palette.card,
                );
            }
        }
    }
}

/// Lay out `fields` on one line no wider than `width`: first by tightening
/// the gaps between them, then by shrinking the font.
fn fit_line(
    painter: &egui::Painter,
    fields: &[String],
    width: f32,
    size: f32,
    color: Color32,
) -> std::sync::Arc<egui::Galley> {
    for gap in ["    ", "   "] {
        let galley = painter.layout_no_wrap(fields.join(gap), FontId::monospace(size), color);
        if galley.size().x <= width {
            return galley;
        }
    }
    fit_text(
        painter,
        fields.join("  "),
        FontId::monospace(size),
        width,
        color,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewports_share_data_and_resync() {
        let session = Session::new();
        let data = session.data.clone();
        let mut bubble = SortVisualizer::new(Algorithm::Bubble, session.clone());
        let mut quick = SortVisualizer::new(Algorithm::Quick, session.clone());
        assert_eq!(bubble.initial, quick.initial);
        data.borrow_mut().shuffle();
        bubble.sync();
        quick.sync();
        assert_eq!(bubble.initial, data.borrow().values());
        assert_eq!(bubble.initial, quick.initial);
    }

    #[test]
    fn seeking_backwards_matches_forwards() {
        let mut v = SortVisualizer::new(Algorithm::Heap, Session::new());
        let total = v.ops.len();
        v.seek(total / 2);
        let middle = v.state.a.clone();
        v.seek(total);
        assert!(v.state.a.windows(2).all(|w| w[0] <= w[1]));
        v.seek(total / 2);
        assert_eq!(v.state.a, middle);
        assert_eq!(v.cursor, total / 2);
    }
}
