// Conway's Game of Life (B3/S23): a cell is born with exactly 3 live neighbors and
// survives with 2 or 3; every other cell dies or stays dead.
//
// A random soup on a wrap-around grid. Live cells start bright and settle to cyan; cells
// that just died glow and cool to deep blue, leaving fading trails. Click or drag to draw
// cells, right-drag to erase, hover to see which rule applies.
// Keys: space pauses, R makes a new soup, C clears.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Rgb = [number, number, number];

const COLS = 100, ROWS = 62, CELL = 8;
const FADE = 28; // generations a dead cell takes to fade back to the background
const NEVER = 0xffff; // age of a cell that has never been alive
const GENS_PER_SECOND = 12;

const wrap = (v: number, n: number): number => ((v % n) + n) % n;

/** Live cells among the 8 neighbors; coordinates wrap around (a torus). */
function neighbors(alive: Uint8Array, x: number, y: number): number {
  let n = 0;
  for (let dy = -1; dy <= 1; dy++)
    for (let dx = -1; dx <= 1; dx++)
      if (dx !== 0 || dy !== 0) n += alive[wrap(y + dy, ROWS) * COLS + wrap(x + dx, COLS)];
  return n;
}

/** The four rules in one line: birth on 3, survival on 2 or 3. */
const nextState = (alive: boolean, n: number): boolean => n === 3 || (alive && n === 2);

function describe(alive: boolean, n: number): string {
  if (alive) return n < 2 ? "dies of underpopulation" : n > 3 ? "dies of overpopulation" : "lives on";
  return n === 3 ? "is born" : "stays dead";
}

let alive = new Uint8Array(COLS * ROWS);
let age = new Uint16Array(COLS * ROWS); // generations alive, or since death

function randomSoup(): void {
  for (let k = 0; k < alive.length; k++) {
    alive[k] = Math.random() < 0.28 ? 1 : 0;
    age[k] = alive[k] ? 0 : NEVER;
  }
}

/** Every cell updates at once, from the old grid into a new one. */
function advance(): void {
  const next = new Uint8Array(alive.length);
  for (let y = 0; y < ROWS; y++)
    for (let x = 0; x < COLS; x++) {
      const k = y * COLS + x;
      next[k] = nextState(alive[k] === 1, neighbors(alive, x, y)) ? 1 : 0;
      const [was, now] = [alive[k], next[k]];
      if (was && now) age[k] = Math.min(age[k] + 1, NEVER - 1);
      else if (was !== now) age[k] = 0;
      else if (age[k] !== NEVER) age[k] = Math.min(age[k] + 1, FADE);
    }
  alive = next;
}

const lerpRgb = (a: Rgb, b: Rgb, t: number): Rgb => [0, 1, 2].map((i) => a[i] + (b[i] - a[i]) * t) as Rgb;
/** Piecewise-linear color ramp through `stops`, t in 0..1. */
function gradient(stops: Rgb[], t: number): Rgb {
  const s = Math.min(Math.max(t, 0), 1) * (stops.length - 1);
  const k = Math.min(Math.floor(s), stops.length - 2);
  return lerpRgb(stops[k], stops[k + 1], s - k);
}
const BG: Rgb = [8, 14, 20];
const LIVE: Rgb[] = [[240, 255, 255], [76, 216, 230]];
const GHOSTS: Rgb[] = [[44, 156, 200], [30, 60, 140], [20, 30, 70], BG];

const canvas = document.createElement("canvas");
canvas.width = COLS * CELL;
canvas.height = ROWS * CELL + 30;
canvas.style.background = "#080e14";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

let playing = true, generation = 0, carry = 0, last = 0;
let hover: [number, number] | null = null;
let stroke: { from: [number, number]; on: boolean } | null = null;
randomSoup();

const cellAt = (e: PointerEvent): [number, number] | null => {
  const x = Math.floor(e.offsetX / CELL), y = Math.floor(e.offsetY / CELL);
  return x >= 0 && x < COLS && y >= 0 && y < ROWS ? [x, y] : null;
};
/** Paint every cell on the line from the last one, so fast drags leave no gaps. */
function paintTo(to: [number, number]): void {
  if (!stroke) return;
  const [fx, fy] = stroke.from, steps = Math.max(Math.abs(to[0] - fx), Math.abs(to[1] - fy), 1);
  for (let s = 0; s <= steps; s++) {
    const k = Math.round(fy + ((to[1] - fy) * s) / steps) * COLS + Math.round(fx + ((to[0] - fx) * s) / steps);
    alive[k] = stroke.on ? 1 : 0;
    age[k] = stroke.on ? 0 : NEVER;
  }
  stroke.from = to;
}
canvas.addEventListener("pointerdown", (e) => {
  const c = cellAt(e);
  if (c) { stroke = { from: c, on: e.button !== 2 }; paintTo(c); }
});
canvas.addEventListener("pointermove", (e) => {
  hover = cellAt(e);
  if (hover && stroke) paintTo(hover);
});
canvas.addEventListener("contextmenu", (e) => e.preventDefault());
window.addEventListener("pointerup", () => (stroke = null));
window.addEventListener("keydown", (e) => {
  if (e.key === " ") playing = !playing;
  if (e.key === "r") { randomSoup(); generation = 0; }
  if (e.key === "c") { alive.fill(0); age.fill(NEVER); generation = 0; }
});

function frame(ms: number): void {
  carry += playing ? (Math.min(ms - last, 100) / 1000) * GENS_PER_SECOND : 0;
  last = ms;
  for (; carry >= 1; carry--) { advance(); generation++; }

  ctx.fillStyle = "#080e14";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  let population = 0;
  for (let k = 0; k < alive.length; k++) {
    population += alive[k];
    if (!alive[k] && age[k] === NEVER) continue;
    const [r, g, b] = alive[k] ? gradient(LIVE, age[k] / 6) : gradient(GHOSTS, age[k] / FADE);
    ctx.fillStyle = `rgb(${r | 0}, ${g | 0}, ${b | 0})`;
    ctx.fillRect((k % COLS) * CELL, Math.floor(k / COLS) * CELL, CELL, CELL);
  }

  // Hover: outline the 3 x 3 neighborhood and name the rule that decides the cell.
  let note = "click/drag: draw   right-drag: erase   space: pause   R: soup   C: clear";
  if (hover) {
    const [x, y] = hover, isAlive = alive[y * COLS + x] === 1, n = neighbors(alive, x, y);
    ctx.strokeStyle = "#e9c46a";
    ctx.strokeRect((x - 1) * CELL, (y - 1) * CELL, 3 * CELL, 3 * CELL);
    note = `${isAlive ? "live" : "dead"} cell, ${n} live neighbors: ${describe(isAlive, n)}`;
  }
  ctx.fillStyle = "#c8d6d0";
  ctx.font = "13px monospace";
  ctx.fillText(`gen ${generation}  alive ${population}${playing ? "" : "  (paused)"}   ${note}`, 8, canvas.height - 10);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
