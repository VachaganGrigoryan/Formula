// Dijkstra's algorithm: dist(n) = min(dist(n), dist(u) + w(n))
//
// A 31 x 19 grid with a start (orange dot) and a goal (green ring). Dijkstra is run once and recorded as a list of operations (a cell joins the priority queue,
// a cell is settled, a cell joins the final path); the animation then replays that list at a fixed speed.
// Cells are settled in order of total cost, and mud costs 5 instead of 1, so the path detours around mud
// when that is cheaper.
// Click a cell to toggle a wall or mud (press w to switch the brush) and run the search
// again, press m for a random maze and r to restart the replay.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Cell = { r: number; c: number };
type Op =
  | { kind: "frontier"; cell: Cell }
  | { kind: "visit"; cell: Cell }
  | { kind: "path"; cell: Cell }
  | { kind: "none" };

const W = 31;
const H = 19;
const SIZE = 28; // pixels per cell
const HEADER = 30;
const STEPS = 4; // recorded operations replayed per frame
const START: Cell = { r: Math.floor(H / 2), c: 1 };
const GOAL: Cell = { r: Math.floor(H / 2), c: W - 2 };
const PAPER = "#16201c";
const WALL = "#d8e0dc";
const FRONTIER = "#9b6a2a";
const VISITED = "#2c4a6a";
const PATH = "#a8c4ff";
const ORANGE = "#f0a040";
const GREEN = "#4cc38a";
const MUD_COLOR = "#5a4a2a";
const MUD = 5; // cost of entering a mud cell

const canvas = document.createElement("canvas");
canvas.width = W * SIZE;
canvas.height = H * SIZE + HEADER;
canvas.style.background = PAPER;
canvas.tabIndex = 0;
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const at = (cell: Cell): number => cell.r * W + cell.c;
const same = (a: Cell, b: Cell): boolean => a.r === b.r && a.c === b.c;

let walls: boolean[] = new Array<boolean>(W * H).fill(false);
let mud: boolean[] = defaultMud();
let brush: "wall" | "mud" = "wall";
let ops: Op[] = [];
let cursor = 0;
let frontier: boolean[] = [];
let visited: boolean[] = [];
let path: boolean[] = [];
let pathLength = 0;
let noPath = false;
let seed = 0;

/** A block of mud across the straight route: the detour around it is cheaper. */
function defaultMud(): boolean[] {
  const grid: boolean[] = new Array<boolean>(W * H).fill(false);
  for (let r = 5; r < 14; r++) for (let c = 12; c < 19; c++) grid[at({ r, c })] = true;
  return grid;
}

/** Open 4-neighbours in the order up, right, down, left. */
function neighbors(cell: Cell): Cell[] {
  const out: Cell[] = [];
  for (const [dr, dc] of [[-1, 0], [0, 1], [1, 0], [0, -1]]) {
    const n = { r: cell.r + dr, c: cell.c + dc };
    if (n.r >= 0 && n.r < H && n.c >= 0 && n.c < W && !walls[at(n)]) out.push(n);
  }
  return out;
}

/** A binary min-heap of [f, h, cell] triples, ordered lexicographically. */
class MinHeap {
  private items: [number, number, number][] = [];

  get size(): number {
    return this.items.length;
  }

  private less(a: [number, number, number], b: [number, number, number]): boolean {
    return a[0] !== b[0] ? a[0] < b[0] : a[1] !== b[1] ? a[1] < b[1] : a[2] < b[2];
  }

  push(item: [number, number, number]): void {
    const a = this.items;
    a.push(item);
    let i = a.length - 1;
    while (i > 0) {
      const parent = (i - 1) >> 1;
      if (!this.less(a[i], a[parent])) break;
      [a[i], a[parent]] = [a[parent], a[i]];
      i = parent;
    }
  }

  pop(): [number, number, number] {
    const a = this.items;
    const top = a[0];
    const last = a.pop()!;
    if (a.length > 0) {
      a[0] = last;
      let i = 0;
      for (;;) {
        const l = 2 * i + 1;
        const r = l + 1;
        let m = i;
        if (l < a.length && this.less(a[l], a[m])) m = l;
        if (r < a.length && this.less(a[r], a[m])) m = r;
        if (m === i) break;
        [a[i], a[m]] = [a[m], a[i]];
        i = m;
      }
    }
    return top;
  }
}

/** Estimated remaining cost: zero (no guess, plain Dijkstra). */
const heuristic = (_n: Cell): number => 0;

/** Run Dijkstra and log every step. */
function record(): Op[] {
  const log: Op[] = [{ kind: "frontier", cell: START }];
  const dist = new Map<number, number>([[at(START), 0]]);
  const parent = new Map<number, Cell>();
  const done = new Set<number>();
  const cells: Cell[] = new Array<Cell>(W * H);
  cells[at(START)] = START;
  const heap = new MinHeap();
  heap.push([heuristic(START), heuristic(START), at(START)]); // [f, h, cell]
  let found = false;
  while (heap.size > 0) {
    const [f, h, id] = heap.pop();
    const cell = cells[id];
    if (done.has(id) || f !== dist.get(id)! + h) continue; // stale entry: skip it
    done.add(id);
    log.push({ kind: "visit", cell });
    if (same(cell, GOAL)) {
      found = true;
      break;
    }
    for (const n of neighbors(cell)) {
      const nd = dist.get(id)! + (mud[at(n)] ? MUD : 1);
      if (nd < (dist.get(at(n)) ?? Infinity)) {
        // relaxation
        dist.set(at(n), nd);
        parent.set(at(n), cell);
        cells[at(n)] = n;
        log.push({ kind: "frontier", cell: n });
        heap.push([nd + heuristic(n), heuristic(n), at(n)]);
      }
    }
  }
  if (!found) return [...log, { kind: "none" }];
  const route: Cell[] = [GOAL];
  while (parent.has(at(route[route.length - 1]))) route.push(parent.get(at(route[route.length - 1]))!);
  return [...log, ...route.reverse().map((cell): Op => ({ kind: "path", cell }))];
}

/** A small seeded generator, so a maze is repeatable. */
function rng(s: number): () => number {
  let x = s * 2654435761 + 12345;
  return () => {
    x = (x ^ (x << 13)) >>> 0;
    x = (x ^ (x >>> 17)) >>> 0;
    x = (x ^ (x << 5)) >>> 0;
    return x / 4294967296;
  };
}

/** A perfect maze by recursive backtracking on the odd cells. */
function maze(s: number): boolean[] {
  const next = rng(s);
  const grid: boolean[] = new Array<boolean>(W * H).fill(true);
  const seen = new Set<number>([at({ r: 1, c: 1 })]);
  grid[at({ r: 1, c: 1 })] = false;
  const stack: Cell[] = [{ r: 1, c: 1 }];
  while (stack.length > 0) {
    const cur = stack[stack.length - 1];
    const options = [[-2, 0], [0, 2], [2, 0], [0, -2]]
      .map(([dr, dc]) => ({ r: cur.r + dr, c: cur.c + dc }))
      .filter((n) => n.r >= 1 && n.r < H - 1 && n.c >= 1 && n.c < W - 1 && !seen.has(at(n)));
    if (options.length === 0) {
      stack.pop();
      continue;
    }
    const n = options[Math.floor(next() * options.length)];
    grid[at({ r: (cur.r + n.r) / 2, c: (cur.c + n.c) / 2 })] = false;
    grid[at(n)] = false;
    seen.add(at(n));
    stack.push(n);
  }
  grid[at(START)] = false;
  grid[at(GOAL)] = false;
  return grid;
}

/** Record the search on the current walls and replay it from the start. */
function restart(): void {
  ops = record();
  cursor = 0;
  frontier = new Array<boolean>(W * H).fill(false);
  visited = new Array<boolean>(W * H).fill(false);
  path = new Array<boolean>(W * H).fill(false);
  pathLength = 0;
  noPath = false;
}

function apply(op: Op): void {
  if (op.kind === "frontier") frontier[at(op.cell)] = true;
  else if (op.kind === "visit") {
    frontier[at(op.cell)] = false;
    visited[at(op.cell)] = true;
  } else if (op.kind === "path") {
    path[at(op.cell)] = true;
    pathLength += 1;
  } else noPath = true;
}

function draw(): void {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  let count = 0;
  let cost = 0;
  for (let i = 0; i < W * H; i++) {
    if (visited[i]) count += 1;
    if (path[i]) cost += mud[i] ? MUD : 1;
    const color = walls[i] ? WALL : mud[i] ? MUD_COLOR : path[i] ? PATH : frontier[i] ? FRONTIER : visited[i] ? VISITED : PAPER;
    ctx.fillStyle = color;
    ctx.fillRect((i % W) * SIZE, HEADER + Math.floor(i / W) * SIZE, SIZE, SIZE);
  }
  ctx.strokeStyle = "#ffffff14";
  ctx.lineWidth = 1;
  for (let c = 0; c <= W; c++) {
    ctx.beginPath();
    ctx.moveTo(c * SIZE, HEADER);
    ctx.lineTo(c * SIZE, HEADER + H * SIZE);
    ctx.stroke();
  }
  for (let r = 0; r <= H; r++) {
    ctx.beginPath();
    ctx.moveTo(0, HEADER + r * SIZE);
    ctx.lineTo(W * SIZE, HEADER + r * SIZE);
    ctx.stroke();
  }
  const mid = (cell: Cell): [number, number] => [(cell.c + 0.5) * SIZE, HEADER + (cell.r + 0.5) * SIZE];
  ctx.fillStyle = ORANGE;
  ctx.beginPath();
  ctx.arc(...mid(START), SIZE * 0.32, 0, 2 * Math.PI);
  ctx.fill();
  ctx.strokeStyle = GREEN;
  ctx.lineWidth = 2.5;
  ctx.beginPath();
  ctx.arc(...mid(GOAL), SIZE * 0.32, 0, 2 * Math.PI);
  ctx.stroke();
  ctx.fillStyle = "#ffffff";
  ctx.font = "14px monospace";
  ctx.textBaseline = "middle";
  const steps = Math.max(pathLength - 1, 0);
  if (pathLength > 0) cost -= 1; // the start cell is not entered
  ctx.fillText(
    noPath ? "No path: the goal is walled off" : `Dijkstra's algorithm  |  visited ${count}  |  path ${steps} steps  |  cost ${cost}`,
    6,
    HEADER / 2,
  );
}

function frame(): void {
  for (let k = 0; k < STEPS && cursor < ops.length; k++) apply(ops[cursor++]);
  draw();
  requestAnimationFrame(frame);
}

canvas.addEventListener("pointerdown", (e) => {
  const box = canvas.getBoundingClientRect();
  const cell = {
    r: Math.floor((e.clientY - box.top - HEADER) / SIZE),
    c: Math.floor((e.clientX - box.left) / SIZE),
  };
  if (cell.r < 0 || cell.r >= H || cell.c < 0 || cell.c >= W) return;
  if (same(cell, START) || same(cell, GOAL)) return;
  if (brush === "mud") {
    if (!walls[at(cell)]) mud[at(cell)] = !mud[at(cell)];
  } else {
    walls[at(cell)] = !walls[at(cell)];
    mud[at(cell)] = false;
  }
  restart();
});

canvas.addEventListener("keydown", (e) => {
  if (e.key === "m") {
    seed += 1;
    walls = maze(seed);
    mud = new Array<boolean>(W * H).fill(false);
    restart();
  } else if (e.key === "w") {
    brush = brush === "mud" ? "wall" : "mud";
  } else if (e.key === "r") {
    restart();
  }
});

restart();
frame();
