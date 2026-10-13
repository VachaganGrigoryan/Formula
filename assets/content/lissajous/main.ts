// Lissajous figures: x(t) = A sin(a t + delta pi),  y(t) = B sin(b t).
//
// One sine wave drives the horizontal position and another the vertical one. The dots
// on the top and left tracks show each wave alone; the point they meet at traces the
// figure. delta drifts slowly forward, so the knot morphs and seems to turn in 3D.
// Left/Right change a, Up/Down change b, drag sideways to scrub delta, D toggles drift,
// Space plays or pauses.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };

const TAU = Math.PI * 2;

/** The point at parameter t; delta is the phase of x in units of pi. */
function lissajous(t: number, p: { ampX: number; ampY: number; a: number; b: number; delta: number }): Vec2 {
  return { x: p.ampX * Math.sin(p.a * t + p.delta * Math.PI), y: p.ampY * Math.sin(p.b * t) };
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const params = { ampX: 1, ampY: 1, a: 3, b: 2, delta: 0.5 };
let t = 0;
let playing = true;
let drift = true;
const driftRate = 0.05; // delta change per second, in units of pi
let dragX: number | null = null;

window.addEventListener("keydown", (e) => {
  if (e.key === "ArrowLeft") params.a = Math.max(1, params.a - 1);
  if (e.key === "ArrowRight") params.a = Math.min(9, params.a + 1);
  if (e.key === "ArrowDown") params.b = Math.max(1, params.b - 1);
  if (e.key === "ArrowUp") params.b = Math.min(9, params.b + 1);
  if (e.key === "d" || e.key === "D") drift = !drift;
  if (e.key === " ") playing = !playing;
});
canvas.addEventListener("pointerdown", (e) => (dragX = e.offsetX));
canvas.addEventListener("pointermove", (e) => {
  if (dragX === null) return;
  params.delta = (((params.delta + (e.offsetX - dragX) / 300) % 2) + 2) % 2;
  dragX = e.offsetX;
});
window.addEventListener("pointerup", () => (dragX = null));

// Square plot area with gutters on the top and left for the two oscillators.
const gutter = 40;
const plot = { x: 230, y: 70, side: 440 };
const toScreen = (p: Vec2): Vec2 => ({
  x: plot.x + plot.side / 2 + p.x * plot.side * 0.45,
  y: plot.y + plot.side / 2 - p.y * plot.side * 0.45,
});

function line(points: Vec2[], color: string, width: number, dash: number[] = []): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.setLineDash(dash);
  ctx.beginPath();
  points.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
  ctx.stroke();
  ctx.setLineDash([]);
}

function dot(p: Vec2, r: number, color: string): void {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(p.x, p.y, r, 0, TAU);
  ctx.fill();
}

let last = 0;
function frame(ms: number): void {
  const dt = Math.min((ms - last) / 1000, 0.1);
  last = ms;
  if (playing) t = (t + 1.0 * dt) % TAU;
  if (drift && dragX === null) params.delta = (params.delta + driftRate * dt) % 2;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.strokeStyle = "#2c3a34";
  ctx.lineWidth = 1;
  ctx.strokeRect(plot.x, plot.y, plot.side, plot.side);

  // The whole closed figure faintly, and the path traced so far brightly.
  const samples = 720;
  const full = Array.from({ length: samples + 1 }, (_, j) => toScreen(lissajous((TAU * j) / samples, params)));
  line(full, "#55635d", 1, [3, 4]);
  line(full.slice(0, Math.floor((t / TAU) * samples) + 1).concat([toScreen(lissajous(t, params))]), "#f0b429", 2.2);

  // Each oscillator on its own track, and how they combine into the moving point.
  const tip = toScreen(lissajous(t, params));
  const xMarker = { x: tip.x, y: plot.y - gutter / 2 };
  const yMarker = { x: plot.x - gutter / 2, y: tip.y };
  line([{ x: plot.x, y: xMarker.y }, { x: plot.x + plot.side, y: xMarker.y }], "rgba(239, 107, 91, 0.3)", 1);
  line([{ x: yMarker.x, y: plot.y }, { x: yMarker.x, y: plot.y + plot.side }], "rgba(91, 200, 138, 0.3)", 1);
  line([xMarker, tip], "#ef6b5b", 1, [5, 5]);
  line([yMarker, tip], "#5bc88a", 1, [5, 5]);
  dot(xMarker, 5, "#ef6b5b");
  dot(yMarker, 5, "#5bc88a");
  dot(tip, 4, "#e4ebe6");

  ctx.font = "34px sans-serif";
  ctx.fillStyle = "#f0b429";
  ctx.textAlign = "right";
  ctx.fillText(`${params.a}:${params.b}`, 780, 580);
  ctx.textAlign = "left";
  ctx.font = "15px monospace";
  ctx.fillStyle = "#e4ebe6";
  ctx.fillText(`x(t) = A sin(${params.a}t + ${params.delta.toFixed(2)} pi)`, 16, 548);
  ctx.fillText(`y(t) = B sin(${params.b}t)`, 16, 570);
  ctx.font = "13px sans-serif";
  ctx.fillStyle = "#93a39b";
  ctx.fillText(`Left/Right: a   Up/Down: b   drag: delta   D: drift ${drift ? "on" : "off"}   Space: pause`, 16, 22);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
