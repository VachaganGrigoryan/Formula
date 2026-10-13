// Selection sort: for i = 0 .. n-2, find m = argmin a[i..n-1] and swap a[i] with a[m].
//
// A shuffled list drawn as bars. The sort runs once up front and records every step
// (compare, new minimum, swap, sorted) in a list; the animation replays that list at a
// steady rate. Blue = unsorted, red = being compared, purple = smallest value found so far
// in this scan, amber = being swapped, green = in final place. Counters track comparisons
// and swaps. Click to shuffle a new list.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Op =
  | { kind: "compare"; i: number; j: number }
  | { kind: "swap"; i: number; j: number }
  | { kind: "min"; i: number | null }
  | { kind: "sorted"; i: number };

const N = 32;
const SECONDS_PER_RUN = 15;
const COLORS = { idle: "#5b8def", compare: "#ff6b6b", swap: "#f5b13d", sorted: "#5ccb7a", min: "#b48cff" };

/** Sort a copy of `input` with selection sort, recording every step. */
function selectionSortOps(input: number[]): Op[] {
  const a = input.slice();
  const ops: Op[] = [];
  const n = a.length;
  for (let i = 0; i < n; i++) {
    let m = i; // index of the smallest value seen in this scan
    ops.push({ kind: "min", i: m });
    for (let j = i + 1; j < n; j++) {
      ops.push({ kind: "compare", i: m, j });
      if (a[j] < a[m]) {
        m = j;
        ops.push({ kind: "min", i: m });
      }
    }
    if (m !== i) {
      [a[i], a[m]] = [a[m], a[i]];
      ops.push({ kind: "swap", i, j: m });
    }
    ops.push({ kind: "min", i: null });
    ops.push({ kind: "sorted", i }); // a[0..i] holds the i + 1 smallest values
  }
  return ops;
}

/** The values 1..n in random order (Fisher-Yates shuffle). */
function shuffled(n: number): number[] {
  const a = Array.from({ length: n }, (_, i) => i + 1);
  for (let i = n - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

/** What the picture needs: the array after `cursor` steps, highlights and counters. */
class Replay {
  ops: Op[];
  a: number[];
  sorted: boolean[];
  cursor = 0;
  comparisons = 0;
  swaps = 0;
  minimum: number | null = null;
  last: Op | null = null;

  constructor(input: number[]) {
    this.ops = selectionSortOps(input);
    this.a = input.slice();
    this.sorted = input.map(() => false);
  }

  done(): boolean {
    return this.cursor >= this.ops.length;
  }

  step(): void {
    const op = this.ops[this.cursor++];
    switch (op.kind) {
      case "compare":
        this.comparisons++;
        break;
      case "swap":
        [this.a[op.i], this.a[op.j]] = [this.a[op.j], this.a[op.i]];
        this.swaps++;
        break;
      case "min":
        this.minimum = op.i;
        break;
      case "sorted":
        this.sorted[op.i] = true;
        break;
    }
    this.last = this.done() ? null : op;
  }

  color(k: number): string {
    const op = this.last;
    if (op && op.kind === "swap" && (op.i === k || op.j === k)) return COLORS.swap;
    if (this.minimum === k) return COLORS.min;
    if (op && op.kind === "compare" && (op.i === k || op.j === k)) return COLORS.compare;
    return this.sorted[k] ? COLORS.sorted : COLORS.idle;
  }
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 500;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

let replay = new Replay(shuffled(N));
let carry = 0; // fractional steps carried between frames
let hold = 0; // seconds spent showing the finished list
let lastMs = 0;
canvas.addEventListener("pointerdown", () => {
  replay = new Replay(shuffled(N));
  carry = 0;
});

function frame(ms: number): void {
  const dt = Math.min((ms - lastMs) / 1000, 0.1);
  lastMs = ms;
  if (!replay.done()) {
    // A full run takes about SECONDS_PER_RUN seconds.
    carry += (replay.ops.length / SECONDS_PER_RUN) * dt;
    while (carry >= 1 && !replay.done()) {
      replay.step();
      carry -= 1;
    }
  } else if ((hold += dt) > 2) {
    replay = new Replay(shuffled(N));
    hold = 0;
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  const left = 40, bottom = 450, width = 720, height = 360;
  const slot = width / N;
  replay.a.forEach((v, k) => {
    const h = (v / N) * height;
    ctx.fillStyle = replay.color(k);
    ctx.fillRect(left + k * slot + slot * 0.1, bottom - h, slot * 0.8, h);
  });

  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  const status = `step ${replay.cursor} / ${replay.ops.length}   comparisons ${replay.comparisons}   swaps ${replay.swaps}`;
  ctx.fillText(status, left, 40);
  ctx.fillStyle = "#93a39b";
  ctx.fillText(replay.done() ? "sorted" : "click to shuffle a new list", left, 480);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
