// Euler's formula: e^(i theta) = cos(theta) + i sin(theta).
//
// The Taylor series 1 + i theta + (i theta)^2/2! + ... drawn as arrows placed head to
// tail. Each term is the previous one turned a quarter turn and scaled by theta / k, so
// the partial sums spiral in to the point at angle theta on the unit circle. At
// theta = pi the point is -1: e^(i pi) + 1 = 0.
// Range inputs under the canvas change theta and the number of terms.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Pt = [number, number];

/** S_0 ... S_n of sum (i theta)^k / k!, each term from the last one. */
function partialSums(theta: number, n: number): Pt[] {
  let term: Pt = [1, 0];
  let sum: Pt = [1, 0];
  const out: Pt[] = [sum];
  for (let k = 1; k <= n; k++) {
    const s = theta / k;
    term = [-term[1] * s, term[0] * s];
    sum = [sum[0] + term[0], sum[1] + term[1]];
    out.push(sum);
  }
  return out;
}

const state = { theta: 2, terms: 8 };
const canvas = document.createElement("canvas");
canvas.width = 700;
canvas.height = 700;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;
for (const [name, max, step] of [["theta", 2 * Math.PI, "any"], ["terms", 30, "1"]] as const) {
  const input = document.createElement("input");
  input.type = "range";
  input.min = "0";
  input.max = String(max);
  input.step = step;
  input.value = String(state[name]);
  input.addEventListener("input", () => (state[name] = Number(input.value)));
  const label = document.createElement("label");
  label.style.display = "block";
  label.append(input, ` ${name}`);
  document.body.appendChild(label);
}

function arrow(from: Pt, to: Pt, color: string): void {
  ctx.strokeStyle = color;
  ctx.fillStyle = color;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(from[0], from[1]);
  ctx.lineTo(to[0], to[1]);
  ctx.stroke();
  const angle = Math.atan2(to[1] - from[1], to[0] - from[0]);
  ctx.beginPath();
  ctx.moveTo(to[0], to[1]);
  ctx.lineTo(to[0] - 8 * Math.cos(angle - 0.4), to[1] - 8 * Math.sin(angle - 0.4));
  ctx.lineTo(to[0] - 8 * Math.cos(angle + 0.4), to[1] - 8 * Math.sin(angle + 0.4));
  ctx.fill();
}

function frame(): void {
  const { theta, terms } = state;
  const sums = partialSums(theta, terms);
  const reach = Math.max(1.2, ...sums.map(([x, y]) => Math.hypot(x, y) * 1.1));
  const k = 330 / reach;
  const at = ([x, y]: Pt): Pt => [350 + x * k, 350 - y * k];
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.strokeStyle = "#55635d";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.arc(350, 350, k, 0, 2 * Math.PI);
  ctx.stroke();
  let from = at([0, 0]);
  sums.forEach((s, i) => {
    const to = at(s);
    arrow(from, to, i % 2 === 0 ? "#5cc2b2" : "#e3a93b");
    from = to;
  });
  const [tx, ty] = at([Math.cos(theta), Math.sin(theta)]);
  ctx.fillStyle = "#9db3ff";
  ctx.beginPath();
  ctx.arc(tx, ty, 6, 0, 2 * Math.PI);
  ctx.fill();
  const [re, im] = sums[sums.length - 1];
  ctx.fillStyle = "#e4ebe6";
  ctx.font = "14px monospace";
  ctx.fillText(`theta = ${theta.toFixed(3)}  sum = ${re.toFixed(4)} ${im >= 0 ? "+" : "-"} ${Math.abs(im).toFixed(4)}i`, 12, 22);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
