// Cubic Bezier curve: B(t) = (1-t)^3 P0 + 3(1-t)^2 t P1 + 3(1-t) t^2 P2 + t^3 P3.
//
// The curve is drawn up to the current t, which sweeps 0 -> 1 -> 0. The colored lines
// are de Casteljau's construction (lerps of lerps) that lands on the same point, and
// the bars below show the four Bernstein weights at t.
// Drag the four points; Space plays or pauses, C toggles the construction.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };
type Quad = [Vec2, Vec2, Vec2, Vec2];

/** The four Bernstein weights (1-t)^3, 3(1-t)^2 t, 3(1-t) t^2, t^3. They sum to 1. */
function bernstein(t: number): [number, number, number, number] {
  const u = 1 - t;
  return [u * u * u, 3 * u * u * t, 3 * u * t * t, t * t * t];
}

/** B(t) as the weighted sum of the control points. */
function bezier(p: Quad, t: number): Vec2 {
  const w = bernstein(t);
  return p.reduce((acc, q, i) => ({ x: acc.x + w[i] * q.x, y: acc.y + w[i] * q.y }), { x: 0, y: 0 });
}

const lerp = (a: Vec2, b: Vec2, t: number): Vec2 => ({ x: a.x + t * (b.x - a.x), y: a.y + t * (b.y - a.y) });

/** De Casteljau: four points -> three -> two -> one, by repeated lerps. */
function deCasteljau(p: Quad, t: number): { a: Vec2[]; b: Vec2[]; c: Vec2 } {
  const a = [lerp(p[0], p[1], t), lerp(p[1], p[2], t), lerp(p[2], p[3], t)];
  const b = [lerp(a[0], a[1], t), lerp(a[1], a[2], t)];
  return { a, b, c: lerp(b[0], b[1], t) };
}

/** Map any real s onto 0..1..0 at constant speed (a triangle wave). */
const pingPong = (s: number): number => 1 - Math.abs(1 - (((s % 2) + 2) % 2));

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const points: Quad = [{ x: 70, y: 380 }, { x: 180, y: 60 }, { x: 560, y: 50 }, { x: 730, y: 330 }];
let dragging: number | null = null;
let playing = true;
let construction = true;
let phase = 0;

canvas.addEventListener("pointerdown", (e) => {
  const i = points.findIndex((q) => Math.hypot(q.x - e.offsetX, q.y - e.offsetY) < 16);
  dragging = i >= 0 ? i : null;
});
canvas.addEventListener("pointermove", (e) => {
  if (dragging !== null) points[dragging] = { x: e.offsetX, y: Math.min(e.offsetY, 420) };
});
window.addEventListener("pointerup", () => (dragging = null));
window.addEventListener("keydown", (e) => {
  if (e.key === " ") playing = !playing;
  if (e.key === "c" || e.key === "C") construction = !construction;
});

function polyline(pts: Vec2[], color: string, width: number, dash: number[] = []): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.setLineDash(dash);
  ctx.beginPath();
  pts.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
  ctx.stroke();
  ctx.setLineDash([]);
}

function dot(p: Vec2, r: number, fill: string, stroke?: string): void {
  ctx.fillStyle = fill;
  ctx.beginPath();
  ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
  ctx.fill();
  if (stroke) {
    ctx.strokeStyle = stroke;
    ctx.lineWidth = 2;
    ctx.stroke();
  }
}

let last = 0;
function frame(ms: number): void {
  const dt = Math.min((ms - last) / 1000, 0.1);
  last = ms;
  if (playing) phase += 0.3 * dt;
  const t = pingPong(phase);
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Control polygon, the whole curve faintly, and the part up to t brightly.
  const steps = 120;
  const curve = Array.from({ length: steps + 1 }, (_, j) => bezier(points, j / steps));
  polyline(points, "#55635d", 1, [4, 4]);
  polyline(curve, "rgba(240, 180, 41, 0.35)", 1.5);
  polyline(curve.slice(0, Math.floor(t * steps) + 1).concat([bezier(points, t)]), "#f0b429", 3);

  const { a, b, c } = deCasteljau(points, t);
  if (construction) {
    polyline(a, "#ef6b5b", 1.5);
    polyline(b, "#5bc88a", 1.5);
    a.forEach((q) => dot(q, 3.5, "#ef6b5b"));
    b.forEach((q) => dot(q, 3.5, "#5bc88a"));
  }
  ctx.font = "13px sans-serif";
  ctx.textAlign = "center";
  points.forEach((q, i) => {
    dot(q, 8, i === 0 || i === 3 ? "#e4ebe6" : "#16201c", "#93a39b");
    ctx.fillStyle = "#93a39b";
    ctx.fillText(`P${i}`, q.x, q.y - 14);
  });
  dot(c, 6.5, "#f0b429", "#16201c");

  // The Bernstein weights: how much each point pulls on B(t) right now.
  const labels = ["(1-t)^3", "3(1-t)^2 t", "3(1-t) t^2", "t^3"];
  const w = bernstein(t);
  const [top, maxH, colW] = [450, 90, 800 / 4];
  for (let i = 0; i < 4; i++) {
    const x0 = colW * i + 30;
    const width = colW - 60;
    ctx.fillStyle = "#2c3a34";
    ctx.fillRect(x0, top, width, maxH);
    ctx.fillStyle = "#f0b429";
    ctx.fillRect(x0, top + maxH - w[i] * maxH, width, w[i] * maxH);
    ctx.fillStyle = "#93a39b";
    ctx.fillText(labels[i], x0 + width / 2, top + maxH + 18);
    ctx.fillStyle = "#e4ebe6";
    ctx.fillText(`P${i}: ${w[i].toFixed(2)}`, x0 + width / 2, top + maxH + 36);
  }
  ctx.textAlign = "left";
  ctx.font = "18px monospace";
  ctx.fillStyle = "#f0b429";
  ctx.fillText(`t = ${t.toFixed(2)}`, 680, 30);
  ctx.font = "13px sans-serif";
  ctx.fillStyle = "#93a39b";
  ctx.fillText("drag the points   Space: play/pause   C: construction", 16, 24);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
