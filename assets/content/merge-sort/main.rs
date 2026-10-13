//! Merge sort: sort each half recursively, then merge; T(n) = 2 T(n/2) + O(n) = O(n log n).
//!
//! A shuffled list drawn as bars. The sort runs once up front and records every step
//! (enter a call, compare, write, sorted, exit) in a Vec; the animation replays it at a
//! steady rate. Red = the two front values being compared, amber = being written,
//! green = placed by the final merge, faded = outside the current call. The rows under
//! the bars are the open recursive calls, outermost on top; the active merge shows its
//! two halves in two colors. Counters track comparisons and writes. Click to shuffle.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 48;
const SECONDS_PER_RUN: f32 = 15.0;
const IDLE: Color = Color::new(0.36, 0.55, 0.94, 1.0);
const FADED: Color = Color::new(0.17, 0.25, 0.37, 1.0);
const COMPARE: Color = Color::new(1.0, 0.42, 0.42, 1.0);
const WRITE: Color = Color::new(0.96, 0.69, 0.24, 1.0);
const SORTED: Color = Color::new(0.36, 0.80, 0.48, 1.0);
const LEFT_HALF: Color = Color::new(0.31, 0.76, 0.79, 1.0);
const RIGHT_HALF: Color = Color::new(0.78, 0.57, 0.92, 1.0);
const OUTER: Color = Color::new(0.34, 0.39, 0.36, 1.0);

#[derive(Clone, Copy)]
enum Op {
    /// A recursive call starts on lo..hi, split at mid.
    Enter { lo: usize, mid: usize, hi: usize },
    /// The innermost call returns.
    Exit,
    Compare(usize, usize),
    Write(usize, u32),
    Sorted(usize),
}

/// Merge sorted a[lo..mid] and a[mid..hi] back into a[lo..hi], recording each step.
fn merge(a: &mut [u32], lo: usize, mid: usize, hi: usize, ops: &mut Vec<Op>) {
    let (left, right) = (a[lo..mid].to_vec(), a[mid..hi].to_vec());
    let last = lo == 0 && hi == a.len(); // the last merge places values for good
    let (mut i, mut j) = (0, 0);
    for k in lo..hi {
        let take_left = if i < left.len() && j < right.len() {
            ops.push(Op::Compare(lo + i, mid + j)); // the front values, where they started
            left[i] <= right[j] // <= keeps equal values in order (stable)
        } else {
            j == right.len() // one half is used up: copy the other
        };
        a[k] = if take_left { left[i] } else { right[j] };
        if take_left {
            i += 1;
        } else {
            j += 1;
        }
        ops.push(Op::Write(k, a[k]));
        if last {
            ops.push(Op::Sorted(k));
        }
    }
}

/// Sort a[lo..hi] in place, recording every step.
fn merge_sort(a: &mut [u32], lo: usize, hi: usize, ops: &mut Vec<Op>) {
    if hi - lo < 2 {
        return;
    }
    let mid = lo + (hi - lo) / 2;
    ops.push(Op::Enter { lo, mid, hi });
    merge_sort(a, lo, mid, ops);
    merge_sort(a, mid, hi, ops);
    merge(a, lo, mid, hi, ops);
    ops.push(Op::Exit);
}

fn merge_sort_ops(input: &[u32]) -> Vec<Op> {
    let mut ops = Vec::new();
    merge_sort(&mut input.to_vec(), 0, input.len(), &mut ops);
    ops
}

/// The values 1..=n in random order (Fisher-Yates shuffle).
fn shuffled(n: usize) -> Vec<u32> {
    let mut a: Vec<u32> = (1..=n as u32).collect();
    for i in (1..n).rev() {
        a.swap(i, rand::gen_range(0, i + 1));
    }
    a
}

/// The array after `cursor` recorded steps, the open calls and counters.
struct Replay {
    ops: Vec<Op>,
    a: Vec<u32>,
    sorted: Vec<bool>,
    /// Open calls (lo, mid, hi), outermost first.
    stack: Vec<(usize, usize, usize)>,
    cursor: usize,
    comparisons: usize,
    writes: usize,
    last: Option<Op>,
}

impl Replay {
    fn new(input: Vec<u32>) -> Self {
        Self {
            ops: merge_sort_ops(&input),
            sorted: vec![false; input.len()],
            a: input,
            stack: Vec::new(),
            cursor: 0,
            comparisons: 0,
            writes: 0,
            last: None,
        }
    }

    fn done(&self) -> bool {
        self.cursor >= self.ops.len()
    }

    fn step(&mut self) {
        let op = self.ops[self.cursor];
        self.cursor += 1;
        match op {
            Op::Enter { lo, mid, hi } => self.stack.push((lo, mid, hi)),
            Op::Exit => {
                self.stack.pop();
            }
            Op::Compare(..) => self.comparisons += 1,
            Op::Write(k, v) => {
                self.a[k] = v;
                self.writes += 1;
            }
            Op::Sorted(k) => self.sorted[k] = true,
        }
        self.last = if self.done() { None } else { Some(op) };
    }

    fn color(&self, k: usize) -> Color {
        match self.last {
            Some(Op::Compare(i, j)) if k == i || k == j => return COMPARE,
            Some(Op::Write(i, _)) | Some(Op::Sorted(i)) if k == i => return WRITE,
            _ => {}
        }
        match self.stack.last() {
            _ if self.sorted[k] || self.done() => SORTED,
            Some(&(lo, _, hi)) if k < lo || k >= hi => FADED,
            _ => IDLE,
        }
    }
}

#[macroquad::main("Merge sort")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let mut replay = Replay::new(shuffled(N));
    let mut carry = 0.0; // fractional steps carried between frames
    let mut hold = 0.0; // seconds spent showing the finished list

    loop {
        let dt = get_frame_time().min(0.1);
        if is_mouse_button_pressed(MouseButton::Left) {
            replay = Replay::new(shuffled(N));
            carry = 0.0;
        }
        if !replay.done() {
            // A full run takes about SECONDS_PER_RUN seconds.
            carry += replay.ops.len() as f32 / SECONDS_PER_RUN * dt;
            while carry >= 1.0 && !replay.done() {
                replay.step();
                carry -= 1.0;
            }
        } else {
            hold += dt;
            if hold > 2.0 {
                replay = Replay::new(shuffled(N));
                hold = 0.0;
            }
        }

        clear_background(Color::from_rgba(22, 32, 28, 255));
        let (left, bottom) = (40.0, screen_height() - 110.0);
        let width = screen_width() - 80.0;
        let height = bottom - 80.0;
        let slot = width / N as f32;
        let x = |i: usize| left + i as f32 * slot;
        for (k, &v) in replay.a.iter().enumerate() {
            let h = v as f32 / N as f32 * height;
            draw_rectangle(x(k) + slot * 0.1, bottom - h, slot * 0.8, h, replay.color(k));
        }

        // One row per open call, outermost on top; the innermost shows its two halves.
        let depth = replay.stack.len();
        for (d, &(lo, mid, hi)) in replay.stack.iter().enumerate() {
            let y = bottom + 12.0 + d as f32 * 8.0;
            if d + 1 < depth {
                draw_rectangle(x(lo) + 1.0, y, x(hi) - x(lo) - 2.0, 4.0, OUTER);
            } else {
                draw_rectangle(x(lo) + 1.0, y, x(mid) - x(lo) - 2.0, 4.0, LEFT_HALF);
                draw_rectangle(x(mid) + 1.0, y, x(hi) - x(mid) - 2.0, 4.0, RIGHT_HALF);
            }
        }

        let status = format!(
            "step {} / {}   comparisons {}   writes {}   depth {depth}",
            replay.cursor,
            replay.ops.len(),
            replay.comparisons,
            replay.writes
        );
        draw_text(&status, left, 40.0, 24.0, WHITE);
        let hint = if replay.done() { "sorted" } else { "click to shuffle a new list" };
        draw_text(hint, left, screen_height() - 20.0, 20.0, GRAY);
        next_frame().await;
    }
}
