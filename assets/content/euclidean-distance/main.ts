// Euclidean distance: d = sqrt((x2 - x1)^2 + (y2 - y1)^2).
//
// Nearest point: a probe follows the mouse (or wanders on its own) and snaps to the
// closest point of a scatter plot, as a chart tooltip does. The legs dx and dy and the
// empty circle of radius d show the Pythagorean triangle. K-means: every point joins
// its nearest centroid, then each centroid moves to the mean of its points.
// Click the tabs to switch modes; in K-means, click the plot for new centroids and
// press 2-6 to change k.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };
type Mode = "nearest" | "kmeans";

const W = 100;
const H = 60; // data space, drawn with equal scale on both axes
const SCALE = 7;
const PLOT = { left: 50, top: 70, width: W * SCALE, height: H * SCALE };
const COLORS = ["#5ab4e6", "#f0b45a", "#7bc67b", "#e5707e", "#b39ddb", "#d4c26a"];

const distance = (a: Vec2, b: Vec2): number => Math.sqrt((b.x - a.x) ** 2 + (b.y - a.y) ** 2);

/** Index of the point closest to `at`, and its distance (a linear scan). */
function nearest(points: Vec2[], at: Vec2): { index: number; d: number } {
  let best = { index: 0, d: Infinity };
  points.forEach((p, index) => {
    const d = distance(p, at);
    if (d < best.d) best = { index, d };
  });
  return best;
}

/** One K-means round: assign each point to its nearest centroid, move centroids to the means. */
function kmeansStep(points: Vec2[], centroids: Vec2[]): boolean {
  const assign = points.map((p) => nearest(centroids, p).index);
  let moved = false;
  centroids.forEach((c, k) => {
    const members = points.filter((_, i) => assign[i] === k);
    if (members.length === 0) return; // an empty cluster keeps its centroid
    const mean = {
      x: members.reduce((s, p) => s + p.x, 0) / members.length,
      y: members.reduce((s, p) => s + p.y, 0) / members.length,
    };
    if (distance(mean, c) > 1e-4) moved = true;
    centroids[k] = mean;
  });
  return moved;
}

const rand = (lo: number, hi: number): number => lo + Math.random() * (hi - lo);
const clamp = (v: number, lo: number, hi: number): number => Math.min(hi, Math.max(lo, v));

function makeData(): Vec2[] {
  const blobs = [[25, 18], [75, 18], [25, 42], [75, 42]].map(([x, y]) => ({
    x: x + rand(-8, 8),
    y: y + rand(-5, 5),
  }));
  const bell = (): number => (Math.random() + Math.random() + Math.random() - 1.5) * 9; // cheap bell curve
  return Array.from({ length: 90 }, (_, i) => {
    const c = blobs[i % blobs.length];
    return { x: clamp(c.x + bell(), 1, W - 1), y: clamp(c.y + bell(), 1, H - 1) };
  });
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const toScreen = (p: Vec2): Vec2 => ({
  x: PLOT.left + p.x * SCALE,
  y: PLOT.top + PLOT.height - p.y * SCALE,
});
const toData = (s: Vec2): Vec2 => ({
  x: (s.x - PLOT.left) / SCALE,
  y: (PLOT.top + PLOT.height - s.y) / SCALE,
});
const inPlot = (s: Vec2): boolean =>
  s.x >= PLOT.left && s.x <= PLOT.left + PLOT.width && s.y >= PLOT.top && s.y <= PLOT.top + PLOT.height;
const TABS: { mode: Mode; label: string; x: number }[] = [
  { mode: "nearest", label: "Nearest point", x: 50 },
  { mode: "kmeans", label: "K-means", x: 210 },
];

const points = makeData();
let mode: Mode = "nearest";
let mouse: Vec2 | null = null;
let k = 4;
let centroids: Vec2[] = [];
let iteration = 0;
let converged = false;
let timer = 0;

function restartKmeans(): void {
  centroids = Array.from({ length: k }, () => ({ x: rand(5, W - 5), y: rand(5, H - 5) }));
  iteration = 0;
  converged = false;
  timer = 0.8;
}
restartKmeans();

canvas.addEventListener("pointermove", (e) => {
  const s = { x: e.offsetX, y: e.offsetY };
  mouse = inPlot(s) ? toData(s) : null;
});
canvas.addEventListener("pointerleave", () => (mouse = null));
canvas.addEventListener("pointerdown", (e) => {
  const s = { x: e.offsetX, y: e.offsetY };
  const tab = TABS.find((t) => s.x >= t.x && s.x <= t.x + 150 && s.y >= 18 && s.y <= 50);
  if (tab) mode = tab.mode;
  else if (mode === "kmeans" && inPlot(s)) restartKmeans();
});
window.addEventListener("keydown", (e) => {
  const n = Number(e.key);
  if (n >= 2 && n <= 6) {
    k = n;
    restartKmeans();
  }
});

function dot(p: Vec2, r: number, color: string): void {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
  ctx.fill();
}

function line(a: Vec2, b: Vec2, color: string, width: number): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.beginPath();
  ctx.moveTo(a.x, a.y);
  ctx.lineTo(b.x, b.y);
  ctx.stroke();
}

function drawNearest(seconds: number): string {
  points.forEach((p) => dot(toScreen(p), 3.5, "#6f7d76"));
  const wander = { x: W / 2 + 40 * Math.sin(1.3 * seconds), y: H / 2 + 22 * Math.sin(2.1 * seconds + 0.5) };
  const a = mouse ?? wander;
  const { index, d } = nearest(points, a);
  const b = points[index];
  const [pa, pb, corner] = [toScreen(a), toScreen(b), toScreen({ x: b.x, y: a.y })];
  // Nothing lies inside this circle: that is what "nearest" means.
  ctx.fillStyle = "rgba(157, 179, 255, 0.12)";
  ctx.beginPath();
  ctx.arc(pa.x, pa.y, d * SCALE, 0, Math.PI * 2);
  ctx.fill();
  line(pa, corner, COLORS[0], 2);
  line(corner, pb, COLORS[3], 2);
  line(pa, pb, "#f0b45a", 2.5);
  dot(pb, 7, "#f0b45a");
  dot(pa, 4, "#e4ebe6");
  ctx.font = "12px sans-serif";
  ctx.fillStyle = COLORS[0];
  ctx.fillText(`dx ${Math.abs(b.x - a.x).toFixed(1)}`, (pa.x + corner.x) / 2 - 16, corner.y + 16);
  ctx.fillStyle = COLORS[3];
  ctx.fillText(`dy ${Math.abs(b.y - a.y).toFixed(1)}`, corner.x + 6, (corner.y + pb.y) / 2);
  // The tooltip, as a scatter plot would show it, on the side away from the probe.
  const tx = clamp(pa.x > pb.x ? pb.x - 138 : pb.x + 12, PLOT.left + 4, PLOT.left + PLOT.width - 130);
  const ty = clamp(pa.y < pb.y ? pb.y + 12 : pb.y - 70, PLOT.top + 4, PLOT.top + PLOT.height - 62);
  ctx.fillStyle = "#24302b";
  ctx.fillRect(tx, ty, 126, 58);
  ctx.fillStyle = "#e4ebe6";
  ctx.font = "13px monospace";
  [`point #${index}`, `(${b.x.toFixed(1)}, ${b.y.toFixed(1)})`, `d = ${d.toFixed(2)}`].forEach((s, i) =>
    ctx.fillText(s, tx + 8, ty + 18 + 16 * i),
  );
  return `checked ${points.length} distances, nearest d = ${d.toFixed(2)}`;
}

function drawKmeans(dt: number): string {
  timer -= dt;
  if (timer <= 0) {
    if (converged) restartKmeans();
    else {
      converged = !kmeansStep(points, centroids);
      iteration += 1;
      timer = converged ? 2.5 : 0.7;
    }
  }
  points.forEach((p) => {
    const c = nearest(centroids, p).index;
    line(toScreen(p), toScreen(centroids[c]), COLORS[c] + "33", 1);
    dot(toScreen(p), 4, COLORS[c]);
  });
  centroids.forEach((c, i) => {
    const s = toScreen(c);
    dot(s, 10, "#16201c");
    ctx.strokeStyle = COLORS[i];
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.arc(s.x, s.y, 9, 0, Math.PI * 2);
    ctx.stroke();
    line({ x: s.x - 5, y: s.y }, { x: s.x + 5, y: s.y }, COLORS[i], 2);
    line({ x: s.x, y: s.y - 5 }, { x: s.x, y: s.y + 5 }, COLORS[i], 2);
  });
  return `k = ${k}   iteration ${iteration}  (${converged ? "converged" : "running"})`;
}

let last = 0;
function frame(ms: number): void {
  const dt = Math.min((ms - last) / 1000, 0.1);
  last = ms;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  TABS.forEach((t) => {
    ctx.fillStyle = t.mode === mode ? "#3a4a8a" : "#24302b";
    ctx.fillRect(t.x, 18, 150, 32);
    ctx.fillStyle = "#e4ebe6";
    ctx.font = "15px sans-serif";
    ctx.fillText(t.label, t.x + 14, 40);
  });

  // Grid, one line every 10 data units.
  for (let g = 0; g <= W; g += 10) line(toScreen({ x: g, y: 0 }), toScreen({ x: g, y: H }), "#26332e", 1);
  for (let g = 0; g <= H; g += 10) line(toScreen({ x: 0, y: g }), toScreen({ x: W, y: g }), "#26332e", 1);

  const caption = mode === "nearest" ? drawNearest(ms / 1000 / 1.6) : drawKmeans(dt);
  ctx.fillStyle = "#93a39b";
  ctx.font = "14px monospace";
  ctx.fillText(caption, PLOT.left, 530);
  ctx.fillText(
    mode === "nearest" ? "move the mouse over the plot" : "click the plot for new centroids, keys 2-6 set k",
    PLOT.left,
    570,
  );
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
