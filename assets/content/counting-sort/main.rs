//! Counting sort: sort small integers without comparing them, in O(n + k) time.
//!
//!   1. Tally:   count[v] += 1 for every value v in the input.
//!   2. Prefix:  count[v] += count[v - 1], so count[v] is where v's block ends in the output.
//!   3. Place:   walking the input right to left, count[v] -= 1, output[count[v]] = v.
//!               (Right to left keeps equal values in their original order: a stable sort.)
//!   4. Copy the output back into the input.
//!
//! The sort runs once up front on a copy of the list and records every step into a
//! Vec<Op>; the main loop replays those steps over time. Top: the input. Middle: the tally
//! histogram count[v], with its numbers. Bottom: the output array filling up. Red marks
//! the value being read, amber the tally or slot being written, green values in their
//! final place. Click for a new list.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 20;
const MAX_VALUE: u32 = 6;
const STEPS_PER_SECOND: f32 = 10.0;
const IDLE: Color = Color::from_hex(0x9db3ff);
const READ: Color = Color::from_hex(0xff6b6b);
const WRITE: Color = Color::from_hex(0xe3a93b);
const SORTED: Color = Color::from_hex(0x5ccb7a);
const TALLY: Color = Color::from_hex(0x6f80b8);
const MUTED: Color = Color::from_hex(0x93a39b);
const EMPTY: Color = Color::from_hex(0x2c3a34);
const LEFT: f32 = 40.0;
const WIDTH: f32 = 720.0;

/// One recorded step of the sort.
#[derive(Clone, Copy)]
enum Op {
    /// count[a[i]] += 1
    Count(usize),
    /// count[v] += count[v - 1]
    Prefix(usize),
    /// count[a[from]] -= 1, output[to] = a[from]
    Place { from: usize, to: usize },
    /// a[i] = v, copied back from the output
    Write(usize, u32),
}

/// Sort a in place via a tally of its values, recording every step.
fn counting_sort(a: &mut [u32], ops: &mut Vec<Op>) {
    let max = a.iter().copied().max().unwrap_or(0) as usize;
    let mut count = vec![0usize; max + 1];
    for (i, &v) in a.iter().enumerate() {
        count[v as usize] += 1;
        ops.push(Op::Count(i));
    }
    for v in 1..count.len() {
        count[v] += count[v - 1]; // count[v] = end of v's block
        ops.push(Op::Prefix(v));
    }
    let mut output = vec![0; a.len()];
    for i in (0..a.len()).rev() {
        let v = a[i] as usize; // backwards, so equal values keep their order
        count[v] -= 1;
        output[count[v]] = a[i];
        ops.push(Op::Place { from: i, to: count[v] });
    }
    for (i, &v) in output.iter().enumerate() {
        a[i] = v;
        ops.push(Op::Write(i, v));
    }
}

/// A recorded sort being replayed: the input, the tallies and the output.
struct Replay {
    a: Vec<u32>,
    count: Vec<u32>,
    output: Vec<Option<u32>>,
    sorted: Vec<bool>,
    ops: Vec<Op>,
    cursor: usize,
    last: Option<Op>,
    writes: usize,
}

impl Replay {
    fn new() -> Self {
        let a: Vec<u32> = (0..N).map(|_| rand::gen_range(1, MAX_VALUE + 1)).collect();
        let mut ops = Vec::new();
        counting_sort(&mut a.clone(), &mut ops);
        let k = *a.iter().max().unwrap() as usize + 1;
        Replay {
            a,
            count: vec![0; k],
            output: vec![None; N],
            sorted: vec![false; N],
            ops,
            cursor: 0,
            last: None,
            writes: 0,
        }
    }

    fn done(&self) -> bool {
        self.cursor == self.ops.len()
    }

    fn step(&mut self) {
        let op = self.ops[self.cursor];
        self.cursor += 1;
        match op {
            Op::Count(i) => self.count[self.a[i] as usize] += 1,
            Op::Prefix(v) => self.count[v] += self.count[v - 1],
            Op::Place { from, to } => {
                self.count[self.a[from] as usize] -= 1;
                self.output[to] = Some(self.a[from]);
                self.writes += 1;
            }
            Op::Write(i, v) => {
                self.a[i] = v;
                self.sorted[i] = true;
                self.writes += 1;
            }
        }
        self.last = if self.done() { None } else { Some(op) };
    }

    /// The input slot whose value is being read in this step.
    fn reading(&self) -> Option<usize> {
        match self.last {
            Some(Op::Count(i)) | Some(Op::Place { from: i, .. }) => Some(i),
            _ => None,
        }
    }

    fn input_color(&self, i: usize) -> Color {
        match self.last {
            _ if self.reading() == Some(i) => READ,
            Some(Op::Write(w, _)) if w == i => WRITE,
            _ if self.sorted[i] => SORTED,
            _ => IDLE,
        }
    }

    fn count_color(&self, v: usize) -> Color {
        match self.last {
            Some(Op::Prefix(p)) if p == v => WRITE,
            Some(Op::Prefix(p)) if p == v + 1 => READ,
            _ if self.reading().is_some_and(|i| self.a[i] as usize == v) => WRITE,
            _ => TALLY,
        }
    }

    fn output_color(&self, i: usize) -> Color {
        match self.last {
            Some(Op::Place { to, .. }) if to == i => WRITE,
            _ => IDLE,
        }
    }
}

/// Bars for `values` between `top` and `bottom`; None leaves an empty stub.
fn bars(
    values: &[Option<u32>],
    top: f32,
    bottom: f32,
    max: u32,
    color: impl Fn(usize) -> Color,
    labels: bool,
) {
    let slot = WIDTH / values.len() as f32;
    for (k, v) in values.iter().enumerate() {
        let (x, w) = (LEFT + k as f32 * slot + slot * 0.15, slot * 0.7);
        match v {
            Some(v) => {
                let h = (*v as f32 / max.max(1) as f32 * (bottom - top)).max(2.0);
                draw_rectangle(x, bottom - h, w, h, color(k));
                if labels {
                    let text = v.to_string();
                    let size = measure_text(&text, None, 18, 1.0);
                    draw_text(&text, x + (w - size.width) / 2.0, bottom - h - 5.0, 18.0, WHITE);
                }
            }
            None => draw_rectangle(x, bottom - 2.0, w, 2.0, EMPTY),
        }
    }
}

#[macroquad::main("Counting sort")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut run = Replay::new();
    let mut carry = 0.0;
    let mut rest = 0.0;

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
        clear_background(Color::from_hex(0x16201c));

        let input: Vec<Option<u32>> = run.a.iter().map(|&v| Some(v)).collect();
        bars(&input, 112.0, 220.0, MAX_VALUE, |i| run.input_color(i), true);
        let tallies: Vec<Option<u32>> = run.count.iter().map(|&c| Some(c)).collect();
        let peak = run.count.iter().copied().max().unwrap_or(1);
        bars(&tallies, 285.0, 380.0, peak, |v| run.count_color(v), true);
        let slot = WIDTH / run.count.len() as f32;
        for v in 0..run.count.len() {
            draw_text(&format!("v={v}"), LEFT + (v as f32 + 0.3) * slot, 400.0, 18.0, MUTED);
        }
        bars(&run.output, 445.0, 525.0, MAX_VALUE, |i| run.output_color(i), false);

        draw_text("input a", LEFT, 84.0, 18.0, MUTED);
        draw_text("count[v]", LEFT, 262.0, 18.0, MUTED);
        draw_text("output", LEFT, 430.0, 18.0, MUTED);
        draw_text(&format!("comparisons 0   writes {}", run.writes), LEFT, 30.0, 22.0, WHITE);
        let phase = match run.last {
            Some(Op::Count(_)) => "1. tally: count[a[i]] += 1",
            Some(Op::Prefix(_)) => "2. running sum: count[v] += count[v - 1]",
            Some(Op::Place { .. }) => "3. place, right to left: count[v] -= 1; output[count[v]] = v",
            Some(Op::Write(..)) => "4. copy the output back into a",
            None if run.done() => "sorted",
            None => "",
        };
        draw_text(phase, LEFT, 56.0, 20.0, WHITE);
        let legend = [
            (IDLE, "unsorted"),
            (READ, "reading a value"),
            (WRITE, "counting / placing"),
            (SORTED, "in final place"),
        ];
        let mut x = LEFT;
        for (color, label) in legend {
            draw_rectangle(x, 560.0, 12.0, 12.0, color);
            draw_text(label, x + 18.0, 571.0, 20.0, MUTED);
            x += 18.0 + measure_text(label, None, 20, 1.0).width + 28.0;
        }
        let hint = "click for a new list";
        let hint_width = measure_text(hint, None, 20, 1.0).width;
        draw_text(hint, LEFT + WIDTH - hint_width, 30.0, 20.0, MUTED);
        next_frame().await;
    }
}
