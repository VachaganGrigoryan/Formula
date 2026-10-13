//! Quick sort (3-way): Dijkstra's Dutch-national-flag partition.
//!
//! Pivot v = a[lo]. Three pointers lt <= i <= gt grow four regions:
//!   a[lo..lt-1] < v,  a[lt..i-1] == v,  a[i..gt] not seen yet,  a[gt+1..hi] > v.
//! Look at a[i]: smaller -> swap it to lt; larger -> swap it to gt; equal -> leave it.
//! When i passes gt, every copy of v is in its final place at once; recurse on the < and >
//! parts only. On lists with few distinct values that saves most of the work.
//!
//! The sort runs once up front on a copy of the list and records every step into a
//! Vec<Op>; the main loop replays those steps over time. Blue unsorted, red comparing,
//! amber swapping, pink pivot, green in final place; the strip under the bars shows the
//! < / = / > regions. Click for a new list (it alternates between few unique values and
//! all distinct values).
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 40;
const STEPS_PER_SECOND: f32 = 30.0;
const IDLE: Color = Color::from_hex(0x9db3ff);
const COMPARE: Color = Color::from_hex(0xff6b6b);
const SWAP: Color = Color::from_hex(0xe3a93b);
const PIVOT: Color = Color::from_hex(0xff7ab8);
const SORTED: Color = Color::from_hex(0x5ccb7a);
const MUTED: Color = Color::from_hex(0x93a39b);
const BACKGROUND: Color = Color::from_hex(0x16201c);

/// The four regions of a partition in progress, as inclusive bounds.
#[derive(Clone, Copy)]
struct Flag {
    lo: usize,
    lt: usize,
    i: usize,
    gt: usize,
    hi: usize,
}

/// One recorded step of the sort.
#[derive(Clone, Copy)]
enum Op {
    /// a[i] is compared with the pivot, which sits at lt.
    Compare(Flag),
    Swap(usize, usize),
    /// a[from..=to] hold their final values.
    Sorted(usize, usize),
}

/// Sort a[lo..=hi] with a three-way partition, recording every step.
fn quick_sort_3way(a: &mut [u32], lo: usize, hi: usize, ops: &mut Vec<Op>) {
    if lo >= hi {
        if lo == hi {
            ops.push(Op::Sorted(lo, lo));
        }
        return;
    }
    let v = a[lo];
    let (mut lt, mut i, mut gt) = (lo, lo + 1, hi);
    while i <= gt {
        ops.push(Op::Compare(Flag { lo, lt, i, gt, hi }));
        if a[i] < v {
            a.swap(lt, i);
            ops.push(Op::Swap(lt, i));
            lt += 1;
            i += 1;
        } else if a[i] > v {
            if i != gt {
                a.swap(i, gt);
                ops.push(Op::Swap(i, gt));
            }
            gt -= 1; // gt >= i >= lo + 1 here, so this never underflows
        } else {
            i += 1;
        }
    }
    ops.push(Op::Sorted(lt, gt)); // every copy of the pivot is final
    if lt > lo {
        quick_sort_3way(a, lo, lt - 1, ops);
    }
    quick_sort_3way(a, gt + 1, hi, ops);
}

fn make_list(few_unique: bool) -> Vec<u32> {
    let n = N as u32;
    if few_unique {
        let levels = [n / 4, n / 2, 3 * n / 4, n]; // like the app's "Few unique"
        return (0..N).map(|_| levels[rand::gen_range(0, 4)]).collect();
    }
    let mut a: Vec<u32> = (1..=n).collect();
    for k in (1..N).rev() {
        a.swap(k, rand::gen_range(0, k + 1)); // Fisher-Yates shuffle
    }
    a
}

/// A recorded sort being replayed onto a fresh copy of the input.
struct Replay {
    few_unique: bool,
    a: Vec<u32>,
    ops: Vec<Op>,
    cursor: usize,
    sorted: Vec<bool>,
    flag: Option<Flag>,
    pivot_value: u32,
    last: Option<Op>,
    comparisons: usize,
    swaps: usize,
}

impl Replay {
    fn new(few_unique: bool) -> Self {
        let a = make_list(few_unique);
        let mut ops = Vec::new();
        quick_sort_3way(&mut a.clone(), 0, N - 1, &mut ops);
        Replay {
            few_unique,
            a,
            ops,
            cursor: 0,
            sorted: vec![false; N],
            flag: None,
            pivot_value: 0,
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
            Op::Compare(f) => {
                self.comparisons += 1;
                self.flag = Some(f);
                self.pivot_value = self.a[f.lt]; // a[lt..i-1] all equal the pivot
            }
            Op::Swap(i, j) => {
                self.a.swap(i, j);
                self.swaps += 1;
            }
            Op::Sorted(from, to) => {
                self.sorted[from..=to].fill(true);
                self.flag = None;
            }
        }
        self.last = Some(op);
    }

    fn color(&self, k: usize) -> Color {
        match self.last {
            Some(Op::Compare(f)) if k == f.i || k == f.lt => return COMPARE,
            Some(Op::Swap(i, j)) if k == i || k == j => return SWAP,
            _ => {}
        }
        if self.flag.is_some_and(|f| f.lt == k) {
            PIVOT // the pivot sits at lt
        } else if self.sorted[k] {
            SORTED
        } else if self.flag.is_some_and(|f| k < f.lo || k > f.hi) {
            Color::new(IDLE.r, IDLE.g, IDLE.b, 0.25)
        } else {
            IDLE
        }
    }
}

#[macroquad::main("Quick sort (3-way)")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut run = Replay::new(true);
    let mut carry = 0.0;
    let mut rest = 0.0;
    let (left, bottom, slot, unit) = (40.0, 450.0, 720.0 / N as f32, 360.0 / N as f32);

    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            run = Replay::new(!run.few_unique);
            rest = 0.0;
        }
        if run.done() {
            rest += get_frame_time();
            if rest > 2.5 {
                run = Replay::new(run.few_unique); // admire the result, then start over
                rest = 0.0;
            }
        } else {
            carry += get_frame_time() * STEPS_PER_SECOND;
            while carry >= 1.0 && !run.done() {
                run.step();
                carry -= 1.0;
            }
        }

        clear_background(BACKGROUND);
        for (k, &v) in run.a.iter().enumerate() {
            let h = v as f32 * unit;
            draw_rectangle(left + k as f32 * slot + 2.0, bottom - h, slot - 4.0, h, run.color(k));
        }
        if let Some(f) = run.flag {
            // The flag: one labelled strip per region under the bars.
            let x = |k: usize| left + k as f32 * slot;
            let bands = [
                (f.lo, f.lt, MUTED, "<"),
                (f.lt, f.i, PIVOT, "="),
                (f.gt + 1, f.hi + 1, GRAY, ">"),
            ];
            for (from, to, color, label) in bands {
                if to > from {
                    let (x0, x1) = (x(from), x(to));
                    draw_rectangle(x0 + 1.0, bottom + 8.0, x1 - x0 - 2.0, 18.0, color);
                    draw_text(label, (x0 + x1) / 2.0 - 5.0, bottom + 23.0, 22.0, BACKGROUND);
                }
            }
            if f.gt + 1 > f.i {
                // Not looked at yet: an empty outline.
                let (x0, x1) = (x(f.i), x(f.gt + 1));
                draw_rectangle_lines(x0 + 1.0, bottom + 8.0, x1 - x0 - 2.0, 18.0, 1.0, MUTED);
            }
            let y = bottom - run.pivot_value as f32 * unit;
            let mut dash = x(f.lo);
            while dash < x(f.hi + 1) {
                draw_line(dash, y, (dash + 6.0).min(x(f.hi + 1)), y, 1.5, PIVOT); // pivot level
                dash += 11.0;
            }
        }

        let status = format!("comparisons {}   swaps {}", run.comparisons, run.swaps);
        draw_text(&status, left, 40.0, 24.0, WHITE);
        let data = if run.few_unique { "few unique values" } else { "all distinct values" };
        let hint = if run.done() { "sorted" } else { "click: other data" };
        draw_text(&format!("{data}   ({hint})"), left, 66.0, 22.0, MUTED);
        let legend = [
            (IDLE, "unsorted"),
            (COMPARE, "comparing"),
            (SWAP, "swapping"),
            (PIVOT, "pivot"),
            (SORTED, "in final place"),
        ];
        for (k, (color, label)) in legend.iter().enumerate() {
            let x = left + k as f32 * 140.0;
            draw_rectangle(x, 540.0, 12.0, 12.0, *color);
            draw_text(label, x + 18.0, 551.0, 20.0, MUTED);
        }
        next_frame().await;
    }
}
