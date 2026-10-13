// 2D rotation: x' = x cos(theta) - y sin(theta),  y' = x sin(theta) + y cos(theta).
//
// Top: a letter F spins about the origin; the colored arrows are the rotated axes,
// (cos, sin) and (-sin, cos), which are the columns of the rotation matrix. Drag P and
// watch its image P' travel the dashed arc. Space pauses the spin.
// Bottom: the same geometry tilts crowded month labels. Drag the slider to set the tilt;
// labels that still collide with a neighbor turn red.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };

/** Rotate p counter-clockwise about the origin (math axes: y points up). */
function rotate(p: Vec2, theta: number): Vec2 {
  const c = Math.cos(theta);
  const s = Math.sin(theta);
  return { x: p.x * c - p.y * s, y: p.x * s + p.y * c };
}

/** Two equal labels `spacing` apart and tilted by `angle` overlap when, in their own
 *  rotated frame, the offset (spacing cos a, spacing sin a) fits inside the box. */
const labelsCollide = (spacing: number, width: number, height: number, angle: number): boolean =>
  spacing * Math.cos(angle) < width && spacing * Math.sin(angle) < height;

const F: Vec2[] = [[-0.25, -0.5], [-0.05, -0.5], [-0.05, -0.05], [0.2, -0.05], [0.2, 0.12],
  [-0.05, 0.12], [-0.05, 0.32], [0.3, 0.32], [0.3, 0.5], [-0.25, 0.5]].map(([x, y]) => ({ x, y }));
const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August",
  "September", "October", "November", "December"];
const SALES = [0.42, 0.55, 0.61, 0.48, 0.72, 0.9, 0.84, 0.77, 0.63, 0.58, 0.69, 0.95];
const CENTER = { x: 500, y: 200 };
const R = 140; // pixels per unit
const SLIDER = { x0: 220, x1: 760, y: 580 };
const BLUE = "#5ab4e6";
const RED = "#e5707e";
const AMBER = "#f0b45a";

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const toScreen = (p: Vec2): Vec2 => ({ x: CENTER.x + p.x * R, y: CENTER.y - p.y * R });
let theta = Math.PI / 6;
let spinning = true;
let point: Vec2 = { x: 0.78, y: 0.32 };
let tilt: number | null = null; // null: animate the tilt until the slider is used
let dragging: "point" | "tilt" | null = null;

function pointerAt(e: PointerEvent): void {
  const m = { x: e.offsetX, y: e.offsetY };
  if (dragging === "point") {
    const p = { x: (m.x - CENTER.x) / R, y: (CENTER.y - m.y) / R };
    const len = Math.hypot(p.x, p.y);
    point = len > 1.1 ? { x: (p.x * 1.1) / len, y: (p.y * 1.1) / len } : p; // stay inside radius 1.1
  } else if (dragging === "tilt") {
    tilt = (Math.min(Math.max((m.x - SLIDER.x0) / (SLIDER.x1 - SLIDER.x0), 0), 1) * Math.PI) / 2;
  }
}
canvas.addEventListener("pointerdown", (e) => {
  const p = toScreen(point);
  if (Math.hypot(e.offsetX - p.x, e.offsetY - p.y) < 16) dragging = "point";
  else if (Math.abs(e.offsetY - SLIDER.y) < 14 && e.offsetX > SLIDER.x0 - 10) dragging = "tilt";
  pointerAt(e);
});
canvas.addEventListener("pointermove", pointerAt);
window.addEventListener("pointerup", () => (dragging = null));
window.addEventListener("keydown", (e) => {
  if (e.key === " ") spinning = !spinning;
});

function path(points: Vec2[], close: boolean): void {
  ctx.beginPath();
  points.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
  if (close) ctx.closePath();
}

function arrow(to: Vec2, color: string, width: number): void {
  const a = toScreen({ x: 0, y: 0 });
  const b = toScreen(to);
  const angle = Math.atan2(b.y - a.y, b.x - a.x);
  ctx.strokeStyle = ctx.fillStyle = color;
  ctx.lineWidth = width;
  path([a, b], false);
  ctx.stroke();
  path([b, { x: b.x - 12 * Math.cos(angle - 0.4), y: b.y - 12 * Math.sin(angle - 0.4) },
    { x: b.x - 12 * Math.cos(angle + 0.4), y: b.y - 12 * Math.sin(angle + 0.4) }], true);
  ctx.fill();
}

function drawPlane(): void {
  ctx.strokeStyle = "#33433c";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.arc(CENTER.x, CENTER.y, R, 0, Math.PI * 2);
  ctx.stroke();
  path([toScreen({ x: -1.15, y: 0 }), toScreen({ x: 1.15, y: 0 })], false);
  ctx.stroke();
  path([toScreen({ x: 0, y: -1.15 }), toScreen({ x: 0, y: 1.15 })], false);
  ctx.stroke();

  // The shape before (dashed) and after rotation (filled).
  ctx.setLineDash([4, 4]);
  ctx.strokeStyle = "#6f7d76";
  path(F.map(toScreen), true);
  ctx.stroke();
  ctx.setLineDash([]);
  path(F.map((p) => toScreen(rotate(p, theta))), true);
  ctx.fillStyle = "rgba(240, 180, 90, 0.25)";
  ctx.fill();
  ctx.strokeStyle = AMBER;
  ctx.lineWidth = 2;
  ctx.stroke();

  // The matrix columns are where (1, 0) and (0, 1) land.
  arrow({ x: 1, y: 0 }, "rgba(90, 180, 230, 0.3)", 1.5);
  arrow({ x: 0, y: 1 }, "rgba(229, 112, 126, 0.3)", 1.5);
  const i = rotate({ x: 1, y: 0 }, theta);
  const j = rotate({ x: 0, y: 1 }, theta);
  arrow(i, BLUE, 3);
  arrow(j, RED, 3);
  // Label each tip just beyond it: wide text needs more room sideways than up.
  const tipLabel = (v: Vec2, text: string, color: string): void => {
    const t = toScreen(v);
    ctx.fillStyle = color;
    ctx.fillText(text, t.x + v.x * 56, t.y - v.y * 18);
  };
  ctx.font = "13px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  tipLabel(i, "(cos \u03b8, sin \u03b8)", BLUE);
  tipLabel(j, "(-sin \u03b8, cos \u03b8)", RED);
  ctx.textAlign = "left";
  ctx.textBaseline = "alphabetic";

  // One point, its image, and the arc between them.
  const q = rotate(point, theta);
  ctx.setLineDash([3, 3]);
  ctx.strokeStyle = "#93a39b";
  ctx.lineWidth = 1.2;
  path(Array.from({ length: 41 }, (_, k) => toScreen(rotate(point, (theta * k) / 40))), false);
  ctx.stroke();
  ctx.setLineDash([]);
  const [ps, qs] = [toScreen(point), toScreen(q)];
  ctx.fillStyle = dragging === "point" ? AMBER : "#16201c";
  ctx.strokeStyle = "#e4ebe6";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.arc(ps.x, ps.y, 8, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = "#e4ebe6";
  ctx.beginPath();
  ctx.arc(qs.x, qs.y, 6, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillText("P", ps.x + 10, ps.y + 18);
  ctx.fillText("P'", qs.x + 10, qs.y - 8);

  const [c, s] = [Math.cos(theta), Math.sin(theta)];
  ctx.font = "14px monospace";
  ctx.fillText(`\u03b8 = ${((theta * 180) / Math.PI).toFixed(0)} deg`, 20, 30);
  ctx.fillStyle = BLUE;
  const fmt = (v: number): string => v.toFixed(2);
  ctx.fillText(`x' = ${fmt(point.x)}*${fmt(c)} - ${fmt(point.y)}*${fmt(s)} = ${fmt(q.x)}`, 20, 52);
  ctx.fillStyle = RED;
  ctx.fillText(`y' = ${fmt(point.x)}*${fmt(s)} + ${fmt(point.y)}*${fmt(c)} = ${fmt(q.y)}`, 20, 74);
}

function drawChart(angle: number): void {
  const [left, right, baseline] = [40, 760, 470];
  const slot = (right - left) / 12;
  ctx.font = "13px sans-serif";
  const widths = MONTHS.map((m) => ctx.measureText(m).width);
  const height = 15;
  let collisions = 0;
  MONTHS.forEach((m, k) => {
    const x = left + slot * (k + 0.5);
    ctx.fillStyle = "rgba(240, 180, 90, 0.75)";
    ctx.fillRect(x - slot * 0.3, baseline - SALES[k] * 60, slot * 0.6, SALES[k] * 60);
    const hitsLeft = k > 0 && labelsCollide(slot, widths[k], height, angle);
    const hitsRight = k < 11 && labelsCollide(slot, widths[k + 1], height, angle);
    if (hitsRight) collisions += 1;
    // Anchor the label's right end under its bar and tilt it upward.
    ctx.save();
    ctx.translate(angle > 0.01 ? x + 4 : x + widths[k] / 2, baseline + 6);
    ctx.rotate(-angle);
    ctx.textAlign = "right";
    ctx.textBaseline = "top";
    ctx.fillStyle = hitsLeft || hitsRight ? RED : "#e4ebe6";
    ctx.fillText(m, 0, 0);
    ctx.restore();
  });
  ctx.strokeStyle = "#93a39b";
  ctx.lineWidth = 1;
  path([{ x: left, y: baseline }, { x: right, y: baseline }], false);
  ctx.stroke();
  ctx.textAlign = "right";
  ctx.font = "13px monospace";
  ctx.fillStyle = collisions ? RED : BLUE;
  ctx.fillText(collisions ? `${collisions} label pairs collide: tilt them` : "no labels collide", right, 395);
  ctx.textAlign = "left";

  // The tilt slider.
  const knob = SLIDER.x0 + (angle / (Math.PI / 2)) * (SLIDER.x1 - SLIDER.x0);
  ctx.fillStyle = "#33433c";
  ctx.fillRect(SLIDER.x0, SLIDER.y - 2, SLIDER.x1 - SLIDER.x0, 4);
  ctx.fillStyle = AMBER;
  ctx.beginPath();
  ctx.arc(knob, SLIDER.y, 8, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = "#93a39b";
  ctx.fillText(`label tilt ${((angle * 180) / Math.PI).toFixed(0)} deg`, 40, SLIDER.y + 5);
}

let last = 0;
function frame(ms: number): void {
  const dt = Math.min((ms - last) / 1000, 0.1);
  last = ms;
  if (spinning) theta = ((theta + 0.5 * dt + 3 * Math.PI) % (2 * Math.PI)) - Math.PI;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawPlane();
  ctx.strokeStyle = "#33433c";
  path([{ x: 20, y: 380 }, { x: 780, y: 380 }], false);
  ctx.stroke();
  drawChart(tilt ?? ((1 - Math.abs(1 - ((ms / 4000) % 2))) * Math.PI) / 2);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
