// Cubic ease-out: f(t) = 1 - (1 - t)^3, with speed f'(t) = 3 (1 - t)^2.
//
// Top left: the curve against the linear diagonal, plus its speed (scaled by 1/3).
// Top right: the same progress t drives a linear and an eased ball; the small dots mark
// equal slices of time, evenly spaced for linear and bunched up at the end for ease-out.
// Bottom: bars grow toward new heights with the eased progress, in a loop.
// Click to replay; keys: E eases the bars or not, L loops, Up/Down change the duration.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

/** 1 - (1 - t)^3 for progress t, clamped to 0..1. */
function easeOutCubic(t: number): number {
  const u = 1 - Math.min(Math.max(t, 0), 1);
  return 1 - u * u * u;
}

/** The derivative 3 (1 - t)^2: three times linear speed at the start, zero at the end. */
function easeOutCubicSpeed(t: number): number {
  const u = 1 - Math.min(Math.max(t, 0), 1);
  return 3 * u * u;
}

const lerp = (a: number, b: number, t: number): number => a + (b - a) * t;
const HOLD = 0.8; // seconds the finished state is held before looping
const BARS = 7;
const GRAPH = { x: 50, y: 30, size: 260 };
const TRACKS = { x0: 370, x1: 760, y: 30, h: 260 };
const CHART = { x0: 30, x1: 770, top: 350, bottom: 555 };
const AMBER = "#f0b45a";
const MUTED = "#93a39b";

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

let duration = 1.6;
let elapsed = 0;
let looping = true;
let eased = true;
let from: number[] = new Array(BARS).fill(0);
let to: number[] = Array.from({ length: BARS }, () => 0.15 + Math.random() * 0.85);

const progress = (): number => Math.min(elapsed / duration, 1);
const barT = (): number => (eased ? easeOutCubic(progress()) : progress());

/** Animate the bars from where they are now toward fresh heights. */
function newTargets(): void {
  const t = barT();
  from = from.map((a, k) => lerp(a, to[k], t));
  to = to.map(() => 0.15 + Math.random() * 0.85);
  elapsed = 0;
}

canvas.addEventListener("pointerdown", () => newTargets());
window.addEventListener("keydown", (e) => {
  if (e.key === "e") eased = !eased;
  if (e.key === "l") looping = !looping;
  if (e.key === "ArrowUp") duration = Math.min(4, duration + 0.1);
  if (e.key === "ArrowDown") duration = Math.max(0.3, duration - 0.1);
});

function dot(x: number, y: number, r: number, color: string): void {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(x, y, r, 0, Math.PI * 2);
  ctx.fill();
}

function plot(f: (u: number) => number, color: string, width: number, dash: number[] = []): void {
  const at = (u: number): [number, number] => [GRAPH.x + u * GRAPH.size, GRAPH.y + (1 - f(u)) * GRAPH.size];
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.setLineDash(dash);
  ctx.beginPath();
  for (let k = 0; k <= 64; k++) ctx.lineTo(...at(k / 64));
  ctx.stroke();
  ctx.setLineDash([]);
}

let last = 0;
function frame(ms: number): void {
  elapsed += Math.min((ms - last) / 1000, 0.1);
  last = ms;
  if (looping && elapsed > duration + HOLD) newTargets();
  const t = progress();
  const e = easeOutCubic(t);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.font = "13px sans-serif";

  // 1. The curve itself, with linear for reference and the speed f'(t) / 3.
  ctx.strokeStyle = "#33433c";
  ctx.lineWidth = 1;
  ctx.strokeRect(GRAPH.x, GRAPH.y, GRAPH.size, GRAPH.size);
  plot((u) => u, MUTED, 1, [4, 4]);
  plot((u) => easeOutCubicSpeed(u) / 3, "rgba(229, 112, 126, 0.7)", 1.2);
  plot(easeOutCubic, AMBER, 2.5);
  const gx = GRAPH.x + t * GRAPH.size;
  const gy = (v: number): number => GRAPH.y + GRAPH.size - v * GRAPH.size;
  ctx.strokeStyle = "rgba(228, 235, 230, 0.4)";
  ctx.beginPath();
  ctx.moveTo(gx, gy(0));
  ctx.lineTo(gx, gy(e));
  ctx.stroke();
  dot(gx, gy(t), 4, MUTED);
  dot(gx, gy(e), 6, AMBER);
  ctx.fillStyle = MUTED;
  ctx.fillText("t = 0", GRAPH.x, GRAPH.y + GRAPH.size + 18);
  ctx.fillText("1", GRAPH.x + GRAPH.size - 6, GRAPH.y + GRAPH.size + 18);
  ctx.fillStyle = "#e5707e";
  ctx.fillText("speed f'(t) / 3", GRAPH.x + 0.55 * GRAPH.size, gy(0.36));

  // 2. The same progress driving position: linear vs eased.
  const lanes: [string, (u: number) => number, string][] = [
    ["linear", (u) => u, MUTED],
    ["ease-out", easeOutCubic, AMBER],
  ];
  lanes.forEach(([label, f, color], i) => {
    const y = TRACKS.y + (TRACKS.h / 2) * (i + 0.55);
    ctx.fillStyle = color;
    ctx.fillText(label, TRACKS.x0, y - 22);
    ctx.fillStyle = "#33433c";
    ctx.fillRect(TRACKS.x0, y - 0.5, TRACKS.x1 - TRACKS.x0, 1);
    // Where the ball is after each tenth of the duration.
    for (let k = 0; k <= 10; k++) dot(lerp(TRACKS.x0, TRACKS.x1, f(k / 10)), y, 2.5, color);
    dot(lerp(TRACKS.x0, TRACKS.x1, f(t)), y, 9, color);
  });

  // 3. A bar chart animating to new values; the thin lines mark the targets.
  const bt = barT();
  const slot = (CHART.x1 - CHART.x0) / BARS;
  const height = CHART.bottom - CHART.top;
  for (let k = 0; k < BARS; k++) {
    const v = lerp(from[k], to[k], bt);
    const x = CHART.x0 + slot * k;
    ctx.fillStyle = eased ? AMBER : MUTED;
    ctx.fillRect(x + slot * 0.18, CHART.bottom - v * height, slot * 0.64, v * height);
    ctx.fillStyle = "rgba(228, 235, 230, 0.6)";
    ctx.fillRect(x + slot * 0.18, CHART.bottom - to[k] * height - 1, slot * 0.64, 2);
  }
  ctx.fillStyle = MUTED;
  ctx.fillRect(CHART.x0, CHART.bottom, CHART.x1 - CHART.x0, 1);

  ctx.font = "14px monospace";
  ctx.fillStyle = "#e4ebe6";
  ctx.fillText(`t = ${t.toFixed(2)}   f(t) = ${e.toFixed(2)}   duration ${duration.toFixed(1)} s`, 30, 335);
  ctx.fillStyle = MUTED;
  const help = `E: ${eased ? "eased" : "linear"} bars   L: loop ${looping ? "on" : "off"}   Up/Down: duration`;
  ctx.fillText(`click: replay   ${help}`, 30, 585);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
