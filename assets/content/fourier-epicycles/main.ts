// Drawn by circles: Fourier epicycles.
//
// x_N(t) = a0 + sum_{k=1..N} r_k cos(k t - phi_k)      (and the same for y)
//
// A hand-drawn outline (a cat's head) is resampled evenly, and each coordinate is
// expanded into a Fourier series. Every harmonic is a circle spinning k times per turn;
// chained largest first, the top chain's tip gives x and the left chain's tip gives y,
// and where they meet the outline is drawn. Drag the slider to change the number of
// terms N and watch the details melt away. Space plays or pauses.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Pt = [number, number];
type Term = { k: number; r: number; phase: number };
type Series = { mean: number; terms: Term[] };

const TAU = Math.PI * 2;
const SAMPLES = 256; // points taken along the outline
const MAX_TERMS = 80; // highest harmonic computed

// A closed outline, written by hand: a cat's head with two pointed ears (y up).
const OUTLINE: Pt[] = [
  [0, -0.9], [0.35, -0.82], [0.62, -0.62], [0.78, -0.32], [0.82, 0], [0.78, 0.3], [0.7, 0.52],
  [0.82, 1.05], [0.4, 0.72], [0.2, 0.75], [0, 0.76], [-0.2, 0.75], [-0.4, 0.72], [-0.82, 1.05],
  [-0.7, 0.52], [-0.78, 0.3], [-0.82, 0], [-0.78, -0.32], [-0.62, -0.62], [-0.35, -0.82],
];

/** m points evenly spaced by arc length along the closed polyline. */
function resample(poly: Pt[], m: number): Pt[] {
  const p = [...poly, poly[0]];
  const len = [0];
  for (let i = 1; i < p.length; i++) len.push(len[i - 1] + Math.hypot(p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]));
  const out: Pt[] = [];
  let j = 1;
  for (let i = 0; i < m; i++) {
    const s = (len[len.length - 1] * i) / m;
    while (j < len.length - 1 && len[j] < s) j++;
    const u = (s - len[j - 1]) / (len[j] - len[j - 1] || 1);
    out.push([p[j - 1][0] + u * (p[j][0] - p[j - 1][0]), p[j - 1][1] + u * (p[j][1] - p[j - 1][1])]);
  }
  return out;
}

/** Center the bounding box, scale the longer side to 1, and flip y for the screen. */
function normalize(p: Pt[]): Pt[] {
  const xs = p.map((q) => q[0]);
  const ys = p.map((q) => q[1]);
  const [x0, x1, y0, y1] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  const side = Math.max(x1 - x0, y1 - y0);
  return p.map(([x, y]) => [(x - (x0 + x1) / 2) / side, -(y - (y0 + y1) / 2) / side]);
}

/** DFT of evenly spaced samples: a cos kt + b sin kt = r cos(kt - phase). */
function fourier(v: number[], terms: number): Series {
  const m = v.length;
  const harmonics: Term[] = [];
  for (let k = 1; k <= terms; k++) {
    let [a, b] = [0, 0];
    v.forEach((x, j) => {
      a += x * Math.cos((TAU * k * j) / m);
      b += x * Math.sin((TAU * k * j) / m);
    });
    harmonics.push({ k, r: (2 / m) * Math.hypot(a, b), phase: Math.atan2(b, a) });
  }
  return { mean: v.reduce((s, x) => s + x, 0) / m, terms: harmonics };
}

/** Partial sum with the first n harmonics at angle t. */
const evaluate = (s: Series, n: number, t: number): number =>
  s.terms.slice(0, n).reduce((sum, q) => sum + q.r * Math.cos(q.k * t - q.phase), s.mean);

const points = normalize(resample(OUTLINE, SAMPLES));
const seriesX = fourier(points.map((q) => q[0]), MAX_TERMS);
const seriesY = fourier(points.map((q) => q[1]), MAX_TERMS);

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const scale = 240;
const center = { x: 520, y: 340 };
const toScreen = (x: number, y: number): Pt => [center.x + x * scale, center.y + y * scale];
const slider = { x0: 150, x1: 650, y: 572 };

let target = 40;
let n = 1;
let growing = 0; // seconds since the grow animation started, or -1 when done
let t = 0;
let playing = true;
let recon: Pt[] = [];

function setN(value: number): void {
  n = Math.max(1, Math.min(MAX_TERMS, Math.round(value)));
  recon = Array.from({ length: SAMPLES + 1 }, (_, j) =>
    toScreen(evaluate(seriesX, n, (TAU * j) / SAMPLES), evaluate(seriesY, n, (TAU * j) / SAMPLES)));
}
setN(1);

let sliding = false;
const slideTo = (x: number): void => {
  const u = (Math.max(slider.x0, Math.min(slider.x1, x)) - slider.x0) / (slider.x1 - slider.x0);
  target = 1 + u * (MAX_TERMS - 1);
  growing = -1;
  setN(target);
};
canvas.addEventListener("pointerdown", (e) => {
  if (Math.abs(e.offsetY - slider.y) < 16) {
    sliding = true;
    slideTo(e.offsetX);
  }
});
canvas.addEventListener("pointermove", (e) => {
  if (sliding) slideTo(e.offsetX);
});
window.addEventListener("pointerup", () => (sliding = false));
window.addEventListener("keydown", (e) => {
  if (e.key === " ") playing = !playing;
});

function stroke(pts: Pt[], color: string, width: number, dash: number[] = []): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.setLineDash(dash);
  ctx.beginPath();
  pts.forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
  ctx.stroke();
  ctx.setLineDash([]);
}

/** Draw one chain of circles, largest first. With swap, cosine drives the vertical axis. */
function chain(s: Series, t: number, origin: Pt, swap: boolean, color: string): Pt {
  let [x, y] = origin;
  ctx.strokeStyle = color;
  ctx.lineWidth = 1;
  for (const q of s.terms.slice(0, n).sort((a, b) => b.r - a.r)) {
    const r = q.r * scale;
    const ang = q.k * t - q.phase;
    const [dx, dy] = swap ? [r * Math.sin(ang), r * Math.cos(ang)] : [r * Math.cos(ang), r * Math.sin(ang)];
    if (r > 0.6) {
      ctx.globalAlpha = 0.3;
      ctx.beginPath();
      ctx.arc(x, y, r, 0, TAU);
      ctx.stroke();
      ctx.globalAlpha = 0.85;
      stroke([[x, y], [x + dx, y + dy]], color, 1);
      ctx.globalAlpha = 1;
    }
    [x, y] = [x + dx, y + dy];
  }
  return [x, y];
}

let last = 0;
function frame(ms: number): void {
  const dt = Math.min((ms - last) / 1000, 0.1);
  last = ms;
  if (growing >= 0) {
    // Ease from 1 term up to the target over 1.8 s.
    growing += dt;
    const u = Math.min(growing / 1.8, 1);
    const value = 1 + (1 - (1 - u) ** 3) * (target - 1);
    if (Math.round(value) !== n) setN(value);
    if (u >= 1) growing = -1;
  }
  if (playing) t = (t + 0.5 * dt) % TAU;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  stroke(points.map(([x, y]) => toScreen(x, y)).concat([toScreen(...points[0])]), "#55635d", 1, [3, 4]);
  ctx.fillStyle = "rgba(240, 180, 41, 0.12)";
  ctx.beginPath();
  recon.forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
  ctx.fill();

  const xTip = chain(seriesX, t, toScreen(seriesX.mean, -0.8), false, "#ef6b5b");
  const yTip = chain(seriesY, t, toScreen(-0.85, seriesY.mean), true, "#5bc88a");
  const pen: Pt = [xTip[0], yTip[1]];
  stroke([xTip, pen], "#ef6b5b", 1, [5, 5]);
  stroke([yTip, pen], "#5bc88a", 1, [5, 5]);
  stroke(recon.slice(0, Math.floor((t / TAU) * SAMPLES) + 1).concat([pen]), "#f0b429", 2.2);
  ctx.fillStyle = "#e4ebe6";
  ctx.beginPath();
  ctx.arc(pen[0], pen[1], 3.5, 0, TAU);
  ctx.fill();

  // The terms slider.
  stroke([[slider.x0, slider.y], [slider.x1, slider.y]], "#2c3a34", 6);
  const knob = slider.x0 + ((n - 1) / (MAX_TERMS - 1)) * (slider.x1 - slider.x0);
  stroke([[slider.x0, slider.y], [knob, slider.y]], "#f0b429", 6);
  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px sans-serif";
  ctx.fillText("Terms N", 60, slider.y + 5);
  ctx.fillText("drag the slider   Space: play/pause", 16, 26);
  ctx.font = "32px sans-serif";
  ctx.fillStyle = "#f0b429";
  ctx.fillText(`${n}`, 690, slider.y + 11);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
