// Relative luminance (WCAG): Y = 0.2126 R + 0.7152 G + 0.0722 B on linearized sRGB,
// and contrast = (Y_light + 0.05) / (Y_dark + 0.05).
//
// Every heatmap cell shows its Y, written in black or white, whichever contrasts more.
// Click a cell to inspect it below: the swatch with both text colors and their contrast
// ratios, and a stacked bar of the red, green and blue contributions to Y with the
// switch point (Y = 0.179) marked. Keys: L toggles linearization, N the naive
// "always white" labels.
//
// Self-contained: compile with `tsc main.ts` and load main.js in any HTML page
// (it creates its own <canvas>), or paste into a TypeScript playground.

type Rgb = [number, number, number];

const WEIGHTS: Rgb = [0.2126, 0.7152, 0.0722]; // the eye is most sensitive to green
const SWITCH = Math.sqrt(1.05 * 0.05) - 0.05; // black and white text contrast equally here

/** Undo the sRGB transfer curve: encoded 0..1 value -> linear light. */
const srgbToLinear = (c: number): number => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);

/** The three weighted terms of Y for an 8-bit color. */
const contributions = (rgb: Rgb, linearize: boolean): Rgb =>
  rgb.map((v, i) => WEIGHTS[i] * (linearize ? srgbToLinear(v / 255) : v / 255)) as Rgb;

const luminance = (rgb: Rgb, linearize: boolean): number =>
  contributions(rgb, linearize).reduce((a, b) => a + b, 0);

/** WCAG contrast ratio between two luminances, from 1:1 up to 21:1. */
const contrast = (a: number, b: number): number => (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);

const prefersBlackText = (y: number): boolean => contrast(y, 0) >= contrast(y, 1);

/** Hue, saturation, value (all 0..1) to 8-bit sRGB. */
function hsvToRgb(h: number, s: number, v: number): Rgb {
  const f = (n: number): number => {
    const k = (n + h * 6) % 6;
    return Math.round(255 * (v - v * s * Math.max(0, Math.min(k, 4 - k, 1))));
  };
  return [f(5), f(3), f(1)];
}

const COLS = 12;
/** Saturation and value for each heatmap row, light to dark. */
const ROWS: [number, number][] = [[0.3, 1], [0.65, 1], [1, 0.95], [1, 0.7], [0.9, 0.42]];
const GRID = { x: 18, y: 18, w: 764, h: 360 };
const cellColor = (col: number, row: number): Rgb => hsvToRgb(col / COLS, ROWS[row][0], ROWS[row][1]);
const css = ([r, g, b]: Rgb): string => `rgb(${r}, ${g}, ${b})`;

const canvas = document.createElement("canvas");
canvas.width = 800;
canvas.height = 600;
canvas.style.background = "#16201c";
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d")!;

let selected: [number, number] = [2, 2]; // column, row
let hovered: [number, number] | null = null;
let linearize = true;
let naive = false;

function cellAt(x: number, y: number): [number, number] | null {
  if (x < GRID.x || x >= GRID.x + GRID.w || y < GRID.y || y >= GRID.y + GRID.h) return null;
  return [Math.floor(((x - GRID.x) / GRID.w) * COLS), Math.floor(((y - GRID.y) / GRID.h) * ROWS.length)];
}
canvas.addEventListener("pointermove", (e) => (hovered = cellAt(e.offsetX, e.offsetY)));
canvas.addEventListener("pointerdown", (e) => {
  const c = cellAt(e.offsetX, e.offsetY);
  if (c) selected = c;
});
window.addEventListener("keydown", (e) => {
  if (e.key === "l") linearize = !linearize;
  if (e.key === "n") naive = !naive;
});

function drawGrid(): void {
  const [w, h] = [GRID.w / COLS, GRID.h / ROWS.length];
  ctx.font = "14px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  for (let row = 0; row < ROWS.length; row++) {
    for (let col = 0; col < COLS; col++) {
      const rgb = cellColor(col, row);
      const y = luminance(rgb, linearize);
      const [x0, y0] = [GRID.x + w * col + 2, GRID.y + h * row + 2];
      ctx.fillStyle = css(rgb);
      ctx.fillRect(x0, y0, w - 4, h - 4);
      const isSelected = selected[0] === col && selected[1] === row;
      const isHovered = hovered !== null && hovered[0] === col && hovered[1] === row;
      if (isSelected || isHovered) {
        ctx.strokeStyle = isSelected ? "#e4ebe6" : "#93a39b";
        ctx.lineWidth = isSelected ? 3 : 1.5;
        ctx.strokeRect(x0 - 2, y0 - 2, w, h);
      }
      ctx.fillStyle = naive || !prefersBlackText(y) ? "#fff" : "#000";
      ctx.fillText(y.toFixed(2), x0 + w / 2 - 2, y0 + h / 2 - 2);
    }
  }
  ctx.textAlign = "left";
  ctx.textBaseline = "alphabetic";
}

function drawPanel(): void {
  const rgb = cellColor(selected[0], selected[1]);
  const parts = contributions(rgb, linearize);
  const y = parts[0] + parts[1] + parts[2];
  const top = 400;

  // Swatch with both text options and their contrast ratios.
  ctx.fillStyle = css(rgb);
  ctx.fillRect(18, top, 260, 180);
  const black = prefersBlackText(y);
  const options: [string, string, number, boolean][] = [
    ["Black text", "#000", contrast(y, 0), black],
    ["White text", "#fff", contrast(y, 1), !black],
  ];
  options.forEach(([label, color, ratio, wins], k) => {
    ctx.font = wins ? "bold 20px sans-serif" : "16px sans-serif";
    ctx.fillStyle = color;
    ctx.fillText(`${label}  ${ratio.toFixed(1)}:1${wins ? "  (best)" : ""}`, 32, top + 60 + 70 * k);
  });

  // Stacked contributions on a 0..1 scale, with the black/white switch point.
  const bar = { x: 310, y: top + 30, w: 470, h: 32 };
  const xOf = (v: number): number => bar.x + v * bar.w;
  ctx.fillStyle = "#33433c";
  ctx.fillRect(bar.x, bar.y, bar.w, bar.h);
  let start = 0;
  ["#d64541", "#2ea043", "#3868d6"].forEach((color, i) => {
    ctx.fillStyle = color;
    ctx.fillRect(xOf(start), bar.y, parts[i] * bar.w, bar.h);
    start += parts[i];
  });
  ctx.fillStyle = "#e4ebe6";
  ctx.fillRect(xOf(SWITCH) - 1, bar.y - 6, 2, bar.h + 12);
  ctx.fillStyle = "#f0b45a";
  ctx.fillRect(xOf(y) - 1.5, bar.y, 3, bar.h + 12);
  ctx.font = "13px sans-serif";
  ctx.fillStyle = "#93a39b";
  ctx.fillText(`text flips at Y = ${SWITCH.toFixed(3)}`, xOf(SWITCH) - 50, bar.y - 10);
  ctx.fillText("0", bar.x, bar.y + bar.h + 18);
  ctx.fillText("1", bar.x + bar.w - 8, bar.y + bar.h + 18);

  const ch = rgb.map((v) => (linearize ? srgbToLinear(v / 255) : v / 255));
  ctx.font = "14px monospace";
  const mode = linearize ? "linearized" : "as is, /255";
  ctx.fillText(`RGB ${rgb.join(" ")}, channels ${mode} (L)`, bar.x, top + 112);
  ctx.fillStyle = "#e4ebe6";
  const [r, g, b] = ch.map((v) => v.toFixed(2));
  ctx.fillText(`Y = 0.2126*${r} + 0.7152*${g} + 0.0722*${b}`, bar.x, top + 136);
  ctx.fillText(`  = ${y.toFixed(3)}`, bar.x, top + 156);
  ctx.fillStyle = "#93a39b";
  ctx.fillText(`labels: ${naive ? "always white" : "best contrast"} (N)   click a cell`, bar.x, top + 178);
}

function frame(): void {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawGrid();
  drawPanel();
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
