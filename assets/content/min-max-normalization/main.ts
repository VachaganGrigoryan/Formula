// Min-max normalization: x_norm = (x - min) / (max - min), height = x_norm * H.
//
// Twelve monthly values sit on a raw axis (0..500), get mapped onto a 0..1 axis, and
// become bar heights. Drag the gold August dot along the raw axis: as it becomes the
// maximum, every other bar is squashed. Hover a bar to see its calculation.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

/** Map values onto 0..1. A flat dataset (max == min) maps to all zeros. */
function minMax(values: number[]): number[] {
  const [lo, hi] = bounds(values);
  const span = hi - lo;
  return values.map((v) => (span > 0 ? (v - lo) / span : 0));
}

function bounds(values: number[]): [number, number] {
  return [Math.min(...values), Math.max(...values)];
}

const lerp = (a: number, b: number, t: number): number => a + t * (b - a);
const clamp = (v: number, lo: number, hi: number): number => Math.min(hi, Math.max(lo, v));

const MONTHS = "JFMAMJJASOND".split("");
const OUTLIER = 7; // August
const RAW_MAX = 500;
const values = MONTHS.map((_, i) => 55 + 18 * Math.sin(0.55 * i) + 10 * Math.sin(2.7 * i + 1));
values[OUTLIER] = 260; // start with August far above the rest
const COLORS = { low: "#2a9d8f", high: "#e76f51", mid: "#7ea7e0", outlier: "#e9c46a", ink: "#e4ebe6", muted: "#8a9a92" };

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 560;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

// Layout: raw axis, normalized axis, then the bar chart.
const LEFT = 60, RIGHT = 740;
const RAW_Y = 100, UNIT_Y = 200;
const CHART_TOP = 260, CHART_BOTTOM = 500;
const H = CHART_BOTTOM - CHART_TOP;
const rawX = (v: number): number => lerp(LEFT, RIGHT, v / RAW_MAX);
const unitX = (u: number): number => lerp(LEFT, RIGHT, u);
const slotW = (RIGHT - LEFT) / 12;

let dragging = false;
let hovered: number | null = null;

canvas.addEventListener("pointerdown", (e) => {
  if (Math.abs(e.offsetY - RAW_Y) < 20) dragging = true;
  if (dragging) values[OUTLIER] = clamp(((e.offsetX - LEFT) / (RIGHT - LEFT)) * RAW_MAX, 0, RAW_MAX);
});
canvas.addEventListener("pointermove", (e) => {
  if (dragging) values[OUTLIER] = clamp(((e.offsetX - LEFT) / (RIGHT - LEFT)) * RAW_MAX, 0, RAW_MAX);
  const i = Math.floor((e.offsetX - LEFT) / slotW);
  hovered = e.offsetY > CHART_TOP - 20 && e.offsetY < CHART_BOTTOM + 25 && i >= 0 && i < 12 ? i : null;
});
window.addEventListener("pointerup", () => (dragging = false));

function dot(x: number, y: number, r: number, color: string): void {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(x, y, r, 0, Math.PI * 2);
  ctx.fill();
}

function line(x0: number, y0: number, x1: number, y1: number, color: string, width = 1): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(x1, y1);
  ctx.stroke();
}

function frame(): void {
  const norm = minMax(values);
  const [lo, hi] = bounds(values);
  const colorOf = (i: number): string =>
    values[i] === lo ? COLORS.low : values[i] === hi ? COLORS.high : COLORS.mid;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.font = "13px sans-serif";
  ctx.textAlign = "left";

  // Raw axis with ticks every 100.
  ctx.fillStyle = COLORS.muted;
  ctx.fillText("raw values (drag the gold August dot)", LEFT, RAW_Y - 40);
  line(LEFT, RAW_Y, RIGHT, RAW_Y, "#4a5852");
  ctx.textAlign = "center";
  for (let tick = 0; tick <= RAW_MAX; tick += 100) {
    line(rawX(tick), RAW_Y - 4, rawX(tick), RAW_Y + 4, "#4a5852");
    ctx.fillText(String(tick), rawX(tick), RAW_Y + 20);
  }
  line(rawX(lo), RAW_Y, rawX(hi), RAW_Y, "rgba(126,167,224,0.4)", 4);

  // Normalized axis: min lands on 0, max on 1.
  line(LEFT, UNIT_Y, RIGHT, UNIT_Y, "rgba(126,167,224,0.4)", 4);
  ctx.fillStyle = COLORS.low;
  ctx.fillText("0", unitX(0), UNIT_Y + 22);
  ctx.fillStyle = COLORS.high;
  ctx.fillText("1", unitX(1), UNIT_Y + 22);
  ctx.fillStyle = COLORS.muted;
  ctx.fillText("normalized", unitX(0.5), UNIT_Y + 22);

  // Each value: a link from its raw position to its normalized position.
  for (let i = 0; i < 12; i++) {
    const active = hovered === i || (dragging && i === OUTLIER);
    line(rawX(values[i]), RAW_Y, unitX(norm[i]), UNIT_Y, active ? COLORS.ink : "rgba(160,180,200,0.3)");
    dot(rawX(values[i]), RAW_Y, i === OUTLIER ? 7 : active ? 6 : 4.5, i === OUTLIER ? COLORS.outlier : colorOf(i));
    dot(unitX(norm[i]), UNIT_Y, active ? 6 : 4.5, colorOf(i));
  }
  ctx.fillStyle = COLORS.low;
  ctx.fillText(`min ${lo.toFixed(0)}`, rawX(lo), RAW_Y - 12);
  ctx.fillStyle = COLORS.high;
  ctx.fillText(`max ${hi.toFixed(0)}`, rawX(hi), RAW_Y - 12);

  // Bars: height = normalized value * available pixels.
  line(LEFT, CHART_BOTTOM, RIGHT, CHART_BOTTOM, "#4a5852");
  ctx.setLineDash([4, 4]);
  line(LEFT, CHART_TOP, RIGHT, CHART_TOP, "#4a5852");
  ctx.setLineDash([]);
  for (let i = 0; i < 12; i++) {
    const h = Math.max(norm[i] * H, 2);
    ctx.fillStyle = hovered === i ? COLORS.ink : i === OUTLIER ? COLORS.outlier : COLORS.mid;
    ctx.fillRect(LEFT + slotW * (i + 0.15), CHART_BOTTOM - h, slotW * 0.7, h);
    ctx.fillStyle = COLORS.muted;
    ctx.fillText(MONTHS[i], LEFT + slotW * (i + 0.5), CHART_BOTTOM + 18);
  }

  ctx.textAlign = "left";
  ctx.font = "14px monospace";
  ctx.fillStyle = COLORS.ink;
  const note = hovered === null
    ? "hover a bar"
    : `(${values[hovered].toFixed(0)} - ${lo.toFixed(0)}) / (${hi.toFixed(0)} - ${lo.toFixed(0)}) = ` +
      `${norm[hovered].toFixed(2)}  ->  ${(norm[hovered] * H).toFixed(0)} px of ${H}`;
  ctx.fillText(note, LEFT, 545);
  ctx.fillText("X_norm = (X - X_min) / (X_max - X_min)", LEFT, 30);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
