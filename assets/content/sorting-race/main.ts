// Sorting race: finish time = steps / speed, with steps = comparisons + moves.
//
// Five algorithms sort copies of the same list side by side. Each sort runs once up
// front and records its steps; every lane then replays its log at the same steps per
// second, so the lane needing the fewest operations finishes first. Markers ("this value
// is final", quick sort's pivot) only change colors and cost no time.
// Red = comparing, amber = swapping / writing, pink = pivot, green = final.
// Click to switch the input: random, nearly sorted, reversed, few unique.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Op =
  | { kind: "compare"; i: number; j: number }
  | { kind: "swap"; i: number; j: number }
  | { kind: "write"; i: number; value: number }
  | { kind: "sorted"; i: number }
  | { kind: "pivot"; i: number | null };

const N = 40;
const SECONDS_FOR_SLOWEST = 15;
const DISTRIBUTIONS = ["Random", "Nearly sorted", "Reversed", "Few unique"];
const COLORS = { idle: "#5b8def", compare: "#ff6b6b", swap: "#f5b13d", sorted: "#5ccb7a", pivot: "#ff7ab8" };

/** Sorts its own copy of the list and records every step. */
class Recorder {
  ops: Op[] = [];
  constructor(public a: number[]) {}
  greater(i: number, j: number): boolean {
    this.ops.push({ kind: "compare", i, j });
    return this.a[i] > this.a[j];
  }
  swap(i: number, j: number): void {
    [this.a[i], this.a[j]] = [this.a[j], this.a[i]];
    this.ops.push({ kind: "swap", i, j });
  }
  write(i: number, value: number): void {
    this.a[i] = value;
    this.ops.push({ kind: "write", i, value });
  }
}

function bubble(r: Recorder): void {
  const n = r.a.length;
  for (let i = 0; i < n; i++) {
    let swapped = false;
    for (let j = 0; j < n - i - 1; j++) {
      if (r.greater(j, j + 1)) {
        r.swap(j, j + 1);
        swapped = true;
      }
    }
    r.ops.push({ kind: "sorted", i: n - i - 1 });
    if (!swapped) return;
  }
}

function insertion(r: Recorder): void {
  for (let i = 1; i < r.a.length; i++) {
    const key = r.a[i];
    let j = i - 1;
    while (j >= 0) {
      r.ops.push({ kind: "compare", i: j, j: j + 1 }); // the key, headed for j + 1, against a[j]
      if (r.a[j] <= key) break;
      r.write(j + 1, r.a[j]);
      j--;
    }
    r.write(j + 1, key);
  }
}

function merge(r: Recorder, lo: number, hi: number): void {
  if (hi - lo < 2) return;
  const mid = lo + Math.floor((hi - lo) / 2);
  merge(r, lo, mid);
  merge(r, mid, hi);
  const left = r.a.slice(lo, mid), right = r.a.slice(mid, hi);
  let i = 0, j = 0;
  for (let k = lo; k < hi; k++) {
    if (i < left.length && j < right.length) r.ops.push({ kind: "compare", i: lo + i, j: mid + j });
    const takeLeft = j === right.length || (i < left.length && left[i] <= right[j]);
    r.write(k, takeLeft ? left[i++] : right[j++]);
    if (hi - lo === r.a.length) r.ops.push({ kind: "sorted", i: k });
  }
}

/** Lomuto partition with the last value as pivot, as in the app. */
function quick(r: Recorder, lo: number, hi: number): void {
  if (lo >= hi) {
    if (lo === hi) r.ops.push({ kind: "sorted", i: lo });
    return;
  }
  r.ops.push({ kind: "pivot", i: hi });
  let i = lo;
  for (let j = lo; j < hi; j++) {
    if (!r.greater(j, hi)) {
      if (i !== j) r.swap(i, j);
      i++;
    }
  }
  if (i !== hi) r.swap(i, hi);
  r.ops.push({ kind: "pivot", i: null }, { kind: "sorted", i }); // the pivot is final
  quick(r, lo, i - 1);
  quick(r, i + 1, hi);
}

function siftDown(r: Recorder, i: number, n: number): void {
  for (;;) {
    let largest = i;
    for (const child of [2 * i + 1, 2 * i + 2]) {
      if (child < n && r.greater(child, largest)) largest = child;
    }
    if (largest === i) return;
    r.swap(i, largest);
    i = largest;
  }
}

function heap(r: Recorder): void {
  const n = r.a.length;
  for (let i = Math.floor(n / 2) - 1; i >= 0; i--) siftDown(r, i, n);
  for (let end = n - 1; end > 0; end--) {
    r.swap(0, end); // the root is the maximum: move it behind the heap
    r.ops.push({ kind: "sorted", i: end });
    siftDown(r, 0, end);
  }
}

const RACERS: [string, (r: Recorder) => void][] = [
  ["Bubble sort", bubble],
  ["Insertion sort", insertion],
  ["Merge sort", (r) => merge(r, 0, r.a.length)],
  ["Quick sort", (r) => quick(r, 0, r.a.length - 1)],
  ["Heap sort", heap],
];

const isStep = (op: Op): boolean => op.kind === "compare" || op.kind === "swap" || op.kind === "write";

/** Values 1..n arranged like the app's data menu. */
function makeList(dist: string): number[] {
  const a = Array.from({ length: N }, (_, i) => i + 1);
  const rand = (k: number): number => Math.floor(Math.random() * k);
  const swap = (i: number, j: number): void => void ([a[i], a[j]] = [a[j], a[i]]);
  if (dist === "Random") for (let i = N - 1; i > 0; i--) swap(i, rand(i + 1));
  if (dist === "Nearly sorted") for (let k = 0; k < 3; k++) swap(rand(N), rand(N));
  if (dist === "Reversed") a.reverse();
  if (dist === "Few unique") return a.map(() => [N / 4, N / 2, (3 * N) / 4, N][rand(4)]);
  return a;
}

class Lane {
  ops: Op[];
  total: number; // comparisons + moves in the whole log
  a: number[];
  sorted: boolean[];
  cursor = 0;
  comparisons = 0;
  moves = 0;
  pivot: number | null = null;
  last: Op | null = null;

  constructor(public name: string, sort: (r: Recorder) => void, data: number[]) {
    const r = new Recorder(data.slice()); // every lane sorts its own copy
    sort(r);
    this.ops = r.ops;
    this.total = r.ops.filter(isStep).length;
    this.a = data.slice();
    this.sorted = data.map(() => false);
  }

  done(): boolean {
    return this.cursor >= this.ops.length;
  }

  /** Replay up to `steps` comparisons and moves; markers along the way are free. */
  advanceTo(steps: number): void {
    while (!this.done()) {
      const op = this.ops[this.cursor];
      if (isStep(op) && this.comparisons + this.moves >= steps) return;
      this.cursor++;
      if (op.kind === "compare") this.comparisons++;
      if (op.kind === "swap") [this.a[op.i], this.a[op.j]] = [this.a[op.j], this.a[op.i]];
      if (op.kind === "write") this.a[op.i] = op.value;
      if (op.kind === "swap" || op.kind === "write") this.moves++;
      if (op.kind === "sorted") this.sorted[op.i] = true;
      if (op.kind === "pivot") this.pivot = op.i;
      if (isStep(op)) this.last = op;
    }
    this.last = null;
  }

  color(k: number): string {
    const op = this.last;
    if (this.done()) return COLORS.sorted;
    if (op && op.kind === "compare" && (op.i === k || op.j === k)) return COLORS.compare;
    if (op && op.kind === "swap" && (op.i === k || op.j === k)) return COLORS.swap;
    if (op && op.kind === "write" && op.i === k) return COLORS.swap;
    if (this.pivot === k) return COLORS.pivot;
    return this.sorted[k] ? COLORS.sorted : COLORS.idle;
  }
}

const canvas = document.createElement("canvas");
canvas.width = 900;
canvas.height = 620;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

let dist = 0;
let lanes: Lane[] = [];
let clock = 0; // seconds of race time
let speed = 1; // steps per second, the same for every lane
let lastMs = 0;

function newRace(): void {
  const data = makeList(DISTRIBUTIONS[dist]);
  lanes = RACERS.map(([name, sort]) => new Lane(name, sort, data));
  speed = Math.max(...lanes.map((l) => l.total)) / SECONDS_FOR_SLOWEST;
  clock = 0;
}
canvas.addEventListener("pointerdown", () => {
  dist = (dist + 1) % DISTRIBUTIONS.length;
  newRace();
});
newRace();

function frame(ms: number): void {
  clock += Math.min((ms - lastMs) / 1000, 0.1);
  lastMs = ms;
  const finished = lanes.every((l) => l.done());
  if (finished && clock * speed > Math.max(...lanes.map((l) => l.total)) + 3 * speed) newRace();
  for (const lane of lanes) lane.advanceTo(Math.floor(clock * speed));

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.font = "15px monospace";
  ctx.fillStyle = "#e4ebe6";
  const header = `${DISTRIBUTIONS[dist]} data, ${N} values   ${speed.toFixed(0)} steps/s   race time ${clock.toFixed(1)} s`;
  ctx.fillText(header, 20, 30);
  const laneH = 112, barsX = 190, barsW = 420, slot = barsW / N;
  lanes.forEach((lane, row) => {
    const top = 50 + row * laneH, bottom = top + laneH - 16;
    lane.a.forEach((v, k) => {
      const h = (v / N) * (laneH - 24);
      ctx.fillStyle = lane.color(k);
      ctx.fillRect(barsX + k * slot + 1, bottom - h, slot - 2, h);
    });
    // Place: 1 + the number of lanes whose logs are shorter.
    const place = 1 + lanes.filter((o) => o.total < lane.total).length;
    const suffix = ["th", "st", "nd", "rd"][place] ?? "th";
    ctx.fillStyle = "#e4ebe6";
    ctx.font = "16px sans-serif";
    ctx.fillText(lane.name, 20, top + 24);
    ctx.fillStyle = lane.done() && place === 1 ? COLORS.sorted : "#93a39b";
    ctx.font = "14px sans-serif";
    const progress = lane.done() ? `${place}${suffix} place` : `${Math.floor((100 * (lane.comparisons + lane.moves)) / lane.total)}%`;
    ctx.fillText(progress, 20, top + 46);
    ctx.fillStyle = "#93a39b";
    ctx.fillText(`finishes at ${(lane.total / speed).toFixed(1)} s`, 20, top + 66);
    ctx.font = "14px monospace";
    ctx.fillStyle = COLORS.compare;
    ctx.fillText(`comparisons ${lane.comparisons}`, barsX + barsW + 30, top + 30);
    ctx.fillStyle = COLORS.swap;
    ctx.fillText(`moves       ${lane.moves}`, barsX + barsW + 30, top + 52);
    ctx.fillStyle = "#e4ebe6";
    ctx.fillText(`steps       ${lane.comparisons + lane.moves} / ${lane.total}`, barsX + barsW + 30, top + 74);
  });
  ctx.fillStyle = "#93a39b";
  ctx.fillText("click for the next input: random, nearly sorted, reversed, few unique", 20, 608);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
