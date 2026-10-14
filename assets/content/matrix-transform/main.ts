// Matrix transformation: x' = a x + b y,  y' = c x + d y.
//
// A 2x2 matrix moves every point of the plane. Its columns are where the basis
// vectors i = (1, 0) and j = (0, 1) land: drag the tips of the two arrows and the
// whole grid follows. The shaded cell is the image of the unit square; its area is
// |det| and it turns red when det < 0 (the plane is mirrored). Dashed lines are the
// eigenvectors, the directions the matrix only stretches: A v = lambda v.
// Keys 1-7 load a preset and play the morph from the identity; space replays it.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };
/** [[a, b], [c, d]] acting on column vectors. */
type Mat2 = { a: number; b: number; c: number; d: number };

const apply = (m: Mat2, p: Vec2): Vec2 => ({ x: m.a * p.x + m.b * p.y, y: m.c * p.x + m.d * p.y });
const det = (m: Mat2): number => m.a * m.d - m.b * m.c;
const IDENTITY: Mat2 = { a: 1, b: 0, c: 0, d: 1 };

type Eigen =
  | { kind: "real"; pairs: { lambda: number; v: Vec2 }[] } // largest first
  | { kind: "all"; lambda: number } // every direction (a uniform scale)
  | { kind: "complex"; re: number; im: number };

function eigen(m: Mat2): Eigen {
  const h = (m.a + m.d) / 2;
  const disc = h * h - det(m);
  const scale = Math.max(1, Math.abs(m.a), Math.abs(m.b), Math.abs(m.c), Math.abs(m.d));
  const eps = 1e-6 * scale * scale;
  if (disc < -eps) return { kind: "complex", re: h, im: Math.sqrt(-disc) };
  const lambdas = disc <= eps ? [h] : [h + Math.sqrt(disc), h - Math.sqrt(disc)];
  const pairs: { lambda: number; v: Vec2 }[] = [];
  for (const lambda of lambdas) {
    // An eigenvector is perpendicular to a nonzero row of A - lambda I.
    const u = { x: m.b, y: lambda - m.a };
    const w = { x: lambda - m.d, y: m.c };
    const v = Math.hypot(u.x, u.y) >= Math.hypot(w.x, w.y) ? u : w;
    const n = Math.hypot(v.x, v.y);
    if (n > 1e-6) pairs.push({ lambda, v: { x: v.x / n, y: v.y / n } });
  }
  return pairs.length ? { kind: "real", pairs } : { kind: "all", lambda: h };
}

const C30 = Math.cos(Math.PI / 6);
const S30 = Math.sin(Math.PI / 6);
const PRESETS: Record<string, Mat2> = {
  "1": IDENTITY,
  "2": { a: 1.5, b: 0, c: 0, d: 0.6 }, // scale
  "3": { a: C30, b: -S30, c: S30, d: C30 }, // rotate 30 degrees
  "4": { a: 1, b: 1, c: 0, d: 1 }, // shear
  "5": { a: 1, b: 0, c: 0, d: -1 }, // reflect in the x axis
  "6": { a: 1, b: 0, c: 0, d: 0 }, // project onto the x axis
  "7": { a: 1, b: 2, c: 0.5, d: 1 }, // singular: both columns on one line
};
const HOUSE: Vec2[] = [[0.2, 0], [0.8, 0], [0.8, 0.6], [1, 0.6], [0.5, 1.05], [0, 0.6], [0.2, 0.6]]
  .map(([x, y]) => ({ x, y }));
const SQUARE: Vec2[] = [[0, 0], [1, 0], [1, 1], [0, 1]].map(([x, y]) => ({ x, y }));
const SIZE = 700;
const UNIT = 100; // pixels per unit
const GRID = 8; // grid lines on each side of the origin
const LIMIT = 5; // matrix entries stay within +-LIMIT
const SECONDS = 1.6; // length of the morph from the identity
const ORANGE = "#f0a040";
const GREEN = "#4cc38a";
const BLUE = "#6a8fd8";
const RED = "#e5707e";

const canvas = document.createElement("canvas");
canvas.width = SIZE;
canvas.height = SIZE;
canvas.style.background = "#16201c";
canvas.tabIndex = 0;
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const toScreen = (p: Vec2): Vec2 => ({ x: SIZE / 2 + p.x * UNIT, y: SIZE / 2 - p.y * UNIT });
const toWorld = (s: Vec2): Vec2 => ({ x: (s.x - SIZE / 2) / UNIT, y: (SIZE / 2 - s.y) / UNIT });
let target: Mat2 = { a: 1.5, b: 0.5, c: 0.25, d: 1 };
let t = 0; // morph progress
let dragging: 0 | 1 | null = null;

/** The matrix on screen: part way from the identity while the morph plays. */
function shown(): Mat2 {
  const s = Math.min(t, 1);
  const e = s * s * (3 - 2 * s);
  return { a: 1 + e * (target.a - 1), b: e * target.b, c: e * target.c, d: 1 + e * (target.d - 1) };
}

const column = (m: Mat2, k: 0 | 1): Vec2 => (k === 0 ? { x: m.a, y: m.c } : { x: m.b, y: m.d });

function line(p: Vec2, q: Vec2, color: string, width = 1, dash: number[] = []): void {
  const a = toScreen(p);
  const b = toScreen(q);
  ctx.setLineDash(dash);
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.beginPath();
  ctx.moveTo(a.x, a.y);
  ctx.lineTo(b.x, b.y);
  ctx.stroke();
  ctx.setLineDash([]);
}

function polygon(points: Vec2[]): void {
  ctx.beginPath();
  points.map(toScreen).forEach((p, k) => (k ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y)));
  ctx.closePath();
}

function arrow(to: Vec2, color: string): void {
  line({ x: 0, y: 0 }, to, color, 3);
  const tip = toScreen(to);
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(tip.x, tip.y, 7, 0, 2 * Math.PI);
  ctx.fill();
}

function draw(): void {
  const m = shown();
  ctx.clearRect(0, 0, SIZE, SIZE);
  for (let k = -GRID; k <= GRID; k++) {
    line({ x: k, y: -GRID }, { x: k, y: GRID }, "#ffffff14");
    line({ x: -GRID, y: k }, { x: GRID, y: k }, "#ffffff14");
  }
  for (let k = -GRID; k <= GRID; k++) {
    const color = k === 0 ? "#ffffff88" : "#6a8fd855";
    line(apply(m, { x: k, y: -GRID }), apply(m, { x: k, y: GRID }), color);
    line(apply(m, { x: -GRID, y: k }), apply(m, { x: GRID, y: k }), color);
  }
  const d = det(m);
  polygon(SQUARE.map((p) => apply(m, p)));
  ctx.fillStyle = d < 0 ? RED + "55" : BLUE + "55";
  ctx.fill();
  ctx.setLineDash([4, 4]);
  polygon(HOUSE);
  ctx.strokeStyle = "#ffffff55";
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.setLineDash([]);
  polygon(HOUSE.map((p) => apply(m, p)));
  ctx.strokeStyle = "#a8c4ff";
  ctx.lineWidth = 2;
  ctx.stroke();

  const e = eigen(m);
  let text: string;
  if (e.kind === "real") {
    for (const { v } of e.pairs) line({ x: -9 * v.x, y: -9 * v.y }, { x: 9 * v.x, y: 9 * v.y }, "#ffffffaa", 1.5, [6, 4]);
    text = "lambda = " + e.pairs.map((p) => p.lambda.toFixed(2)).join(", ");
  } else if (e.kind === "all") {
    text = `lambda = ${e.lambda.toFixed(2)}: every direction is an eigenvector`;
  } else {
    text = `lambda = ${e.re.toFixed(2)} +- ${e.im.toFixed(2)}i: no real eigenvectors`;
  }
  arrow(column(m, 0), ORANGE);
  arrow(column(m, 1), GREEN);

  const scale = Math.max(1, Math.abs(m.a), Math.abs(m.b), Math.abs(m.c), Math.abs(m.d));
  const area = Math.abs(d) < 1e-4 * scale * scale ? "det = 0: the plane collapses"
    : `det = ${d.toFixed(2)}` + (d < 0 ? " (mirrored)" : "");
  const f = (x: number): string => x.toFixed(2).padStart(6);
  ctx.fillStyle = "#e8ecef";
  ctx.font = "13px monospace";
  [`[${f(m.a)} ${f(m.b)} ]`, `[${f(m.c)} ${f(m.d)} ]`, area, text, "drag the tips | keys 1-7 presets | space replays"]
    .forEach((s, k) => ctx.fillText(s, 12, 22 + 17 * k));
}

canvas.addEventListener("pointerdown", (e) => {
  const m = shown();
  for (const k of [0, 1] as const) {
    const tip = toScreen(column(m, k));
    if (Math.hypot(e.offsetX - tip.x, e.offsetY - tip.y) < 16) {
      target = m;
      t = 1;
      dragging = k;
    }
  }
});
canvas.addEventListener("pointermove", (e) => {
  if (dragging === null) return;
  const p = toWorld({ x: e.offsetX, y: e.offsetY });
  const snap = (x: number): number => Math.min(Math.max(Math.round(x * 2) / 2, -LIMIT), LIMIT);
  target = dragging === 0 ? { ...target, a: snap(p.x), c: snap(p.y) } : { ...target, b: snap(p.x), d: snap(p.y) };
});
window.addEventListener("pointerup", () => (dragging = null));
window.addEventListener("keydown", (e) => {
  if (e.key in PRESETS) {
    target = PRESETS[e.key];
    t = 0;
  } else if (e.key === " ") {
    t = 0;
    e.preventDefault();
  }
});

let last = performance.now();
function frame(now: number): void {
  t += (now - last) / 1000 / SECONDS;
  last = now;
  draw();
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
