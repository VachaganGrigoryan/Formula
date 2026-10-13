// Linear interpolation: v(t) = v0 + t * (v1 - v0).
//
// A point slides between two draggable endpoints, a number moves along a number line,
// and a color bar shows the blend, all driven by one t that sweeps 0 -> 1 -> 0.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };
type Rgb = [number, number, number];

const lerp = (a: number, b: number, t: number): number => (1 - t) * a + t * b;
const lerpVec = (a: Vec2, b: Vec2, t: number): Vec2 => ({ x: lerp(a.x, b.x, t), y: lerp(a.y, b.y, t) });
const lerpRgb = (a: Rgb, b: Rgb, t: number): Rgb => [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)];
const css = ([r, g, b]: Rgb): string => `rgb(${r | 0}, ${g | 0}, ${b | 0})`;
/** Map any real s onto 0..1..0 at constant speed (a triangle wave). */
const pingPong = (s: number): number => 1 - Math.abs(1 - (((s % 2) + 2) % 2));

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const ends: [Vec2, Vec2] = [{ x: 120, y: 330 }, { x: 680, y: 90 }];
const [v0, v1] = [20, 80];
const [c0, c1]: [Rgb, Rgb] = [[35, 120, 110], [168, 112, 15]];
let dragging: number | null = null;

canvas.addEventListener("pointerdown", (e) => {
  const p = { x: e.offsetX, y: e.offsetY };
  ends.forEach((q, i) => {
    if (Math.hypot(p.x - q.x, p.y - q.y) < 14) dragging = i;
  });
});
canvas.addEventListener("pointermove", (e) => {
  if (dragging !== null) ends[dragging] = { x: e.offsetX, y: Math.min(e.offsetY, 400) };
});
window.addEventListener("pointerup", () => (dragging = null));

function frame(ms: number): void {
  const t = pingPong(ms / 2000);
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // The plane: segment, endpoints, and the interpolated point.
  ctx.strokeStyle = "#556";
  ctx.setLineDash([5, 5]);
  ctx.beginPath();
  ctx.moveTo(ends[0].x, ends[0].y);
  ctx.lineTo(ends[1].x, ends[1].y);
  ctx.stroke();
  ctx.setLineDash([]);
  ends.forEach((q, i) => {
    ctx.fillStyle = css(i === 0 ? c0 : c1);
    ctx.beginPath();
    ctx.arc(q.x, q.y, 9, 0, Math.PI * 2);
    ctx.fill();
  });
  const p = lerpVec(ends[0], ends[1], t);
  ctx.fillStyle = "#9db3ff";
  ctx.beginPath();
  ctx.arc(p.x, p.y, 7, 0, Math.PI * 2);
  ctx.fill();

  // The number line from 0 to 100.
  const x = (v: number): number => 60 + (v / 100) * 680;
  ctx.strokeStyle = "#93a39b";
  ctx.beginPath();
  ctx.moveTo(x(0), 460);
  ctx.lineTo(x(100), 460);
  ctx.stroke();
  const v = lerp(v0, v1, t);
  ctx.fillStyle = "#9db3ff";
  ctx.beginPath();
  ctx.moveTo(x(v), 452);
  ctx.lineTo(x(v) - 7, 440);
  ctx.lineTo(x(v) + 7, 440);
  ctx.fill();

  // The color bar: the gradient, and a marker at t.
  for (let i = 0; i < 680; i++) {
    ctx.fillStyle = css(lerpRgb(c0, c1, i / 679));
    ctx.fillRect(60 + i, 500, 1, 40);
  }
  ctx.strokeStyle = "#fff";
  ctx.lineWidth = 3;
  ctx.strokeRect(60 + t * 680 - 2, 496, 4, 48);
  ctx.lineWidth = 1;

  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  ctx.fillText(`t = ${t.toFixed(2)}   v = ${v.toFixed(1)}   p = (${p.x.toFixed(0)}, ${p.y.toFixed(0)})`, 60, 40);
  ctx.fillText("drag the endpoints", 60, 580);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
