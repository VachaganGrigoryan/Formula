// Perspective projection: x' = f * x / z + cx,  y' = f * y / z + cy.
//
// A cube, a floor grid and a cloud of random 3D points are rotated, pushed in front of
// the camera, and flattened onto the screen by dividing by depth z: far points shrink
// toward the center. The inset shows the highlighted point's ray from above.
// Drag to orbit, scroll to move the camera, Up/Down change f, D toggles a dolly zoom.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec3 = [number, number, number];
type Vec2 = { x: number; y: number };

const NEAR = 0.05; // points closer than this to the eye are not drawn

/** Rotate around the vertical axis (yaw), then the horizontal axis (pitch). */
function rotate([x, y, z]: Vec3, yaw: number, pitch: number): Vec3 {
  const [sy, cy] = [Math.sin(yaw), Math.cos(yaw)];
  [x, z] = [x * cy + z * sy, -x * sy + z * cy];
  const [sp, cp] = [Math.sin(pitch), Math.cos(pitch)];
  [y, z] = [y * cp - z * sp, y * sp + z * cp];
  return [x, y, z];
}

/** Divide by depth; y is flipped so up is up on screen. Null when behind the near plane. */
function project([x, y, z]: Vec3, f: number, c: Vec2): Vec2 | null {
  return z > NEAR ? { x: (f * x) / z + c.x, y: (-f * y) / z + c.y } : null;
}

// A tiny seeded xorshift so the scatter is the same every run.
let seed = 3;
const random = (): number => {
  seed ^= seed << 13;
  seed ^= seed >>> 17;
  seed ^= seed << 5;
  return (seed >>> 0) / 4294967296;
};
const scatter: Vec3[] = Array.from({ length: 70 }, () => [random() * 2 - 1, random() * 2 - 1, random() * 2 - 1]);
const corner = (i: number): Vec3 => [i & 1 ? 1 : -1, i & 2 ? 1 : -1, i & 4 ? 1 : -1];
const cubeEdges: [number, number][] = [
  [0, 1], [1, 3], [3, 2], [2, 0], [4, 5], [5, 7], [7, 6], [6, 4], [0, 4], [1, 5], [2, 6], [3, 7],
];

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

const cam = { focal: 1, distance: 4, yaw: 0.6, pitch: -0.35, spinning: true };
let drag: Vec2 | null = null;
canvas.addEventListener("pointerdown", (e) => (drag = { x: e.offsetX, y: e.offsetY }));
canvas.addEventListener("pointermove", (e) => {
  if (!drag) return;
  cam.yaw += (e.offsetX - drag.x) * 0.01;
  cam.pitch = Math.max(-1.4, Math.min(1.4, cam.pitch + (e.offsetY - drag.y) * 0.01));
  drag = { x: e.offsetX, y: e.offsetY };
  cam.spinning = false;
});
window.addEventListener("pointerup", () => (drag = null));
canvas.addEventListener("wheel", (e) => {
  e.preventDefault();
  cam.distance = Math.max(2.2, Math.min(12, cam.distance + Math.sign(e.deltaY) * 0.5));
});
window.addEventListener("keydown", (e) => {
  if (e.key === "ArrowUp") cam.focal = Math.min(2.5, cam.focal + 0.1);
  if (e.key === "ArrowDown") cam.focal = Math.max(0.3, cam.focal - 0.1);
  if (e.key === "d" || e.key === "D") {
    // Dolly zoom: move the camera and scale f with it, so the cube keeps its size.
    cam.distance = cam.distance < 7 ? 10 : 3;
    cam.focal = 0.25 * cam.distance;
  }
});

const toCamera = (p: Vec3): Vec3 => {
  const [x, y, z] = rotate(p, cam.yaw, cam.pitch);
  return [x, y, z + cam.distance];
};

function segment(a: Vec2 | null, b: Vec2 | null, color: string, width = 1, dash: number[] = []): void {
  if (!a || !b) return;
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.setLineDash(dash);
  ctx.beginPath();
  ctx.moveTo(a.x, a.y);
  ctx.lineTo(b.x, b.y);
  ctx.stroke();
  ctx.setLineDash([]);
}

function dot(p: Vec2, r: number, color: string): void {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
  ctx.fill();
}

/** Top-down diagram: eye, image plane at f, and the ray to the highlighted point. */
function sideView(): void {
  const box = { x: 560, y: 440, w: 225, h: 145 };
  ctx.fillStyle = "#1e2a25";
  ctx.fillRect(box.x, box.y, box.w, box.h);
  const [x, , z] = toCamera(scatter[0]);
  const maxZ = cam.distance + 2;
  const eye = { x: box.x + 16, y: box.y + box.h / 2 };
  const to = (zz: number, xx: number): Vec2 => ({
    x: eye.x + (zz / maxZ) * (box.w - 28),
    y: eye.y - (xx / 2.2) * box.h * 0.42,
  });
  segment(to(cam.focal, 2.2), to(cam.focal, -2.2), "#f0b429", 2);
  const point = to(z, x);
  const hit = to(cam.focal, (cam.focal * x) / z);
  segment(eye, point, "#ef6b5b", 1, [4, 3]);
  dot(point, 4.5, "#ef6b5b");
  dot(hit, 3.5, "#e4ebe6");
  dot(eye, 4, "#e4ebe6");
  ctx.fillStyle = "#93a39b";
  ctx.font = "12px sans-serif";
  ctx.fillText("seen from above", box.x + 8, box.y + 16);
  ctx.fillText("x'", hit.x - 14, hit.y - 4);
}

let last = 0;
function frame(ms: number): void {
  const dt = Math.min((ms - last) / 1000, 0.1);
  last = ms;
  if (cam.spinning) cam.yaw += 0.35 * dt;
  const f = cam.focal * Math.min(canvas.width, canvas.height);
  const c = { x: canvas.width / 2, y: canvas.height / 2 };
  const screen = (p: Vec3): Vec2 | null => project(toCamera(p), f, c);
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Floor grid at y = -1 for a sense of depth, then the cube.
  for (let i = 0; i <= 8; i++) {
    const u = -1.6 + (3.2 * i) / 8;
    segment(screen([u, -1, -1.6]), screen([u, -1, 1.6]), "#2c3a34");
    segment(screen([-1.6, -1, u]), screen([1.6, -1, u]), "#2c3a34");
  }
  for (const [i, j] of cubeEdges) segment(screen(corner(i)), screen(corner(j)), "#7d8c85", 1.2);

  // Painter's algorithm: far points first so near ones cover them.
  const points = scatter.map((p, i) => ({ i, cam: toCamera(p) })).sort((a, b) => b.cam[2] - a.cam[2]);
  const [near, far] = [cam.distance - 1.8, cam.distance + 1.8];
  for (const { i, cam: q } of points) {
    const s = project(q, f, c);
    if (!s) continue;
    const depth = Math.max(0, Math.min(1, (q[2] - near) / (far - near)));
    const shade = Math.round(230 - depth * 150);
    const radius = Math.max(1.5, Math.min(18, (0.045 * f) / q[2]));
    dot(s, radius + 1, "#16201c");
    dot(s, radius, i === 0 ? "#ef6b5b" : `rgb(${shade}, ${Math.round(shade * 0.75)}, 60)`);
  }

  sideView();
  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px monospace";
  ctx.fillText(`f = ${cam.focal.toFixed(2)}   camera distance = ${cam.distance.toFixed(1)}`, 16, 28);
  ctx.fillStyle = "#93a39b";
  ctx.font = "13px sans-serif";
  ctx.fillText("drag: orbit   wheel: distance   Up/Down: f   D: dolly zoom", 16, 50);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
