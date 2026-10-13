// Insertion sort: lift key = a[i], shift every larger a[j] (j < i) one step right, then
// drop the key into the gap. Time O(n + inversions): linear on sorted input.
//
// A shuffled list drawn as bars. The sort runs once up front and records every step
// (lift, compare, shift, insert) in a list; the animation replays that list at a steady
// rate. The lifted key is the purple outline over its hole (red while it is compared),
// amber = being written, faded bars are not yet reached, green = sorted. Counters track
// comparisons and writes. Click to shuffle a new list.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Op =
  | { kind: "lift"; i: number } // key = a[i]; slot i becomes the hole
  | { kind: "compare"; i: number } // compare the key with a[i]
  | { kind: "shift"; i: number } // a[i + 1] = a[i]; the hole moves to i
  | { kind: "insert"; i: number }; // a[i] = key; the hole closes

const N = 32;
const SECONDS_PER_RUN = 15;
const COLORS = {
  idle: "#5b8def", faded: "#2b3f5e", compare: "#ff6b6b", swap: "#f5b13d", sorted: "#5ccb7a", key: "#b48cff",
};

/** Sort a copy of `input` with insertion sort, recording every step. */
function insertionSortOps(input: number[]): Op[] {
  const a = input.slice();
  const ops: Op[] = [];
  for (let i = 1; i < a.length; i++) {
    const key = a[i];
    ops.push({ kind: "lift", i });
    let j = i - 1;
    while (j >= 0) {
      ops.push({ kind: "compare", i: j });
      if (a[j] <= key) break; // first value not larger: the key goes right of it
      a[j + 1] = a[j];
      ops.push({ kind: "shift", i: j });
      j--;
    }
    a[j + 1] = key;
    ops.push({ kind: "insert", i: j + 1 });
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

/** What the picture needs: the array after `cursor` steps, the lifted key and counters. */
class Replay {
  ops: Op[];
  a: number[];
  cursor = 0;
  comparisons = 0;
  writes = 0;
  key = 0;
  hole: number | null = null;
  boundary = 0; // a[0..boundary] is the prefix being extended
  last: Op | null = null;

  constructor(input: number[]) {
    this.ops = insertionSortOps(input);
    this.a = input.slice();
  }

  done(): boolean {
    return this.cursor >= this.ops.length;
  }

  step(): void {
    const op = this.ops[this.cursor++];
    switch (op.kind) {
      case "lift":
        this.key = this.a[op.i];
        this.hole = this.boundary = op.i;
        break;
      case "compare":
        this.comparisons++;
        break;
      case "shift":
        this.a[op.i + 1] = this.a[op.i];
        this.hole = op.i;
        this.writes++;
        break;
      case "insert":
        this.a[op.i] = this.key;
        this.hole = null;
        this.writes++;
        break;
    }
    this.last = this.done() ? null : op;
  }

  color(k: number): string {
    const op = this.last;
    if (this.done()) return COLORS.sorted;
    if (op && op.kind === "compare" && op.i === k) return COLORS.compare;
    if (op && op.kind === "shift" && op.i + 1 === k) return COLORS.swap;
    if (op && op.kind === "insert" && op.i === k) return COLORS.swap;
    return k <= this.boundary ? COLORS.idle : COLORS.faded;
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
    if (k === replay.hole) return; // the hole is empty while the key is lifted out
    const h = (v / N) * height;
    ctx.fillStyle = replay.color(k);
    ctx.fillRect(left + k * slot + slot * 0.1, bottom - h, slot * 0.8, h);
  });
  if (replay.hole !== null) {
    // The lifted key hovers over its hole; red while it is being compared.
    const comparing = replay.last !== null && replay.last.kind === "compare";
    const h = (replay.key / N) * height;
    const x = left + replay.hole * slot + slot * 0.1;
    ctx.fillStyle = comparing ? "rgba(255, 107, 107, 0.25)" : "rgba(180, 140, 255, 0.25)";
    ctx.fillRect(x, bottom - h, slot * 0.8, h);
    ctx.strokeStyle = comparing ? COLORS.compare : COLORS.key;
    ctx.lineWidth = 2;
    ctx.strokeRect(x + 1, bottom - h + 1, slot * 0.8 - 2, h - 2);
  }

  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  const status = `step ${replay.cursor} / ${replay.ops.length}   comparisons ${replay.comparisons}   writes ${replay.writes}`;
  ctx.fillText(status, left, 40);
  ctx.fillStyle = "#93a39b";
  ctx.fillText(replay.done() ? "sorted" : "click to shuffle a new list", left, 480);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
