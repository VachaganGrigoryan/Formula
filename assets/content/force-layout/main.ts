// Springs and repulsion: F_s = -k (d - x0) on every edge, F_r = kr / d^2 between every pair.
//
// A random graph of four clusters relaxes into a readable layout. Edge color shows the
// spring's strain (red stretched, blue squeezed). Drag a node to pin it and watch the
// graph rebalance; hover a node to see its spring and repulsion forces.
// Keys: S shuffles the positions, N makes a new graph.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Vec2 = { x: number; y: number };
type GraphNode = { pos: Vec2; vel: Vec2; cluster: number };
type Edge = [number, number];

const K = 0.04, REST = 60, KR = 3000; // stiffness, rest length, repulsion strength
const GRAVITY = 0.004, DAMPING = 0.85, MAX_SPEED = 12;

/** Hooke's law along an edge: negative pulls the two ends together. */
const spring = (k: number, d: number, rest: number): number => -k * (d - rest);
/** Coulomb-style repulsion: always pushes apart, falls off as 1 / d^2. */
const repulsion = (kr: number, d: number): number => kr / (d * d);

/** Spring and repulsion forces on every node. */
function forces(nodes: GraphNode[], edges: Edge[]): [Vec2[], Vec2[]] {
  const springs = nodes.map(() => ({ x: 0, y: 0 }));
  const repel = nodes.map(() => ({ x: 0, y: 0 }));
  const push = (out: Vec2[], i: number, j: number, minD: number, force: (d: number) => number): void => {
    const dx = nodes[i].pos.x - nodes[j].pos.x, dy = nodes[i].pos.y - nodes[j].pos.y;
    const d = Math.max(Math.hypot(dx, dy), minD);
    const f = force(d) / d;
    out[i].x += dx * f; out[i].y += dy * f;
    out[j].x -= dx * f; out[j].y -= dy * f;
  };
  for (const [i, j] of edges) push(springs, i, j, 1e-3, (d) => spring(K, d, REST));
  for (let i = 0; i < nodes.length; i++)
    for (let j = i + 1; j < nodes.length; j++) push(repel, i, j, 4, (d) => repulsion(KR, d));
  return [springs, repel];
}

/** One damped integration step; `pinned` stays put. Returns the kinetic energy. */
function step(nodes: GraphNode[], edges: Edge[], pinned: number | null): number {
  const [springs, repel] = forces(nodes, edges);
  let kinetic = 0;
  nodes.forEach((n, i) => {
    if (i === pinned) { n.vel = { x: 0, y: 0 }; return; }
    n.vel.x = (n.vel.x + springs[i].x + repel[i].x - GRAVITY * n.pos.x) * DAMPING;
    n.vel.y = (n.vel.y + springs[i].y + repel[i].y - GRAVITY * n.pos.y) * DAMPING;
    const speed = Math.hypot(n.vel.x, n.vel.y);
    if (speed > MAX_SPEED) { n.vel.x *= MAX_SPEED / speed; n.vel.y *= MAX_SPEED / speed; }
    n.pos.x += n.vel.x;
    n.pos.y += n.vel.y;
    kinetic += 0.5 * (n.vel.x ** 2 + n.vel.y ** 2);
  });
  return kinetic;
}

const rand = (lo: number, hi: number): number => lo + Math.random() * (hi - lo);
const randInt = (n: number): number => Math.floor(Math.random() * n);

/** Four clusters, each a random tree plus a few extra edges, chained together. */
function randomGraph(): [GraphNode[], Edge[]] {
  const nodes: GraphNode[] = [], edges: Edge[] = [], firsts: number[] = [];
  for (let c = 0; c < 4; c++) {
    const start = nodes.length, size = 5 + randInt(5);
    firsts.push(start);
    for (let i = 0; i < size; i++) {
      nodes.push({ pos: { x: rand(-150, 150), y: rand(-110, 110) }, vel: { x: 0, y: 0 }, cluster: c });
      if (i > 0) edges.push([start + i, start + randInt(i)]);
    }
    for (let e = 0; e < Math.floor(size / 3); e++) {
      const a = start + randInt(size), b = start + randInt(size);
      if (a !== b && !edges.some(([p, q]) => (p === a && q === b) || (p === b && q === a))) edges.push([a, b]);
    }
    if (c > 0) edges.push([firsts[c - 1], start]);
  }
  return [nodes, edges];
}

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;
const CLUSTER_COLORS = ["#2a9d8f", "#e9c46a", "#7ea7e0", "#c38bd6"];
const toScreen = (p: Vec2): Vec2 => ({ x: canvas.width / 2 + p.x, y: canvas.height / 2 + p.y });

let [nodes, edges] = randomGraph();
let dragging: number | null = null;
let hovered: number | null = null;
let kinetic = 0;

const pick = (e: PointerEvent, radius: number): number | null => {
  let best: number | null = null, bestD = radius;
  for (let i = 0; i < nodes.length; i++) {
    const s = toScreen(nodes[i].pos), d = Math.hypot(s.x - e.offsetX, s.y - e.offsetY);
    if (d < bestD) { best = i; bestD = d; }
  }
  return best;
};
canvas.addEventListener("pointerdown", (e) => (dragging = pick(e, 20)));
canvas.addEventListener("pointermove", (e) => {
  hovered = pick(e, 16);
  if (dragging !== null) nodes[dragging].pos = { x: e.offsetX - canvas.width / 2, y: e.offsetY - canvas.height / 2 };
});
window.addEventListener("pointerup", () => (dragging = null));
window.addEventListener("keydown", (e) => {
  if (e.key === "n") [nodes, edges] = randomGraph();
  if (e.key === "s") nodes.forEach((n) => { n.pos = { x: rand(-200, 200), y: rand(-140, 140) }; n.vel = { x: 0, y: 0 }; });
});

function arrow(from: Vec2, v: Vec2, color: string): void {
  const len = Math.hypot(v.x, v.y);
  if (len < 1) return;
  const s = Math.min(len, 120) / len, to = { x: from.x + v.x * s, y: from.y + v.y * s };
  const a = Math.atan2(to.y - from.y, to.x - from.x);
  ctx.strokeStyle = ctx.fillStyle = color;
  ctx.lineWidth = 2.5;
  ctx.beginPath(); ctx.moveTo(from.x, from.y); ctx.lineTo(to.x, to.y); ctx.stroke();
  ctx.beginPath(); ctx.moveTo(to.x, to.y);
  ctx.lineTo(to.x - 9 * Math.cos(a - 0.4), to.y - 9 * Math.sin(a - 0.4));
  ctx.lineTo(to.x - 9 * Math.cos(a + 0.4), to.y - 9 * Math.sin(a + 0.4));
  ctx.fill();
}

function frame(): void {
  for (let s = 0; s < 2; s++) kinetic = step(nodes, edges, dragging);
  const hw = canvas.width / 2 - 12, hh = canvas.height / 2 - 12;
  for (const n of nodes) { n.pos.x = Math.max(-hw, Math.min(hw, n.pos.x)); n.pos.y = Math.max(-hh, Math.min(hh, n.pos.y)); }
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Edges: gray at rest length, red when stretched, blue when squeezed.
  for (const [i, j] of edges) {
    const a = toScreen(nodes[i].pos), b = toScreen(nodes[j].pos);
    const strain = Math.max(-1, Math.min(1, (Math.hypot(a.x - b.x, a.y - b.y) - REST) / REST));
    const t = Math.abs(strain), target = strain > 0 ? [230, 80, 80] : [80, 140, 240];
    const c = [130, 140, 135].map((g, k) => Math.round(g + (target[k] - g) * t));
    ctx.strokeStyle = `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
    ctx.lineWidth = 1.5 + 2 * t;
    ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke();
  }
  nodes.forEach((n, i) => {
    const p = toScreen(n.pos), active = i === dragging || i === hovered;
    ctx.fillStyle = CLUSTER_COLORS[n.cluster % 4];
    ctx.strokeStyle = "#16201c";
    ctx.lineWidth = 2;
    ctx.beginPath(); ctx.arc(p.x, p.y, active ? 9 : 7, 0, Math.PI * 2); ctx.fill(); ctx.stroke();
  });

  // Forces on the focused node, scaled up so they are visible.
  const focus = dragging ?? hovered;
  if (focus !== null) {
    const [springs, repel] = forces(nodes, edges), p = toScreen(nodes[focus].pos);
    arrow(p, { x: springs[focus].x * 12, y: springs[focus].y * 12 }, "#ff6b6b");
    arrow(p, { x: repel[focus].x * 12, y: repel[focus].y * 12 }, "#4ecdc4");
  }
  ctx.font = "13px monospace";
  ctx.fillStyle = "#8a9a92";
  ctx.fillText("drag a node   red: spring force   teal: repulsion   S: shuffle   N: new graph", 14, 22);
  ctx.fillText(`kinetic energy ${kinetic.toFixed(3)}  (${kinetic < 0.01 ? "settled" : "moving"})`, 14, canvas.height - 14);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
