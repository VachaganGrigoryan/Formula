//! Heap sort: build a max-heap in the array, then move the root behind it n - 1 times.
//!
//! The array is read as a binary tree: node i has children 2i + 1 and 2i + 2, and in a
//! max-heap a[parent(i)] >= a[i], so the largest value sits at the root a[0].
//!   1. Build: sift down every parent, from the last one (n / 2 - 1) back to the root.
//!   2. Extract: swap the root with the last heap slot (now final), shrink the heap by one,
//!      and sift the new root down to repair the heap. Repeat.
//!
//! The sort runs once up front on a copy of the list and records every step into a
//! Vec<Op>; the main loop replays those steps over time. Top: the array as bars. Bottom:
//! the same array drawn as the heap tree. Blue in the heap, red comparing, amber swapping,
//! green in final place. Click for a new list.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const N: usize = 31; // 2^5 - 1: a perfect tree with five levels
const STEPS_PER_SECOND: f32 = 25.0;
const HEAP: Color = Color::from_hex(0x9db3ff);
const COMPARE: Color = Color::from_hex(0xff6b6b);
const SWAP: Color = Color::from_hex(0xe3a93b);
const SORTED: Color = Color::from_hex(0x5ccb7a);
const MUTED: Color = Color::from_hex(0x93a39b);
const BACKGROUND: Color = Color::from_hex(0x16201c);

/// One recorded step of the sort.
#[derive(Clone, Copy)]
enum Op {
    Compare(usize, usize),
    Swap(usize, usize),
    /// The root was swapped to a[end], which is now final; the heap is a[..end].
    Extract(usize),
    /// a[i..] are all final (the last root).
    Sorted(usize),
}

/// Move a[i] down until it is at least as big as its children (the heap is a[..n]).
fn sift_down(a: &mut [u32], mut i: usize, n: usize, ops: &mut Vec<Op>) {
    loop {
        let (mut largest, left, right) = (i, 2 * i + 1, 2 * i + 2);
        if left < n {
            ops.push(Op::Compare(left, largest));
            if a[left] > a[largest] {
                largest = left;
            }
        }
        if right < n {
            ops.push(Op::Compare(right, largest));
            if a[right] > a[largest] {
                largest = right;
            }
        }
        if largest == i {
            return;
        }
        a.swap(i, largest);
        ops.push(Op::Swap(i, largest));
        i = largest;
    }
}

fn heap_sort(a: &mut [u32], ops: &mut Vec<Op>) {
    let n = a.len();
    for i in (0..n / 2).rev() {
        sift_down(a, i, n, ops); // build the heap bottom-up
    }
    for end in (1..n).rev() {
        a.swap(0, end); // the root is the max: park it behind the heap
        ops.push(Op::Extract(end));
        sift_down(a, 0, end, ops);
    }
    ops.push(Op::Sorted(0));
}

/// Node k of the heap drawn level by level, inside a box at (x, y) of width w.
fn tree_position(k: usize, x: f32, y: f32, w: f32, row_height: f32) -> Vec2 {
    let level = (k + 1).ilog2();
    let first = (1usize << level) - 1;
    let across = (1usize << level) as f32;
    vec2(x + w * ((k - first) as f32 + 0.5) / across, y + row_height * (level as f32 + 0.5))
}

/// A recorded sort being replayed onto a fresh copy of the input.
struct Replay {
    a: Vec<u32>,
    ops: Vec<Op>,
    cursor: usize,
    heap_size: usize,
    last: Option<Op>,
    comparisons: usize,
    swaps: usize,
}

impl Replay {
    fn new() -> Self {
        let mut a: Vec<u32> = (1..=N as u32).collect();
        for k in (1..N).rev() {
            a.swap(k, rand::gen_range(0, k + 1)); // Fisher-Yates shuffle
        }
        let mut ops = Vec::new();
        heap_sort(&mut a.clone(), &mut ops);
        Replay { a, ops, cursor: 0, heap_size: N, last: None, comparisons: 0, swaps: 0 }
    }

    fn done(&self) -> bool {
        self.cursor == self.ops.len()
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
            Op::Extract(end) => {
                self.a.swap(0, end);
                self.swaps += 1;
                self.heap_size = end;
            }
            Op::Sorted(i) => self.heap_size = i,
        }
        self.last = Some(op);
    }

    fn color(&self, k: usize) -> Color {
        match self.last {
            Some(Op::Compare(i, j)) if k == i || k == j => COMPARE,
            Some(Op::Swap(i, j)) if k == i || k == j => SWAP,
            Some(Op::Extract(end)) if k == 0 || k == end => SWAP,
            _ if k < self.heap_size => HEAP,
            _ => SORTED,
        }
    }
}

#[macroquad::main("Heap sort")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut run = Replay::new();
    let mut carry = 0.0;
    let mut rest = 0.0;
    let (left, width, bar_bottom) = (40.0, 720.0, 250.0);
    let (slot, unit) = (width / N as f32, 180.0 / N as f32);

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
        clear_background(BACKGROUND);

        // The array as bars, with a rule under the part that is still a heap.
        for (k, &v) in run.a.iter().enumerate() {
            let h = v as f32 * unit;
            let x = left + k as f32 * slot;
            draw_rectangle(x + 2.0, bar_bottom - h, slot - 4.0, h, run.color(k));
        }
        draw_rectangle(left, bar_bottom + 6.0, run.heap_size as f32 * slot, 3.0, MUTED);

        // The same array as a tree: node k has children 2k + 1 and 2k + 2.
        let at = |k: usize| tree_position(k, left, 280.0, width, 48.0);
        for k in 1..N {
            let (p, c) = (at((k - 1) / 2), at(k));
            let edge = if k < run.heap_size { MUTED } else { Color::from_hex(0x2c3a34) };
            draw_line(p.x, p.y, c.x, c.y, 1.0, edge);
        }
        for (k, &v) in run.a.iter().enumerate() {
            let p = at(k);
            draw_circle(p.x, p.y, 13.0, run.color(k));
            let label = v.to_string();
            let size = measure_text(&label, None, 18, 1.0);
            draw_text(&label, p.x - size.width / 2.0, p.y + 5.0, 18.0, BACKGROUND);
        }

        let phase = if run.done() {
            "sorted"
        } else if run.heap_size == N {
            "building the heap"
        } else {
            "extracting the max"
        };
        let status = format!("comparisons {}   swaps {}   {phase}", run.comparisons, run.swaps);
        draw_text(&status, left, 36.0, 24.0, WHITE);
        let legend = [
            (HEAP, "in the heap"),
            (COMPARE, "comparing"),
            (SWAP, "swapping"),
            (SORTED, "in final place"),
        ];
        for (k, (color, label)) in legend.iter().enumerate() {
            let x = left + k as f32 * 150.0;
            draw_rectangle(x, 560.0, 12.0, 12.0, *color);
            draw_text(label, x + 18.0, 571.0, 20.0, MUTED);
        }
        let hint = "click for a new list";
        let hint_width = measure_text(hint, None, 20, 1.0).width;
        draw_text(hint, left + width - hint_width, 36.0, 20.0, MUTED);
        next_frame().await;
    }
}
