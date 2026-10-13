// Counting sort: sort small integers without comparing them, in O(n + k) time.
//
//   1. Tally:   count[v] += 1 for every value v in the input.
//   2. Prefix:  count[v] += count[v - 1], so count[v] is where v's block ends in the output.
//   3. Place:   walking the input right to left, count[v] -= 1, output[count[v]] = v.
//               (Right to left keeps equal values in their original order: a stable sort.)
//   4. Copy the output back into the input.
//
// The sort is a generator function that yields one step at a time while it fills count[]
// and output[]. Top: the input. Middle: the tally histogram count[v], with its numbers.
// Bottom: the output array filling up. Red marks the value being read, amber the tally or
// slot being written, green values in their final place. Click for a new list.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Step =
  | { kind: "count"; i: number } // tally a[i]
  | { kind: "prefix"; v: number } // count[v] += count[v - 1]
  | { kind: "place"; from: number; to: number } // output[to] = a[from]
  | { kind: "write"; i: number }; // a[i] = output[i]

const N = 20;
const MAX_VALUE = 6;
const STEPS_PER_SECOND = 10;
const COLOR = {
  idle: "#9db3ff", read: "#ff6b6b", write: "#e3a93b", sorted: "#5ccb7a", tally: "#6f80b8",
};
const PHASE = {
  count: "1. tally: count[a[i]] += 1",
  prefix: "2. running sum: count[v] += count[v - 1]",
  place: "3. place, right to left: count[v] -= 1; output[count[v]] = v",
  write: "4. copy the output back into a",
};

/** Sort a using count (max(a) + 1 zeros) and output (a.length slots), yielding every step. */
function* countingSort(
  a: number[], count: number[], output: (number | null)[],
): Generator<Step, void> {
  for (let i = 0; i < a.length; i++) {
    count[a[i]]++;
    yield { kind: "count", i };
  }
  for (let v = 1; v < count.length; v++) {
    count[v] += count[v - 1]; // count[v] = end of v's block
    yield { kind: "prefix", v };
  }
  for (let i = a.length - 1; i >= 0; i--) {
    const to = --count[a[i]]; // backwards, so equal values keep their order
    output[to] = a[i];
    yield { kind: "place", from: i, to };
  }
  for (let i = 0; i < a.length; i++) {
    a[i] = output[i]!;
    yield { kind: "write", i };
  }
}

/** One random list being sorted, plus what the picture needs to know. */
class Run {
  a = Array.from({ length: N }, () => 1 + Math.floor(Math.random() * MAX_VALUE));
  count: number[] = new Array(Math.max(...this.a) + 1).fill(0);
  output: (number | null)[] = new Array(N).fill(null);
  steps = countingSort(this.a, this.count, this.output);
  sorted: boolean[] = new Array(N).fill(false);
  last: Step | null = null;
  writes = 0;
  doneAt = -1;

  advance(now: number): void {
    const r = this.steps.next();
    if (r.done) {
      this.doneAt = now;
      this.last = null;
      return;
    }
    this.last = r.value;
    if (r.value.kind === "place") this.writes++;
    if (r.value.kind === "write") {
      this.writes++;
      this.sorted[r.value.i] = true;
    }
  }

  /** The input slot whose value is being read in this step, or -1. */
  reading(): number {
    const s = this.last;
    return s?.kind === "count" ? s.i : s?.kind === "place" ? s.from : -1;
  }

  inputColor(i: number): string {
    if (i === this.reading()) return COLOR.read;
    if (this.last?.kind === "write" && this.last.i === i) return COLOR.write;
    return this.sorted[i] ? COLOR.sorted : COLOR.idle;
  }

  countColor(v: number): string {
    const s = this.last;
    if (s?.kind === "prefix") return s.v === v ? COLOR.write : s.v - 1 === v ? COLOR.read : COLOR.tally;
    const r = this.reading();
    return r >= 0 && this.a[r] === v ? COLOR.write : COLOR.tally;
  }
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const LEFT = 40, WIDTH = 720, SLOT = WIDTH / N;
let run = new Run();
let carry = 0;
let last = 0;

canvas.addEventListener("pointerdown", () => (run = new Run()));

/** Bars for values in the box (top..bottom), one per slot; null leaves a stub. */
function bars(
  values: (number | null)[], top: number, bottom: number, max: number,
  color: (k: number) => string, labels: boolean,
): void {
  const slot = WIDTH / values.length;
  values.forEach((v, k) => {
    const x = LEFT + k * slot + slot * 0.15, w = slot * 0.7;
    const h = v === null ? 2 : Math.max(2, (v / max) * (bottom - top));
    ctx.fillStyle = v === null ? "#2c3a34" : color(k);
    ctx.fillRect(x, bottom - h, w, h);
    if (labels && v !== null) {
      ctx.fillStyle = "#e4ebe6";
      ctx.fillText(String(v), x + w / 2, bottom - h - 5);
    }
  });
}

function frame(now: number): void {
  const dt = Math.min(0.1, (now - last) / 1000);
  last = now;
  if (run.doneAt < 0) {
    carry += dt * STEPS_PER_SECOND;
    for (; carry >= 1 && run.doneAt < 0; carry--) run.advance(now);
  } else if (now - run.doneAt > 2500) {
    run = new Run(); // admire the result, then start over
  }
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.textAlign = "center";
  ctx.font = "13px monospace";

  bars(run.a, 112, 220, MAX_VALUE, (i) => run.inputColor(i), true);
  const peak = Math.max(1, ...run.count);
  bars(run.count, 285, 380, peak, (v) => run.countColor(v), true);
  ctx.fillStyle = "#93a39b";
  run.count.forEach((_, v) => ctx.fillText(`v=${v}`, LEFT + (v + 0.5) * (WIDTH / run.count.length), 398));
  const placed = run.last && run.last.kind === "place" ? run.last.to : -1;
  bars(run.output, 445, 525, MAX_VALUE, (i) => (i === placed ? COLOR.write : COLOR.idle), false);

  ctx.textAlign = "left";
  ctx.fillStyle = "#93a39b";
  ctx.fillText("input a", LEFT, 84);
  ctx.fillText("count[v]", LEFT, 262);
  ctx.fillText("output", LEFT, 430);
  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  ctx.fillText(`comparisons 0   writes ${run.writes}`, LEFT, 30);
  ctx.fillText(run.last ? PHASE[run.last.kind] : run.doneAt >= 0 ? "sorted" : "", LEFT, 54);
  const legend: [string, string][] = [
    [COLOR.idle, "unsorted"], [COLOR.read, "reading a value"], [COLOR.write, "counting / placing"],
    [COLOR.sorted, "in final place"],
  ];
  ctx.font = "13px sans-serif";
  legend.forEach(([color, label], k) => {
    ctx.fillStyle = color;
    ctx.fillRect(LEFT + k * 150, 565, 12, 12);
    ctx.fillStyle = "#93a39b";
    ctx.fillText(label, LEFT + k * 150 + 18, 576);
  });
  ctx.fillText("click for a new list", LEFT + 610, 576);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
