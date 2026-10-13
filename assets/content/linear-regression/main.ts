// Line of best fit (least squares):
//   m = sum((x - mean_x) * (y - mean_y)) / sum((x - mean_x)^2),   b = mean_y - m * mean_x.
//
// The line through the mean point that makes the squared vertical misses (residuals)
// as small as possible. Drag points, click to add one, right-click to remove one.
// Keys: R residuals, S squared errors (drawn as real squares), N new data,
// Up/Down change the noise of new data.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };
type Fit = { slope: number; intercept: number; meanX: number; meanY: number; r2: number };

/** Least-squares fit; null with fewer than two points or when every x is the same. */
function fit(points: Vec2[]): Fit | null {
  const n = points.length;
  if (n < 2) return null;
  const meanX = points.reduce((s, p) => s + p.x, 0) / n;
  const meanY = points.reduce((s, p) => s + p.y, 0) / n;
  const sxy = points.reduce((s, p) => s + (p.x - meanX) * (p.y - meanY), 0);
  const sxx = points.reduce((s, p) => s + (p.x - meanX) ** 2, 0);
  if (sxx <= 1e-12) return null; // a vertical line has no slope
  const slope = sxy / sxx;
  const intercept = meanY - slope * meanX;
  const sst = points.reduce((s, p) => s + (p.y - meanY) ** 2, 0);
  const r2 = sst > 0 ? 1 - sumSquaredResiduals(points, slope, intercept) / sst : 1;
  return { slope, intercept, meanX, meanY, r2 };
}

const sumSquaredResiduals = (points: Vec2[], m: number, b: number): number =>
  points.reduce((s, p) => s + (p.y - (m * p.x + b)) ** 2, 0);

const SPAN = 10; // data space is 0..10 on both axes
const PLOT = { left: 60, top: 40, size: 480 };
const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const toScreen = (p: Vec2): Vec2 => ({
  x: PLOT.left + (p.x / SPAN) * PLOT.size,
  y: PLOT.top + PLOT.size - (p.y / SPAN) * PLOT.size,
});
const clamp = (v: number): number => Math.min(SPAN, Math.max(0, v));
const toData = (s: Vec2): Vec2 => ({
  x: clamp(((s.x - PLOT.left) / PLOT.size) * SPAN),
  y: clamp(((PLOT.top + PLOT.size - s.y) / PLOT.size) * SPAN),
});

let noise = 1.2;
let points: Vec2[] = [];
let showResiduals = true;
let showSquares = false;
let dragging: number | null = null;

function newData(): void {
  const slope = -0.5 + Math.random() * 1.4;
  const intercept = 5 - slope * 5;
  const bell = (): number => Math.random() + Math.random() + Math.random() - 1.5; // cheap bell curve
  points = Array.from({ length: 18 }, () => {
    const x = 0.6 + Math.random() * 8.8;
    return { x, y: Math.min(9.8, Math.max(0.2, slope * x + intercept + bell() * 2 * noise)) };
  });
}
newData();

const pick = (s: Vec2): number =>
  points.findIndex((p) => Math.hypot(toScreen(p).x - s.x, toScreen(p).y - s.y) < 14);
const inPlot = (s: Vec2): boolean =>
  s.x >= PLOT.left && s.x <= PLOT.left + PLOT.size && s.y >= PLOT.top && s.y <= PLOT.top + PLOT.size;
canvas.addEventListener("contextmenu", (e) => e.preventDefault());
canvas.addEventListener("pointerdown", (e) => {
  const s = { x: e.offsetX, y: e.offsetY };
  const k = pick(s);
  if (e.button === 2) {
    if (k >= 0) points.splice(k, 1);
  } else if (k >= 0) dragging = k;
  else if (inPlot(s)) points.push(toData(s));
});
canvas.addEventListener("pointermove", (e) => {
  if (dragging !== null) points[dragging] = toData({ x: e.offsetX, y: e.offsetY });
});
window.addEventListener("pointerup", () => (dragging = null));
window.addEventListener("keydown", (e) => {
  if (e.key === "r") showResiduals = !showResiduals;
  if (e.key === "s") showSquares = !showSquares;
  if (e.key === "n") newData();
  if (e.key === "ArrowUp" || e.key === "ArrowDown") {
    noise = Math.min(3, Math.max(0, noise + (e.key === "ArrowUp" ? 0.25 : -0.25)));
    newData();
  }
});

function segment(a: Vec2, b: Vec2, color: string, width: number, dash: number[] = []): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.setLineDash(dash);
  ctx.beginPath();
  ctx.moveTo(a.x, a.y);
  ctx.lineTo(b.x, b.y);
  ctx.stroke();
  ctx.setLineDash([]);
}

function frame(): void {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.font = "12px sans-serif";
  for (let k = 0; k <= SPAN; k++) {
    const a = toScreen({ x: k, y: 0 });
    const b = toScreen({ x: 0, y: k });
    segment(a, toScreen({ x: k, y: SPAN }), "#24302b", 1);
    segment(b, toScreen({ x: SPAN, y: k }), "#24302b", 1);
    if (k % 2 === 0) {
      ctx.fillStyle = "#93a39b";
      ctx.fillText(String(k), a.x - 4, a.y + 16);
      ctx.fillText(String(k), b.x - 22, b.y + 4);
    }
  }

  const f = fit(points);
  const lines: string[] = [];
  if (f) {
    const predict = (x: number): number => f.slope * x + f.intercept;
    for (const p of points) {
      const [a, b] = [toScreen(p), toScreen({ x: p.x, y: predict(p.x) })];
      if (showSquares) {
        // The square of each residual, drawn as an actual square.
        const side = b.y - a.y;
        const dir = f.slope >= 0 ? -1 : 1;
        ctx.fillStyle = "rgba(229, 112, 126, 0.12)";
        ctx.strokeStyle = "rgba(229, 112, 126, 0.5)";
        ctx.lineWidth = 1;
        ctx.fillRect(a.x, a.y, dir * Math.abs(side), side);
        ctx.strokeRect(a.x, a.y, dir * Math.abs(side), side);
      }
      if (showResiduals) segment(a, b, "#e5707e", 1.5);
    }
    // The fitted line always passes through the mean point.
    const mean = toScreen({ x: f.meanX, y: f.meanY });
    const [x0, x1, y0, y1] = [PLOT.left, PLOT.left + PLOT.size, PLOT.top, PLOT.top + PLOT.size];
    segment({ x: mean.x, y: y0 }, { x: mean.x, y: y1 }, "rgba(90, 180, 230, 0.6)", 1, [4, 4]);
    segment({ x: x0, y: mean.y }, { x: x1, y: mean.y }, "rgba(229, 112, 126, 0.6)", 1, [4, 4]);
    ctx.save();
    ctx.beginPath();
    ctx.rect(PLOT.left, PLOT.top, PLOT.size, PLOT.size);
    ctx.clip();
    segment(toScreen({ x: 0, y: predict(0) }), toScreen({ x: SPAN, y: predict(SPAN) }), "#f0b45a", 3);
    ctx.restore();
    ctx.fillStyle = "#16201c";
    ctx.strokeStyle = "#f0b45a";
    ctx.lineWidth = 2.5;
    ctx.beginPath();
    ctx.arc(mean.x, mean.y, 6, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "#f0b45a";
    ctx.fillText("(mean x, mean y)", mean.x + 9, mean.y - 9);
    const sign = f.intercept < 0 ? "-" : "+";
    const sse = sumSquaredResiduals(points, f.slope, f.intercept);
    lines.push(`y = ${f.slope.toFixed(2)}x ${sign} ${Math.abs(f.intercept).toFixed(2)}`);
    lines.push(`R^2 = ${f.r2.toFixed(2)}`, `sum residual^2 = ${sse.toFixed(1)}`);
  } else {
    lines.push("add points with", "different x", "to fit a line");
  }
  points.forEach((p, i) => {
    const s = toScreen(p);
    ctx.fillStyle = "#e4ebe6";
    ctx.beginPath();
    ctx.arc(s.x, s.y, i === dragging ? 7 : 5, 0, Math.PI * 2);
    ctx.fill();
  });

  // Side panel: the fit, then the controls.
  lines.push(`n = ${points.length}`, "", `[R] residuals ${showResiduals ? "on" : "off"}`,
    `[S] squared errors ${showSquares ? "on" : "off"}`, "[N] new data", `[Up/Down] noise ${noise.toFixed(2)}`,
    "", "drag: move a point", "click: add a point", "right-click: remove");
  ctx.font = "15px monospace";
  lines.forEach((text, i) => {
    ctx.fillStyle = i < 4 ? "#e4ebe6" : "#93a39b";
    ctx.fillText(text, 570, 70 + 26 * i);
  });
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
