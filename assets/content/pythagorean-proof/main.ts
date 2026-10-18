// Pythagorean theorem: a proof by rearrangement.
//
// Four copies of a right triangle with legs a and b sit in the corners of a square of
// side a + b, leaving a tilted square of side c uncovered. Three of them then slide,
// one after another, until the uncovered part is a square of side a and one of side b.
// The big square and the triangles never changed, so c^2 = a^2 + b^2.
// Range inputs under the canvas change a and b.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Pt = [number, number];

/** The four triangles around the c^2 square: right angle, end of leg a, end of leg b. */
function tilted(a: number, b: number): Pt[][] {
  const s = a + b;
  return [
    [[0, 0], [a, 0], [0, b]],
    [[s, 0], [s, a], [a, 0]],
    [[s, s], [b, s], [s, a]],
    [[0, s], [0, b], [b, s]],
  ];
}

const smoothstep = (u: number): number => {
  const v = Math.min(Math.max(u, 0), 1);
  return v * v * (3 - 2 * v);
};

function triangles(a: number, b: number, t: number): Pt[][] {
  const tris = tilted(a, b);
  const moves: Pt[] = [[0, a], [0, 0], [-b, 0], [a, -b]];
  [3, 0, 2].forEach((i, turn) => {
    const u = smoothstep(t * 3 - turn);
    tris[i] = tris[i].map(([x, y]): Pt => [x + moves[i][0] * u, y + moves[i][1] * u]);
  });
  return tris;
}

/** Hold at 0, move to 1, hold, move back. */
function cycle(phase: number, period = 10): number {
  const u = (phase % period) / period;
  if (u < 0.1) return 0;
  if (u < 0.45) return (u - 0.1) / 0.35;
  if (u < 0.6) return 1;
  return 1 - (u - 0.6) / 0.4;
}

const legs = { a: 3, b: 4 };
const canvas = document.createElement("canvas");
canvas.width = 600;
canvas.height = 640;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;
for (const name of ["a", "b"] as const) {
  const input = document.createElement("input");
  input.type = "range";
  input.min = "0.5";
  input.max = "6";
  input.step = "any";
  input.value = String(legs[name]);
  input.addEventListener("input", () => (legs[name] = Number(input.value)));
  const label = document.createElement("label");
  label.style.display = "block";
  label.append(input, ` ${name}`);
  document.body.appendChild(label);
}

let last = 0;
let phase = 0;
function frame(ms: number): void {
  phase += Math.min((ms - last) / 1000, 0.1);
  last = ms;
  const { a, b } = legs;
  const s = a + b;
  const t = cycle(phase);
  const k = 520 / s;
  const at = ([x, y]: Pt): Pt => [40 + x * k, 590 - y * k];
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "rgba(227, 169, 59, 0.25)";
  ctx.fillRect(40, 590 - s * k, s * k, s * k);
  for (const tri of triangles(a, b, t)) {
    ctx.beginPath();
    tri.map(at).forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
    ctx.closePath();
    ctx.fillStyle = "#16201c";
    ctx.fill();
    ctx.strokeStyle = "#9db3ff";
    ctx.lineWidth = 1.5;
    ctx.stroke();
  }
  ctx.fillStyle = "#e4ebe6";
  ctx.font = "22px sans-serif";
  ctx.textAlign = "center";
  if (t < 0.5) {
    const [x, y] = at([s / 2, s / 2]);
    ctx.fillText("c^2", x, y);
  } else {
    const [x1, y1] = at([a / 2, a / 2]);
    const [x2, y2] = at([a + b / 2, a + b / 2]);
    ctx.fillText("a^2", x1, y1);
    ctx.fillText("b^2", x2, y2);
  }
  ctx.textAlign = "left";
  ctx.font = "15px sans-serif";
  ctx.fillText(`a^2 + b^2 = ${(a * a + b * b).toFixed(2)} = c^2`, 16, 26);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
