//! Sorting race: finish time = steps / speed, with steps = comparisons + moves.
//!
//! Five algorithms sort copies of the same list side by side. Each sort runs once up
//! front and records its steps in a Vec; every lane then replays its log at the same steps
//! per second, so the lane needing the fewest operations finishes first. Markers ("this
//! value is final", quick sort's pivot) only change colors and cost no time.
//! Red = comparing, amber = swapping / writing, pink = pivot, green = final.
//! Click to switch the input: random, nearly sorted, reversed, few unique.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 40;
const SECONDS_FOR_SLOWEST: f32 = 15.0;
const DISTRIBUTIONS: [&str; 4] = ["Random", "Nearly sorted", "Reversed", "Few unique"];
const IDLE: Color = Color::new(0.36, 0.55, 0.94, 1.0);
const COMPARE: Color = Color::new(1.0, 0.42, 0.42, 1.0);
const MOVE: Color = Color::new(0.96, 0.69, 0.24, 1.0);
const SORTED: Color = Color::new(0.36, 0.80, 0.48, 1.0);
const PIVOT: Color = Color::new(1.0, 0.48, 0.72, 1.0);

#[derive(Clone, Copy)]
enum Op {
    Compare(usize, usize),
    Swap(usize, usize),
    Write(usize, u32),
    Sorted(usize),
    Pivot(Option<usize>),
}

impl Op {
    /// Comparisons and moves cost one step; markers are free.
    fn is_step(self) -> bool {
        matches!(self, Op::Compare(..) | Op::Swap(..) | Op::Write(..))
    }
}

/// Sorts its own copy of the list and records every step.
struct Recorder {
    a: Vec<u32>,
    ops: Vec<Op>,
}

impl Recorder {
    fn greater(&mut self, i: usize, j: usize) -> bool {
        self.ops.push(Op::Compare(i, j));
        self.a[i] > self.a[j]
    }
    fn swap(&mut self, i: usize, j: usize) {
        self.a.swap(i, j);
        self.ops.push(Op::Swap(i, j));
    }
    fn write(&mut self, i: usize, v: u32) {
        self.a[i] = v;
        self.ops.push(Op::Write(i, v));
    }
}

fn bubble(r: &mut Recorder) {
    let n = r.a.len();
    for i in 0..n {
        let mut swapped = false;
        for j in 0..n - i - 1 {
            if r.greater(j, j + 1) {
                r.swap(j, j + 1);
                swapped = true;
            }
        }
        r.ops.push(Op::Sorted(n - i - 1));
        if !swapped {
            return;
        }
    }
}

fn insertion(r: &mut Recorder) {
    for i in 1..r.a.len() {
        let key = r.a[i];
        let mut j = i; // the hole the key will fill
        while j > 0 {
            r.ops.push(Op::Compare(j - 1, j)); // the key, headed for j, against a[j - 1]
            if r.a[j - 1] <= key {
                break;
            }
            let v = r.a[j - 1];
            r.write(j, v);
            j -= 1;
        }
        r.write(j, key);
    }
}

fn merge(r: &mut Recorder, lo: usize, hi: usize) {
    if hi - lo < 2 {
        return;
    }
    let mid = lo + (hi - lo) / 2;
    merge(r, lo, mid);
    merge(r, mid, hi);
    let (left, right) = (r.a[lo..mid].to_vec(), r.a[mid..hi].to_vec());
    let (mut i, mut j) = (0, 0);
    for k in lo..hi {
        if i < left.len() && j < right.len() {
            r.ops.push(Op::Compare(lo + i, mid + j));
        }
        if j == right.len() || (i < left.len() && left[i] <= right[j]) {
            r.write(k, left[i]);
            i += 1;
        } else {
            r.write(k, right[j]);
            j += 1;
        }
        if hi - lo == r.a.len() {
            r.ops.push(Op::Sorted(k));
        }
    }
}

/// Sort lo..hi with Lomuto's partition and the last value as pivot, as in the app.
fn quick(r: &mut Recorder, lo: usize, hi: usize) {
    if hi - lo < 2 {
        r.ops.extend((lo..hi).map(Op::Sorted));
        return;
    }
    let pivot = hi - 1;
    r.ops.push(Op::Pivot(Some(pivot)));
    let mut i = lo;
    for j in lo..pivot {
        if !r.greater(j, pivot) {
            if i != j {
                r.swap(i, j);
            }
            i += 1;
        }
    }
    if i != pivot {
        r.swap(i, pivot);
    }
    r.ops.extend([Op::Pivot(None), Op::Sorted(i)]); // the pivot is in its final place
    quick(r, lo, i);
    quick(r, i + 1, hi);
}

fn sift_down(r: &mut Recorder, mut i: usize, n: usize) {
    loop {
        let mut largest = i;
        for child in [2 * i + 1, 2 * i + 2] {
            if child < n && r.greater(child, largest) {
                largest = child;
            }
        }
        if largest == i {
            return;
        }
        r.swap(i, largest);
        i = largest;
    }
}

fn heap(r: &mut Recorder) {
    let n = r.a.len();
    for i in (0..n / 2).rev() {
        sift_down(r, i, n);
    }
    for end in (1..n).rev() {
        r.swap(0, end); // the root is the maximum: move it behind the heap
        r.ops.push(Op::Sorted(end));
        sift_down(r, 0, end);
    }
}

fn merge_all(r: &mut Recorder) {
    let n = r.a.len();
    merge(r, 0, n);
}

fn quick_all(r: &mut Recorder) {
    let n = r.a.len();
    quick(r, 0, n);
}

const RACERS: [(&str, fn(&mut Recorder)); 5] = [
    ("Bubble sort", bubble),
    ("Insertion sort", insertion),
    ("Merge sort", merge_all),
    ("Quick sort", quick_all),
    ("Heap sort", heap),
];

/// Values 1..=N arranged like the app's data menu.
fn make_list(dist: usize) -> Vec<u32> {
    let mut a: Vec<u32> = (1..=N as u32).collect();
    match DISTRIBUTIONS[dist] {
        "Random" => (1..N).rev().for_each(|i| a.swap(i, rand::gen_range(0, i + 1))),
        "Nearly sorted" => (0..3).for_each(|_| a.swap(rand::gen_range(0, N), rand::gen_range(0, N))),
        "Reversed" => a.reverse(),
        _ => {
            let levels = [N / 4, N / 2, 3 * N / 4, N].map(|v| v as u32);
            a.iter_mut().for_each(|v| *v = levels[rand::gen_range(0, 4)]);
        }
    }
    a
}

struct Lane {
    name: &'static str,
    ops: Vec<Op>,
    /// Comparisons + moves in the whole log.
    total: usize,
    a: Vec<u32>,
    sorted: Vec<bool>,
    cursor: usize,
    comparisons: usize,
    moves: usize,
    pivot: Option<usize>,
    last: Option<Op>,
}

impl Lane {
    fn new(name: &'static str, sort: fn(&mut Recorder), data: &[u32]) -> Self {
        let mut r = Recorder { a: data.to_vec(), ops: Vec::new() }; // its own copy
        sort(&mut r);
        Self {
            name,
            total: r.ops.iter().filter(|op| op.is_step()).count(),
            ops: r.ops,
            a: data.to_vec(),
            sorted: vec![false; data.len()],
            cursor: 0,
            comparisons: 0,
            moves: 0,
            pivot: None,
            last: None,
        }
    }

    fn done(&self) -> bool {
        self.cursor >= self.ops.len()
    }

    /// Replay up to `steps` comparisons and moves; markers along the way are free.
    fn advance_to(&mut self, steps: usize) {
        while let Some(&op) = self.ops.get(self.cursor) {
            if op.is_step() && self.comparisons + self.moves >= steps {
                return;
            }
            self.cursor += 1;
            match op {
                Op::Compare(..) => self.comparisons += 1,
                Op::Swap(i, j) => self.a.swap(i, j),
                Op::Write(i, v) => self.a[i] = v,
                Op::Sorted(i) => self.sorted[i] = true,
                Op::Pivot(p) => self.pivot = p,
            }
            if matches!(op, Op::Swap(..) | Op::Write(..)) {
                self.moves += 1;
            }
            if op.is_step() {
                self.last = Some(op);
            }
        }
    }

    fn color(&self, k: usize) -> Color {
        match self.last {
            _ if self.done() => SORTED,
            Some(Op::Compare(i, j)) if k == i || k == j => COMPARE,
            Some(Op::Swap(i, j)) if k == i || k == j => MOVE,
            Some(Op::Write(i, _)) if k == i => MOVE,
            _ if self.pivot == Some(k) => PIVOT,
            _ if self.sorted[k] => SORTED,
            _ => IDLE,
        }
    }
}

fn ordinal(k: usize) -> String {
    let suffix = match k {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{k}{suffix}")
}

/// A fresh race on one list; returns the lanes and the shared speed in steps per second.
fn new_race(dist: usize) -> (Vec<Lane>, f32) {
    let data = make_list(dist);
    let lanes: Vec<Lane> = RACERS.iter().map(|&(name, sort)| Lane::new(name, sort, &data)).collect();
    let slowest = lanes.iter().map(|l| l.total).max().unwrap_or(1);
    (lanes, slowest as f32 / SECONDS_FOR_SLOWEST)
}

#[macroquad::main("Sorting race")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let mut dist = 0;
    let (mut lanes, mut speed) = new_race(dist);
    let mut clock = 0.0; // seconds of race time

    loop {
        clock += get_frame_time().min(0.1);
        if is_mouse_button_pressed(MouseButton::Left) {
            dist = (dist + 1) % DISTRIBUTIONS.len();
            (lanes, speed) = new_race(dist);
            clock = 0.0;
        }
        // Three seconds after the slowest lane finishes, race again on a fresh list.
        if lanes.iter().all(Lane::done) && clock > SECONDS_FOR_SLOWEST + 3.0 {
            (lanes, speed) = new_race(dist);
            clock = 0.0;
        }
        for lane in &mut lanes {
            lane.advance_to((clock * speed) as usize);
        }

        clear_background(Color::from_rgba(22, 32, 28, 255));
        let header = format!(
            "{} data, {N} values   {speed:.0} steps/s   race time {clock:.1} s",
            DISTRIBUTIONS[dist]
        );
        draw_text(&header, 16.0, 28.0, 22.0, WHITE);
        let lane_h = (screen_height() - 70.0) / lanes.len() as f32;
        let (bars_x, bars_w) = (170.0, screen_width() - 400.0);
        let slot = bars_w / N as f32;
        let text_x = bars_x + bars_w + 20.0;
        for (row, lane) in lanes.iter().enumerate() {
            let top = 44.0 + row as f32 * lane_h;
            let bottom = top + lane_h - 10.0;
            for (k, &v) in lane.a.iter().enumerate() {
                let h = v as f32 / N as f32 * (lane_h - 18.0);
                draw_rectangle(bars_x + k as f32 * slot + 1.0, bottom - h, slot - 2.0, h, lane.color(k));
            }
            // Place: 1 + the number of lanes whose logs are shorter.
            let place = 1 + lanes.iter().filter(|o| o.total < lane.total).count();
            let status = if lane.done() {
                format!("{} place", ordinal(place))
            } else {
                format!("{}%", 100 * (lane.comparisons + lane.moves) / lane.total.max(1))
            };
            let winner = lane.done() && place == 1;
            draw_text(lane.name, 16.0, top + 22.0, 22.0, WHITE);
            draw_text(&status, 16.0, top + 44.0, 20.0, if winner { SORTED } else { GRAY });
            let finish = format!("finishes at {:.1} s", lane.total as f32 / speed);
            draw_text(&finish, 16.0, top + 64.0, 18.0, GRAY);
            let comparisons = format!("comparisons {}", lane.comparisons);
            draw_text(&comparisons, text_x, top + 24.0, 20.0, COMPARE);
            draw_text(&format!("moves {}", lane.moves), text_x, top + 46.0, 20.0, MOVE);
            let steps = format!("steps {} / {}", lane.comparisons + lane.moves, lane.total);
            draw_text(&steps, text_x, top + 68.0, 20.0, WHITE);
        }
        let hint = "click for the next input: random, nearly sorted, reversed, few unique";
        draw_text(hint, 16.0, screen_height() - 10.0, 18.0, GRAY);
        next_frame().await;
    }
}
