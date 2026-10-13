// Quick sort (Lomuto partition, last element as pivot).
//
// partition(lo, hi): pivot = a[hi]; every a[j] <= pivot is swapped to the front, then the
// pivot drops in right behind them, at its final index p, so that
// a[lo..p-1] <= a[p] < a[p+1..hi]. Recurse on lo..p-1 and p+1..hi.
//
// The sort is a generator function that yields one step at a time (compare, swap, final
// place) while it reorders the list in place, so the animation can play it bar by bar.
// Blue unsorted, red comparing, amber swapping, pink pivot, green in final place. The
// dashed line is the pivot's value, the bracket the partition in progress. Click for a
// new list.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Step =
  | { kind: "range"; lo: number; hi: number }
  | { kind: "compare"; i: number; j: number }
  | { kind: "swap"; i: number; j: number }
  | { kind: "sorted"; i: number };

const N = 40;
const STEPS_PER_SECOND = 40;
const COLOR = { idle: "#9db3ff", compare: "#ff6b6b", swap: "#e3a93b", pivot: "#ff7ab8", sorted: "#5ccb7a" };

function swap(a: number[], i: number, j: number): void {
  [a[i], a[j]] = [a[j], a[i]];
}

/** Lomuto partition of a[lo..hi]; yields steps, returns the pivot's final index. */
function* partition(a: number[], lo: number, hi: number): Generator<Step, number> {
  yield { kind: "range", lo, hi };
  let i = lo; // a[lo..i-1] holds the values <= pivot
  for (let j = lo; j < hi; j++) {
    yield { kind: "compare", i: j, j: hi };
    if (a[j] <= a[hi]) {
      if (i !== j) {
        swap(a, i, j);
        yield { kind: "swap", i, j };
      }
      i++;
    }
  }
  if (i !== hi) {
    swap(a, i, hi); // drop the pivot in behind the small ones
    yield { kind: "swap", i, j: hi };
  }
  return i;
}

/** Sort a[lo..hi] in place, yielding every step. */
function* quickSort(a: number[], lo: number, hi: number): Generator<Step, void> {
  if (lo < hi) {
    const p = yield* partition(a, lo, hi);
    yield { kind: "sorted", i: p };
    yield* quickSort(a, lo, p - 1);
    yield* quickSort(a, p + 1, hi);
  } else if (lo === hi) {
    yield { kind: "sorted", i: lo };
  }
}

function shuffled(n: number): number[] {
  const a = Array.from({ length: n }, (_, i) => i + 1);
  for (let i = n - 1; i > 0; i--) swap(a, i, Math.floor(Math.random() * (i + 1)));
  return a;
}

/** One shuffled list being sorted, plus what the picture needs to know. */
class Run {
  a = shuffled(N);
  steps = quickSort(this.a, 0, N - 1);
  sorted: boolean[] = new Array(N).fill(false);
  hot = new Map<number, string>(); // index -> color, for the step just taken
  range: [number, number] | null = null;
  pivot: { index: number; value: number } | null = null;
  comparisons = 0;
  swaps = 0;
  doneAt = -1;

  advance(now: number): void {
    this.hot.clear();
    const r = this.steps.next();
    if (r.done) {
      this.doneAt = now;
      this.range = null;
      return;
    }
    const s = r.value;
    if (s.kind === "range") {
      this.range = [s.lo, s.hi];
      this.pivot = { index: s.hi, value: this.a[s.hi] };
    } else if (s.kind === "compare") {
      this.comparisons++;
      this.hot.set(s.i, COLOR.compare).set(s.j, COLOR.compare);
    } else if (s.kind === "swap") {
      this.swaps++;
      this.hot.set(s.i, COLOR.swap).set(s.j, COLOR.swap);
    } else {
      this.sorted[s.i] = true;
      this.pivot = null;
    }
  }

  color(k: number): string {
    const hot = this.hot.get(k);
    if (hot) return hot;
    if (this.pivot && this.pivot.index === k) return COLOR.pivot;
    if (this.sorted[k]) return COLOR.sorted;
    if (this.range && (k < this.range[0] || k > this.range[1])) return "rgba(157, 179, 255, 0.25)";
    return COLOR.idle;
  }
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const LEFT = 40, BOTTOM = 470, WIDTH = 720, UNIT = 380 / N;
const slot = WIDTH / N;
let run = new Run();
let carry = 0;
let last = 0;

canvas.addEventListener("pointerdown", () => (run = new Run()));

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
  run.a.forEach((v, k) => {
    ctx.fillStyle = run.color(k);
    ctx.fillRect(LEFT + k * slot + 2, BOTTOM - v * UNIT, slot - 4, v * UNIT);
  });
  if (run.range) {
    const [lo, hi] = run.range;
    ctx.fillStyle = "#93a39b";
    ctx.fillRect(LEFT + lo * slot + 1, BOTTOM + 10, (hi - lo + 1) * slot - 2, 4);
    if (run.pivot) {
      const y = BOTTOM - run.pivot.value * UNIT;
      ctx.strokeStyle = COLOR.pivot;
      ctx.setLineDash([6, 5]);
      ctx.beginPath();
      ctx.moveTo(LEFT + lo * slot, y);
      ctx.lineTo(LEFT + (hi + 1) * slot, y);
      ctx.stroke();
      ctx.setLineDash([]);
    }
  }

  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  ctx.fillText(`comparisons ${run.comparisons}   swaps ${run.swaps}`, LEFT, 40);
  ctx.fillText(run.doneAt < 0 ? "click for a new list" : "sorted", LEFT + 560, 40);
  const legend: [string, string][] = [
    [COLOR.idle, "unsorted"], [COLOR.compare, "comparing"], [COLOR.swap, "swapping"],
    [COLOR.pivot, "pivot"], [COLOR.sorted, "in final place"],
  ];
  ctx.font = "13px sans-serif";
  legend.forEach(([color, label], i) => {
    ctx.fillStyle = color;
    ctx.fillRect(LEFT + i * 140, 540, 12, 12);
    ctx.fillStyle = "#93a39b";
    ctx.fillText(label, LEFT + i * 140 + 18, 551);
  });
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
