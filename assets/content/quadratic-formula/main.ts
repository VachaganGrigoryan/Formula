// Quadratic formula: where a x^2 + b x + c = 0 crosses the x axis.
//
// The parabola y = a x^2 + b x + c with its real roots, its vertex and its axis of
// symmetry. The discriminant b^2 - 4ac decides how many roots there are: two when it
// is positive, one double root when it is zero, none when it is negative.
// Range inputs under the canvas change a, b and c.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

const EPS = 1e-12;

/** Real roots, smaller first, computed without cancellation. */
function roots(a: number, b: number, c: number): number[] {
  if (Math.abs(a) < EPS) return Math.abs(b) < EPS ? [] : [-c / b];
  const d = b * b - 4 * a * c;
  if (Math.abs(d) < EPS) return [-b / (2 * a)];
  if (d < 0) return [];
  const q = -0.5 * (b + (b < 0 ? -1 : 1) * Math.sqrt(d));
  return [q / a, c / q].sort((x, y) => x - y);
}

const coef = { a: 1, b: -1, c: -2 };
const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

for (const name of ["a", "b", "c"] as const) {
  const label = document.createElement("label");
  label.style.display = "block";
  label.style.fontFamily = "monospace";
  const input = document.createElement("input");
  input.type = "range";
  input.min = "-5";
  input.max = "5";
  input.step = "any";
  input.value = String(coef[name]);
  input.addEventListener("input", () => (coef[name] = Number(input.value)));
  label.append(input, ` ${name}`);
  document.body.appendChild(label);
}

const scale = 60; // pixels per unit
const toScreen = (x: number, y: number): [number, number] => [canvas.width / 2 + x * scale, canvas.height / 2 - y * scale];

function frame(): void {
  const { a, b, c } = coef;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.strokeStyle = "#55635d";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, canvas.height / 2);
  ctx.lineTo(canvas.width, canvas.height / 2);
  ctx.moveTo(canvas.width / 2, 0);
  ctx.lineTo(canvas.width / 2, canvas.height);
  ctx.stroke();

  ctx.strokeStyle = "#9db3ff";
  ctx.lineWidth = 2.4;
  ctx.beginPath();
  for (let px = 0; px <= canvas.width; px++) {
    const x = (px - canvas.width / 2) / scale;
    const [sx, sy] = toScreen(x, a * x * x + b * x + c);
    if (px === 0) ctx.moveTo(sx, sy);
    else ctx.lineTo(sx, sy);
  }
  ctx.stroke();

  if (Math.abs(a) >= EPS) {
    const vx = -b / (2 * a);
    const [sx, sy] = toScreen(vx, a * vx * vx + b * vx + c);
    ctx.setLineDash([5, 5]);
    ctx.strokeStyle = "#e3a93b";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(sx, 0);
    ctx.lineTo(sx, canvas.height);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.fillStyle = "#e3a93b";
    ctx.beginPath();
    ctx.arc(sx, sy, 4.5, 0, 2 * Math.PI);
    ctx.fill();
  }

  const found = roots(a, b, c);
  ctx.fillStyle = "#5cc2b2";
  for (const x of found) {
    const [sx, sy] = toScreen(x, 0);
    ctx.beginPath();
    ctx.arc(sx, sy, 6, 0, 2 * Math.PI);
    ctx.fill();
  }
  const d = b * b - 4 * a * c;
  ctx.fillStyle = "#e4ebe6";
  ctx.font = "15px sans-serif";
  const text = found.length ? found.map((x) => `x = ${x.toFixed(3)}`).join(", ") : "no real roots";
  ctx.fillText(`discriminant = ${d.toFixed(2)}: ${text}`, 16, 24);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
