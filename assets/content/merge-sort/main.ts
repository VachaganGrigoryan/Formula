// Merge sort: sort each half recursively, then merge; T(n) = 2 T(n/2) + O(n) = O(n log n).
//
// A shuffled list drawn as bars. The sort runs once up front and records every step
// (enter a call, compare, write, sorted, exit) in a list; the animation replays that list
// at a steady rate. Red = the two front values being compared, amber = being written,
// green = placed by the final merge, faded = outside the current call. The rows under the
// bars are the open recursive calls, outermost on top; the active merge shows its two
// halves in two colors. Counters track comparisons and writes. Click to shuffle.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Op =
  | { kind: "enter"; lo: number; mid: number; hi: number }
  | { kind: "exit" }
  | { kind: "compare"; i: number; j: number }
  | { kind: "write"; k: number; value: number }
  | { kind: "sorted"; k: number };

const N = 48;
const SECONDS_PER_RUN = 15;
const COLORS = {
  idle: "#5b8def", faded: "#2b3f5e", compare: "#ff6b6b", swap: "#f5b13d", sorted: "#5ccb7a",
  leftHalf: "#4fc1c9", rightHalf: "#c792ea", outer: "#56645d",
};

/** Merge sorted a[lo..mid) and a[mid..hi) back into a[lo..hi), recording each step. */
function merge(a: number[], lo: number, mid: number, hi: number, ops: Op[]): void {
  const left = a.slice(lo, mid);
  const right = a.slice(mid, hi);
  const final = lo === 0 && hi === a.length; // the last merge places values for good
  let i = 0, j = 0;
  for (let k = lo; k < hi; k++) {
    let takeLeft: boolean;
    if (i < left.length && j < right.length) {
      ops.push({ kind: "compare", i: lo + i, j: mid + j }); // the front values, where they started
      takeLeft = left[i] <= right[j]; // <= keeps equal values in order (stable)
    } else {
      takeLeft = j === right.length; // one half is used up: copy the other
    }
    a[k] = takeLeft ? left[i++] : right[j++];
    ops.push({ kind: "write", k, value: a[k] });
    if (final) ops.push({ kind: "sorted", k });
  }
}

/** Sort a[lo..hi) in place, recording every step into `ops`. */
function mergeSort(a: number[], lo: number, hi: number, ops: Op[]): void {
  if (hi - lo < 2) return;
  const mid = lo + Math.floor((hi - lo) / 2);
  ops.push({ kind: "enter", lo, mid, hi });
  mergeSort(a, lo, mid, ops);
  mergeSort(a, mid, hi, ops);
  merge(a, lo, mid, hi, ops);
  ops.push({ kind: "exit" });
}

function mergeSortOps(input: number[]): Op[] {
  const ops: Op[] = [];
  mergeSort(input.slice(), 0, input.length, ops);
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

/** What the picture needs: the array after `cursor` steps, the call stack and counters. */
class Replay {
  ops: Op[];
  a: number[];
  sorted: boolean[];
  stack: { lo: number; mid: number; hi: number }[] = [];
  cursor = 0;
  comparisons = 0;
  writes = 0;
  last: Op | null = null;

  constructor(input: number[]) {
    this.ops = mergeSortOps(input);
    this.a = input.slice();
    this.sorted = input.map(() => false);
  }

  done(): boolean {
    return this.cursor >= this.ops.length;
  }

  step(): void {
    const op = this.ops[this.cursor++];
    switch (op.kind) {
      case "enter":
        this.stack.push(op);
        break;
      case "exit":
        this.stack.pop();
        break;
      case "compare":
        this.comparisons++;
        break;
      case "write":
        this.a[op.k] = op.value;
        this.writes++;
        break;
      case "sorted":
        this.sorted[op.k] = true;
        break;
    }
    this.last = this.done() ? null : op;
  }

  color(k: number): string {
    const op = this.last;
    if (op && op.kind === "compare" && (op.i === k || op.j === k)) return COLORS.compare;
    if (op && (op.kind === "write" || op.kind === "sorted") && op.k === k) return COLORS.swap;
    if (this.sorted[k] || this.done()) return COLORS.sorted;
    const call = this.stack[this.stack.length - 1];
    if (call && (k < call.lo || k >= call.hi)) return COLORS.faded;
    return COLORS.idle;
  }
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 540;
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
  const left = 40, bottom = 420, width = 720, height = 340;
  const slot = width / N;
  const x = (i: number): number => left + i * slot;
  replay.a.forEach((v, k) => {
    const h = (v / N) * height;
    ctx.fillStyle = replay.color(k);
    ctx.fillRect(x(k) + slot * 0.1, bottom - h, slot * 0.8, h);
  });

  // One row per open call, outermost on top; the innermost shows its two halves.
  replay.stack.forEach((call, depth) => {
    const y = bottom + 12 + depth * 8;
    const segments: [number, number, string][] =
      depth + 1 < replay.stack.length
        ? [[call.lo, call.hi, COLORS.outer]]
        : [[call.lo, call.mid, COLORS.leftHalf], [call.mid, call.hi, COLORS.rightHalf]];
    for (const [from, to, color] of segments) {
      ctx.fillStyle = color;
      ctx.fillRect(x(from) + 1, y, to * slot - from * slot - 2, 4);
    }
  });

  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  const status = `step ${replay.cursor} / ${replay.ops.length}   comparisons ${replay.comparisons}   writes ${replay.writes}   depth ${replay.stack.length}`;
  ctx.fillText(status, left, 40);
  ctx.fillStyle = "#93a39b";
  ctx.fillText(replay.done() ? "sorted" : "click to shuffle a new list", left, 520);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
