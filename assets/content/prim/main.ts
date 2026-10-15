// Prim's algorithm: grow one tree; add the lightest edge leaving it
//
// 14 random points joined by every pair closer than 0.35 (the seed is retried
// until the graph is connected). The algorithm is run once and recorded as a list
// of operations; the animation then replays that list. Vertices in the tree are filled; the tree grows from vertex 0. Click the canvas
// to draw a new set of points.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Edge = { a: number; b: number; w: number };
type Op =
  | { kind: "consider" | "accept" | "reject"; edge: number }
  | { kind: "reach"; vertex: number };

const N = 14;
const RADIUS = 0.35;
const SIZE = 560; // pixels per side of the unit square
const HEADER = 30;
const STEPS = 1; // recorded operations replayed per frame
const FRAME_MS = 120;
const PAPER = "#16201c";
const RULE = "#3a4a44";
const CONSIDER = "#f0a040";
const TREE = "#a8c4ff";
const REJECT = "#7a8a84";
const INK = "#d8e0dc";
const CYCLE = ["#4cc38a", "#f0a040", "#a8c4ff", "#d8e0dc", "#7a8a84", "#2c8a6a"];

const canvas = document.createElement("canvas");
canvas.width = SIZE;
canvas.height = SIZE + HEADER;
canvas.style.background = PAPER;
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

/** A small seeded generator, so a graph is repeatable. */
function rng(s: number): () => number {
  let x = (s * 2654435761 + 12345) >>> 0;
  return () => {
    x = (x ^ (x << 13)) >>> 0;
    x = (x ^ (x >>> 17)) >>> 0;
    x = (x ^ (x << 5)) >>> 0;
    return x / 4294967296;
  };
}

function find(parent: number[], v: number): number {
  while (parent[v] !== v) v = parent[v];
  return v;
}

/** Random points and every pair within RADIUS; retry until connected. */
function makeGraph(seed: number): { pts: [number, number][]; edges: Edge[] } {
  for (;;) {
    const next = rng(seed);
    const pts: [number, number][] = [];
    for (let i = 0; i < N; i++) pts.push([0.08 + next() * 0.84, 0.08 + next() * 0.84]);
    const edges: Edge[] = [];
    for (let a = 0; a < N; a++) {
      for (let b = a + 1; b < N; b++) {
        const w = Math.hypot(pts[a][0] - pts[b][0], pts[a][1] - pts[b][1]) * 100;
        if (w <= RADIUS * 100) edges.push({ a, b, w });
      }
    }
    const parent = Array.from({ length: N }, (_, i) => i);
    for (const e of edges) parent[find(parent, e.a)] = find(parent, e.b);
    if (new Set(parent.map((_, v) => find(parent, v))).size === 1) return { pts, edges };
    seed += 1000;
  }
}

/** Run Prim from vertex 0 and log every step (a linear scan stands in for the heap). */
function record(edges: Edge[]): Op[] {
  const inTree = new Array<boolean>(N).fill(false);
  inTree[0] = true;
  const log: Op[] = [{ kind: "reach", vertex: 0 }];
  const done = new Array<boolean>(edges.length).fill(false);
  for (;;) {
    // lightest not-yet-popped edge with at least one end in the tree
    let best = -1;
    edges.forEach((e, i) => {
      if (done[i] || !(inTree[e.a] || inTree[e.b])) return;
      if (best < 0 || e.w < edges[best].w) best = i;
    });
    if (best < 0) break;
    done[best] = true;
    log.push({ kind: "consider", edge: best });
    const { a, b } = edges[best];
    if (inTree[a] && inTree[b]) {
      log.push({ kind: "reject", edge: best });
      continue;
    }
    const v = inTree[a] ? b : a;
    log.push({ kind: "accept", edge: best });
    inTree[v] = true;
    log.push({ kind: "reach", vertex: v });
  }
  return log;
}

let seed = 7;
let graph = makeGraph(seed);
let ops: Op[] = [];
let cursor = 0;
let edgeState: string[] = [];
let inTree: boolean[] = [];
let weight = 0;
let accepted = 0;
let last = 0;

function restart(): void {
  graph = makeGraph(seed);
  ops = record(graph.edges);
  cursor = 0;
  edgeState = graph.edges.map(() => "unseen");
  inTree = new Array<boolean>(N).fill(false);
  weight = 0;
  accepted = 0;
}

function apply(op: Op): void {
  if (op.kind === "reach") {
    inTree[op.vertex] = true;
    return;
  }
  edgeState[op.edge] = op.kind;
  if (op.kind === "accept") {
    weight += graph.edges[op.edge].w;
    accepted += 1;
  }
}

function draw(): void {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  const at = (v: number): [number, number] => [graph.pts[v][0] * SIZE, HEADER + graph.pts[v][1] * SIZE];
  for (const pass of ["unseen", "reject", "accept", "consider"]) {
    graph.edges.forEach((e, i) => {
      if (edgeState[i] !== pass) return;
      ctx.strokeStyle = pass === "consider" ? CONSIDER : pass === "accept" ? TREE : pass === "reject" ? REJECT : RULE;
      ctx.lineWidth = pass === "consider" || pass === "accept" ? 3 : 1.2;
      ctx.setLineDash(pass === "reject" ? [5, 4] : []);
      ctx.beginPath();
      ctx.moveTo(...at(e.a));
      ctx.lineTo(...at(e.b));
      ctx.stroke();
    });
  }
  ctx.setLineDash([]);
  for (let v = 0; v < N; v++) {
    ctx.fillStyle = inTree[v] ? TREE : PAPER;
    ctx.strokeStyle = INK;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(...at(v), 8, 0, 2 * Math.PI);
    ctx.fill();
    ctx.stroke();
  }
  ctx.fillStyle = "#ffffff";
  ctx.font = "14px monospace";
  ctx.textBaseline = "middle";
  ctx.fillText(`Prim's algorithm  |  tree weight ${weight.toFixed(0)}  |  edges ${accepted} / ${N - 1}`, 6, HEADER / 2);
}

function frame(time: number): void {
  if (time - last >= FRAME_MS) {
    last = time;
    for (let k = 0; k < STEPS && cursor < ops.length; k++) apply(ops[cursor++]);
  }
  draw();
  requestAnimationFrame(frame);
}

canvas.addEventListener("pointerdown", () => {
  seed += 1;
  restart();
});

restart();
requestAnimationFrame(frame);
