// Depth-first search: follow the newest frontier cell first (a stack)
//
// A 31 x 19 grid with a start (orange dot) and a goal (green ring). DFS is run once and recorded as a list of operations (a cell joins the stack, a cell is
// visited, a cell joins the final path); the animation then replays that list at a fixed speed. It dives
// down one corridor (up first) before backing up, so the path it finds is valid but usually far from the
// shortest.
// Click a cell to toggle a wall and run the search again, press m for a random maze and r to
// restart the replay.
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
let ops: Op[] = [];
let cursor = 0;
let frontier: boolean[] = [];
let visited: boolean[] = [];
let path: boolean[] = [];
let pathLength = 0;
let noPath = false;
let seed = 0;

/** Open 4-neighbours in the order up, right, down, left. */
function neighbors(cell: Cell): Cell[] {
  const out: Cell[] = [];
  for (const [dr, dc] of [[-1, 0], [0, 1], [1, 0], [0, -1]]) {
    const n = { r: cell.r + dr, c: cell.c + dc };
    if (n.r >= 0 && n.r < H && n.c >= 0 && n.c < W && !walls[at(n)]) out.push(n);
  }
  return out;
}

/** Run DFS and log every step. */
function record(): Op[] {
  const log: Op[] = [{ kind: "frontier", cell: START }];
  const visited = new Set<number>();
  const parent = new Map<number, Cell>();
  const stack: { cell: Cell; from: Cell | null }[] = [{ cell: START, from: null }];
  let found = false;
  while (stack.length > 0) {
    const { cell, from } = stack.pop()!;
    if (visited.has(at(cell))) continue; // mark on pop: stale duplicates are skipped
    visited.add(at(cell));
    if (from !== null) parent.set(at(cell), from);
    log.push({ kind: "visit", cell });
    if (same(cell, GOAL)) {
      found = true;
      break;
    }
    // reversed, so "up" ends on top of the stack and is tried first
    for (const n of neighbors(cell).reverse()) {
      if (!visited.has(at(n))) {
        log.push({ kind: "frontier", cell: n });
        stack.push({ cell: n, from: cell });
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
  for (let i = 0; i < W * H; i++) {
    if (visited[i]) count += 1;
    const color = walls[i] ? WALL : path[i] ? PATH : frontier[i] ? FRONTIER : visited[i] ? VISITED : PAPER;
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
  ctx.fillText(
    noPath ? "No path: the goal is walled off" : `Depth-first search  |  visited ${count}  |  path ${steps} steps`,
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
  walls[at(cell)] = !walls[at(cell)];
  restart();
});

canvas.addEventListener("keydown", (e) => {
  if (e.key === "m") {
    seed += 1;
    walls = maze(seed);
    restart();
  } else if (e.key === "r") {
    restart();
  }
});

restart();
frame();
