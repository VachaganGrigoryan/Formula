//! Quick sort (Lomuto partition, last element as pivot).
//!
//! partition(lo, hi): pivot = a[hi]; every a[j] <= pivot is swapped to the front, then the
//! pivot drops in right behind them, at its final index p, so that
//! a[lo..p-1] <= a[p] < a[p+1..hi]. Recurse on both sides.
//!
//! The sort runs once up front on a copy of the list and records every step into a
//! Vec<Op>; the main loop replays those steps over time. Blue unsorted, red comparing,
//! amber swapping, pink pivot, green in final place. The dashed line is the pivot's
//! value, the bracket the partition in progress. Click for a new list.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 40;
const STEPS_PER_SECOND: f32 = 40.0;
const IDLE: Color = Color::from_hex(0x9db3ff);
const COMPARE: Color = Color::from_hex(0xff6b6b);
const SWAP: Color = Color::from_hex(0xe3a93b);
const PIVOT: Color = Color::from_hex(0xff7ab8);
const SORTED: Color = Color::from_hex(0x5ccb7a);
const MUTED: Color = Color::from_hex(0x93a39b);

/// One recorded step of the sort.
#[derive(Clone, Copy)]
enum Op {
    /// A partition of a[lo..=hi] starts; the pivot is a[hi].
    Range(usize, usize),
    Compare(usize, usize),
    Swap(usize, usize),
    /// This index holds its final value.
    Sorted(usize),
}

/// Lomuto partition of a[lo..=hi]; returns the pivot's final index.
fn partition(a: &mut [u32], lo: usize, hi: usize, ops: &mut Vec<Op>) -> usize {
    ops.push(Op::Range(lo, hi));
    let mut i = lo; // a[lo..i] holds the values <= pivot
    for j in lo..hi {
        ops.push(Op::Compare(j, hi));
        if a[j] <= a[hi] {
            if i != j {
                a.swap(i, j);
                ops.push(Op::Swap(i, j));
            }
            i += 1;
        }
    }
    if i != hi {
        a.swap(i, hi); // drop the pivot in behind the small ones
        ops.push(Op::Swap(i, hi));
    }
    i
}

/// Sort the half-open range a[lo..hi], recording every step.
fn quick_sort(a: &mut [u32], lo: usize, hi: usize, ops: &mut Vec<Op>) {
    if hi - lo > 1 {
        let p = partition(a, lo, hi - 1, ops);
        ops.push(Op::Sorted(p));
        quick_sort(a, lo, p, ops);
        quick_sort(a, p + 1, hi, ops);
    } else if hi - lo == 1 {
        ops.push(Op::Sorted(lo));
    }
}

/// A recorded sort being replayed onto a fresh copy of the input.
struct Replay {
    a: Vec<u32>,
    ops: Vec<Op>,
    cursor: usize,
    sorted: Vec<bool>,
    range: Option<(usize, usize)>,
    pivot: Option<(usize, u32)>,
    last: Option<Op>,
    comparisons: usize,
    swaps: usize,
}

impl Replay {
    fn new() -> Self {
        let mut a: Vec<u32> = (1..=N as u32).collect();
        for i in (1..N).rev() {
            a.swap(i, rand::gen_range(0, i + 1)); // Fisher-Yates shuffle
        }
        let mut ops = Vec::new();
        quick_sort(&mut a.clone(), 0, N, &mut ops);
        Replay {
            a,
            ops,
            cursor: 0,
            sorted: vec![false; N],
            range: None,
            pivot: None,
            last: None,
            comparisons: 0,
            swaps: 0,
        }
    }

    fn done(&self) -> bool {
        self.cursor == self.ops.len()
    }

    fn step(&mut self) {
        let op = self.ops[self.cursor];
        self.cursor += 1;
        match op {
            Op::Range(lo, hi) => {
                self.range = Some((lo, hi));
                self.pivot = Some((hi, self.a[hi]));
            }
            Op::Compare(..) => self.comparisons += 1,
            Op::Swap(i, j) => {
                self.a.swap(i, j);
                self.swaps += 1;
            }
            Op::Sorted(i) => {
                self.sorted[i] = true;
                self.pivot = None;
            }
        }
        self.last = Some(op);
        if self.done() {
            self.range = None;
        }
    }

    fn color(&self, k: usize) -> Color {
        match self.last {
            Some(Op::Compare(i, j)) if k == i || k == j => return COMPARE,
            Some(Op::Swap(i, j)) if k == i || k == j => return SWAP,
            _ => {}
        }
        if self.pivot.is_some_and(|(p, _)| p == k) {
            PIVOT
        } else if self.sorted[k] {
            SORTED
        } else if self.range.is_some_and(|(lo, hi)| k < lo || k > hi) {
            Color::new(IDLE.r, IDLE.g, IDLE.b, 0.25)
        } else {
            IDLE
        }
    }
}

#[macroquad::main("Quick sort")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut run = Replay::new();
    let mut carry = 0.0;
    let mut rest = 0.0;
    let (left, bottom, slot, unit) = (40.0, 470.0, 720.0 / N as f32, 380.0 / N as f32);

    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            run = Replay::new();
            rest = 0.0;
        }
        if run.done() {
            rest += get_frame_time();
            if rest > 2.5 {
                run = Replay::new(); // admire the result, then start over
                rest = 0.0;
            }
        } else {
            carry += get_frame_time() * STEPS_PER_SECOND;
            while carry >= 1.0 && !run.done() {
                run.step();
                carry -= 1.0;
            }
        }

        clear_background(Color::from_rgba(22, 32, 28, 255));
        for (k, &v) in run.a.iter().enumerate() {
            let h = v as f32 * unit;
            draw_rectangle(left + k as f32 * slot + 2.0, bottom - h, slot - 4.0, h, run.color(k));
        }
        if let Some((lo, hi)) = run.range {
            let (x0, x1) = (left + lo as f32 * slot, left + (hi + 1) as f32 * slot);
            draw_rectangle(x0 + 1.0, bottom + 10.0, x1 - x0 - 2.0, 4.0, MUTED);
            if let Some((_, value)) = run.pivot {
                let y = bottom - value as f32 * unit;
                let mut x = x0;
                while x < x1 {
                    draw_line(x, y, (x + 6.0).min(x1), y, 1.5, PIVOT); // dashed pivot level
                    x += 11.0;
                }
            }
        }

        let status = format!("comparisons {}   swaps {}", run.comparisons, run.swaps);
        draw_text(&status, left, 40.0, 24.0, WHITE);
        let hint = if run.done() { "sorted" } else { "click for a new list" };
        draw_text(hint, left + 520.0, 40.0, 22.0, MUTED);
        let legend = [
            (IDLE, "unsorted"),
            (COMPARE, "comparing"),
            (SWAP, "swapping"),
            (PIVOT, "pivot"),
            (SORTED, "in final place"),
        ];
        for (i, (color, label)) in legend.iter().enumerate() {
            let x = left + i as f32 * 140.0;
            draw_rectangle(x, 540.0, 12.0, 12.0, *color);
            draw_text(label, x + 18.0, 551.0, 20.0, MUTED);
        }
        next_frame().await;
    }
}
