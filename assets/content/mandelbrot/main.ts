// Mandelbrot set: z_{n+1} = z_n^2 + c, z_0 = 0; c is in the set while |z_n| <= 2.
//
// Every pixel is a value of c, colored by its smooth escape count
// mu = n + 1 - log2(log2 |z_n|); black points never escape. Hover to see the orbit of
// the point under the cursor; click to zoom in 2x, right-click (or shift-click) to zoom out.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Rgb = [number, number, number];

const BAILOUT = 256; // |z|^2 escape test; large so the smooth count is smooth

/** Main cardioid or period-2 disc: these never escape, so skip iterating them. */
function inMainBulbs(x: number, y: number): boolean {
  const q = (x - 0.25) ** 2 + y * y;
  return q * (q + (x - 0.25)) <= 0.25 * y * y || (x + 1) ** 2 + y * y <= 0.0625;
}

/** Smooth escape count of c = cx + i cy, or null if z stays bounded for maxIter steps. */
function escapeTime(cx: number, cy: number, maxIter: number): number | null {
  if (inMainBulbs(cx, cy)) return null;
  let x = 0, y = 0;
  for (let n = 0; n < maxIter; n++) {
    const x2 = x * x, y2 = y * y;
    if (x2 + y2 > BAILOUT) return n + 1 - Math.log2(Math.log2(x2 + y2) / 2);
    y = 2 * x * y + cy;
    x = x2 - y2 + cx;
  }
  return null;
}

/** The first points of the orbit of c, stopping once it clearly escapes. */
function orbit(cx: number, cy: number, steps: number): [number, number][] {
  const out: [number, number][] = [[0, 0]];
  let x = 0, y = 0;
  for (let k = 0; k < steps && x * x + y * y <= 16; k++) {
    [x, y] = [x * x - y * y + cx, 2 * x * y + cy];
    out.push([x, y]);
  }
  return out;
}

// A repeating color ramp, indexed by the square root of the escape count.
const STOPS: Rgb[] = [[13, 18, 31], [41, 115, 158], [242, 204, 115], [217, 84, 64], [13, 18, 31]];
function color(mu: number): Rgb {
  const t = ((Math.sqrt(Math.max(mu - 1, 0)) * 0.06) % 1) * (STOPS.length - 1);
  const k = Math.floor(t), f = t - k;
  const a = STOPS[k], b = STOPS[k + 1];
  return [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f];
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 560;
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;
const image = ctx.createImageData(canvas.width, canvas.height);

const HOME_SPAN = 2.6;
const view = { cx: -0.6, cy: 0, span: HOME_SPAN }; // center and height in the complex plane
let hover: { x: number; y: number } | null = null;
let dirty = true;

const pixel = (): number => view.span / canvas.height;
const toComplex = (px: number, py: number): [number, number] =>
  [view.cx + (px - canvas.width / 2) * pixel(), view.cy - (py - canvas.height / 2) * pixel()];
const toScreen = (x: number, y: number): [number, number] =>
  [canvas.width / 2 + (x - view.cx) / pixel(), canvas.height / 2 - (y - view.cy) / pixel()];
/** Deeper zooms need more iterations to separate the set from its surroundings. */
const maxIter = (): number => Math.round(300 + 60 * Math.max(0, Math.log2(HOME_SPAN / view.span)));

function render(): void {
  const { width: w, height: h } = canvas, iter = maxIter();
  for (let j = 0; j < h; j++) {
    for (let i = 0; i < w; i++) {
      const [cx, cy] = toComplex(i + 0.5, j + 0.5);
      const mu = escapeTime(cx, cy, iter);
      const [r, g, b] = mu === null ? [0, 0, 0] : color(mu);
      const k = 4 * (j * w + i);
      image.data[k] = r;
      image.data[k + 1] = g;
      image.data[k + 2] = b;
      image.data[k + 3] = 255;
    }
  }
}

/** Zoom by `factor` (> 1 zooms in) keeping the point under the cursor fixed. */
function zoomAt(px: number, py: number, factor: number): void {
  const [x, y] = toComplex(px, py);
  view.cx = x + (view.cx - x) / factor;
  view.cy = y + (view.cy - y) / factor;
  view.span /= factor;
  dirty = true;
}

canvas.addEventListener("pointerdown", (e) => {
  zoomAt(e.offsetX, e.offsetY, e.button === 2 || e.shiftKey ? 0.5 : 2);
});
canvas.addEventListener("contextmenu", (e) => e.preventDefault());
canvas.addEventListener("pointermove", (e) => (hover = { x: e.offsetX, y: e.offsetY }));
canvas.addEventListener("pointerleave", () => (hover = null));

function frame(): void {
  if (dirty) {
    render();
    dirty = false;
  }
  ctx.putImageData(image, 0, 0);

  // The orbit of the point under the cursor: inside it settles, outside it flies off.
  ctx.font = "14px monospace";
  if (hover) {
    const [cx, cy] = toComplex(hover.x, hover.y);
    const pts = orbit(cx, cy, 60).map(([x, y]) => toScreen(x, y));
    ctx.strokeStyle = "rgba(255,255,255,0.8)";
    ctx.fillStyle = "#fff";
    ctx.lineWidth = 1.2;
    ctx.beginPath();
    pts.forEach(([x, y], k) => (k === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
    ctx.stroke();
    for (const [x, y] of pts) ctx.fillRect(x - 2, y - 2, 4, 4);
    ctx.strokeStyle = "#ffd166";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(hover.x, hover.y, 6, 0, Math.PI * 2);
    ctx.stroke();
    const mu = escapeTime(cx, cy, maxIter());
    const fate = mu === null ? `bounded for ${maxIter()} iterations` : `escapes after ${Math.max(mu, 1).toFixed(0)} iterations`;
    ctx.fillStyle = "rgba(0,0,0,0.65)";
    ctx.fillRect(10, 10, 330, 48);
    ctx.fillStyle = "#fff";
    ctx.fillText(`c = ${cx.toFixed(6)} ${cy < 0 ? "-" : "+"} ${Math.abs(cy).toFixed(6)}i`, 18, 30);
    ctx.fillText(fate, 18, 49);
  }
  const zoom = `zoom x${(HOME_SPAN / view.span).toFixed(0)}   click: in   right-click: out`;
  ctx.fillStyle = "rgba(0,0,0,0.65)";
  ctx.fillRect(canvas.width - 380, canvas.height - 34, 370, 24);
  ctx.fillStyle = "#ffd166";
  ctx.fillText(zoom, canvas.width - 372, canvas.height - 17);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
