// Quick sort (3-way): Dijkstra's Dutch-national-flag partition.
//
// Pivot v = a[lo]. Three pointers lt <= i <= gt grow four regions:
//   a[lo..lt-1] < v,  a[lt..i-1] == v,  a[i..gt] not seen yet,  a[gt+1..hi] > v.
// Look at a[i]: smaller -> swap it to lt; larger -> swap it to gt; equal -> leave it.
// When i passes gt, every copy of v is in its final place at once; recurse on the < and >
// parts only. On lists with few distinct values that saves most of the work.
//
// The sort is a generator function that yields one step at a time while it reorders the
// list in place. Blue unsorted, red comparing, amber swapping, pink pivot, green in final
// place; the strip under the bars shows the < / = / > regions. Click for a new list (it
// alternates between few unique values and all distinct values).
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Flag = { lo: number; lt: number; i: number; gt: number; hi: number };
type Step =
  | { kind: "compare"; flag: Flag }
  | { kind: "swap"; i: number; j: number }
  | { kind: "sorted"; from: number; to: number };

const N = 40;
const STEPS_PER_SECOND = 30;
const COLOR = { idle: "#9db3ff", compare: "#ff6b6b", swap: "#e3a93b", pivot: "#ff7ab8", sorted: "#5ccb7a" };

function swap(a: number[], i: number, j: number): void {
  [a[i], a[j]] = [a[j], a[i]];
}

/** Sort a[lo..hi] in place with a three-way partition, yielding every step. */
function* quickSort3(a: number[], lo: number, hi: number): Generator<Step, void> {
  if (lo >= hi) {
    if (lo === hi) yield { kind: "sorted", from: lo, to: lo };
    return;
  }
  const v = a[lo];
  let [lt, i, gt] = [lo, lo + 1, hi];
  while (i <= gt) {
    yield { kind: "compare", flag: { lo, lt, i, gt, hi } }; // one three-way comparison
    if (a[i] < v) {
      swap(a, lt, i);
      yield { kind: "swap", i: lt, j: i };
      lt++;
      i++;
    } else if (a[i] > v) {
      if (i !== gt) {
        swap(a, i, gt);
        yield { kind: "swap", i, j: gt };
      }
      gt--;
    } else {
      i++;
    }
  }
  yield { kind: "sorted", from: lt, to: gt }; // every copy of the pivot is final
  yield* quickSort3(a, lo, lt - 1);
  yield* quickSort3(a, gt + 1, hi);
}

function makeList(fewUnique: boolean): number[] {
  if (fewUnique) {
    const levels = [N / 4, N / 2, (3 * N) / 4, N]; // like the app's "Few unique"
    return Array.from({ length: N }, () => levels[Math.floor(Math.random() * 4)]);
  }
  const a = Array.from({ length: N }, (_, k) => k + 1);
  for (let k = N - 1; k > 0; k--) swap(a, k, Math.floor(Math.random() * (k + 1)));
  return a;
}

/** One list being sorted, plus what the picture needs to know. */
class Run {
  a: number[];
  steps: Generator<Step, void>;
  sorted: boolean[] = new Array(N).fill(false);
  hot = new Map<number, string>(); // index -> color, for the step just taken
  flag: Flag | null = null;
  pivotValue = 0;
  comparisons = 0;
  swaps = 0;
  doneAt = -1;

  constructor(readonly fewUnique: boolean) {
    this.a = makeList(fewUnique);
    this.steps = quickSort3(this.a, 0, N - 1);
  }

  advance(now: number): void {
    this.hot.clear();
    const r = this.steps.next();
    if (r.done) {
      this.doneAt = now;
      return;
    }
    const s = r.value;
    if (s.kind === "compare") {
      this.comparisons++;
      this.flag = s.flag;
      this.pivotValue = this.a[s.flag.lt]; // a[lt..i-1] all equal the pivot
      this.hot.set(s.flag.i, COLOR.compare).set(s.flag.lt, COLOR.compare);
    } else if (s.kind === "swap") {
      this.swaps++;
      this.hot.set(s.i, COLOR.swap).set(s.j, COLOR.swap);
    } else {
      for (let k = s.from; k <= s.to; k++) this.sorted[k] = true;
      this.flag = null;
    }
  }

  color(k: number): string {
    const hot = this.hot.get(k);
    if (hot) return hot;
    if (this.flag && this.flag.lt === k) return COLOR.pivot; // the pivot sits at lt
    if (this.sorted[k]) return COLOR.sorted;
    if (this.flag && (k < this.flag.lo || k > this.flag.hi)) return "rgba(157, 179, 255, 0.25)";
    return COLOR.idle;
  }
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const LEFT = 40, BOTTOM = 450, WIDTH = 720, UNIT = 360 / N;
const slot = WIDTH / N;
let run = new Run(true);
let carry = 0;
let last = 0;

canvas.addEventListener("pointerdown", () => (run = new Run(!run.fewUnique)));

/** One band of the flag drawn as a labelled strip under the bars. */
function band(from: number, to: number, fill: string, label: string): void {
  if (to < from) return;
  const x = LEFT + from * slot, w = (to - from + 1) * slot;
  ctx.fillStyle = fill;
  ctx.fillRect(x + 1, BOTTOM + 8, w - 2, 18);
  ctx.fillStyle = "#16201c";
  ctx.fillText(label, x + w / 2 - 4, BOTTOM + 22);
}

function frame(now: number): void {
  const dt = Math.min(0.1, (now - last) / 1000);
  last = now;
  if (run.doneAt < 0) {
    carry += dt * STEPS_PER_SECOND;
    for (; carry >= 1 && run.doneAt < 0; carry--) run.advance(now);
  } else if (now - run.doneAt > 2500) {
    run = new Run(run.fewUnique); // admire the result, then start over
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  run.a.forEach((v, k) => {
    ctx.fillStyle = run.color(k);
    ctx.fillRect(LEFT + k * slot + 2, BOTTOM - v * UNIT, slot - 4, v * UNIT);
  });
  const f = run.flag;
  if (f) {
    ctx.font = "bold 14px monospace";
    band(f.lo, f.lt - 1, "#93a39b", "<");
    band(f.lt, f.i - 1, COLOR.pivot, "=");
    band(f.gt + 1, f.hi, "#5f6f68", ">");
    ctx.strokeStyle = "#93a39b";
    if (f.i <= f.gt) ctx.strokeRect(LEFT + f.i * slot + 1, BOTTOM + 8, (f.gt - f.i + 1) * slot - 2, 18);
    const y = BOTTOM - run.pivotValue * UNIT;
    ctx.strokeStyle = COLOR.pivot;
    ctx.setLineDash([6, 5]);
    ctx.beginPath();
    ctx.moveTo(LEFT + f.lo * slot, y);
    ctx.lineTo(LEFT + (f.hi + 1) * slot, y);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  ctx.fillText(`comparisons ${run.comparisons}   swaps ${run.swaps}`, LEFT, 40);
  const data = run.fewUnique ? "few unique values" : "all distinct values";
  ctx.fillText(`${data}   (${run.doneAt < 0 ? "click: other data" : "sorted"})`, LEFT, 64);
  const legend: [string, string][] = [
    [COLOR.idle, "unsorted"], [COLOR.compare, "comparing"], [COLOR.swap, "swapping"],
    [COLOR.pivot, "pivot"], [COLOR.sorted, "in final place"],
  ];
  ctx.font = "13px sans-serif";
  legend.forEach(([color, label], k) => {
    ctx.fillStyle = color;
    ctx.fillRect(LEFT + k * 140, 540, 12, 12);
    ctx.fillStyle = "#93a39b";
    ctx.fillText(label, LEFT + k * 140 + 18, 551);
  });
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
