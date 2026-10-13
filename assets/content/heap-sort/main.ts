// Heap sort: build a max-heap in the array, then move the root behind it n - 1 times.
//
// The array is read as a binary tree: node i has children 2i + 1 and 2i + 2, and in a
// max-heap a[parent(i)] >= a[i], so the largest value sits at the root a[0].
//   1. Build: sift down every parent, from the last one (n / 2 - 1) back to the root.
//   2. Extract: swap the root with the last heap slot (now final), shrink the heap by one,
//      and sift the new root down to repair the heap. Repeat.
//
// The sort is a generator function that yields one step at a time while it reorders the
// list in place. Top: the array as bars. Bottom: the same array drawn as the heap tree.
// Blue in the heap, red comparing, amber swapping, green in final place. Click for a new
// list.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Step =
  | { kind: "compare"; i: number; j: number }
  | { kind: "swap"; i: number; j: number }
  | { kind: "extract"; end: number } // root swapped to a[end], which is now final
  | { kind: "sorted"; i: number }; // a[i..] are all final (the last root)

const N = 31; // 2^5 - 1: a perfect tree with five levels
const STEPS_PER_SECOND = 25;
const COLOR = { heap: "#9db3ff", compare: "#ff6b6b", swap: "#e3a93b", sorted: "#5ccb7a" };

function swap(a: number[], i: number, j: number): void {
  [a[i], a[j]] = [a[j], a[i]];
}

/** Move a[i] down until it is at least as big as its children (the heap is a[0..n-1]). */
function* siftDown(a: number[], i: number, n: number): Generator<Step, void> {
  for (;;) {
    let largest = i;
    const [left, right] = [2 * i + 1, 2 * i + 2];
    if (left < n) {
      yield { kind: "compare", i: left, j: largest };
      if (a[left] > a[largest]) largest = left;
    }
    if (right < n) {
      yield { kind: "compare", i: right, j: largest };
      if (a[right] > a[largest]) largest = right;
    }
    if (largest === i) return;
    swap(a, i, largest);
    yield { kind: "swap", i, j: largest };
    i = largest;
  }
}

function* heapSort(a: number[]): Generator<Step, void> {
  const n = a.length;
  for (let i = Math.floor(n / 2) - 1; i >= 0; i--) yield* siftDown(a, i, n); // build
  for (let end = n - 1; end > 0; end--) {
    swap(a, 0, end); // the root is the max: park it behind the heap
    yield { kind: "extract", end };
    yield* siftDown(a, 0, end);
  }
  yield { kind: "sorted", i: 0 };
}

/** Node k of the heap drawn level by level inside the given box. */
function treePosition(k: number, x: number, y: number, w: number, rowHeight: number): [number, number] {
  const level = Math.floor(Math.log2(k + 1));
  const first = 2 ** level - 1;
  return [x + (w * (k - first + 0.5)) / 2 ** level, y + rowHeight * (level + 0.5)];
}

/** One shuffled list being sorted, plus what the picture needs to know. */
class Run {
  a = Array.from({ length: N }, (_, k) => k + 1);
  steps: Generator<Step, void>;
  heapSize = N;
  hot = new Map<number, string>(); // index -> color, for the step just taken
  comparisons = 0;
  swaps = 0;
  doneAt = -1;

  constructor() {
    for (let k = N - 1; k > 0; k--) swap(this.a, k, Math.floor(Math.random() * (k + 1)));
    this.steps = heapSort(this.a);
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
      this.hot.set(s.i, COLOR.compare).set(s.j, COLOR.compare);
    } else if (s.kind === "swap") {
      this.swaps++;
      this.hot.set(s.i, COLOR.swap).set(s.j, COLOR.swap);
    } else if (s.kind === "extract") {
      this.swaps++;
      this.hot.set(0, COLOR.swap).set(s.end, COLOR.swap);
      this.heapSize = s.end; // a[end] is final; the heap shrinks
    } else {
      this.heapSize = s.i;
    }
  }

  color(k: number): string {
    return this.hot.get(k) ?? (k < this.heapSize ? COLOR.heap : COLOR.sorted);
  }
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const LEFT = 40, WIDTH = 720, BAR_BOTTOM = 250, UNIT = 180 / N;
const TREE_TOP = 280, ROW = 48, RADIUS = 13;
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

  // The array as bars, with a tick where the heap ends.
  run.a.forEach((v, k) => {
    ctx.fillStyle = run.color(k);
    ctx.fillRect(LEFT + k * slot + 2, BAR_BOTTOM - v * UNIT, slot - 4, v * UNIT);
  });
  ctx.fillStyle = "#93a39b";
  ctx.fillRect(LEFT, BAR_BOTTOM + 6, run.heapSize * slot, 3);

  // The same array as a tree: node k has children 2k + 1 and 2k + 2.
  const at = (k: number) => treePosition(k, LEFT, TREE_TOP, WIDTH, ROW);
  for (let k = 1; k < N; k++) {
    const [[px, py], [cx, cy]] = [at((k - 1) >> 1), at(k)];
    ctx.strokeStyle = k < run.heapSize ? "#93a39b" : "#2c3a34";
    ctx.beginPath();
    ctx.moveTo(px, py);
    ctx.lineTo(cx, cy);
    ctx.stroke();
  }
  ctx.font = "bold 12px sans-serif";
  ctx.textAlign = "center";
  run.a.forEach((v, k) => {
    const [x, y] = at(k);
    ctx.fillStyle = run.color(k);
    ctx.beginPath();
    ctx.arc(x, y, RADIUS, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = "#16201c";
    ctx.fillText(String(v), x, y + 4);
  });
  ctx.textAlign = "left";

  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  const phase = run.doneAt >= 0 ? "sorted" : run.heapSize === N ? "building the heap" : "extracting the max";
  ctx.fillText(`comparisons ${run.comparisons}   swaps ${run.swaps}   ${phase}`, LEFT, 36);
  const legend: [string, string][] = [
    [COLOR.heap, "in the heap"], [COLOR.compare, "comparing"], [COLOR.swap, "swapping"],
    [COLOR.sorted, "in final place"],
  ];
  ctx.font = "13px sans-serif";
  legend.forEach(([color, label], k) => {
    ctx.fillStyle = color;
    ctx.fillRect(LEFT + k * 140, 560, 12, 12);
    ctx.fillStyle = "#93a39b";
    ctx.fillText(label, LEFT + k * 140 + 18, 571);
  });
  ctx.fillText("click for a new list", LEFT + 590, 571);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
