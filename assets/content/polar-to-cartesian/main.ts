// Polar to Cartesian: x = r cos(theta), y = r sin(theta).
//
// A point given by a radius r and an angle theta, with the x and y components it
// produces drawn along the axes while theta sweeps around. Press M to switch to the
// rose curve r = cos(k theta), drawn one angle at a time (Up/Down change k).
// Click or drag to place the point (this uses the inverse, atan2 and hypot);
// Space plays or pauses.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };

const TAU = Math.PI * 2;
/** (r, theta) -> (x, y), theta in radians, counterclockwise from the +x axis. */
const toCartesian = (r: number, theta: number): Vec2 => ({ x: r * Math.cos(theta), y: r * Math.sin(theta) });
/** (x, y) -> (r, theta) with theta in [0, 2 pi). */
const toPolar = (x: number, y: number): { r: number; theta: number } => ({
  r: Math.hypot(x, y),
  theta: ((Math.atan2(y, x) % TAU) + TAU) % TAU,
});

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

// Math space (y up) to screen space (y down).
const center: Vec2 = { x: 400, y: 270 };
const scale = 220;
const toScreen = (p: Vec2): Vec2 => ({ x: center.x + p.x * scale, y: center.y - p.y * scale });

let mode: "point" | "rose" = "point";
let r = 0.8;
let theta = 0.75;
let k = 5;
let playing = true;
let dragging = false;

/** The radius actually plotted: free in point mode, cos(k theta) on the rose. */
const radius = (a: number): number => (mode === "point" ? r : Math.cos(k * a));

function placeAt(e: PointerEvent): void {
  const p = toPolar((e.offsetX - center.x) / scale, (center.y - e.offsetY) / scale);
  theta = p.theta;
  if (mode === "point") r = Math.min(p.r, 1);
  playing = false;
}
canvas.addEventListener("pointerdown", (e) => {
  dragging = true;
  placeAt(e);
});
canvas.addEventListener("pointermove", (e) => {
  if (dragging) placeAt(e);
});
window.addEventListener("pointerup", () => (dragging = false));
window.addEventListener("keydown", (e) => {
  if (e.key === " ") playing = !playing;
  if (e.key === "m" || e.key === "M") mode = mode === "point" ? "rose" : "point";
  if (e.key === "ArrowUp") k = Math.min(k + 1, 9);
  if (e.key === "ArrowDown") k = Math.max(k - 1, 1);
});

function path(points: Vec2[], color: string, width: number, dash: number[] = []): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.setLineDash(dash);
  ctx.beginPath();
  points.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
  ctx.stroke();
  ctx.setLineDash([]);
}

/** Sample the polar curve r = radius(a) for a from 0 to `upTo`, in screen space. */
function polarCurve(upTo: number, steps: number): Vec2[] {
  const n = Math.max(1, Math.ceil((upTo / TAU) * steps));
  return Array.from({ length: n + 1 }, (_, j) => {
    const a = Math.min((TAU * j) / steps, upTo);
    return toScreen(toCartesian(radius(a), a));
  });
}

let last = 0;
function frame(ms: number): void {
  const dt = Math.min((ms - last) / 1000, 0.1);
  last = ms;
  if (playing) theta = (theta + 0.45 * dt) % TAU;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Polar grid: rings of equal r, spokes of equal theta.
  const grid = "#2c3a34";
  for (let ring = 1; ring <= 4; ring++) {
    ctx.strokeStyle = grid;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(center.x, center.y, (scale * ring) / 4, 0, TAU);
    ctx.stroke();
  }
  ctx.font = "13px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  for (let spoke = 0; spoke < 12; spoke++) {
    const a = (spoke * Math.PI) / 6;
    path([center, toScreen(toCartesian(1, a))], grid, 1);
    if (spoke % 3 === 0) {
      const label = toScreen(toCartesian(1.1, a));
      ctx.fillStyle = "#7d8c85";
      ctx.fillText(`${spoke * 30} deg`, label.x, label.y);
    }
  }

  if (mode === "rose") {
    path(polarCurve(TAU, 720), "#55635d", 1, [3, 4]);
    path(polarCurve(theta, 720), "#f0b429", 2.2);
  }

  const rad = radius(theta);
  const p = toCartesian(rad, theta);
  const s = toScreen(p);
  const footX = toScreen({ x: p.x, y: 0 });
  const footY = toScreen({ x: 0, y: p.y });

  // The angle, swept from the positive x axis.
  const arc = Array.from({ length: 49 }, (_, j) => toScreen(toCartesian(0.16, (theta * j) / 48)));
  path(arc, "#93a39b", 1.5);

  // The two components the conversion produces, and the radius itself.
  path([center, footX], "#ef6b5b", 4);
  path([center, footY], "#5bc88a", 4);
  path([s, footX], "#ef6b5b", 1, [4, 4]);
  path([s, footY], "#5bc88a", 1, [4, 4]);
  path([center, s], "#f0b429", 2);
  ctx.fillStyle = "#f0b429";
  ctx.beginPath();
  ctx.arc(s.x, s.y, 6.5, 0, TAU);
  ctx.fill();

  ctx.textAlign = "left";
  ctx.textBaseline = "alphabetic";
  ctx.font = "15px monospace";
  const deg = (theta * 180) / Math.PI;
  ctx.fillStyle = "#e4ebe6";
  ctx.fillText(`r = ${rad >= 0 ? "+" : ""}${rad.toFixed(2)}   theta = ${deg.toFixed(0)} deg`, 20, 530);
  ctx.fillStyle = "#ef6b5b";
  ctx.fillText(`x = r cos(theta) = ${p.x.toFixed(2)}`, 20, 552);
  ctx.fillStyle = "#5bc88a";
  ctx.fillText(`y = r sin(theta) = ${p.y.toFixed(2)}`, 20, 574);
  ctx.fillStyle = "#7d8c85";
  ctx.font = "13px sans-serif";
  const hint = mode === "point" ? "M: rose curve" : `M: point   Up/Down: k = ${k}`;
  ctx.fillText(`drag to place   Space: play/pause   ${hint}`, 420, 574);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
