// Lorenz attractor: dx/dt = sigma (y - x), dy/dt = x (rho - z) - y, dz/dt = x y - beta z.
//
// Two paths start 0.00001 apart and are stepped forward with RK4. Their 3D trails are
// projected onto the screen over a faint trace of the whole attractor while the view
// slowly spins; at first they overlap, then they split onto different lobes. The inset
// plots log10 of the distance between them. Drag to orbit the view.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type V3 = [number, number, number];

const SIGMA = 10, RHO = 28, BETA = 8 / 3; // Lorenz's own values from 1963
const H = 0.004; // integration step (model time)
const SPEED = 1; // model time units per second
const TRAIL = 2500; // points kept per path
const PLOT_SPAN = 40; // time covered by the separation plot

/** The right-hand side: how fast x, y and z are changing at state s. */
const derivative = ([x, y, z]: V3): V3 => [SIGMA * (y - x), x * (RHO - z) - y, x * y - BETA * z];
const add = (a: V3, b: V3, k: number): V3 => [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k];

/** One fourth-order Runge-Kutta step of size h. */
function rk4(s: V3, h: number): V3 {
  const k1 = derivative(s);
  const k2 = derivative(add(s, k1, h / 2));
  const k3 = derivative(add(s, k2, h / 2));
  const k4 = derivative(add(s, k3, h));
  return [0, 1, 2].map((i) => s[i] + (h / 6) * (k1[i] + 2 * k2[i] + 2 * k3[i] + k4[i])) as V3;
}

const distance = (a: V3, b: V3): number => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#10161a";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

let yaw = -0.8, pitch = 0.25, spinning = true; // yaw -0.8: the wings face the viewer
/** Orthographic view: spin about the vertical z axis, then tilt toward the viewer. */
function project([x, y, z]: V3): [number, number] {
  const zc = z - 0.85 * RHO, scale = canvas.height / (1.85 * RHO);
  const x1 = x * Math.cos(yaw) - y * Math.sin(yaw), depth = x * Math.sin(yaw) + y * Math.cos(yaw);
  const up = zc * Math.cos(pitch) - depth * Math.sin(pitch);
  return [canvas.width / 2 + x1 * scale, canvas.height / 2 - up * scale];
}

// The attractor itself, traced once in advance as a faint backdrop.
const backdrop: V3[] = [[1, 1, 20]];
for (let k = 0; k < 8000; k++) backdrop.push(rk4(backdrop[backdrop.length - 1], H));
backdrop.splice(0, 1000);

const START: V3 = [-8, 8, 27];
const paths: V3[][] = [[START], [[START[0] + 1e-5, START[1], START[2]]]]; // trails; last = current
const separation: [number, number][] = []; // (time, log10 distance)
let time = 0, carry = 0, last = 0;

let drag: { x: number; y: number } | null = null;
canvas.addEventListener("pointerdown", (e) => (drag = { x: e.offsetX, y: e.offsetY }));
canvas.addEventListener("pointermove", (e) => {
  if (!drag) return;
  yaw += (e.offsetX - drag.x) * 0.01;
  pitch = Math.max(-1.5, Math.min(1.5, pitch + (e.offsetY - drag.y) * 0.01));
  drag = { x: e.offsetX, y: e.offsetY };
  spinning = false;
});
window.addEventListener("pointerup", () => (drag = null));

function polyline(points: V3[], from: number, to: number): void {
  ctx.beginPath();
  for (let k = from; k <= to; k++) {
    const [px, py] = project(points[k]);
    if (k === from) ctx.moveTo(px, py);
    else ctx.lineTo(px, py);
  }
  ctx.stroke();
}

function frame(ms: number): void {
  const dt = Math.min(ms - last, 100) / 1000;
  last = ms;
  if (spinning) yaw += 0.12 * dt;
  for (carry += (SPEED * dt) / H; carry >= 1; carry--) {
    for (const trail of paths) {
      trail.push(rk4(trail[trail.length - 1], H));
      if (trail.length > TRAIL) trail.shift();
    }
    time += H;
    const d = distance(paths[0][paths[0].length - 1], paths[1][paths[1].length - 1]);
    if (time <= PLOT_SPAN && (separation.length === 0 || time - separation[separation.length - 1][0] >= 0.05))
      separation.push([time, Math.log10(Math.max(d, 1e-12))]);
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.strokeStyle = "rgba(160,170,175,0.12)";
  ctx.lineWidth = 0.7;
  polyline(backdrop, 0, backdrop.length - 1);

  // Each trail in 40 chunks that fade toward the tail.
  const colors = ["77, 204, 230", "255, 140, 77"];
  paths.forEach((trail, p) => {
    const chunks = 40, per = Math.ceil((trail.length - 1) / chunks);
    ctx.lineWidth = 1.4;
    for (let k = 0; k < trail.length - 1; k += per) {
      const end = Math.min(k + per, trail.length - 1);
      ctx.strokeStyle = `rgba(${colors[p]}, ${0.1 + 0.9 * (end / (trail.length - 1))})`;
      polyline(trail, k, end);
    }
    const [hx, hy] = project(trail[trail.length - 1]);
    ctx.fillStyle = `rgb(${colors[p]})`;
    ctx.beginPath();
    ctx.arc(hx, hy, 4, 0, Math.PI * 2);
    ctx.fill();
  });

  // Readout for the first path, and the separation plot (log10 distance from 1e-6 to 1e2).
  const [x, y, z] = paths[0][paths[0].length - 1];
  ctx.fillStyle = "#c8d6d0";
  ctx.font = "13px monospace";
  [`x = ${x.toFixed(2)}`, `y = ${y.toFixed(2)}`, `z = ${z.toFixed(2)}`, `t = ${time.toFixed(2)}`, "drag to orbit"]
    .forEach((text, k) => ctx.fillText(text, 14, 22 + 17 * k));
  const box = { x: 14, y: canvas.height - 100, w: 240, h: 70 };
  const at = (t: number, l: number): [number, number] =>
    [box.x + (t / PLOT_SPAN) * box.w, box.y + box.h - ((Math.max(-6, Math.min(2, l)) + 6) / 8) * box.h];
  ctx.strokeStyle = "rgba(200,214,208,0.3)";
  ctx.lineWidth = 1;
  ctx.strokeRect(box.x, box.y, box.w, box.h);
  ctx.strokeStyle = "#ff8c4d";
  ctx.beginPath();
  separation.forEach(([t, l], k) => (k === 0 ? ctx.moveTo(...at(t, l)) : ctx.lineTo(...at(t, l))));
  ctx.stroke();
  ctx.fillText("distance between the paths (log scale)", box.x, box.y + box.h + 18);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
