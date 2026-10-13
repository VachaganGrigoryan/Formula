//! Selection sort: for i = 0 .. n-2, find m = argmin a[i..n-1] and swap a[i] with a[m].
//!
//! A shuffled list drawn as bars. The sort runs once up front and records every step
//! (compare, new minimum, swap, sorted) in a Vec; the animation replays it at a steady
//! rate. Blue = unsorted, red = being compared, purple = smallest value found so far in
//! this scan, amber = being swapped, green = in final place. Counters track comparisons
//! and swaps. Click to shuffle a new list.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 32;
const SECONDS_PER_RUN: f32 = 15.0;
const IDLE: Color = Color::new(0.36, 0.55, 0.94, 1.0);
const COMPARE: Color = Color::new(1.0, 0.42, 0.42, 1.0);
const SWAP: Color = Color::new(0.96, 0.69, 0.24, 1.0);
const SORTED: Color = Color::new(0.36, 0.80, 0.48, 1.0);
const MINIMUM: Color = Color::new(0.71, 0.55, 1.0, 1.0);

#[derive(Clone, Copy)]
enum Op {
    Compare(usize, usize),
    Swap(usize, usize),
    /// The smallest value found so far in this scan (None between scans).
    Min(Option<usize>),
    Sorted(usize),
}

/// Sort a copy of `input` with selection sort, recording every step.
fn selection_sort_ops(input: &[u32]) -> Vec<Op> {
    let mut a = input.to_vec();
    let mut ops = Vec::new();
    let n = a.len();
    for i in 0..n {
        let mut m = i;
        ops.push(Op::Min(Some(m)));
        for j in i + 1..n {
            ops.push(Op::Compare(m, j));
            if a[j] < a[m] {
                m = j;
                ops.push(Op::Min(Some(m)));
            }
        }
        if m != i {
            a.swap(i, m);
            ops.push(Op::Swap(i, m));
        }
        ops.push(Op::Min(None));
        // a[0..=i] now holds the i + 1 smallest values.
        ops.push(Op::Sorted(i));
    }
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

/// The array after `cursor` recorded steps, plus highlights and counters.
struct Replay {
    ops: Vec<Op>,
    a: Vec<u32>,
    sorted: Vec<bool>,
    cursor: usize,
    comparisons: usize,
    swaps: usize,
    minimum: Option<usize>,
    last: Option<Op>,
}

impl Replay {
    fn new(input: Vec<u32>) -> Self {
        Self {
            ops: selection_sort_ops(&input),
            sorted: vec![false; input.len()],
            a: input,
            cursor: 0,
            comparisons: 0,
            swaps: 0,
            minimum: None,
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
            Op::Compare(..) => self.comparisons += 1,
            Op::Swap(i, j) => {
                self.a.swap(i, j);
                self.swaps += 1;
            }
            Op::Min(m) => self.minimum = m,
            Op::Sorted(i) => self.sorted[i] = true,
        }
        self.last = if self.done() { None } else { Some(op) };
    }

    fn color(&self, k: usize) -> Color {
        match self.last {
            Some(Op::Swap(i, j)) if k == i || k == j => SWAP,
            _ if self.minimum == Some(k) => MINIMUM,
            Some(Op::Compare(i, j)) if k == i || k == j => COMPARE,
            _ if self.sorted[k] => SORTED,
            _ => IDLE,
        }
    }
}

#[macroquad::main("Selection sort")]
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
        let (left, bottom) = (40.0, screen_height() - 50.0);
        let width = screen_width() - 80.0;
        let height = bottom - 80.0;
        let slot = width / N as f32;
        for (k, &v) in replay.a.iter().enumerate() {
            let h = v as f32 / N as f32 * height;
            let x = left + k as f32 * slot + slot * 0.1;
            draw_rectangle(x, bottom - h, slot * 0.8, h, replay.color(k));
        }

        let status = format!(
            "step {} / {}   comparisons {}   swaps {}",
            replay.cursor,
            replay.ops.len(),
            replay.comparisons,
            replay.swaps
        );
        draw_text(&status, left, 40.0, 24.0, WHITE);
        let hint = if replay.done() { "sorted" } else { "click to shuffle a new list" };
        draw_text(hint, left, screen_height() - 20.0, 20.0, GRAY);
        next_frame().await;
    }
}
