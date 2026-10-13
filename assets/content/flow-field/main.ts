// Perlin noise flow field: theta(x, y) = N(x * s, y * s, t) * 2 pi * k,
// p' = p + v * dt * (cos theta, sin theta).
//
// Improved Perlin noise (Ken Perlin, 2002) turns every point into an angle; particles
// follow the angle under them and paint trails, colored by heading. Each frame a
// translucent rectangle fades the old trails. Hover to read the field; click for new noise.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

const COUNT = 1500; // particles
const ZOOM = 0.005, TURNS = 1.5; // s and k in the formula
const SPEED = 70; // pixels per second
const FADE = 0.04; // alpha of the background painted over the trails each frame
const EVOLVE = 0.06; // noise time per second
const BG = "10, 16, 14";

/** 6t^5 - 15t^4 + 10t^3: blends each cell smoothly, so no seams show. */
const fade = (t: number): number => t * t * t * (t * (t * 6 - 15) + 10);
const lerp = (t: number, a: number, b: number): number => a + t * (b - a);

/** Dot product with one of 12 gradients pointing at the edges of a cube. */
function grad(hash: number, x: number, y: number, z: number): number {
  const h = hash & 15;
  const u = h < 8 ? x : y;
  const v = h < 4 ? y : h === 12 || h === 14 ? x : z;
  return (h & 1 ? -u : u) + (h & 2 ? -v : v);
}

/** A shuffled 0..255, repeated to 512 entries so p[i + 1] never overflows. */
function permutation(): Uint8Array {
  const p = Array.from({ length: 256 }, (_, i) => i);
  for (let i = 255; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [p[i], p[j]] = [p[j], p[i]];
  }
  return Uint8Array.from([...p, ...p]);
}

/** Improved Perlin noise in roughly -1..1; exactly 0 on integer lattice points. */
function noise(p: Uint8Array, x: number, y: number, z: number): number {
  const [xf, yf, zf] = [Math.floor(x), Math.floor(y), Math.floor(z)];
  const [X, Y, Z] = [xf & 255, yf & 255, zf & 255];
  [x, y, z] = [x - xf, y - yf, z - zf];
  const [u, v, w] = [fade(x), fade(y), fade(z)];
  const a = p[X] + Y, aa = p[a] + Z, ab = p[a + 1] + Z; // hash the 8 cell corners
  const b = p[X + 1] + Y, ba = p[b] + Z, bb = p[b + 1] + Z;
  return lerp(w,
    lerp(v, lerp(u, grad(p[aa], x, y, z), grad(p[ba], x - 1, y, z)),
      lerp(u, grad(p[ab], x, y - 1, z), grad(p[bb], x - 1, y - 1, z))),
    lerp(v, lerp(u, grad(p[aa + 1], x, y, z - 1), grad(p[ba + 1], x - 1, y, z - 1)),
      lerp(u, grad(p[ab + 1], x, y - 1, z - 1), grad(p[bb + 1], x - 1, y - 1, z - 1))));
}

/** N rescaled to 0..1, and theta = N * 2 pi * k. */
function field(x: number, y: number): [number, number] {
  const n = Math.min(1, Math.max(0, noise(perm, x * ZOOM, y * ZOOM, time) * 0.5 + 0.5));
  return [n, n * 2 * Math.PI * TURNS];
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 500;
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;
// Trails accumulate on an offscreen canvas; the visible one adds the hover readout on top.
const trails = document.createElement("canvas");
trails.width = canvas.width;
trails.height = canvas.height;
const tctx = trails.getContext("2d")!;

type Particle = { x: number; y: number; life: number };
const spawn = (): Particle =>
  ({ x: Math.random() * canvas.width, y: Math.random() * canvas.height, life: 2 + Math.random() * 5 });
const particles: Particle[] = Array.from({ length: COUNT }, spawn);
let perm = permutation(), time = 0, last = 0;
let hover: { x: number; y: number } | null = null;

function clearTrails(): void {
  tctx.fillStyle = `rgb(${BG})`;
  tctx.fillRect(0, 0, trails.width, trails.height);
}
clearTrails();
canvas.addEventListener("pointermove", (e) => (hover = { x: e.offsetX, y: e.offsetY }));
canvas.addEventListener("pointerleave", () => (hover = null));
canvas.addEventListener("pointerdown", () => { perm = permutation(); clearTrails(); });

function arrow(x: number, y: number, theta: number, len: number): void {
  const dx = Math.cos(theta) * len / 2, dy = Math.sin(theta) * len / 2;
  ctx.beginPath();
  ctx.moveTo(x - dx, y - dy);
  ctx.lineTo(x + dx, y + dy);
  ctx.lineTo(x + dx - 9 * Math.cos(theta - 0.45), y + dy - 9 * Math.sin(theta - 0.45));
  ctx.moveTo(x + dx, y + dy);
  ctx.lineTo(x + dx - 9 * Math.cos(theta + 0.45), y + dy - 9 * Math.sin(theta + 0.45));
  ctx.stroke();
}

function frame(ms: number): void {
  const dt = Math.min(ms - last, 50) / 1000;
  last = ms;

  // Fade: paint the background over everything at low opacity.
  tctx.fillStyle = `rgba(${BG}, ${FADE})`;
  tctx.fillRect(0, 0, trails.width, trails.height);
  tctx.lineWidth = 1;
  for (let k = 0; k < particles.length; k++) {
    const p = particles[k], [, theta] = field(p.x, p.y);
    const nx = p.x + Math.cos(theta) * SPEED * dt, ny = p.y + Math.sin(theta) * SPEED * dt;
    tctx.strokeStyle = `hsla(${((theta / (2 * Math.PI)) % 1) * 360}, 65%, 62%, 0.35)`;
    tctx.beginPath();
    tctx.moveTo(p.x, p.y);
    tctx.lineTo(nx, ny);
    tctx.stroke();
    p.x = nx;
    p.y = ny;
    p.life -= dt;
    if (p.life <= 0 || nx < 0 || ny < 0 || nx >= canvas.width || ny >= canvas.height) particles[k] = spawn();
  }
  time += EVOLVE * dt;
  ctx.drawImage(trails, 0, 0);

  if (hover) {
    const [n, theta] = field(hover.x, hover.y);
    ctx.fillStyle = "rgba(10, 16, 14, 0.75)";
    ctx.beginPath();
    ctx.arc(hover.x, hover.y, 26, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = "#fff";
    ctx.lineWidth = 2.4;
    arrow(hover.x, hover.y, theta, 40);
    const degrees = ((theta * 180) / Math.PI) % 360;
    const text = `N = ${n.toFixed(3)}   theta = N * 2pi * ${TURNS} = ${degrees.toFixed(0)} deg`;
    const bx = Math.min(Math.max(hover.x - 170, 6), canvas.width - 346), by = hover.y + (hover.y < 420 ? 34 : -60);
    ctx.fillRect(bx, by, 340, 26);
    ctx.fillStyle = "#fff";
    ctx.font = "13px monospace";
    ctx.fillText(text, bx + 8, by + 17);
  }
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
