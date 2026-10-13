//! Insertion sort: lift key = a[i], shift every larger a[j] (j < i) one step right, then
//! drop the key into the gap. Time O(n + inversions): linear on sorted input.
//!
//! A shuffled list drawn as bars. The sort runs once up front and records every step
//! (lift, compare, shift, insert) in a Vec; the animation replays it at a steady rate.
//! The lifted key is the purple outline over its hole (red while it is compared),
//! amber = being written, faded bars are not yet reached, green = sorted. Counters track
//! comparisons and writes. Click to shuffle a new list.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 32;
const SECONDS_PER_RUN: f32 = 15.0;
const IDLE: Color = Color::new(0.36, 0.55, 0.94, 1.0);
const FADED: Color = Color::new(0.17, 0.25, 0.37, 1.0);
const COMPARE: Color = Color::new(1.0, 0.42, 0.42, 1.0);
const WRITE: Color = Color::new(0.96, 0.69, 0.24, 1.0);
const SORTED: Color = Color::new(0.36, 0.80, 0.48, 1.0);
const KEY: Color = Color::new(0.71, 0.55, 1.0, 1.0);

#[derive(Clone, Copy)]
enum Op {
    /// key = a[i]; slot i becomes the hole.
    Lift(usize),
    /// Compare the key with a[i].
    Compare(usize),
    /// a[i + 1] = a[i]; the hole moves to i.
    Shift(usize),
    /// a[i] = key; the hole closes.
    Insert(usize),
}

/// Sort a copy of `input` with insertion sort, recording every step.
fn insertion_sort_ops(input: &[u32]) -> Vec<Op> {
    let mut a = input.to_vec();
    let mut ops = Vec::new();
    for i in 1..a.len() {
        let key = a[i];
        ops.push(Op::Lift(i));
        let mut j = i; // the hole; compare the key with a[j - 1]
        while j > 0 {
            ops.push(Op::Compare(j - 1));
            if a[j - 1] <= key {
                break; // first value not larger: the key goes right of it
            }
            a[j] = a[j - 1];
            ops.push(Op::Shift(j - 1));
            j -= 1;
        }
        a[j] = key;
        ops.push(Op::Insert(j));
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

/// The array after `cursor` recorded steps, the lifted key and counters.
struct Replay {
    ops: Vec<Op>,
    a: Vec<u32>,
    cursor: usize,
    comparisons: usize,
    writes: usize,
    key: u32,
    hole: Option<usize>,
    /// a[0..=boundary] is the prefix being extended; the rest is faded.
    boundary: usize,
    last: Option<Op>,
}

impl Replay {
    fn new(input: Vec<u32>) -> Self {
        Self {
            ops: insertion_sort_ops(&input),
            a: input,
            cursor: 0,
            comparisons: 0,
            writes: 0,
            key: 0,
            hole: None,
            boundary: 0,
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
            Op::Lift(i) => {
                self.key = self.a[i];
                self.hole = Some(i);
                self.boundary = i;
            }
            Op::Compare(_) => self.comparisons += 1,
            Op::Shift(i) => {
                self.a[i + 1] = self.a[i];
                self.hole = Some(i);
                self.writes += 1;
            }
            Op::Insert(i) => {
                self.a[i] = self.key;
                self.hole = None;
                self.writes += 1;
            }
        }
        self.last = if self.done() { None } else { Some(op) };
    }

    fn color(&self, k: usize) -> Color {
        match self.last {
            _ if self.done() => SORTED,
            Some(Op::Compare(i)) if i == k => COMPARE,
            Some(Op::Shift(i)) if i + 1 == k => WRITE,
            Some(Op::Insert(i)) if i == k => WRITE,
            _ if k <= self.boundary => IDLE,
            _ => FADED,
        }
    }
}

#[macroquad::main("Insertion sort")]
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
            if replay.hole == Some(k) {
                continue; // the hole is empty while the key is lifted out
            }
            let h = v as f32 / N as f32 * height;
            let x = left + k as f32 * slot + slot * 0.1;
            draw_rectangle(x, bottom - h, slot * 0.8, h, replay.color(k));
        }
        if let Some(hole) = replay.hole {
            // The lifted key hovers over its hole; red while it is being compared.
            let edge = match replay.last {
                Some(Op::Compare(_)) => COMPARE,
                _ => KEY,
            };
            let h = replay.key as f32 / N as f32 * height;
            let x = left + hole as f32 * slot + slot * 0.1;
            draw_rectangle(x, bottom - h, slot * 0.8, h, Color { a: 0.25, ..edge });
            draw_rectangle_lines(x, bottom - h, slot * 0.8, h, 3.0, edge);
        }

        let status = format!(
            "step {} / {}   comparisons {}   writes {}",
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
