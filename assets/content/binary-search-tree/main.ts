// AVL binary search tree: left subtree < key < right subtree, |h(left) - h(right)| <= 1.
//
// Ten keys are inserted with AVL balancing and the tree is drawn with one circle
// per key. Click to insert a random key and press d to delete the last key you
// inserted. Every rotation is shown for half a second: the nodes glide from their
// old place to the new one (a node is identified by its key, so it is tracked
// through the rotation). When a deletion has to rebalance while it relinks a node
// with two children, that rebalancing is shown as a single step.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Link = number | null;
/** key -> [x in 0..1 by in-order rank, depth] */
type Layout = Map<number, [number, number]>;

const START_KEYS = [50, 30, 70, 20, 40, 60, 80, 35, 45, 65];
const STEP = 0.5; // seconds each rotation is shown
const WIDTH = 900;
const HEIGHT = 560;
const HEADER = 30;
const RADIUS = 17;
const PAPER = "#16201c";
const RULE = "#5c6b64";
const NODE = "#243a52";
const INK = "#e6ece8";

const canvas = document.createElement("canvas");
canvas.width = WIDTH;
canvas.height = HEIGHT;
canvas.style.background = PAPER;
canvas.tabIndex = 0;
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

/** An AVL tree whose nodes are identified by their (unique) keys. */
class Tree {
  left = new Map<number, Link>();
  right = new Map<number, Link>();
  height = new Map<number, number>();
  root: Link = null;
  /** A layout after every rotation, so the animation can glide between them. */
  snaps: Layout[] = [];
  rotated = false; // a rotation happened and is not shown yet
  quiet = false; // no snapshots while a subtree is detached

  private l(k: number): Link {
    return this.left.get(k)!;
  }
  private r(k: number): Link {
    return this.right.get(k)!;
  }
  h(k: Link): number {
    return k === null ? 0 : this.height.get(k)!;
  }
  private fix(k: number): void {
    this.height.set(k, 1 + Math.max(this.h(this.l(k)), this.h(this.r(k))));
  }
  private bf(k: number): number {
    return this.h(this.l(k)) - this.h(this.r(k));
  }

  private rotateRight(k: number): number {
    const y = this.l(k)!;
    this.left.set(k, this.r(y));
    this.right.set(y, k);
    this.fix(k);
    this.fix(y);
    return y;
  }
  private rotateLeft(k: number): number {
    const y = this.r(k)!;
    this.right.set(k, this.l(y));
    this.left.set(y, k);
    this.fix(k);
    this.fix(y);
    return y;
  }

  /** Restore |bf| <= 1 at k; returns the new root of that subtree. */
  private balance(k: number): number {
    this.fix(k);
    const b = this.bf(k);
    if (b > 1) {
      if (this.bf(this.l(k)!) < 0) {
        this.left.set(k, this.rotateLeft(this.l(k)!));
        this.snap();
      }
      this.rotated = true;
      return this.rotateRight(k);
    }
    if (b < -1) {
      if (this.bf(this.r(k)!) > 0) {
        this.right.set(k, this.rotateRight(this.r(k)!));
        this.snap();
      }
      this.rotated = true;
      return this.rotateLeft(k);
    }
    return k;
  }

  /** Called right after a subtree was linked back in: show a pending rotation. */
  private flush(): void {
    if (this.rotated) this.snap();
    this.rotated = false;
  }

  private insertAt(k: Link, key: number): number {
    if (k === null) {
      this.left.set(key, null);
      this.right.set(key, null);
      this.height.set(key, 1);
      return key;
    }
    if (key < k) this.left.set(k, this.insertAt(this.l(k), key));
    else this.right.set(k, this.insertAt(this.r(k), key));
    this.flush();
    return this.balance(k);
  }

  private removeMin(k: number): Link {
    const lo = this.l(k);
    if (lo === null) return this.r(k);
    this.left.set(k, this.removeMin(lo));
    this.flush();
    return this.balance(k);
  }

  private deleteAt(k: Link, key: number): Link {
    if (k === null) return null;
    if (key < k) this.left.set(k, this.deleteAt(this.l(k), key));
    else if (key > k) this.right.set(k, this.deleteAt(this.r(k), key));
    else {
      const lo = this.l(k);
      const hi = this.r(k);
      this.left.delete(k);
      this.right.delete(k);
      this.height.delete(k);
      if (lo === null) return hi;
      if (hi === null) return lo;
      // Two children: the smallest key of the right side takes this place.
      let s = hi;
      while (this.l(s) !== null) s = this.l(s)!;
      this.quiet = true;
      const rest = this.removeMin(hi);
      this.left.set(s, lo);
      this.right.set(s, rest);
      const top = this.balance(s);
      this.quiet = false;
      return top;
    }
    this.flush();
    return this.balance(k);
  }

  insert(key: number): void {
    if (this.height.has(key)) return;
    this.snaps = [this.layout()];
    this.root = this.insertAt(this.root, key);
    this.flush();
    this.finish();
  }

  delete(key: number): void {
    if (!this.height.has(key)) return;
    this.snaps = [this.layout()];
    this.root = this.deleteAt(this.root, key);
    this.flush();
    this.finish();
  }

  private finish(): void {
    this.rotated = false;
    this.snap();
  }

  /** x by in-order rank, and the depth, of every key. */
  layout(): Layout {
    const order: [number, number][] = [];
    const walk = (k: Link, depth: number): void => {
      if (k === null) return;
      walk(this.l(k), depth + 1);
      order.push([k, depth]);
      walk(this.r(k), depth + 1);
    };
    walk(this.root, 0);
    const out: Layout = new Map();
    order.forEach(([k, d], i) => out.set(k, [(i + 0.5) / order.length, d]));
    return out;
  }

  private snap(): void {
    if (!this.quiet) this.snaps.push(this.layout());
  }

  get size(): number {
    return this.height.size;
  }
  get depth(): number {
    return this.h(this.root);
  }
  children(k: number): Link[] {
    return [this.l(k), this.r(k)];
  }
}

const tree = new Tree();
const inserted: number[] = [];
for (const key of START_KEYS) {
  tree.insert(key);
  inserted.push(key);
}

/** A small seeded generator, so a run is repeatable. */
let state = 12345;
function random(): number {
  state = (state ^ (state << 13)) >>> 0;
  state = (state ^ (state >>> 17)) >>> 0;
  state = (state ^ (state << 5)) >>> 0;
  return state / 4294967296;
}

let shots: Layout[] = [tree.layout()]; // layouts being glided through
let clock = 0; // seconds since the first of `shots`

function run(operation: () => void): void {
  operation();
  shots = tree.snaps;
  clock = 0;
}

/** Node positions at the current time, interpolated by key. */
function positions(): Map<number, [number, number]> {
  const seg = Math.min(Math.floor(clock / STEP), Math.max(shots.length - 2, 0));
  const a = shots[seg];
  const b = shots[Math.min(seg + 1, shots.length - 1)];
  let t = Math.min(Math.max(clock / STEP - seg, 0), 1);
  t = t * t * (3 - 2 * t); // smoothstep
  const out = new Map<number, [number, number]>();
  for (const [key, [bx, bd]] of b) {
    const [ax, ad] = a.get(key) ?? [bx, bd];
    out.set(key, [ax + (bx - ax) * t, ad + (bd - ad) * t]);
  }
  return out;
}

function draw(): void {
  ctx.clearRect(0, 0, WIDTH, HEIGHT);
  const pos = positions();
  const levels = Math.max(tree.depth, 1);
  const at = (x: number, d: number): [number, number] => [
    RADIUS + 8 + x * (WIDTH - 2 * RADIUS - 16),
    HEADER + RADIUS + 10 + (d / Math.max(levels - 1, 1)) * (HEIGHT - HEADER - 2 * RADIUS - 20),
  ];
  ctx.strokeStyle = RULE;
  ctx.lineWidth = 1.5;
  for (const [key, [x, d]] of pos) {
    for (const child of tree.children(key)) {
      const c = child === null ? undefined : pos.get(child);
      if (!c) continue;
      ctx.beginPath();
      ctx.moveTo(...at(x, d));
      ctx.lineTo(...at(c[0], c[1]));
      ctx.stroke();
    }
  }
  ctx.font = "13px monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  for (const [key, [x, d]] of pos) {
    const [px, py] = at(x, d);
    ctx.fillStyle = NODE;
    ctx.beginPath();
    ctx.arc(px, py, RADIUS, 0, 2 * Math.PI);
    ctx.fill();
    ctx.strokeStyle = INK;
    ctx.lineWidth = 1.2;
    ctx.stroke();
    ctx.fillStyle = INK;
    ctx.fillText(String(key), px, py);
  }
  ctx.textAlign = "left";
  ctx.fillText(`n = ${tree.size}, height = ${tree.depth}`, 6, HEADER / 2);
}

canvas.addEventListener("pointerdown", () => {
  const free: number[] = [];
  for (let k = -99; k <= 99; k++) if (!tree.height.has(k)) free.push(k);
  if (free.length === 0) return;
  const key = free[Math.floor(random() * free.length)];
  inserted.push(key);
  run(() => tree.insert(key));
});

canvas.addEventListener("keydown", (e) => {
  if (e.key === "d" && inserted.length > 0) {
    const key = inserted.pop()!;
    run(() => tree.delete(key));
  }
});

let last = performance.now();
function frame(now: number): void {
  clock += (now - last) / 1000;
  last = now;
  draw();
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
