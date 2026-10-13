// Voronoi diagram: V(p_i) = { x : |x - p_i| <= |x - p_j| for all j != i }.
//
// Each seed owns every location closer to it than to any other seed. Cells are computed
// exactly: start from the whole canvas and cut away, for every other seed, the half-plane
// beyond the perpendicular bisector. Seeds drift; hover to compare the distances to the
// nearest seed and the runner-up. Drag a seed, click to add one, right-click to remove.
// Keys: R toggles Lloyd relaxation (seeds slide to their cell centroids), D toggles drift.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };
type Seed = { at: Vec2; vel: Vec2; hue: number };

const W = 800;
const H = 540; // the diagram's box; a status line sits below it
const dist = (a: Vec2, b: Vec2): number => Math.hypot(a.x - b.x, a.y - b.y);

/** Keep the part of a convex polygon closer to p than to q (one Sutherland-Hodgman pass). */
function clipCloser(poly: Vec2[], p: Vec2, q: Vec2): Vec2[] {
  const m = { x: (p.x + q.x) / 2, y: (p.y + q.y) / 2 };
  const side = (a: Vec2): number => (a.x - m.x) * (q.x - p.x) + (a.y - m.y) * (q.y - p.y); // <= 0: p's side
  const out: Vec2[] = [];
  poly.forEach((a, k) => {
    const b = poly[(k + 1) % poly.length];
    const [sa, sb] = [side(a), side(b)];
    if (sa <= 0) out.push(a);
    if ((sa <= 0) !== (sb <= 0)) {
      const t = sa / (sa - sb); // where the edge crosses the bisector
      out.push({ x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t });
    }
  });
  return out;
}

/** The Voronoi cell of points[i] inside the box. */
function cell(points: Vec2[], i: number): Vec2[] {
  let poly: Vec2[] = [{ x: 0, y: 0 }, { x: W, y: 0 }, { x: W, y: H }, { x: 0, y: H }];
  points.forEach((q, j) => {
    const p = points[i];
    if (j !== i && (q.x !== p.x || q.y !== p.y) && poly.length > 0) poly = clipCloser(poly, p, q);
  });
  return poly;
}

/** Center of mass of a polygon (shoelace formula). */
function centroid(poly: Vec2[]): Vec2 | null {
  let [a2, cx, cy] = [0, 0, 0];
  poly.forEach((p, k) => {
    const q = poly[(k + 1) % poly.length];
    const cross = p.x * q.y - q.x * p.y;
    a2 += cross;
    cx += (p.x + q.x) * cross;
    cy += (p.y + q.y) * cross;
  });
  return Math.abs(a2) > 1e-6 ? { x: cx / (3 * a2), y: cy / (3 * a2) } : null;
}

const canvas = document.createElement("canvas");
canvas.width = W;
canvas.height = H + 60;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

function newSeed(at: Vec2): Seed {
  const angle = Math.random() * Math.PI * 2;
  const speed = 12 + Math.random() * 20;
  return { at, vel: { x: Math.cos(angle) * speed, y: Math.sin(angle) * speed }, hue: Math.random() * 360 };
}
const randomPoint = (): Vec2 => ({ x: 20 + Math.random() * (W - 40), y: 20 + Math.random() * (H - 40) });
const seeds: Seed[] = Array.from({ length: 24 }, () => newSeed(randomPoint()));
let drift = true;
let relax = false;
let dragging: number | null = null;
let hover: Vec2 | null = null;

const pick = (at: Vec2): number => seeds.findIndex((s) => dist(s.at, at) < 12);
canvas.addEventListener("contextmenu", (e) => e.preventDefault());
canvas.addEventListener("pointerdown", (e) => {
  const at = { x: e.offsetX, y: e.offsetY };
  if (at.y > H) return;
  const k = pick(at);
  if (e.button === 2) {
    if (k >= 0 && seeds.length > 2) seeds.splice(k, 1);
  } else if (k >= 0) dragging = k;
  else if (seeds.length < 200) seeds.push(newSeed(at));
});
canvas.addEventListener("pointermove", (e) => {
  hover = e.offsetY <= H ? { x: e.offsetX, y: e.offsetY } : null;
  if (dragging !== null && dragging < seeds.length && hover) seeds[dragging].at = hover;
});
canvas.addEventListener("pointerleave", () => (hover = null));
window.addEventListener("pointerup", () => (dragging = null));
window.addEventListener("keydown", (e) => {
  if (e.key === "r") relax = !relax;
  if (e.key === "d") drift = !drift;
});

function animate(dt: number): void {
  seeds.forEach((s, k) => {
    if (!drift || k === dragging) return;
    s.at = { x: s.at.x + s.vel.x * dt, y: s.at.y + s.vel.y * dt };
    if (s.at.x < 5 || s.at.x > W - 5) s.vel.x = -s.vel.x; // bounce off the walls
    if (s.at.y < 5 || s.at.y > H - 5) s.vel.y = -s.vel.y;
    s.at = { x: Math.min(Math.max(s.at.x, 5), W - 5), y: Math.min(Math.max(s.at.y, 5), H - 5) };
  });
  if (relax) {
    // Lloyd's algorithm: each seed slides toward the centroid of its cell.
    const points = seeds.map((s) => s.at);
    seeds.forEach((s, k) => {
      const c = centroid(cell(points, k));
      if (!c || k === dragging) return;
      const f = Math.min(dt * 2, 1);
      s.at = { x: s.at.x + (c.x - s.at.x) * f, y: s.at.y + (c.y - s.at.y) * f };
    });
  }
}

function labelAt(a: Vec2, b: Vec2, text: string, color: string): void {
  const mid = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
  const w = ctx.measureText(text).width;
  ctx.fillStyle = "#16201c";
  ctx.fillRect(mid.x - w / 2 - 4, mid.y - 10, w + 8, 18);
  ctx.fillStyle = color;
  ctx.fillText(text, mid.x - w / 2, mid.y + 4);
}

let last = 0;
function frame(ms: number): void {
  const dt = Math.min((ms - last) / 1000, 0.1);
  last = ms;
  animate(dt);
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const points = seeds.map((s) => s.at);
  const at = hover;
  const order = at ? points.map((_, i) => i).sort((a, b) => dist(points[a], at) - dist(points[b], at)) : [];
  points.forEach((_, i) => {
    const poly = cell(points, i);
    if (poly.length < 3) return;
    ctx.beginPath();
    poly.forEach((p, k) => (k === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
    ctx.closePath();
    ctx.fillStyle = `hsl(${seeds[i].hue}, 45%, ${order[0] === i ? 52 : 40}%)`;
    ctx.fill();
    ctx.strokeStyle = "#16201c";
    ctx.lineWidth = 1.5;
    ctx.stroke();
  });
  points.forEach((p, i) => {
    ctx.fillStyle = i === dragging ? "#f0b45a" : "#e4ebe6";
    ctx.beginPath();
    ctx.arc(p.x, p.y, i === dragging ? 6 : 3.5, 0, Math.PI * 2);
    ctx.fill();
  });

  // Hover: the nearest seed wins; the runner-up is shown for comparison.
  ctx.font = "13px sans-serif";
  if (at && order.length > 1) {
    const [i, j] = [points[order[0]], points[order[1]]];
    for (const [p, color, width] of [[j, "#e5707e", 1.5], [i, "#5ab4e6", 2.5]] as [Vec2, string, number][]) {
      ctx.strokeStyle = color;
      ctx.lineWidth = width;
      ctx.beginPath();
      ctx.moveTo(at.x, at.y);
      ctx.lineTo(p.x, p.y);
      ctx.stroke();
    }
    labelAt(at, j, dist(at, j).toFixed(0), "#e5707e");
    labelAt(at, i, `${dist(at, i).toFixed(0)} nearest`, "#5ab4e6");
  }

  ctx.fillStyle = "#93a39b";
  ctx.font = "14px monospace";
  const onOff = (b: boolean): string => (b ? "on" : "off");
  ctx.fillText(`${seeds.length} seeds   drift ${onOff(drift)} (D)   Lloyd relax ${onOff(relax)} (R)`, 16, H + 26);
  ctx.fillText("drag a seed, click to add, right-click to remove", 16, H + 48);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
